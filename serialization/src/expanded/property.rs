use std::hash::Hash;

use json_ld_core::{object::node::Multiset, Indexed, StrippedIndexedNode, StrippedIndexedObject};
use linked_data::{AsRdfLiteral, LinkedDataResource};
use locspan::Meta;
use rdf_types::{
	interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
	Interpretation, IriVocabularyMut, ReverseLiteralInterpretation, Vocabulary,
};

use crate::Error;

use super::{object::serialize_object_with, serialize_node_with};

pub struct SerializeProperty<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: Multiset<StrippedIndexedObject<V::Iri, V::BlankId>>,
}

impl<'a, I, V: Vocabulary> SerializeProperty<'a, I, V> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			result: Multiset::new(),
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::PredicateObjectsVisitor<I, V>
	for SerializeProperty<'a, I, V>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	V::LanguageTag: Clone,
	V::Value: AsRdfLiteral<V>,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Ok = Multiset<StrippedIndexedObject<V::Iri, V::BlankId>>;
	type Error = Error;

	fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
	{
		let object = serialize_object_with(self.vocabulary, self.interpretation, value)?;
		self.result
			.insert(locspan::Stripped(Meta::none(Indexed::none(object))));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}

pub struct SerializeReverseProperty<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: Multiset<StrippedIndexedNode<V::Iri, V::BlankId>>,
}

impl<'a, I, V: Vocabulary> SerializeReverseProperty<'a, I, V> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			result: Multiset::new(),
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::PredicateObjectsVisitor<I, V>
	for SerializeReverseProperty<'a, I, V>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	V::LanguageTag: Clone,
	V::Value: AsRdfLiteral<V>,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Ok = Multiset<StrippedIndexedNode<V::Iri, V::BlankId>>;
	type Error = Error;

	fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
	{
		let object = serialize_node_with(self.vocabulary, self.interpretation, value)?;
		self.result
			.insert(locspan::Stripped(Meta::none(Indexed::none(object))));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}
