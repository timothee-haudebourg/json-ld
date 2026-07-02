use crate::IndexedNode;
use std::collections::HashSet;

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub type JsonLdFlattenedDocument = Vec<IndexedNode>;

pub type JsonLdUnorderedFlattenedDocument = HashSet<IndexedNode>;
