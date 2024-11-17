use super::rl_graph::RLEdge;
use super::rl_graph::RLGraph;
use super::rl_graph::RLGraphEdge;
use super::rl_graph::RLGraphNode;
use super::rl_graph::RLIndex;
use super::rl_graph::RLNode;

use rustworkx_core::petgraph::csr::IndexType;
use rustworkx_core::petgraph::graph;

unsafe impl IndexType for RLIndex {
    fn new(value: usize) -> Self {
        RLIndex::new(value)
    }

    fn index(&self) -> usize {
        self.value()
    }

    fn max() -> Self {
        Self::new(usize::MAX)
    }
}

impl From<graph::NodeIndex> for RLIndex {
    fn from(node_index: graph::NodeIndex) -> Self {
        Self::new(node_index.index())
    }
}

impl RLGraph for graph::DiGraph<RLNode, RLEdge, RLIndex> {
    type Node = RLNode;
    type Edge = RLEdge;
    type Index = RLIndex;

    fn rl_add_node(&mut self, node: Self::Node) -> Self::Index {
        Self::Index::new(self.add_node(node).index())
    }

    fn rl_add_edge(&mut self, source: Self::Index, target: Self::Index, edge: Self::Edge) {
        self.add_edge(source.into(), target.into(), edge);
    }

    fn print_dot(&self) {
        use rustworkx_core::petgraph::dot::{Config, Dot};

        let get_node_attr =
            |_g: &&graph::DiGraph<RLNode, RLEdge, RLIndex>,
             node: (graph::NodeIndex<RLIndex>, &RLNode)| {
                let index = node.0.index();
                let node = node.1;
                format!("label=\"i{}: {:?}\"", index, node.def_id())
            };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &|_g, e| format!("label=\"{:.2}\"", e.weight().total_weight()),
                &get_node_attr,
            )
        )
    }
}
