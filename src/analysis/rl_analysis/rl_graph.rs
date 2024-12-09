use rustc_hir::def_id::{CrateNum, DefIndex};
use rustc_middle::mir::Promoted;
use rustc_span::def_id::DefId;
use serde::{Deserialize, Serialize};

/// The `RLGraphEdge` trait represents an edge in a graph.
pub trait RLGraphEdge {
    fn create(_arg_weights: Vec<f32>) -> Self;
    fn calc_total_weight(arg_weights: &[f32]) -> f32;
    fn total_weight(&self) -> f32;
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
        let def_id_str_2 = other.def_id_str();
        def_id_str_1.split('~').collect::<Vec<_>>()[1]
            == def_id_str_2.split('~').collect::<Vec<_>>()[1]
    }
}

impl RLGraphNode for RLNode {
    fn create(def_id: DefId, promoted: Option<Promoted>) -> Self {
        let def_id_str = format!("{:?}", def_id);
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
        serializer.serialize_str(&format!(
            "{}:{}:{}:{}",
            self.def_id.krate.as_u32(),
            self.def_id.index.as_u32(),
            promoted,
            self.def_id_str
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
        let index = parts[1].parse().unwrap();
        let promoted = match parts[2] {
            "4294967295" => None,
            _ => Some(Promoted::from_u32(parts[2].parse().unwrap())),
        };
        // We need to join the rest of the parts because the def_id_str can contain ':' characters.
        // And remove the last character because it is a trailing '"' character.
        let def_id_str = parts[3..].join(":");
        Ok(Self {
            def_id: DefId {
                krate: CrateNum::from_u32(krate),
                index: DefIndex::from_u32(index),
            },
            promoted,
            def_id_str,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RLEdge {
    // It represents the weights of the arguments of the function call.
    // Each weight is associated with an argument, and it is calculated based on the
    // ownership semantics and the borrowing semantics of the argument.
    // The weights are used to calculate the total weight of the edge.
    //
    // Example:
    // ```rust,ignore
    // fn foo(a: i32, b: i32) {}
    //
    // fn main() {
    //     let x = 1;
    //     let y = 2;
    //     foo(x, y);
    // }
    // ```
    // The weights of the arguments in this example are both moved, so the total weight of the edge is 2.
    _arg_weights: Vec<f32>,
    total_weight: f32,
}

impl RLGraphEdge for RLEdge {
    fn create(_arg_weights: Vec<f32>) -> Self {
        let total_weight = Self::calc_total_weight(&_arg_weights);
        Self {
            _arg_weights,
            total_weight,
        }
    }

    fn calc_total_weight(arg_weights: &[f32]) -> f32 {
        // FIXME
        arg_weights.iter().sum()
    }

    fn total_weight(&self) -> f32 {
        self.total_weight
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
