use std::hash::Hash;

use json_ld_core::{ExpandedDocument, Indexed, Object};
use linked_data::{AsRdfLiteral, CowRdfTerm, LinkedDataResource};
use locspan::Meta;
use rdf_types::{
	interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
	Interpretation, IriVocabularyMut, ReverseLiteralInterpretation, Term, Vocabulary,
};

use crate::Error;

use super::{node::SerializeNode, value::literal_to_value};

pub struct SerializeDefaultGraph<'a, V: Vocabulary, I> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: &'a mut ExpandedDocument<V::Iri, V::BlankId>,
}

impl<'a, V: Vocabulary, I> SerializeDefaultGraph<'a, V, I> {
	pub fn new(
		vocabulary: &'a mut V,
		interpretation: &'a mut I,
		result: &'a mut ExpandedDocument<V::Iri, V::BlankId>,
	) -> Self {
		Self {
			vocabulary,
			interpretation,
			result,
		}
	}
}

impl<'a, V: Vocabulary, I: Interpretation> linked_data::GraphVisitor<V, I>
	for SerializeDefaultGraph<'a, V, I>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	V::Value: AsRdfLiteral<V>,
	V::LanguageTag: Clone,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Ok = ();
	type Error = Error;

	fn subject<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<V, I> + linked_data::LinkedDataSubject<V, I>,
	{
		let id = match value
			.lexical_representation(self.vocabulary, self.interpretation)
			.map(CowRdfTerm::into_owned)
		{
			Some(Term::Literal(lit)) => {
				let value = literal_to_value(self.vocabulary, lit);
				self.result
					.insert(Meta::none(Indexed::new(Object::Value(value), None)));
				return Ok(());
			}
			Some(Term::Id(id)) => Some(json_ld_core::Id::Valid(id)),
			_ => None,
		};

		let serializer = SerializeNode::new(self.vocabulary, self.interpretation, id);

		let node = value.visit_subject(serializer)?;
		self.result
			.insert(Meta::none(Indexed::new(Object::node(node), None)));
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(())
	}
}
