use core::panic;

use crate::analysis::utils::RUSTC_DEPENDENCIES;
use crate::analysis::Analyzer;

use super::rl_graph::RLGraph;
use super::rl_graph::{RLEdge, RLIndex, RLNode};
use rustc_const_eval::interpret::GlobalAlloc;
use rustc_hash::{FxHashMap, FxHashSet};
use rustc_hir::def_id::DefIndex;
use rustc_middle::mir::{self, Operand, Promoted, Rvalue};
use rustc_middle::ty;
use rustc_span::def_id::DefId;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, PartialEq)]
pub enum CallKind {
    Clone,
    StaticMut,
    Const,
    Static,
    Function,
    Closure,
    Method,
    Unknown,
}

impl From<ty::Mutability> for CallKind {
    fn from(mutability: ty::Mutability) -> Self {
        match mutability {
            ty::Mutability::Mut => CallKind::StaticMut,
            ty::Mutability::Not => CallKind::Static,
        }
    }
}

/// RlRy is a struct that represents the type of a place (local variable).
/// It is used to weight the edges of the graph.
/// At the beginning, all the places are assigned to its RlTy, since
/// all the type are known in the local_decls of the MIR.
pub struct RLTy<'tcx, 'a> {
    _kind: &'a ty::TyKind<'tcx>,
    _mutability: ty::Mutability,
    _user_binding: Option<mir::BindingForm<'tcx>>,
}

