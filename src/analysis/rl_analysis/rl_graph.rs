use rustc_span::def_id::DefId;

// FIXME
#[allow(dead_code)]
/// The `RLGraphEdge` trait represents an edge in a graph.
pub trait RLGraphEdge {
    fn weight(&self) -> f32;
    fn set_weight(&mut self, weight: f32);
}

// FIXME
#[allow(dead_code)]
/// The `RLGraphNode` trait represents a node in a graph.
pub trait RLGraphNode {}

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

    pub fn def_id(&self) -> DefId {
        self.def_id
    }
}

#[derive(Debug, Clone)]
pub struct RLEdge {
    weight: f32,
}

impl RLEdge {
    pub fn new(weight: f32) -> Self {
        Self { weight }
    }

    pub fn weight(&self) -> f32 {
        self.weight
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
    fn weight(&self) -> f32 {
        self.weight
    }

    fn set_weight(&mut self, weight: f32) {
        self.weight = weight;
    }
}

impl RLGraphNode for RLNode {}

impl RLGraphIndex for RLIndex {}
