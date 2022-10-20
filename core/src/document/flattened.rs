use rdf_types::Vocabulary;

use crate::{id, IdentifyAll, IndexedNode, StrippedIndexedNode};
use std::collections::HashSet;

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub type FlattenedDocument<T, B, M> = Vec<IndexedNode<T, B, M>>;

impl<T, B, M> IdentifyAll<T, B, M> for FlattenedDocument<T, B, M> {
	#[inline(always)]
	fn identify_all_with<V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&mut self,
		vocabulary: &mut V,
		mut generator: G,
	) where
		M: Clone,
	{
		for node in self {
			node.identify_all_with(vocabulary, &mut generator)
		}
	}

	#[inline(always)]
	fn identify_all<G: id::Generator<(), M>>(&mut self, generator: G)
	where
		M: Clone,
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.identify_all_with(&mut (), generator)
	}
}

pub type UnorderedFlattenedDocument<T, B, M> = HashSet<StrippedIndexedNode<T, B, M>>;
