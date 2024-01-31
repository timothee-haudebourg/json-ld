use std::hash::Hash;

use json_ld_core::{object::Graph, Indexed};
use linked_data::{AsRdfLiteral, LinkedDataResource};
use rdf_types::{
	interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
	Interpretation, IriVocabularyMut, ReverseLiteralInterpretation, Vocabulary,
};

use crate::Error;

use super::object::serialize_object_with;

pub struct SerializeGraph<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: Graph<V::Iri, V::BlankId>,
}

impl<'a, I, V: Vocabulary> SerializeGraph<'a, I, V> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			result: Graph::new(),
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::GraphVisitor<I, V>
	for SerializeGraph<'a, I, V>
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
		T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
	{
		let object = serialize_object_with(self.vocabulary, self.interpretation, value)?;
		self.result.insert(Indexed::new(object, None));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}
