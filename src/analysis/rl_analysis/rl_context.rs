use super::rl_graph::RLGraph;
use super::rl_graph::{RLEdge, RLIndex, RLNode};
use rustc_hash::{FxHashMap, FxHashSet};
use rustc_middle::mir::{self, Promoted};
use rustc_middle::ty;
use rustc_span::def_id::DefId;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq)]
pub enum CallKind {
    Clone,
    StaticMut,
    Const,
    Static,
    Function,
    Closure,
    Method,
    StaticallyUnknown,
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// RlRy is a struct that represents the type of a place (local variable).
/// It is used to weight the edges of the graph.
/// At the beginning, all the places are assigned to its RlTy, since
/// all the type are known in the local_decls of the MIR.
pub struct RLTy<'tcx, 'a> {
    kind: &'a ty::TyKind<'tcx>,
    mutability: ty::Mutability,
    user_binding: Option<mir::BindingForm<'tcx>>,
}

impl<'tcx, 'a> RLTy<'tcx, 'a> {
    pub fn new(
        kind: &'a ty::TyKind<'tcx>,
        mutability: ty::Mutability,
        user_binding: Option<mir::BindingForm<'tcx>>,
    ) -> Self {
        Self {
            kind,
            mutability,
            user_binding,
        }
    }

    pub fn kind(&self) -> &'a ty::TyKind<'tcx> {
        self.kind
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
    /// A terminator call with the def_id of the function that is called.
    /// The function is not known statically.
    TermCallStaticallyUnknown(DefId),
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

    /// Abstract state.
    /// Map of places and their rvalues, this refers to the local_def_id we are visiting.
    /// It is used to keep track of the rvalue of a local variable.
    /// The value is an option because the rvalues are not initialized at the beginning.
    ///
    /// Basically, it is used to retrieve the function that is called
    /// when it is aliased to a local variable.
    ///
    /// See `visit_local` function.
    pub map_place_rlvalue: FxHashMap<mir::Local, Option<RLValue<'tcx>>>,

    /// Abstract domain.
    /// Map from basic block to the map of places and their rvalues.
    /// It is used to retrieve the rvalues of the places that are used in the basic block.
    /// The value is an option because the rvalues are not initialized at the beginning.
    ///
    /// Basically, after visiting all the basic blocks, we have the rvalues of the places
    /// that are used in the basic block.
    pub map_bb_to_map_place_rlvalue:
        FxHashMap<mir::BasicBlock, FxHashMap<mir::Local, Option<RLValue<'tcx>>>>,

    /// Map from basic block to the places that are used in the basic block.
    /// It is used to retrieve the places that are used in the basic block.
    pub map_bb_used_locals: FxHashMap<mir::BasicBlock, FxHashSet<mir::Local>>,

    /// Map from def_id to the index of the node in the graph.
    /// It is used to retrieve the index of the node in the graph
    /// when we need to add an edge.
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
}
