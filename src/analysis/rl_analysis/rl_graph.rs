use rustc_span::def_id::DefId;

// FIXME
#[allow(dead_code)]
/// The `RLGraphEdge` trait represents an edge in a graph.
pub trait RLGraphEdge {
    fn total_weight(&self) -> f32;
}

/// The `RLGraphNode` trait represents a node in a graph.
pub trait RLGraphNode {
    fn def_id(&self) -> DefId;
}

// FIXME
#[allow(dead_code)]
/// The `RLGraphIndex` trait represents the index of a node in a graph.
pub trait RLGraphIndex {}

/// The `RLGraph` trait represents a graph where the nodes are of type `RLGraphNode`,
/// the edges are of type `RLGraphEdge`, and the indices are of type `RLGraphIndex`.
/// The graph is mutable, and it is possible to add nodes and edges to it.
pub trait RLGraph {
    type Node: RLGraphNode;
    type Edge: RLGraphEdge;
    type Index: RLGraphIndex;

    fn rl_add_node(&mut self, node: Self::Node) -> Self::Index;
    fn rl_add_edge(&mut self, source: Self::Index, target: Self::Index, edge: Self::Edge);
    fn print_dot(&self);
}

#[derive(Debug, Clone)]
pub struct RLNode {
    def_id: DefId,
}

impl RLNode {
    pub fn new(def_id: DefId) -> Self {
        Self { def_id }
    }
}

#[derive(Debug, Clone)]
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

impl RLEdge {
    pub fn new(_arg_weights: Vec<f32>) -> Self {
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
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Copy, Clone)]
pub struct RLIndex {
    value: usize,
}

impl RLIndex {
    pub fn new(value: usize) -> Self {
        Self { value }
    }

    pub fn value(&self) -> usize {
        self.value
    }
}

impl RLGraphEdge for RLEdge {
    fn total_weight(&self) -> f32 {
        self.total_weight
    }
}

impl RLGraphNode for RLNode {
    fn def_id(&self) -> DefId {
        self.def_id
    }
}

impl RLGraphIndex for RLIndex {}
