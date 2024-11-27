use crate::analysis::utils::RUSTC_DEPENDENCIES;
use crate::analysis::Analyzer;

use super::rl_graph::RLGraph;
use super::rl_graph::{RLEdge, RLIndex, RLNode};
use rustc_hir::def_id::DefIndex;
use rustc_index::IndexVec;
use rustc_middle::mir::{self, Operand, Rvalue};
use rustc_middle::ty;
use rustc_span::def_id::DefId;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, PartialEq)]
pub enum CallKind {
    Clone,
    Function,
    Closure,
    Method,
    Unknown,
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
pub enum RLValue<'tcx> {
    // A MIR rvalue.
    Rvalue(mir::Rvalue<'tcx>),
    // A terminator call with the def_id of the function that is called.
    TermCall(DefId),
    // A terminator call with the def_id of the operand that is cloned.
    TermCallClone(mir::Operand<'tcx>),
}
pub struct RLContext<'tcx, 'a, G>
where
    G: RLGraph + Default + Clone + Serialize,
{
    // Stack of local_def_id and local_decls.
    // It should enought to keep track the current function and its local variables,
    // becuase the MIR does not allow nested functions.
    pub stack_local_def_id: Vec<(DefId, &'a IndexVec<mir::Local, mir::LocalDecl<'tcx>>)>,

    // Abstract domain/state.
    // Map of places and their rvalues, this refers to the local_def_id we are visiting.
    // It is used to keep track of the rvalue of a local variable.
    // It is a vector because a local variable can be assigned multiple times.
    // During the visit the last rvalue is always the last assigned value.
    //
    // Basically, it is used to retrieve the function that is called
    // when it is aliased to a local variable.
    //
    // See `visit_local` function.
    pub map_place_rlvalue: rustc_hash::FxHashMap<mir::Local, Vec<RLValue<'tcx>>>,

    // Abstract domain/state.
    // Map of places and their types, this refers to the local_def_id we are visiting.
    // It is used to keep track of the type of a local variable.
    //
    // Basically, it is used to weight the edges of the graph.
    // The weight of the edge is the type of the argument.
    pub map_place_ty: rustc_hash::FxHashMap<mir::Local, RLTy<'tcx, 'a>>,

    // Map from def_id to the index of the node in the graph.
    // It is used to retrieve the index of the node in the graph
    // when we need to add an edge.
    pub rl_graph_index_map: rustc_hash::FxHashMap<DefId, G::Index>,
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
            stack_local_def_id: Vec::new(),
            map_place_rlvalue: rustc_hash::FxHashMap::default(),
            map_place_ty: rustc_hash::FxHashMap::default(),
            rl_graph_index_map: rustc_hash::FxHashMap::default(),
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
    pub fn push_or_insert_map_place_rlvalue(&mut self, local: mir::Local, rl_value: RLValue<'tcx>) {
        match self.map_place_rlvalue.get_mut(&local) {
            Some(rvalues) => rvalues.push(rl_value),
            None => {
                self.map_place_rlvalue.insert(local, vec![rl_value]);
            }
        }
    }

    /// Retrieve the def_id of the function that is called.
    /// This function call the `retrieve_fun_def_id` or `retrieve_fun_or_closure_def_id`
    /// which recursively retrieve the def_id of the function.
    pub fn retrieve_call_def_id(
        &self,
        func: &mir::Operand<'tcx>,
        analyzer: &Analyzer,
    ) -> (DefId, CallKind) {
        match func {
            mir::Operand::Copy(place) => {
                let (def_id, call_kind) = self.retrieve_fun_or_method_def_id(place.local, analyzer);
                log::debug!(
                    "Retrieving(Copy) the def_id of the function (local: {:?}) that is called",
                    place.local
                );
                (def_id, call_kind)
            }
            mir::Operand::Move(place) => {
                let (def_id, call_kind) = self.retrieve_fun_or_method_def_id(place.local, analyzer);
                log::debug!(
                    "Retrieving(Move) the def_id of the function (local: {:?}) that is called",
                    place.local
                );
                (def_id, call_kind)
            }
            mir::Operand::Constant(const_operand) => {
                let (def_id, call_kind) = self.get_fun_meth_closure_def_id(const_operand, analyzer);
                log::debug!(
                    "Retrieving(Constant) the def_id {:?} of the {:?} that is called",
                    def_id,
                    call_kind
                );
                (def_id, call_kind)
            }
        }
    }

    /// Recursively retrieve the def_id of the function.
    /// This function assumes that the `local` is a function, so
    /// it panics if it is not.
    ///
    /// For instance, in the following MIR:
    /// ```rust,ignore
    /// bb4: {
    ///     // ...
    ///     _11 = test_own;
    ///    // ...
    ///    _13 = copy _11;
    ///    // ...
    ///    _15 = &_1;
    ///    _14 = <T as std::clone::Clone>::clone(move _15) -> [return: bb5, unwind continue];
    /// }
    /// ```
    /// The function `test_own` is assigned to the local `_11`.
    /// The local `_13` is a copy of the local `_11`.
    /// The local `_15` is a reference to the local `_1`.
    /// The function `test_own` is then called with the local `_15`.
    /// At this point, we need to retrieve the def_id of the function `test_own`.
    pub fn retrieve_fun_or_method_def_id(
        &self,
        local: mir::Local,
        analyzer: &Analyzer,
    ) -> (DefId, CallKind) {
        fn fun_or_method(def_id: DefId, analyzer: &Analyzer) -> (DefId, CallKind) {
            if let Some(def_id) = analyzer.tcx.impl_of_method(def_id) {
                return (def_id, CallKind::Method);
            }
            (def_id, CallKind::Function)
        }

        match &self.map_place_rlvalue[&local].last() {
            // _5 = function
            // _5 = const main::promoted[0]
            Some(RLValue::Rvalue(Rvalue::Use(Operand::Constant(const_operand)))) => {
                // It seems to be safa at this point to assume that the constant is a function.
                // A closure (as terminator) can never be in the form:
                // ```rust, ignore
                // _5 = move|copy _6
                // ```
                // because the closure is always in the form:
                // ```rust, ignore
                // _5 = {closure@src/main.rs:18:18: 18:20} ...
                // ```
                // and this case is handled in the `retrieve_fun_meth_closure_def_id` which is called
                // by `retrieve_call_def_id` in case of a constant (which is exactly this case).
                match const_operand.const_.ty().kind() {
                    ty::TyKind::FnDef(def_id, _generic_args) => fun_or_method(*def_id, analyzer),
                    ty::TyKind::Ref(_region, ty, _mutability) => match ty.kind() {
                        ty::TyKind::FnDef(def_id, _generic_args) => {
                            fun_or_method(*def_id, analyzer)
                        }
                        _ => unreachable!(),
                    },
                    _ => {
                        log::error!(
                            "The local ({:?}) is not a function, but a constant: {:?}",
                            local,
                            const_operand.const_.ty()
                        );
                        unreachable!()
                    }
                }
            }
            // _5 = copy (*_6)
            Some(RLValue::Rvalue(Rvalue::Use(Operand::Copy(place)))) => {
                self.retrieve_fun_or_method_def_id(place.local, analyzer)
            }
            // _5 = move _6
            Some(RLValue::Rvalue(Rvalue::Use(Operand::Move(place)))) => {
                self.retrieve_fun_or_method_def_id(place.local, analyzer)
            }
            // _5 = &(*10)
            Some(RLValue::Rvalue(Rvalue::Ref(_region, _borrow_kind, place))) => {
                self.retrieve_fun_or_method_def_id(place.local, analyzer)
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
            Some(RLValue::Rvalue(Rvalue::Cast(_cast_kind, operand, _ty))) => {
                self.retrieve_call_def_id(operand, analyzer)
            }
            _ => unreachable!(),
        }
    }

    /// Get the def_id of the function or closure.
    ///
    /// This function assumes that the const_operand is a function.
    /// In case it is a method, it tries to interpret it as a method.
    /// In case it is a closure, it tries to interpret it as a closure.
    ///
    /// For instance, in the following MIR:
    /// ```rust,ignore
    /// bb5: {
    ///     _15 = {closure@src/main.rs:18:18: 18:20};
    ///     _17 = &_15;
    ///     _18 = ();
    ///     _16 = <{closure@src/main.rs:18:18: 18:20} as std::ops::Fn<()>>::call(move _17, move _18) -> [return: bb6, unwind continue];
    /// }
    /// ```
    /// In this case the first arguments it `move _17` that is a reference to the closure.
    fn get_fun_meth_closure_def_id(
        &self,
        const_operand: &mir::ConstOperand<'tcx>,
        analyzer: &Analyzer,
    ) -> (DefId, CallKind) {
        match const_operand.const_.ty().kind() {
            ty::TyKind::FnDef(def_id, generic_args) => {
                // Check if it is a clone call
                if !def_id.is_local() {
                    let krate_name = analyzer.tcx.crate_name(def_id.krate);
                    if krate_name == rustc_span::Symbol::intern("core") {
                        let fun_name = analyzer.tcx.def_path_str(*def_id);
                        if fun_name == "std::clone::Clone::clone" {
                            return (*def_id, CallKind::Clone);
                        }
                    }
                }

                // Check if the def_id is a method
                if let Some(def_id) = analyzer.tcx.impl_of_method(*def_id) {
                    return (def_id, CallKind::Method);
                }

                // Interpret the generic_args as a closure
                let closure_args = generic_args.as_closure().args;

                if closure_args.len() > 1 {
                    if let Some(ty) = closure_args[0].as_type() {
                        if let ty::TyKind::Closure(closure_def_id, _substs) = ty.kind() {
                            return (*closure_def_id, CallKind::Closure);
                        }
                    }
                }

                // Check if the def_id is a local function
                if def_id.is_local() {
                    return (*def_id, CallKind::Function);
                }

                // Check if the def_id is external
                if !def_id.is_local() {
                    // let def_path = self.analyzer.tcx.def_path(*def_id);
                    let krate_name = analyzer.tcx.crate_name(def_id.krate);

                    // Check if it is in the core crate
                    if krate_name == rustc_span::Symbol::intern("core") {
                        return (*def_id, CallKind::Function);
                    }

                    // Check if it is in the std crate
                    if krate_name == rustc_span::Symbol::intern("std") {
                        return (*def_id, CallKind::Function);
                    }

                    // Check if it is external but specified as dependency in the Cargo.toml
                    if !RUSTC_DEPENDENCIES.contains(&krate_name.as_str()) {
                        // From external crates we can inkove only functions
                        return (*def_id, CallKind::Function);
                    }
                }

                // The def_id should not be handled
                (
                    DefId {
                        krate: def_id.krate,
                        index: DefIndex::from_usize(0),
                    },
                    CallKind::Unknown,
                )
            }
            _ => unreachable!(),
        }
    }
}
