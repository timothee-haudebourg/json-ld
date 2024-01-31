use json_ld_core::{ExpandedDocument, Indexed, Node, Object};
use linked_data::{AsRdfLiteral, CowRdfTerm};
use rdf_types::{
	interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
	Interpretation, IriVocabularyMut, ReverseLiteralInterpretation, Term, Vocabulary,
};
use std::hash::Hash;

use crate::Error;

mod default_graph;
mod graph;
mod list;
mod node;
mod object;
mod property;
mod value;

use default_graph::SerializeDefaultGraph;
use graph::SerializeGraph;

pub use node::serialize_node_with;
pub use object::serialize_object_with;

pub struct SerializeExpandedDocument<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: ExpandedDocument<V::Iri, V::BlankId>,
}

impl<'a, I, V: Vocabulary> SerializeExpandedDocument<'a, I, V> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			result: ExpandedDocument::new(),
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::Visitor<I, V>
	for SerializeExpandedDocument<'a, I, V>
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
	type Ok = ExpandedDocument<V::Iri, V::BlankId>;
	type Error = Error;

	fn default_graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + linked_data::LinkedDataGraph<I, V>,
	{
		let serializer =
			SerializeDefaultGraph::new(self.vocabulary, self.interpretation, &mut self.result);

		value.visit_graph(serializer)
	}

	fn named_graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + linked_data::LinkedDataResource<I, V> + linked_data::LinkedDataGraph<I, V>,
	{
		let mut node = match value
			.lexical_representation(self.vocabulary, self.interpretation)
			.map(CowRdfTerm::into_owned)
		{
			Some(Term::Literal(_)) => return Err(Error::InvalidGraph),
			Some(Term::Id(id)) => Node::with_id(json_ld_core::Id::Valid(id)),
			None => Node::new(),
		};

		let serializer = SerializeGraph::new(self.vocabulary, self.interpretation);

		let graph = value.visit_graph(serializer)?;

		node.graph = Some(graph);
		self.result.insert(Indexed::new(Object::node(node), None));

		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}
