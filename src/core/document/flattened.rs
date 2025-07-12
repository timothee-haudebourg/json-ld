use crate::IndexedNode;
use std::collections::HashSet;

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub type FlattenedDocument = Vec<IndexedNode>;

// impl IdentifyAll for FlattenedDocument {
// 	#[inline(always)]
// 	fn identify_all_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
// 		&mut self,
// 		vocabulary: &mut V,
// 		generator: &mut G,
// 	) where
// 		T: Eq + Hash,
// 		B: Eq + Hash,
// 	{
// 		for node in self {
// 			node.identify_all_with(vocabulary, generator)
// 		}
// 	}
// }

// impl Relabel for FlattenedDocument {
// 	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: Generator<N>>(
// 		&mut self,
// 		vocabulary: &mut N,
// 		generator: &mut G,
// 		relabeling: &mut hashbrown::HashMap<B, rdf_types::Subject>,
// 	) where
// 		T: Clone + Eq + Hash,
// 		B: Clone + Eq + Hash,
// 	{
// 		for node in self {
// 			node.relabel_with(vocabulary, generator, relabeling)
// 		}
// 	}
// }

pub type UnorderedFlattenedDocument = HashSet<IndexedNode>;
