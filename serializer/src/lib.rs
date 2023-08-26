use std::hash::Hash;

use indexmap::IndexSet;
use json_ld_core::{object::Literal, ExpandedDocument, Indexed, LangString, Node, Object, Value};
use locspan::Meta;
use rdf_types::{literal, IriVocabularyMut, LanguageTagVocabulary, Term};
use serde_ld::{rdf_types::Vocabulary, LexicalRepresentation, RdfLiteral, SerializeLd};
use xsd_types::XsdDatatype;

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
