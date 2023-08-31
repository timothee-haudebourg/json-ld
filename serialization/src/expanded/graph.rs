use std::hash::Hash;

use json_ld_core::{object::Graph, Indexed};
use linked_data::{AsRdfLiteral, LinkedDataResource};
use locspan::{Meta, Stripped};
use rdf_types::{
	interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
	Interpretation, IriVocabularyMut, ReverseLiteralInterpretation, Vocabulary,
};

use crate::Error;

use super::object::serialize_object_with;

pub struct SerializeGraph<'a, V: Vocabulary, I> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: Graph<V::Iri, V::BlankId>,
}

impl<'a, V: Vocabulary, I> SerializeGraph<'a, V, I> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			result: Graph::new(),
		}
	}
}

impl<'a, V: Vocabulary, I: Interpretation> linked_data::GraphVisitor<V, I>
	for SerializeGraph<'a, V, I>
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
	type Ok = Graph<V::Iri, V::BlankId>;
	type Error = Error;

	fn subject<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<V, I> + linked_data::LinkedDataSubject<V, I>,
	{
		let object = serialize_object_with(self.vocabulary, self.interpretation, value)?;
		self.result
			.insert(Stripped(Meta::none(Indexed::new(object, None))));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}
