//! Flattening algorithm and related types.
use crate::{id, ExpandedDocument, FlattenedDocument, Id, Indexed, Node, StrippedIndexedNode, Object};
use locspan::Stripped;
use std::collections::HashSet;

mod namespace;
mod node_map;

pub use namespace::Namespace;
pub use node_map::*;

impl<T: Id, M: Clone> ExpandedDocument<T, M> {
	pub fn flatten<G: id::Generator<T>>(
		self,
		generator: G,
		ordered: bool,
	) -> Result<FlattenedDocument<T, M>, ConflictingIndexes<T>> {
		let nodes = self.generate_node_map(generator)?.flatten(ordered);
		Ok(FlattenedDocument::new(nodes))
	}

	pub fn flatten_unordered<G: id::Generator<T>>(
		self,
		generator: G,
	) -> Result<HashSet<StrippedIndexedNode<T, M>>, ConflictingIndexes<T>> {
		Ok(self.generate_node_map(generator)?.flatten_unordered())
	}
}

fn filter_graph<T: Id, M>(node: Indexed<Node<T, M>>) -> Option<Indexed<Node<T, M>>> {
	if node.index().is_none() && node.is_empty() {
		None
	} else {
		Some(node)
	}
}

fn filter_sub_graph<T: Id, M>(
	mut node: Indexed<Node<T, M>>,
) -> Option<Indexed<Object<T, M>>> {
	if node.index().is_none() && node.properties().is_empty() {
		None
	} else {
		node.set_graph(None);
		node.set_included(None);
		node.reverse_properties_mut().clear();
		Some(node.map_inner(Object::Node))
	}
}

impl<T: Id, M> NodeMap<T, M> {
	pub fn flatten(self, ordered: bool) -> Vec<Indexed<Node<T, M>>> {
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
				nodes.into_iter().filter_map(filter_sub_graph).map(Stripped).collect(),
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

	pub fn flatten_unordered(self) -> HashSet<Stripped<Indexed<Node<T, M>>>> {
		let (mut default_graph, named_graphs) = self.into_parts();

		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).unwrap();
			entry.set_graph(Some(
				graph.into_nodes().filter_map(filter_sub_graph).map(Stripped).collect(),
			));
		}

		default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.map(Stripped)
			.collect()
	}
}
