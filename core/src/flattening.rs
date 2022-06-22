//! Flattening algorithm and related types.
use crate::{id, ExpandedDocument, FlattenedDocument, Id, Indexed, Node, Object};
use generic_json::{JsonClone, JsonHash};
use std::collections::HashSet;

mod namespace;
mod node_map;

pub use namespace::Namespace;
pub use node_map::*;

impl<F, J: JsonHash + JsonClone, T: Id> ExpandedDocument<F, J, T> {
	pub fn flatten<G: id::Generator<T>>(
		self,
		generator: G,
		ordered: bool,
	) -> Result<FlattenedDocument<F, J, T>, ConflictingIndexes<T>> {
		let nodes = self.generate_node_map(generator)?.flatten(ordered);
		Ok(FlattenedDocument::new(nodes, self.into_warnings()))
	}

	pub fn flatten_unordered<G: id::Generator<T>>(
		self,
		generator: G,
	) -> Result<HashSet<Indexed<Node<J, T>>>, ConflictingIndexes<T>> {
		Ok(self.generate_node_map(generator)?.flatten_unordered())
	}
}

fn filter_graph<J: JsonHash, T: Id>(node: Indexed<Node<J, T>>) -> Option<Indexed<Node<J, T>>> {
	if node.index().is_none() && node.is_empty() {
		None
	} else {
		Some(node)
	}
}

fn filter_sub_graph<J: JsonHash, T: Id>(
	mut node: Indexed<Node<J, T>>,
) -> Option<Indexed<Object<J, T>>> {
	if node.index().is_none() && node.properties().is_empty() {
		None
	} else {
		node.set_graph(None);
		node.set_included(None);
		node.reverse_properties_mut().clear();
		Some(node.map_inner(Object::Node))
	}
}

impl<J: JsonHash, T: Id> NodeMap<J, T> {
	pub fn flatten(self, ordered: bool) -> Vec<Indexed<Node<J, T>>> {
		let (mut default_graph, named_graphs) = self.into_parts();

		let mut named_graphs: Vec<_> = named_graphs.into_iter().collect();
		if ordered {
			named_graphs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
		}

		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).unwrap();
			let mut nodes: Vec<_> = graph.into_nodes().collect();
			if ordered {
				nodes.sort_by(|a, b| a.id().unwrap().as_str().cmp(b.id().unwrap().as_str()));
			}
			entry.set_graph(Some(
				nodes.into_iter().filter_map(filter_sub_graph).collect(),
			));
		}

		let mut nodes: Vec<_> = default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.collect();

		if ordered {
			nodes.sort_by(|a, b| a.id().unwrap().as_str().cmp(b.id().unwrap().as_str()));
		}

		nodes
	}

	pub fn flatten_unordered(self) -> HashSet<Indexed<Node<J, T>>> {
		let (mut default_graph, named_graphs) = self.into_parts();

		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).unwrap();
			entry.set_graph(Some(
				graph.into_nodes().filter_map(filter_sub_graph).collect(),
			));
		}

		default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.collect()
	}
}
