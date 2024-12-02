use super::rl_graph::RLEdge;
use super::rl_graph::RLGraph;
use super::rl_graph::RLGraphEdge;
use super::rl_graph::RLGraphIndex;
use super::rl_graph::RLGraphNode;
use super::rl_graph::RLIndex;
use super::rl_graph::RLNode;

use petgraph::visit::EdgeRef;
use rustworkx_core::petgraph::csr::IndexType;
use rustworkx_core::petgraph::graph;

unsafe impl IndexType for RLIndex {
    fn new(value: usize) -> Self {
        RLIndex::create(value)
    }

    fn index(&self) -> usize {
        self.value()
    }

    fn max() -> Self {
        Self::create(usize::MAX)
    }
}

impl From<graph::NodeIndex> for RLIndex {
    fn from(node_index: graph::NodeIndex) -> Self {
        Self::create(node_index.index())
    }
}

impl From<graph::NodeIndex<RLIndex>> for RLIndex {
    fn from(value: graph::NodeIndex<RLIndex>) -> Self {
        Self::create(value.index())
    }
}

impl RLGraph for graph::DiGraph<RLNode, RLEdge, RLIndex> {
    type Node = RLNode;
    type Edge = RLEdge;
    type Index = RLIndex;

    fn rl_add_node(&mut self, node: Self::Node) -> Self::Index {
        Self::Index::create(self.add_node(node).index())
    }

    fn rl_add_edge(&mut self, source: Self::Index, target: Self::Index, edge: Self::Edge) {
        self.add_edge(source.into(), target.into(), edge);
    }

    fn merge(&mut self, other: &Self) {
        for node in other.node_indices() {
            let node = other.node_weight(node).unwrap().clone();
            if !self.node_indices().any(|n| self[n] == node) {
                let _index = self.rl_add_node(node);
            }
        }

        for edge in other.edge_references() {
            // We can not do something like `let source = edge.source()`
            // because in all graphs the nodes start from 0, so we need to find the
            // correct node in the current (merged) graph.
            let source = self
                .node_indices()
                .find(|n| self[*n] == other[edge.source()])
                .unwrap();
            let target = self
                .node_indices()
                .find(|n| self[*n] == other[edge.target()])
                .unwrap();
            let edge = edge.weight().clone();
            self.rl_add_edge(source.into(), target.into(), edge);
        }
    }

    fn print_dot(&self) {
        use rustworkx_core::petgraph::dot::{Config, Dot};

        let get_node_attr =
            |_g: &&graph::DiGraph<RLNode, RLEdge, RLIndex>,
             node: (graph::NodeIndex<RLIndex>, &RLNode)| {
                let index = node.0.index();
                let node = node.1;
                let promoted = match node.promoted() {
                    Some(promoted) => format!("{:?}", promoted),
                    None => "None".to_string(),
                };
                format!("label=\"i{}: {} - {}\"", index, node.def_id_str(), promoted)
                // format!("label=\"i{}: {:?}\"", index, node.def_id())
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
