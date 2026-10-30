use crate::IndexedJsonLdNode;
use std::collections::HashSet;

/// Result of the document flattening algorithm.
///
/// It is just an alias for a set of (indexed) nodes.
pub type JsonLdFlattenedDocument = Vec<IndexedJsonLdNode>;

pub type JsonLdUnorderedFlattenedDocument = HashSet<IndexedJsonLdNode>;
