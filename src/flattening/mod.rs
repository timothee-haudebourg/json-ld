//! Flattening algorithm and related types.
use crate::{id, Id, Indexed, Node, Object, ExpandedDocument};
use generic_json::{JsonHash, JsonClone};
use std::collections::HashSet;

mod namespace;
mod node_map;

pub use namespace::Namespace;
pub use node_map::*;

impl<J: JsonHash + JsonClone, T: Id> ExpandedDocument<J, T> {
	pub fn flatten_ordered<G: id::Generator<T>>(self, generator: G) -> Result<Vec<Indexed<Node<J, T>>>, ConflictingIndexes<T>> {
		Ok(self.generate_node_map(generator)?.flatten_ordered())
	}

	pub fn flatten<G: id::Generator<T>>(self, generator: G) -> Result<HashSet<Indexed<Node<J, T>>>, ConflictingIndexes<T>> {
		Ok(self.generate_node_map(generator)?.flatten())
	}
}

impl<J: JsonHash, T: Id> NodeMap<J, T> {
	pub fn flatten_ordered(self) -> Vec<Indexed<Node<J, T>>> {
		let (mut default_graph, named_graphs) = self.into_parts();
		let mut named_graphs: Vec<_> = named_graphs.into_iter().collect();
		named_graphs.sort_by(|a, b| a.0.as_str().cmp(&b.0.as_str()));

		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).unwrap();
			let mut nodes: Vec<_> = graph.into_nodes().filter(|node| !node.is_empty() && node.id().is_some()).collect();
			nodes.sort_by(|a, b| a.id().unwrap().as_str().cmp(&b.id().unwrap().as_str()));
			entry.set_graph(Some(nodes.into_iter().map(|node| node.map_inner(Object::Node)).collect()));
		}

		let mut nodes: Vec<_> = default_graph.into_nodes().filter(|node| !node.is_empty() && node.id().is_some()).collect();
		nodes.sort_by(|a, b| a.id().unwrap().as_str().cmp(&b.id().unwrap().as_str()));
		nodes
	}

	pub fn flatten(self) -> HashSet<Indexed<Node<J, T>>> {
		let (mut default_graph, named_graphs) = self.into_parts();
	
		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).unwrap();
			entry.set_graph(Some(graph.into_nodes().filter(|node| !node.is_empty()).map(|node| node.map_inner(Object::Node)).collect()));
		}
	
		default_graph.into_nodes().filter(|node| !node.is_empty()).collect()
	}
}