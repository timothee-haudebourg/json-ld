use rdf_types::{Generator, Vocabulary};

use crate::{IdentifyAll, IndexedNode, Relabel};
use std::{collections::HashSet, hash::Hash};

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub type FlattenedDocument<T, B> = Vec<IndexedNode<T, B>>;

impl<T, B> IdentifyAll<T, B> for FlattenedDocument<T, B> {
	#[inline(always)]
	fn identify_all_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		T: Eq + Hash,
		B: Eq + Hash,
	{
		for node in self {
			node.identify_all_with(vocabulary, generator)
		}
	}
}

impl<T, B> Relabel<T, B> for FlattenedDocument<T, B> {
	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: Generator<N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
		relabeling: &mut hashbrown::HashMap<B, rdf_types::Subject<T, B>>,
	) where
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		for node in self {
			node.relabel_with(vocabulary, generator, relabeling)
		}
	}
}

pub type UnorderedFlattenedDocument<T, B> = HashSet<IndexedNode<T, B>>;
