use super::rl_graph::RLGraph;
use super::rl_graph::{RLEdge, RLIndex, RLNode};
use rustc_index::IndexVec;
use rustc_middle::mir;
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
pub struct RlTy<'tcx, 'a> {
    _kind: &'a ty::TyKind<'tcx>,
    _mutability: ty::Mutability,
    _user_binding: Option<mir::BindingForm<'tcx>>,
}

impl<'tcx, 'a> RlTy<'tcx, 'a> {
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
pub enum RlValue<'tcx> {
    // A MIR rvalue.
    Rvalue(mir::Rvalue<'tcx>),
    // A terminator call with the def_id of the function that is called.
    TermCall(DefId),
    // A terminator call with the def_id of the operand that is cloned.
    TermCallClone(mir::Operand<'tcx>),
}
pub struct RlContext<'tcx, 'a, G>
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
    pub map_place_rlvalue: rustc_hash::FxHashMap<mir::Local, Vec<RlValue<'tcx>>>,

    // Abstract domain/state.
    // Map of places and their types, this refers to the local_def_id we are visiting.
    // It is used to keep track of the type of a local variable.
    //
    // Basically, it is used to weight the edges of the graph.
    // The weight of the edge is the type of the argument.
    pub map_place_ty: rustc_hash::FxHashMap<mir::Local, RlTy<'tcx, 'a>>,

    // Map from def_id to the index of the node in the graph.
    // It is used to retrieve the index of the node in the graph
    // when we need to add an edge.
    pub rl_graph_index_map: rustc_hash::FxHashMap<DefId, G::Index>,
}

impl<G> RlContext<'_, '_, G>
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
