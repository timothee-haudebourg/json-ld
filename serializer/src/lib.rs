use std::hash::Hash;

use json_ld_core::ExpandedDocument;

use linked_data::{rdf_types::Vocabulary, LinkedData, RdfLiteralValue};
use rdf_types::{
    interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
    Interpretation, IriVocabularyMut, ReverseLiteralInterpretation,
};

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
pub fn serialize(value: &impl LinkedData) -> Result<ExpandedDocument, Error> {
    serialize_with(&mut (), &mut (), value)
}

/// Serialize the given Linked-Data value into a JSON-LD document using a
/// custom vocabulary and interpretation.
pub fn serialize_with<V: Vocabulary, I: Interpretation>(
    vocabulary: &mut V,
    interpretation: &mut I,
    value: &impl LinkedData<V, I>,
) -> Result<ExpandedDocument<V::Iri, V::BlankId>, Error>
where
    V: IriVocabularyMut,
    V::Iri: Clone + Eq + Hash,
    V::BlankId: Clone + Eq + Hash,
    V::LanguageTag: Clone,
    V::Value: RdfLiteralValue<V>,
    I: ReverseIriInterpretation<Iri = V::Iri>
        + ReverseBlankIdInterpretation<BlankId = V::BlankId>
        + ReverseLiteralInterpretation<Literal = V::Literal>,
{
    let serializer = SerializeExpandedDocument::new(vocabulary, interpretation);

    value.visit(serializer)
}
