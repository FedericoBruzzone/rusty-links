use super::{
    rl_context::{CallKind, RLContext},
    rl_graph::RLGraph,
};
use crate::analysis::{rl_analysis::rl_context::RLValue, utils::RUSTC_DEPENDENCIES, Analyzer};
use rustc_const_eval::interpret::GlobalAlloc;
use rustc_hash::FxHashSet;
use rustc_hir::def_id::{DefId, DefIndex};
use rustc_middle::{
    mir::{self, Operand, Promoted, Rvalue},
    ty,
};
use serde::Serialize;

pub struct RLCallResolver<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    ctx: &'a RLContext<'tcx, 'a, G>,
    analyzer: &'a Analyzer<'tcx>,
}

impl<'tcx, 'a, G> RLCallResolver<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    pub fn new(ctx: &'a RLContext<'tcx, 'a, G>, analyzer: &'a Analyzer<'tcx>) -> Self {
        Self { ctx, analyzer }
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
        bb: mir::BasicBlock,
    ) -> Vec<((DefId, Option<Promoted>), CallKind)> {
        match func {
            Operand::Copy(place) => {
                let res = self.retrieve_def_id(place.local, bb);
                log::debug!(
                    "Retrieved(Copy) the def_id of the function (local: {:?}) that is called",
                    place.local
                );
                res
            }
            Operand::Move(place) => {
                let res = self.retrieve_def_id(place.local, bb);
                log::debug!(
                    "Retrieved(Move) the def_id of the function (local: {:?}) that is called",
                    place.local
                );
                res
            }
            Operand::Constant(const_operand) => {
                let (def_id, call_kind) = self.get_def_id(const_operand);
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
        bb: mir::BasicBlock,
    ) -> Vec<((DefId, Option<Promoted>), CallKind)> {
        log::debug!(
            "Retrieving the def_id of the function (local: {:?}) that is called",
            local
        );
        // If we are on bb0 or if the bb has only one parent
        if self.ctx.map_parent_bb.is_empty() || self.ctx.map_parent_bb[&bb].len() == 1 {
            // It can be done because in the visit_terminator we always update set the map_bb_to_map_place_rlvalue
            // with the current map_place_rlvalue.
            match self.ctx.map_bb_to_map_place_rlvalue[&bb][&local]
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
                            vec![self.retrieve_const_val(const_value, ty)]
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
                                vec![self.def_id_as_static_or_const(unevaluated_const.def)]
                            }
                            // An unevaluated constant can be a reference to a const function.
                            // ```rust, ignore
                            // struct T { _value: i32, }
                            // const TEST: fn(T) = |t| { let _ = t; };
                            // fn main() {
                            //     let x = T { _value: 10 };
                            //     let y = &TEST;
                            //     y(x);
                            // }
                            // ```
                            ty::TyKind::Ref(_, _, _) => match unevaluated_const.promoted {
                                Some(x) => {
                                    let promoted =
                                        self.analyzer.tcx.promoted_mir(unevaluated_const.def);
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
                    self.retrieve_def_id(place.local, bb)
                }
                // _5 = move _6
                RLValue::Rvalue(Rvalue::Use(Operand::Move(place))) => {
                    self.retrieve_def_id(place.local, bb)
                }

                RLValue::Rvalue(Rvalue::Ref(_, _, place)) => self.retrieve_def_id(place.local, bb),
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
                    self.resolve_call_def_id(operand, bb)
                }
                _ => unreachable!(),
            }
        } else {
            // We need to retrieve the upper local in order to perform the search of it in the parent basic blocks.
            let upper_local =
                self.retrieve_upper_local_non_const(local, bb, &self.ctx.map_bb_used_locals[&bb]);

            // All possibile basic blocks that can be reached from the current basic block after a SwitchInt.
            let all_targets =
                self.ctx.map_parent_bb[&self.ctx.current_basic_block.unwrap()].clone();
            let mut res = Vec::new();
            for target in all_targets {
                let res_target = self.retrieve_def_id(upper_local, target);
                res.extend(res_target);
            }
            res
        }
    }

    /// Retrieve the upper local that is not a constant.
    /// This function is used to retrieve the upper local that is not a constant.
    fn retrieve_upper_local_non_const(
        &self,
        local: mir::Local,
        bb: mir::BasicBlock,
        candidates: &FxHashSet<mir::Local>,
    ) -> mir::Local {
        if !candidates.contains(&local) {
            return local;
        }
        match self.ctx.map_bb_to_map_place_rlvalue[&bb][&local]
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
    ) -> ((DefId, Option<Promoted>), CallKind) {
        match const_operand.const_ {
            mir::Const::Val(_, ty) => match ty.kind() {
                ty::TyKind::FnDef(def_id, generic_args) => {
                    // Check if it is a clone call
                    if !def_id.is_local() {
                        let krate_name = self.analyzer.tcx.crate_name(def_id.krate);
                        if krate_name == rustc_span::Symbol::intern("core") {
                            let fun_name = self.analyzer.tcx.def_path_str(*def_id);
                            if fun_name == "std::clone::Clone::clone" {
                                return ((*def_id, None), CallKind::Clone);
                            }
                        }
                    }

                    if self.analyzer.tcx.is_closure_like(*def_id) {
                        return ((*def_id, None), CallKind::Closure);
                    }

                    // Interpret the generic_args as a closure
                    let closure_args = generic_args.as_closure().args;

                    if closure_args.len() > 1 {
                        if let Some(ty) = closure_args[0].as_type() {
                            if let ty::TyKind::Closure(closure_def_id, _substs) = ty.kind() {
                                assert!(
                                    self.analyzer.tcx.def_kind(*closure_def_id)
                                        == rustc_hir::def::DefKind::Closure
                                );
                                return ((*closure_def_id, None), CallKind::Closure);
                            }
                        }
                    }

                    // Check if the def_id is a method
                    if self.analyzer.tcx.def_kind(*def_id) == rustc_hir::def::DefKind::AssocFn {
                        let assoc_item = self.analyzer.tcx.associated_item(*def_id);
                        if assoc_item.fn_has_self_parameter {
                            return ((*def_id, None), CallKind::Method);
                        }
                    }

                    if self.analyzer.tcx.is_const_default_method(*def_id) {
                        return ((*def_id, None), CallKind::Method);
                    }

                    if self
                        .analyzer
                        .tcx
                        .impl_method_has_trait_impl_trait_tys(*def_id)
                    {
                        return ((*def_id, None), CallKind::Method);
                    }

                    if let Some(trait_def_id) = self.analyzer.tcx.trait_of_item(*def_id) {
                        let assoc_items = self.analyzer.tcx.associated_items(trait_def_id);
                        for assoc_item in assoc_items.in_definition_order() {
                            if assoc_item.fn_has_self_parameter && assoc_item.def_id == *def_id {
                                return ((*def_id, None), CallKind::Method);
                            }
                        }
                    }

                    // Check if the def_id is a local function
                    if def_id.is_local() {
                        assert!(matches!(
                            self.analyzer.tcx.def_kind(def_id),
                            rustc_hir::def::DefKind::Fn | rustc_hir::def::DefKind::AssocFn
                        ));
                        return ((*def_id, None), CallKind::Function);
                    }

                    // Check if the def_id is external
                    if !def_id.is_local() {
                        // let def_path = self.analyzer.tcx.def_path(*def_id);
                        let krate_name = self.analyzer.tcx.crate_name(def_id.krate);

                        // Check if it is in the core crate
                        if krate_name == rustc_span::Symbol::intern("core") {
                            return ((*def_id, None), CallKind::Function);
                        }

                        // Check if it is in the std crate
                        if krate_name == rustc_span::Symbol::intern("std") {
                            return ((*def_id, None), CallKind::Function);
                        }

                        // Check if it is in the alloc crate
                        if krate_name == rustc_span::Symbol::intern("alloc") {
                            return ((*def_id, None), CallKind::Function);
                        }

                        // Check if it is external but specified as dependency in the Cargo.toml
                        if !RUSTC_DEPENDENCIES.contains(&krate_name.as_str()) {
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
                    self.def_id_as_static_or_const(unevaluated_const.def)
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
    ) -> ((DefId, Option<Promoted>), CallKind) {
        match ty.kind() {
            ty::TyKind::FnDef(def_id, _generic_args) => self.def_id_as_fun_or_method(*def_id),
            ty::TyKind::Ref(_, ty, mutability) => match ty.kind() {
                ty::TyKind::FnDef(def_id, _generic_args) => self.def_id_as_fun_or_method(*def_id),
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
                            self.analyzer,
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
                            self.analyzer,
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

    fn def_id_as_static_or_const(&self, def_id: DefId) -> ((DefId, Option<Promoted>), CallKind) {
        if self.analyzer.tcx.is_static(def_id) {
            let mutability = self.analyzer.tcx.static_mutability(def_id);
            assert!(matches!(
                self.analyzer.tcx.def_kind(def_id),
                rustc_hir::def::DefKind::Static { .. }
            ));
            return ((def_id, None), CallKind::from(mutability.unwrap()));
        }
        assert!(self.analyzer.tcx.def_kind(def_id) == rustc_hir::def::DefKind::Const);
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

    fn def_id_as_fun_or_method(&self, def_id: DefId) -> ((DefId, Option<Promoted>), CallKind) {
        if let Some(def_id) = self.analyzer.tcx.impl_of_method(def_id) {
            assert!(matches!(
                self.analyzer.tcx.def_kind(def_id),
                rustc_hir::def::DefKind::Impl { .. }
            ));
            return ((def_id, None), CallKind::Method);
        }
        assert!(self.analyzer.tcx.def_kind(def_id) == rustc_hir::def::DefKind::Fn);
        ((def_id, None), CallKind::Function)
    }
}
