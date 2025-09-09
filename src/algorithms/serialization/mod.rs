//! This crate implements JSON-LD serialization (from RDF dataset to JSON-LD)
//! through the [`linked_data`](https://github.com/spruceid/linked-data-rs)
//! crate.
//! The input value can be an RDF dataset, or any type implementing
//! [`linked_data::LinkedData`].
use linked_data::{
	de::LinkedDataSubjectDeserializer, DeserializeLinkedData, LinkedDataDeserializer,
};
use rdf_types::{
	dataset::IndexedBTreeDataset, pattern::CanonicalQuadPattern, Quad, Term, RDF_FIRST,
};

use crate::{ExpandedDocument, Indexed};

mod graph;
mod object;

use graph::ProtoGraph;
use object::ProtoObject;

impl DeserializeLinkedData for ExpandedDocument {
	fn deserialize_rdf<D>(mut deserializer: D, _: Option<&Term>) -> Result<Self, D::Error>
	where
		D: LinkedDataDeserializer<Term>,
	{
		let mut result = Self::new();

		// Unused quads.
		// let mut unused: IndexedBTreeDataset = IndexedBTreeDataset::new();

		while let Some(Quad(_, _, _, g)) = deserializer.peek_quad(CanonicalQuadPattern::ANY)? {
			let g = g.cloned();
			let graph: ProtoGraph =
				DeserializeLinkedData::deserialize_rdf(&mut deserializer, g.as_ref())?;
		}

		// unused.extract_pattern_matching(pattern);

		// for subject in subjects {
		// 	result.insert(DeserializeLinkedData::deserialize_rdf(
		// 		ObjectDeserializer {
		// 			deserializer: &mut deserializer,
		// 			subject: Some(subject),
		// 		},
		// 		None,
		// 	)?);
		// }

		Ok(result)
	}
}

impl<T: DeserializeLinkedData> DeserializeLinkedData for Indexed<T> {
	fn deserialize_rdf<D>(deserializer: D, graph: Option<&Term>) -> Result<Self, D::Error>
	where
		D: LinkedDataDeserializer<Term>,
	{
		T::deserialize_rdf(deserializer, graph).map(Self::unindexed)
	}
}

// use std::hash::Hash;

// use json_ld_core::{ExpandedDocument, Node, Object};

// use linked_data::{rdf_types::Vocabulary, LinkedData, LinkedDataResource, LinkedDataSubject};
// use rdf_types::{
// 	interpretation::{
// 		ReverseBlankIdInterpretation, ReverseIriInterpretation, ReverseLiteralInterpretation,
// 	},
// 	vocabulary::IriVocabularyMut,
// 	Interpretation,
// };

// mod expanded;

// use expanded::SerializeExpandedDocument;

// pub use expanded::{serialize_node_with, serialize_object_with};

// #[derive(Debug, thiserror::Error)]
// pub enum Error {
// 	#[error("invalid graph label")]
// 	InvalidGraph,

// 	#[error("invalid predicate")]
// 	InvalidPredicate,

// 	#[error("invalid node object")]
// 	InvalidNode,

// 	#[error("reverse properties on lists are not supported")]
// 	ListReverseProperty,

// 	#[error("included nodes on lists are not supported")]
// 	ListInclude,
// }

// /// Serialize the given Linked-Data value into a JSON-LD document.
// pub fn serialize(value: &impl LinkedData) -> Result<ExpandedDocument, Error> {
// 	serialize_with(&mut (), &mut (), value)
// }

// /// Serialize the given Linked-Data value into a JSON-LD document using a
// /// custom vocabulary and interpretation.
// pub fn serialize_with<V, I>(
// 	vocabulary: &mut V,
// 	interpretation: &mut I,
// 	value: &impl LinkedData<I, V>,
// ) -> Result<ExpandedDocument<V::Iri, V::BlankId>, Error>
// where
// 	V: Vocabulary + IriVocabularyMut,
// 	V::Iri: Clone + Eq + Hash,
// 	V::BlankId: Clone + Eq + Hash,
// 	I: Interpretation
// 		+ ReverseIriInterpretation<Iri = V::Iri>
// 		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
// 		+ ReverseLiteralInterpretation<Literal = V::Literal>,
// {
// 	let serializer = SerializeExpandedDocument::new(vocabulary, interpretation);

// 	value.visit(serializer)
// }

// /// Serialize the given Linked-Data value into a JSON-LD object.
// pub fn serialize_object(
// 	value: &(impl LinkedDataSubject + LinkedDataResource),
// ) -> Result<Object, Error> {
// 	serialize_object_with(&mut (), &mut (), value)
// }

// /// Serialize the given Linked-Data value into a JSON-LD node object.
// pub fn serialize_node(
// 	value: &(impl LinkedDataSubject + LinkedDataResource),
// ) -> Result<Node, Error> {
// 	serialize_node_with(&mut (), &mut (), value)
// }
