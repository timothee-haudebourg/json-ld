use std::hash::Hash;

use json_ld_core::{ExpandedDocument, Node, Object};

use linked_data::{rdf_types::Vocabulary, LinkedData, LinkedDataResource, LinkedDataSubject};
use rdf_types::{
	interpretation::{
		ReverseBlankIdInterpretation, ReverseIriInterpretation, ReverseLiteralInterpretation,
	},
	vocabulary::IriVocabularyMut,
	Interpretation,
};

mod expanded;

use expanded::SerializeExpandedDocument;

pub use expanded::{serialize_node_with, serialize_object_with};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("invalid graph label")]
	InvalidGraph,

	#[error("invalid predicate")]
	InvalidPredicate,

	#[error("invalid node object")]
	InvalidNode,

	#[error("reverse properties on lists are not supported")]
	ListReverseProperty,

	#[error("included nodes on lists are not supported")]
	ListInclude,
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
	value: &impl LinkedData<I, V>,
) -> Result<ExpandedDocument<V::Iri, V::BlankId>, Error>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	let serializer = SerializeExpandedDocument::new(vocabulary, interpretation);

	value.visit(serializer)
}

/// Serialize the given Linked-Data value into a JSON-LD object.
pub fn serialize_object(
	value: &(impl LinkedDataSubject + LinkedDataResource),
) -> Result<Object, Error> {
	serialize_object_with(&mut (), &mut (), value)
}

/// Serialize the given Linked-Data value into a JSON-LD node object.
pub fn serialize_node(
	value: &(impl LinkedDataSubject + LinkedDataResource),
) -> Result<Node, Error> {
	serialize_node_with(&mut (), &mut (), value)
}
