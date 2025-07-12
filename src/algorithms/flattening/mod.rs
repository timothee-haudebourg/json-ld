//! Flattening algorithm and related types.
use crate::flattened::UnorderedFlattenedDocument;
use crate::{ExpandedDocument, FlattenedDocument, IndexedNode, IndexedObject, Object};
use contextual::WithContext;
use rdf_types::{Generator, Vocabulary};
use std::collections::HashSet;
use std::hash::Hash;

mod environment;
mod node_map;

pub use environment::Environment;
pub use node_map::*;

pub type FlattenResult<I, B> = Result<FlattenedDocument<I, B>, ConflictingIndexes<I, B>>;

pub type FlattenUnorderedResult<I, B> =
	Result<UnorderedFlattenedDocument<I, B>, ConflictingIndexes<I, B>>;

pub trait Flatten<I, B> {
	fn flatten_with<V, G: Generator<V>>(
		self,
		vocabulary: &mut V,
		generator: G,
		ordered: bool,
	) -> FlattenResult<I, B>
	where
		V: Vocabulary<Iri = I, BlankId = B>;

	fn flatten_unordered_with<V, G: Generator<V>>(
		self,
		vocabulary: &mut V,
		generator: G,
	) -> FlattenUnorderedResult<I, B>
	where
		V: Vocabulary<Iri = I, BlankId = B>;

	fn flatten<G: Generator>(self, generator: G, ordered: bool) -> FlattenResult<I, B>
	where
		(): Vocabulary<Iri = I, BlankId = B>,
		Self: Sized,
	{
		self.flatten_with(
			rdf_types::vocabulary::no_vocabulary_mut(),
			generator,
			ordered,
		)
	}

	fn flatten_unordered<G: Generator>(self, generator: G) -> FlattenUnorderedResult<I, B>
	where
		(): Vocabulary<Iri = I, BlankId = B>,
		Self: Sized,
	{
		self.flatten_unordered_with(rdf_types::vocabulary::no_vocabulary_mut(), generator)
	}
}

impl<I: Clone + Eq + Hash, B: Clone + Eq + Hash> Flatten<I, B> for ExpandedDocument<I, B> {
	fn flatten_with<V, G: Generator<V>>(
		self,
		vocabulary: &mut V,
		generator: G,
		ordered: bool,
	) -> FlattenResult<I, B>
	where
		V: Vocabulary<Iri = I, BlankId = B>,
	{
		Ok(self
			.generate_node_map_with(vocabulary, generator)?
			.flatten_with(vocabulary, ordered))
	}

	fn flatten_unordered_with<V, G: Generator<V>>(
		self,
		vocabulary: &mut V,
		generator: G,
	) -> FlattenUnorderedResult<I, B>
	where
		V: Vocabulary<Iri = I, BlankId = B>,
	{
		Ok(self
			.generate_node_map_with(vocabulary, generator)?
			.flatten_unordered())
	}
}

fn filter_graph<T, B>(node: IndexedNode<T, B>) -> Option<IndexedNode<T, B>> {
	if node.index().is_none() && node.is_empty() {
		None
	} else {
		Some(node)
	}
}

fn filter_sub_graph<T, B>(mut node: IndexedNode<T, B>) -> Option<IndexedObject<T, B>> {
	if node.index().is_none() && node.properties().is_empty() {
		None
	} else {
		node.set_graph_entry(None);
		node.set_included(None);
		node.set_reverse_properties(None);
		Some(node.map_inner(Object::node))
	}
}

impl<T: Clone + Eq + Hash, B: Clone + Eq + Hash> NodeMap<T, B> {
	pub fn flatten(self, ordered: bool) -> FlattenedDocument<T, B>
	where
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.flatten_with(&(), ordered)
	}

	pub fn flatten_with<V>(self, vocabulary: &V, ordered: bool) -> FlattenedDocument<T, B>
	where
		V: Vocabulary<Iri = T, BlankId = B>,
	{
		let (mut default_graph, named_graphs) = self.into_parts();

		let mut named_graphs: Vec<_> = named_graphs.into_iter().collect();
		if ordered {
			named_graphs.sort_by(|a, b| {
				a.0.with(vocabulary)
					.as_str()
					.cmp(b.0.with(vocabulary).as_str())
			});
		}

		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).ok().unwrap();
			let mut nodes: Vec<_> = graph.into_nodes().collect();
			if ordered {
				nodes.sort_by(|a, b| {
					a.id.as_ref()
						.unwrap()
						.with(vocabulary)
						.as_str()
						.cmp(b.id.as_ref().unwrap().with(vocabulary).as_str())
				});
			}
			entry.set_graph_entry(Some(
				nodes.into_iter().filter_map(filter_sub_graph).collect(),
			));
		}

		let mut nodes: Vec<_> = default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.collect();

		if ordered {
			nodes.sort_by(|a, b| {
				a.id.as_ref()
					.unwrap()
					.with(vocabulary)
					.as_str()
					.cmp(b.id.as_ref().unwrap().with(vocabulary).as_str())
			});
		}

		nodes
	}

	pub fn flatten_unordered(self) -> HashSet<IndexedNode<T, B>> {
		let (mut default_graph, named_graphs) = self.into_parts();

		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).ok().unwrap();
			entry.set_graph_entry(Some(
				graph.into_nodes().filter_map(filter_sub_graph).collect(),
			));
		}

		default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.collect()
	}
}
