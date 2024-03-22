use std::hash::Hash;

use json_ld_core::{
	rdf::{RDF_FIRST, RDF_REST},
	Indexed, IndexedObject, Object,
};
use linked_data::{CowRdfTerm, LinkedDataResource};
use rdf_types::{
	interpretation::{
		ReverseBlankIdInterpretation, ReverseIriInterpretation, ReverseLiteralInterpretation,
	},
	vocabulary::IriVocabularyMut,
	Id, Interpretation, Term, Vocabulary,
};

use crate::Error;

use super::object::serialize_object_with;

pub struct SerializeList<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	first: Option<Object<V::Iri, V::BlankId>>,
	rest: Vec<IndexedObject<V::Iri, V::BlankId>>,
}

impl<'a, I, V: Vocabulary> SerializeList<'a, I, V> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			first: None,
			rest: Vec::new(),
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::SubjectVisitor<I, V>
	for SerializeList<'a, I, V>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Ok = Vec<IndexedObject<V::Iri, V::BlankId>>;
	type Error = Error;

	fn predicate<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I, V>,
		T: ?Sized + linked_data::LinkedDataPredicateObjects<I, V>,
	{
		let repr = predicate
			.interpretation(self.vocabulary, self.interpretation)
			.into_lexical_representation(self.vocabulary, self.interpretation)
			.map(CowRdfTerm::into_term);

		match repr {
			Some(Term::Id(id)) => {
				if let Id::Iri(iri) = id {
					let iri = self.vocabulary.iri(iri.as_ref()).unwrap();
					if iri == RDF_FIRST {
						let serializer =
							SerializeListFirst::new(self.vocabulary, self.interpretation);
						self.first = value.visit_objects(serializer)?;
					} else if iri == RDF_REST {
						let serializer =
							SerializeListRest::new(self.vocabulary, self.interpretation);
						self.rest = value.visit_objects(serializer)?;
					}
				}

				Ok(())
			}
			_ => Err(Error::InvalidPredicate),
		}
	}

	fn reverse_predicate<L, T>(&mut self, _predicate: &L, _subjects: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I, V>,
		T: ?Sized + linked_data::LinkedDataPredicateObjects<I, V>,
	{
		Err(Error::ListReverseProperty)
	}

	fn graph<T>(&mut self, _value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + linked_data::LinkedDataGraph<I, V>,
	{
		Ok(())
	}

	fn include<T>(&mut self, _value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
	{
		Err(Error::ListInclude)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let first = self.first.unwrap_or_else(Object::null);
		let mut result = self.rest;
		result.push(Indexed::none(first));
		Ok(result)
	}
}

pub struct SerializeListFirst<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: Option<Object<V::Iri, V::BlankId>>,
}

impl<'a, I, V: Vocabulary> SerializeListFirst<'a, I, V> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			result: None,
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::PredicateObjectsVisitor<I, V>
	for SerializeListFirst<'a, I, V>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Ok = Option<Object<V::Iri, V::BlankId>>;
	type Error = Error;

	fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
	{
		self.result = Some(serialize_object_with(
			self.vocabulary,
			self.interpretation,
			value,
		)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}

pub struct SerializeListRest<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: Vec<IndexedObject<V::Iri, V::BlankId>>,
}

impl<'a, I, V: Vocabulary> SerializeListRest<'a, I, V> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			result: Vec::new(),
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::PredicateObjectsVisitor<I, V>
	for SerializeListRest<'a, I, V>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Ok = Vec<IndexedObject<V::Iri, V::BlankId>>;
	type Error = Error;

	fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
	{
		let serializer = SerializeList::new(self.vocabulary, self.interpretation);
		self.result = value.visit_subject(serializer)?;
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}
