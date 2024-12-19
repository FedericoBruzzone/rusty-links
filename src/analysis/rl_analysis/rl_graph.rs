use crate::analysis::utils::{DUMMY_CRATE_NUM, DUMMY_DEF_INDEX};

use super::rl_context::{CallKind, MutabilityKind, OperandKind, RLTyKind};
use rustc_hir::def_id::{CrateNum, DefIndex};
use rustc_middle::mir::Promoted;
use rustc_span::def_id::DefId;
use serde::{Deserialize, Serialize};

/// The `RLGraphEdge` trait represents an edge in a graph.
pub trait RLGraphEdge {
    fn create(edge: (CallKind, Vec<(OperandKind, MutabilityKind, RLTyKind)>)) -> Self;
}

/// The `RLGraphNode` trait represents a node in a graph.
pub trait RLGraphNode {
    fn create(def_id: DefId, promoted: Option<Promoted>) -> Self;
    fn def_id_str(&self) -> String;
    fn promoted(&self) -> Option<Promoted>;
}

#[allow(unused)]
/// The `RLGraphIndex` trait represents the index of a node in a graph.
pub trait RLGraphIndex {
    fn create(value: usize) -> Self;
    fn value(&self) -> usize;
}

/// The `RLGraph` trait represents a graph where the nodes are of type `RLGraphNode`,
/// the edges are of type `RLGraphEdge`, and the indices are of type `RLGraphIndex`.
/// The graph is mutable, and it is possible to add nodes and edges to it.
pub trait RLGraph {
    type Node: RLGraphNode + Serialize;
    type Edge: RLGraphEdge + Serialize;
    type Index: RLGraphIndex + Serialize;

    fn rl_add_node(&mut self, node: Self::Node) -> Self::Index;
    fn rl_add_edge(&mut self, source: Self::Index, target: Self::Index, edge: Self::Edge);
    fn merge(&mut self, other: &Self);
    fn as_dot_str(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct RLNode {
    def_id: DefId,
    promoted: Option<Promoted>,
    def_id_str: String,
}

impl PartialEq for RLNode {
    // This implementation is necessary because the `RLNode` for merge correctly the graphs.
    // def_id_str_1 -> DefId(20:4 ~ crate_a[132e]::add)
    // def_id_str_2 -> DefId(0:4  ~ crate_a[132e]::add)
    //                            ^^^^^^^^^^^^^^^^^^^^
    // The `def_id_str` contains the crate name, so we need to compare only this part in order to
    // merge the graphs correctly.
    // This is caused by the fact that when we analyze a call to a function `add` from the `crate_X`
    // and the declaration of the function `add` in the `crate_a`, the `def_id` is different but
    // we can check if the function is the same by comparing the `def_id_str` the part after the
    // `~` character.
    fn eq(&self, other: &Self) -> bool {
        let def_id_str_1 = self.def_id_str();
        let promoted_1 = self
            .promoted()
            .map_or(u32::MAX, |promoted| promoted.as_u32());
        let def_id_str_2 = other.def_id_str();
        let promoted_2 = other
            .promoted()
            .map_or(u32::MAX, |promoted| promoted.as_u32());
        def_id_str_1.split('~').collect::<Vec<_>>()[1]
            == def_id_str_2.split('~').collect::<Vec<_>>()[1]
            && promoted_1 == promoted_2
    }
}

impl RLGraphNode for RLNode {
    fn create(def_id: DefId, promoted: Option<Promoted>) -> Self {
        let def_id_str = match def_id {
            DefId {
                krate: DUMMY_CRATE_NUM,
                index: DUMMY_DEF_INDEX,
            } => "STATICALLY_UNKNOWN".to_string(),
            _ => format!("{:?}", def_id),
        };
        Self {
            def_id,
            promoted,
            def_id_str,
        }
    }

    fn def_id_str(&self) -> String {
        self.def_id_str.clone()
    }

    fn promoted(&self) -> Option<Promoted> {
        self.promoted
    }
}

impl Serialize for RLNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let promoted = match self.promoted {
            Some(promoted) => promoted.as_u32(),
            None => u32::MAX,
        };
        let binding = self.def_id.index.as_u32();
        let index = match self.def_id.index {
            DUMMY_DEF_INDEX => "STATICALLY_UNKNOWN",
            _ => &binding.to_string(),
        };
        serializer.serialize_str(&format!(
            "{}:{}:{}:{}",
            self.def_id.krate.as_u32(),
            index,
            promoted,
            self.def_id_str,
        ))
    }
}

impl<'de> Deserialize<'de> for RLNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(':').collect();
        let krate = parts[0].parse().unwrap();
        let index = match parts[1] {
            "STATICALLY_UNKNOWN" => DUMMY_DEF_INDEX,
            _ => DefIndex::from_u32(parts[1].parse().unwrap()),
        };
        let promoted = match parts[2] {
            "4294967295" /* u32::MAX */ => None,
            _ => Some(Promoted::from_u32(parts[2].parse().unwrap())),
        };
        // We need to join the rest of the parts because the def_id_str can contain ':' characters.
        // And remove the last character because it is a trailing '"' character.
        let def_id_str = parts[3..].join(":");
        Ok(Self {
            def_id: DefId {
                krate: CrateNum::from_u32(krate),
                index,
            },
            promoted,
            def_id_str,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RLEdge {
    // It represents the kind of the call and the multiplier of the call.
    // The multiplier is used to calculate the total weight of the edge.
    // A call can be a static call, a method call, a function call, etc.
    call_multiplier: CallKind,
    // It represents the kind of the arguments.
    arg_weights: Vec<(OperandKind, MutabilityKind, RLTyKind)>,
}

impl RLGraphEdge for RLEdge {
    fn create(edge: (CallKind, Vec<(OperandKind, MutabilityKind, RLTyKind)>)) -> Self {
        let (call_multiplier, arg_weights) = edge;
        Self {
            call_multiplier,
            arg_weights,
        }
    }
}

#[derive(
    Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Copy, Clone, Serialize, Deserialize,
)]
pub struct RLIndex {
    index: usize,
}

impl RLGraphIndex for RLIndex {
    fn create(index: usize) -> Self {
        Self { index }
    }

    fn value(&self) -> usize {
        self.index
    }
}
