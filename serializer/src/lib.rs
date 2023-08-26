use std::hash::Hash;

use json_ld_core::ExpandedDocument;

use rdf_types::IriVocabularyMut;
use serde_ld::{rdf_types::Vocabulary, SerializeLd};

mod expanded;

use expanded::SerializeExpandedDocument;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid graph label")]
    InvalidGraph,

    #[error("invalid predicate")]
    InvalidPredicate,
}

/// Serialize the given Linked-Data value into a JSON-LD document.
pub fn serialize(value: &impl SerializeLd) -> Result<ExpandedDocument, Error> {
    serialize_with(&mut (), &mut (), value)
}

/// Serialize the given Linked-Data value into a JSON-LD document.
pub fn serialize_with<V: Vocabulary, I>(
    vocabulary: &mut V,
    interpretation: &mut I,
    value: &impl SerializeLd<V, I>,
) -> Result<ExpandedDocument<V::Iri, V::BlankId>, Error>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    let serializer = SerializeExpandedDocument::new(vocabulary, interpretation);

    value.serialize(serializer)
}
