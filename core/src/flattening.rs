//! Flattening algorithm and related types.
use crate::flattened::UnorderedFlattenedDocument;
use crate::{
	id, ExpandedDocument, FlattenedDocument, IndexedNode, IndexedObject, Object,
	StrippedIndexedNode,
};
use contextual::WithContext;
use json_ld_syntax::Entry;
use locspan::{Meta, Stripped};
use rdf_types::Vocabulary;
use std::collections::HashSet;
use std::hash::Hash;

mod environment;
mod node_map;

pub use environment::Environment;
pub use node_map::*;

pub type FlattenResult<I, B, M> =
	Result<Meta<FlattenedDocument<I, B, M>, M>, ConflictingIndexes<I, B, M>>;

pub type FlattenUnorderedResult<I, B, M> =
	Result<Meta<UnorderedFlattenedDocument<I, B, M>, M>, ConflictingIndexes<I, B, M>>;

pub trait FlattenMeta<I, B, M> {
	fn flatten_meta<N, G: id::Generator<I, B, M, N>>(
		self,
		meta: M,
		vocabulary: &mut N,
		generator: G,
		ordered: bool,
	) -> FlattenResult<I, B, M>
	where
		N: Vocabulary<Iri = I, BlankId = B>;

	fn flatten_unordered_meta<N, G: id::Generator<I, B, M, N>>(
		self,
		meta: M,
		vocabulary: &mut N,
		generator: G,
	) -> FlattenUnorderedResult<I, B, M>;
}

pub trait Flatten<I, B, M> {
	fn flatten_with<N, G: id::Generator<I, B, M, N>>(
		self,
		vocabulary: &mut N,
		generator: G,
		ordered: bool,
	) -> FlattenResult<I, B, M>
	where
		N: Vocabulary<Iri = I, BlankId = B>;

	fn flatten_unordered_with<N, G: id::Generator<I, B, M, N>>(
		self,
		vocabulary: &mut N,
		generator: G,
	) -> FlattenUnorderedResult<I, B, M>;
}

impl<T: FlattenMeta<I, B, M>, I, B, M> Flatten<I, B, M> for Meta<T, M> {
	fn flatten_with<N, G: id::Generator<I, B, M, N>>(
		self,
		vocabulary: &mut N,
		generator: G,
		ordered: bool,
	) -> FlattenResult<I, B, M>
	where
		N: Vocabulary<Iri = I, BlankId = B>,
	{
		T::flatten_meta(self.0, self.1, vocabulary, generator, ordered)
	}

	fn flatten_unordered_with<N, G: id::Generator<I, B, M, N>>(
		self,
		vocabulary: &mut N,
		generator: G,
	) -> FlattenUnorderedResult<I, B, M> {
		T::flatten_unordered_meta(self.0, self.1, vocabulary, generator)
	}
}

impl<I: Clone + Eq + Hash, B: Clone + Eq + Hash, M: Clone> FlattenMeta<I, B, M>
	for ExpandedDocument<I, B, M>
{
	fn flatten_meta<N, G: id::Generator<I, B, M, N>>(
		self,
		meta: M,
		vocabulary: &mut N,
		generator: G,
		ordered: bool,
	) -> FlattenResult<I, B, M>
	where
		N: Vocabulary<Iri = I, BlankId = B>,
	{
		Ok(Meta(
			self.generate_node_map_with(vocabulary, generator)?
				.flatten_with(vocabulary, ordered),
			meta,
		))
	}

	fn flatten_unordered_meta<N, G: id::Generator<I, B, M, N>>(
		self,
		meta: M,
		vocabulary: &mut N,
		generator: G,
	) -> FlattenUnorderedResult<I, B, M> {
		Ok(Meta(
			self.generate_node_map_with(vocabulary, generator)?
				.flatten_unordered(),
			meta,
		))
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
		Some(Meta(node.map_inner(Object::node), meta))
	}
}

impl<T: Clone + Eq + Hash, B: Clone + Eq + Hash, M: Clone> NodeMap<T, B, M> {
	pub fn flatten(self, ordered: bool) -> Vec<IndexedNode<T, B, M>>
	where
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.flatten_with(&(), ordered)
	}

	pub fn flatten_with<N>(self, vocabulary: &N, ordered: bool) -> Vec<IndexedNode<T, B, M>>
	where
		N: Vocabulary<Iri = T, BlankId = B>,
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
			let (id_metadata, graph) = graph.into_parts();
			let entry = default_graph
				.declare_node(Meta(graph_id, id_metadata.clone()), None)
				.ok()
				.unwrap();
			let mut nodes: Vec<_> = graph.into_nodes().collect();
			if ordered {
				nodes.sort_by(|a, b| {
					a.id()
						.unwrap()
						.0
						.with(vocabulary)
						.as_str()
						.cmp(b.id().unwrap().0.with(vocabulary).as_str())
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
				a.id()
					.unwrap()
					.0
					.with(vocabulary)
					.as_str()
					.cmp(b.id().unwrap().0.with(vocabulary).as_str())
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
