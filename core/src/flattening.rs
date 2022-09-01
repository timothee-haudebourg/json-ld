//! Flattening algorithm and related types.
use crate::flattened::UnorderedFlattenedDocument;
use crate::{
	id, ExpandedDocument, FlattenedDocument, IndexedNode, IndexedObject, Object,
	StrippedIndexedNode,
};
use json_ld_syntax::Entry;
use locspan::{Meta, Stripped};
use std::collections::HashSet;
use std::hash::Hash;

mod environment;
mod node_map;

pub use environment::Environment;
pub use node_map::*;

impl<T: Clone + Eq + Hash, B: Clone + Eq + Hash, M: Clone> ExpandedDocument<T, B, M> {
	pub fn flatten_in<N, G: id::Generator<T, B, M, N>>(
		self,
		vocabulary: &mut N,
		generator: G,
		ordered: bool,
	) -> Result<FlattenedDocument<T, B, M>, ConflictingIndexes<T, B, M>>
	where
		T: AsRef<str>,
		B: AsRef<str>,
	{
		Ok(self
			.generate_node_map_in(vocabulary, generator)?
			.flatten(ordered))
	}

	pub fn flatten_unordered_in<N, G: id::Generator<T, B, M, N>>(
		self,
		vocabulary: &mut N,
		generator: G,
	) -> Result<UnorderedFlattenedDocument<T, B, M>, ConflictingIndexes<T, B, M>> {
		Ok(self
			.generate_node_map_in(vocabulary, generator)?
			.flatten_unordered())
	}
}

fn filter_graph<T, B, M>(node: IndexedNode<T, B, M>) -> Option<IndexedNode<T, B, M>> {
	if node.index().is_none() && node.is_empty() {
		None
	} else {
		Some(node)
	}
}

fn filter_sub_graph<T, B, M>(
	Meta(mut node, meta): IndexedNode<T, B, M>,
) -> Option<IndexedObject<T, B, M>> {
	if node.index().is_none() && node.properties().is_empty() {
		None
	} else {
		node.set_graph(None);
		node.set_included(None);
		node.set_reverse_properties(None);
		Some(Meta(node.map_inner(Object::Node), meta))
	}
}

impl<T: Clone + Eq + Hash, B: Clone + Eq + Hash, M: Clone> NodeMap<T, B, M> {
	pub fn flatten(self, ordered: bool) -> Vec<IndexedNode<T, B, M>>
	where
		T: AsRef<str>,
		B: AsRef<str>,
	{
		let (mut default_graph, named_graphs) = self.into_parts();

		let mut named_graphs: Vec<_> = named_graphs.into_iter().collect();
		if ordered {
			named_graphs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
		}

		for (graph_id, graph) in named_graphs {
			let (id_metadata, graph) = graph.into_parts();
			let entry = default_graph
				.declare_node(Meta(graph_id, id_metadata.clone()), None)
				.ok()
				.unwrap();
			let mut nodes: Vec<_> = graph.into_nodes().collect();
			if ordered {
				nodes.sort_by(|a, b| {
					a.id_entry()
						.unwrap()
						.as_str()
						.cmp(b.id_entry().unwrap().as_str())
				});
			}
			entry.set_graph(Some(Entry::new(
				id_metadata.clone(),
				Meta(
					nodes
						.into_iter()
						.filter_map(filter_sub_graph)
						.map(Stripped)
						.collect(),
					id_metadata,
				),
			)));
		}

		let mut nodes: Vec<_> = default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.collect();

		if ordered {
			nodes.sort_by(|a, b| {
				a.id_entry()
					.unwrap()
					.as_str()
					.cmp(b.id_entry().unwrap().as_str())
			});
		}

		nodes
	}

	pub fn flatten_unordered(self) -> HashSet<StrippedIndexedNode<T, B, M>> {
		let (mut default_graph, named_graphs) = self.into_parts();

		for (graph_id, graph) in named_graphs {
			let (id_metadata, graph) = graph.into_parts();
			let entry = default_graph
				.declare_node(Meta(graph_id, id_metadata.clone()), None)
				.ok()
				.unwrap();
			entry.set_graph(Some(Entry::new(
				id_metadata.clone(),
				Meta(
					graph
						.into_nodes()
						.filter_map(filter_sub_graph)
						.map(Stripped)
						.collect(),
					id_metadata,
				),
			)));
		}

		default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.map(Stripped)
			.collect()
	}
}