impl<'tcx, 'a> RLTy<'tcx, 'a> {
    pub fn new(
        kind: &'a ty::TyKind<'tcx>,
        mutability: ty::Mutability,
        user_binding: Option<mir::BindingForm<'tcx>>,
    ) -> Self {
        Self {
            _kind: kind,
            _mutability: mutability,
            _user_binding: user_binding,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum RLValue<'tcx> {
    /// A MIR rvalue.
    Rvalue(mir::Rvalue<'tcx>),
    /// A terminator call with the def_id of the function that is called.
    TermCall(DefId),
    /// A terminator call with the def_id of the operand that is cloned.
    TermCallClone(mir::Operand<'tcx>),
    /// A terminator call with the def_id of the const that is called.
    ///
    /// For example, in the following MIR:
    /// ```rust,ignore
    /// bb0: {
    ///     _1 = const TEST_LAMBDA_C() -> [return: bb1, unwind continue];
    /// }
    /// ```
    TermCallConst(DefId),
    /// A terminator call with the def_id of the static that is called.
    TermCallStatic(DefId),
    /// A terminator call with the def_id of the mutable static that is called.
    /// It means the used of `unsafe` to mutate the static.
    TermCallStaticMut(DefId),
}

pub struct ComeFromSwitchCache<'tcx> {
    pub cache: FxHashMap<mir::Local, Option<RLValue<'tcx>>>,
    pub set_targets: FxHashSet<mir::BasicBlock>,
}

impl<'tcx> ComeFromSwitchCache<'tcx> {
    pub fn new(
        cache: FxHashMap<mir::Local, Option<RLValue<'tcx>>>,
        set_targets: FxHashSet<mir::BasicBlock>,
    ) -> Self {
        Self { cache, set_targets }
    }
}

pub struct RLContext<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    /// The `local_def_id` is used to keep track of the current function that is visited.
    /// The MIR does not allow nested functions.
    pub current_local_def_id: Option<DefId>, // , &'a IndexVec<mir::Local, mir::LocalDecl<'tcx>>

    /// The `basic_block` is used to keep track of the current basic block that is visited.
    /// The MIR does not allow nested functions.
    pub current_basic_block: Option<mir::BasicBlock>,

    /// The cache is used only when a SwitchInt terminator is encauntered.
    /// It is used to simulate the visit of the basic blocks that are the target of the SwitchInt.
    ///
    /// We store the `map_place_rlvalue` and the `targets`.
    /// The map is used to keep the state and when a basic block
    /// in the targets is visited we restore the real `map_place_rlvalue`
    /// with the cache one.
    pub stack_come_from_switch_cache: Vec<ComeFromSwitchCache<'tcx>>,

    // Map from basic block to the basic blocks that are the parent of the current basic block.
    // Vector size is not 1 only when a SwitchInt terminator was encoutered.
    pub map_parent_bb: FxHashMap<mir::BasicBlock, Vec<mir::BasicBlock>>,

    // Map of places and their types, this refers to the local_def_id we are visiting.
    // It is used to keep track of the type of a local variable.
    //
    // Basically, it is used to weight the edges of the graph.
    // The weight of the edge is the type of the argument.
    pub map_place_ty: FxHashMap<mir::Local, RLTy<'tcx, 'a>>,

    // Abstract state.
    // Map of places and their rvalues, this refers to the local_def_id we are visiting.
    // It is used to keep track of the rvalue of a local variable.
    // The value is an option because the rvalues are not initialized at the beginning.
    //
    // Basically, it is used to retrieve the function that is called
    // when it is aliased to a local variable.
    //
    // See `visit_local` function.
    pub map_place_rlvalue: FxHashMap<mir::Local, Option<RLValue<'tcx>>>,

    // Abstract domain.
    pub map_bb_to_map_place_rlvalue:
        FxHashMap<mir::BasicBlock, FxHashMap<mir::Local, Option<RLValue<'tcx>>>>,

    /// Map from basic block to the places that are used in the basic block.
    /// It is used to retrieve the places that are used in the basic block.
    pub map_bb_used_locals: FxHashMap<mir::BasicBlock, FxHashSet<mir::Local>>,

    // Map from def_id to the index of the node in the graph.
    // It is used to retrieve the index of the node in the graph
    // when we need to add an edge.
    pub rl_graph_index_map: FxHashMap<(DefId, Option<Promoted>), G::Index>,
}

impl<G> RLContext<'_, '_, G>
where
    G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex>
        + Default
        + Clone
        + Serialize
        + DeserializeOwned,
{
    pub fn new() -> Self {
        Self {
            current_local_def_id: None,
            current_basic_block: None,
            map_parent_bb: FxHashMap::default(),
            stack_come_from_switch_cache: Vec::new(),
            map_place_rlvalue: FxHashMap::default(),
            map_bb_to_map_place_rlvalue: FxHashMap::default(),
            map_place_ty: FxHashMap::default(),
            map_bb_used_locals: FxHashMap::default(),
            rl_graph_index_map: FxHashMap::default(),
        }
    }
}

impl<'tcx, G> RLContext<'tcx, '_, G>
where
    G: RLGraph<Node = RLNode, Edge = RLEdge, Index = RLIndex>
        + Default
        + Clone
        + Serialize
        + DeserializeOwned,
{
    pub fn insert_map_place_rlvalue(&mut self, local: mir::Local, rl_value: RLValue<'tcx>) {
        self.map_place_rlvalue.insert(local, Some(rl_value));
    }

    pub fn add_current_bb_as_parent_of(&mut self, bb: mir::BasicBlock) {
        if let Some(parents) = self.map_parent_bb.get_mut(&bb) {
            parents.push(self.current_basic_block.unwrap());
        } else {
            self.map_parent_bb
                .insert(bb, vec![self.current_basic_block.unwrap()]);
        }
    }

    /// Retrieve the def_id of the function that is called.
    /// This function call the `retrieve_def_id` or `get_def_id`
    /// which recursively retrieve the def_id of the function.
    ///
    /// The difference between the two functions is that the `retrieve_def_id`
    /// is called when the operand is a place (local variable) so we need to go deeper
    /// to retrieve the def_id of the function, it operates in O(n) where n is the depth of the
    /// recursion.
    /// The `get_def_id` is called when the operand is a constant, so we can directly
    /// retrieve the def_id of the function, it operates in O(1).
    ///
    /// *NOTE*: This function is called always from the `visit_terminator` since the functions
    /// can be called only in it.
    pub fn resolve_call_def_id(
        &self,
        func: &mir::Operand<'tcx>,
        analyzer: &Analyzer,
        bb: mir::BasicBlock,
    ) -> Vec<((DefId, Option<Promoted>), CallKind)> {
        match func {
            mir::Operand::Copy(place) => {
                let res = self.retrieve_def_id(place.local, analyzer, bb);
                log::debug!(
                    "Retrieved(Copy) the def_id of the function (local: {:?}) that is called",
                    place.local
                );
                res
            }
            Operand::Move(place) => {
                let res = self.retrieve_def_id(place.local, analyzer, bb);
                log::debug!(
                    "Retrieved(Move) the def_id of the function (local: {:?}) that is called",
                    place.local
                );
                res
            }
            Operand::Constant(const_operand) => {
                let (def_id, call_kind) = self.get_def_id(const_operand, analyzer);
                log::debug!(
                    "Retrieved(Constant) the def_id {:?} of the {:?} that is called",
                    def_id,
                    call_kind
                );
                vec![(def_id, call_kind)]
            }
        }
    }

    /// Recursively retrieve the `def_id` of the called function.
    /// This function assumes that the `local` is one of the following, otherwise it panics:
    /// - A function
    /// - A method
    /// - A static function
    /// - A const function
    ///
    /// Note that the `closure` is handled in the `get_def_id` function, not in this one.
    ///
    /// This function operates in O(n) where n is the depth of the recursion.
    pub fn retrieve_def_id(
        &self,
        local: mir::Local,
        analyzer: &Analyzer,
        bb: mir::BasicBlock,
    ) -> Vec<((DefId, Option<Promoted>), CallKind)> {
        log::debug!(
            "Retrieving the def_id of the function (local: {:?}) that is called",
            local
        );
        if self.map_parent_bb.is_empty() || self.map_parent_bb[&bb].len() == 1 {
            // It can be done because in the visit_terminator we always update set the map_bb_to_map_place_rlvalue
            // with the current map_place_rlvalue.
            match self.map_bb_to_map_place_rlvalue[&bb][&local]
                .as_ref()
                .unwrap_or_else(|| unreachable!())
            {
                // _5 = const T
                // T := main::promoted[0] // this could be function, method or a const
                //   | {alloc1: &[char; 5]}
                RLValue::Rvalue(Rvalue::Use(Operand::Constant(const_operand))) => {
                    // BASE CASE
                    // It safe at this point to assume that the constant is a function call.
                    // A closure (as terminator) can never be in the form:
                    // ```rust, ignore
                    // _5 = move|copy _6
                    // ```
                    // because the closure is always in the form:
                    // ```rust, ignore
                    // _5 = {closure@src/main.rs:18:18: 18:20} ...
                    // ```
                    // and this case is handled in the `get_def_id` which is called
                    // by `retrieve_call_def_id` in case of a constant.
                    match const_operand.const_ {
                        mir::Const::Val(const_value, ty) => {
                            vec![self.retrieve_const_val(const_value, ty, analyzer)]
                        }
                        mir::Const::Unevaluated(unevaluated_const, ty) => match ty.kind() {
                            ty::TyKind::FnPtr(_, _) => {
                                // The static in this case is difficult to replicate in the MIR
                                // but we convert it.
                                //
                                // *NOTE* This is an expected case, since we are not able to replicate.
                                // For instance, in the following MIR:
                                // ```rust,ignore
                                // bb0: {
                                //     _1 = const  const {alloc11: &fn()}-> [return: bb1, unwind continue];
                                // }
                                // ```
                                vec![self.def_id_as_static_or_const(unevaluated_const.def, analyzer)]
                            }
                            ty::TyKind::Ref(_, _, _) => match unevaluated_const.promoted {
                                Some(x) => {
                                    let promoted = analyzer.tcx.promoted_mir(unevaluated_const.def);
                                    let def_id = promoted[x].source.instance.def_id();
                                    let promoted = promoted[x].source.promoted.unwrap();
                                    vec![((def_id, Some(promoted)), CallKind::Const)]
                                }
                                None => unreachable!(),
                            },
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    }
                }
                // _5 = copy (*_6)
                RLValue::Rvalue(Rvalue::Use(Operand::Copy(place))) => {
                    self.retrieve_def_id(place.local, analyzer, bb)
                }
                // _5 = move _6
                RLValue::Rvalue(Rvalue::Use(Operand::Move(place))) => {
                    self.retrieve_def_id(place.local, analyzer, bb)
                }

                RLValue::Rvalue(Rvalue::Ref(_, _, place)) => {
                    self.retrieve_def_id(place.local, analyzer, bb)
                }
                // In rust is:
                //
                // ```rust, ignore
                // let mut x = test_own as fn(T);
                // x = test as fn(T);
                // x(T { _value: 10 });
                // ```
                //
                // In MIR is translated as:
                // ```rust, ignore
                // bb0: {
                //     _1 = test_own as fn(T) (PointerCoercion(ReifyFnPointer, AsCast));
                //     _2 = test as fn(T) (PointerCoercion(ReifyFnPointer, AsCast));
                //     _1 = move _2;
                //     _4 = copy _1;
                //     _5 = T { _value: const 10_i32 };
                //     _3 = move _4(move _5) -> [return: bb1, unwind continue];
                // }
                // ```
                RLValue::Rvalue(Rvalue::Cast(_, operand, _)) => {
                    self.resolve_call_def_id(operand, analyzer, bb)
                }
                _ => unreachable!(),
            }
        } else {
            let upper_local =
                self.retrieve_upper_local_non_const(local, bb, &self.map_bb_used_locals[&bb]);

            let all_targets = self.map_parent_bb[&self.current_basic_block.unwrap()].clone();
            let mut res = Vec::new();
            for target in all_targets {
                let res_target = self.retrieve_def_id(upper_local, analyzer, target);
                res.extend(res_target);
            }
            res
        }
    }

    fn retrieve_upper_local_non_const(
        &self,
        local: mir::Local,
        bb: mir::BasicBlock,
        candidates: &FxHashSet<mir::Local>,
    ) -> mir::Local {
        if !candidates.contains(&local) {
            return local;
        }
        match self.map_bb_to_map_place_rlvalue[&bb][&local]
            .as_ref()
            .unwrap_or_else(|| unreachable!())
        {
            // _5 = copy (*_6)
            RLValue::Rvalue(Rvalue::Use(Operand::Copy(place))) => {
                self.retrieve_upper_local_non_const(place.local, bb, candidates)
            }
            // _5 = move _6
            RLValue::Rvalue(Rvalue::Use(Operand::Move(place))) => {
                self.retrieve_upper_local_non_const(place.local, bb, candidates)
            }
            // _5 = &(*10)
            RLValue::Rvalue(Rvalue::Ref(_, _, place)) => {
                self.retrieve_upper_local_non_const(place.local, bb, candidates)
            }
            // In rust is:
            //
            // ```rust, ignore
            // let mut x = test_own as fn(T);
            // x = test as fn(T);
            // x(T { _value: 10 });
            // ```
            //
            // In MIR is translated as:
            // ```rust, ignore
            // bb0: {
            //     _1 = test_own as fn(T) (PointerCoercion(ReifyFnPointer, AsCast));
            //     _2 = test as fn(T) (PointerCoercion(ReifyFnPointer, AsCast));
            //     _1 = move _2;
            //     _4 = copy _1;
            //     _5 = T { _value: const 10_i32 };
            //     _3 = move _4(move _5) -> [return: bb1, unwind continue];
            // }
            // ```
            RLValue::Rvalue(Rvalue::Cast(_, operand, _)) => match operand {
                mir::Operand::Copy(place) => {
                    self.retrieve_upper_local_non_const(place.local, bb, candidates)
                }
                mir::Operand::Move(place) => {
                    self.retrieve_upper_local_non_const(place.local, bb, candidates)
                }
                _ => unreachable!(),
            },
            RLValue::Rvalue(Rvalue::CopyForDeref(place)) => {
                self.retrieve_upper_local_non_const(place.local, bb, candidates)
            }
            x => panic!("The RLValue {:?} is not handled", x),
        }
    }

    /// Get the def_id of the function that is called.
    /// This function assumes that the `const_operand` is one of the following, otherwise it panics:
    /// - A function
    /// - A method
    /// - A static function
    /// - A const function
    /// - A closure
    ///
    /// The closure are always in the form:
    /// ```rust,ignore
    /// bb5: {
    ///     _15 = {closure@src/main.rs:18:18: 18:20};
    ///     _17 = &_15;
    ///     _18 = ();
    ///     _16 = <{closure@src/main.rs:18:18: 18:20} as std::ops::Fn<()>>::call(move _17, move _18) -> [return: bb6, unwind continue];
    /// }
    /// ```
    ///
    /// It operates in O(1).
    fn get_def_id(
        &self,
        const_operand: &mir::ConstOperand<'tcx>,
        analyzer: &Analyzer,
    ) -> ((DefId, Option<Promoted>), CallKind) {
        match const_operand.const_ {
            mir::Const::Val(_, ty) => match ty.kind() {
                ty::TyKind::FnDef(def_id, generic_args) => {
                    // Check if it is a clone call
                    if !def_id.is_local() {
                        let krate_name = analyzer.tcx.crate_name(def_id.krate);
                        if krate_name == rustc_span::Symbol::intern("core") {
                            let fun_name = analyzer.tcx.def_path_str(*def_id);
                            if fun_name == "std::clone::Clone::clone" {
                                return ((*def_id, None), CallKind::Clone);
                            }
                        }
                    }

                    // Check if the def_id is a method
                    if let Some(def_id) = analyzer.tcx.impl_of_method(*def_id) {
                        return ((def_id, None), CallKind::Method);
                    }

                    // Interpret the generic_args as a closure
                    let closure_args = generic_args.as_closure().args;

                    if closure_args.len() > 1 {
                        if let Some(ty) = closure_args[0].as_type() {
                            if let ty::TyKind::Closure(closure_def_id, _substs) = ty.kind() {
                                assert!(
                                    analyzer.tcx.def_kind(*closure_def_id)
                                        == rustc_hir::def::DefKind::Closure
                                );
                                return ((*closure_def_id, None), CallKind::Closure);
                            }
                        }
                    }

                    // Check if the def_id is a local function
                    if def_id.is_local() {
                        assert!(analyzer.tcx.def_kind(def_id) == rustc_hir::def::DefKind::Fn);
                        return ((*def_id, None), CallKind::Function);
                    }

                    // Check if the def_id is external
                    if !def_id.is_local() {
                        // let def_path = self.analyzer.tcx.def_path(*def_id);
                        let krate_name = analyzer.tcx.crate_name(def_id.krate);

                        // Check if it is in the core crate
                        if krate_name == rustc_span::Symbol::intern("core") {
                            return ((*def_id, None), CallKind::Function);
                        }

                        // Check if it is in the std crate
                        if krate_name == rustc_span::Symbol::intern("std") {
                            return ((*def_id, None), CallKind::Function);
                        }

                        // Check if it is external but specified as dependency in the Cargo.toml
                        if !RUSTC_DEPENDENCIES.contains(&krate_name.as_str()) {
                            // From external crates we can inkove only functions
                            return ((*def_id, None), CallKind::Function);
                        }
                    }

                    // The def_id should not be handled
                    (
                        (
                            DefId {
                                krate: def_id.krate,
                                index: DefIndex::from_usize(0),
                            },
                            None,
                        ),
                        CallKind::Unknown,
                    )
                }
                _ => unreachable!(),
            },
            mir::Const::Unevaluated(unevaluated_const, ty) => match ty.kind() {
                ty::TyKind::FnPtr(_, _) => {
                    // The static in this case is difficult to replicate in the MIR
                    // but we convert it.
                    //
                    // *NOTE* This is an expected case, since we are not able to replicate.
                    // For instance, in the following MIR:
                    // ```rust,ignore
                    // bb0: {
                    //     _1 = const  const {alloc11: &fn()}-> [return: bb1, unwind continue];
                    // }
                    // ```
                    self.def_id_as_static_or_const(unevaluated_const.def, analyzer)
                }
                _ => unreachable!(),
            },
            mir::Const::Ty(_, _) => unreachable!(),
        }
    }

    fn retrieve_const_val(
        &self,
        const_value: mir::ConstValue,
        ty: ty::Ty<'tcx>,
        analyzer: &Analyzer,
    ) -> ((DefId, Option<Promoted>), CallKind) {
        match ty.kind() {
            ty::TyKind::FnDef(def_id, _generic_args) => {
                self.def_id_as_fun_or_method(*def_id, analyzer)
            }
            ty::TyKind::Ref(_, ty, mutability) => match ty.kind() {
                ty::TyKind::FnDef(def_id, _generic_args) => {
                    self.def_id_as_fun_or_method(*def_id, analyzer)
                }
                // This is something like:
                // ```rust, ignore
                // static TEST: fn() = || {};
                // TEST();
                // ```
                // The static in the compiler are allocated directly in the memory.
                ty::TyKind::FnPtr(_, _) => {
                    if let mir::ConstValue::Scalar(mir::interpret::Scalar::Ptr(pointer, _)) =
                        const_value
                    {
                        return self.alloc_id_as_static(
                            pointer.provenance.alloc_id(),
                            mutability,
                            analyzer,
                        );
                    }
                    unreachable!()
                }
                _ => unreachable!(),
            },
            ty::TyKind::RawPtr(ty, mutability) => match ty.kind() {
                // This could something like:
                // ```rust, ignore
                // static mut TEST: fn() = || {};
                // unsafe { TEST(); }
                // ```
                // The static in the compiler are allocated directly in the memory.
                ty::TyKind::FnPtr(_, _) => {
                    if let mir::ConstValue::Scalar(mir::interpret::Scalar::Ptr(pointer, _)) =
                        const_value
                    {
                        return self.alloc_id_as_static(
                            pointer.provenance.alloc_id(),
                            mutability,
                            analyzer,
                        );
                    }
                    unreachable!()
                }
                _ => unreachable!(),
            },
            ty::TyKind::FnPtr(binder, _fn_header) => {
                log::error!(
                    "The const_value ({:?}) is a function pointer: {:?}",
                    const_value,
                    binder
                );
                unreachable!()
            }
            _ => unreachable!(),
        }
    }

    fn def_id_as_static_or_const(
        &self,
        def_id: DefId,
        analyzer: &Analyzer,
    ) -> ((DefId, Option<Promoted>), CallKind) {
        if analyzer.tcx.is_static(def_id) {
            let mutability = analyzer.tcx.static_mutability(def_id);
            assert!(matches!(
                analyzer.tcx.def_kind(def_id),
                rustc_hir::def::DefKind::Static { .. }
            ));
            return ((def_id, None), CallKind::from(mutability.unwrap()));
        }
        assert!(analyzer.tcx.def_kind(def_id) == rustc_hir::def::DefKind::Const);
        ((def_id, None), CallKind::Const)
    }

    fn alloc_id_as_static(
        &self,
        alloc_id: mir::interpret::AllocId,
        mutability: &ty::Mutability,
        analyzer: &Analyzer,
    ) -> ((DefId, Option<Promoted>), CallKind) {
        if let GlobalAlloc::Static(def_id) = analyzer.tcx.global_alloc(alloc_id) {
            assert!(matches!(
                analyzer.tcx.def_kind(def_id),
                rustc_hir::def::DefKind::Static { .. }
            ));
            return ((def_id, None), CallKind::from(*mutability));
        }
        unreachable!()
    }

    fn def_id_as_fun_or_method(
        &self,
        def_id: DefId,
        analyzer: &Analyzer,
    ) -> ((DefId, Option<Promoted>), CallKind) {
        if let Some(def_id) = analyzer.tcx.impl_of_method(def_id) {
            assert!(matches!(
                analyzer.tcx.def_kind(def_id),
                rustc_hir::def::DefKind::Impl { .. }
            ));
            return ((def_id, None), CallKind::Method);
        }
        assert!(analyzer.tcx.def_kind(def_id) == rustc_hir::def::DefKind::Fn);
        ((def_id, None), CallKind::Function)
    }
}
