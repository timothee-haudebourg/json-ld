use rdf_types::Vocabulary;

use crate::{id, IdentifyAll, IndexedNode, Relabel, StrippedIndexedNode};
use std::{collections::HashSet, hash::Hash};

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub type FlattenedDocument<T, B, M> = Vec<IndexedNode<T, B, M>>;

impl<T, B, M> IdentifyAll<T, B, M> for FlattenedDocument<T, B, M> {
	#[inline(always)]
	fn identify_all_with<V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		M: Clone,
		T: Eq + Hash,
		B: Eq + Hash,
	{
		for node in self {
			node.identify_all_with(vocabulary, generator)
		}
	}
}

impl<T, B, M> Relabel<T, B, M> for FlattenedDocument<T, B, M> {
	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: rdf_types::MetaGenerator<N, M>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
		relabeling: &mut hashbrown::HashMap<B, locspan::Meta<rdf_types::Subject<T, B>, M>>,
	) where
		M: Clone,
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		for node in self {
			node.relabel_with(vocabulary, generator, relabeling)
		}
	}
}

pub type UnorderedFlattenedDocument<T, B, M> = HashSet<StrippedIndexedNode<T, B, M>>;
