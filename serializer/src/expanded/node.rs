use std::hash::Hash;

use json_ld_core::Node;
use linked_data::{CowRdfTerm, Interpret, RdfLiteralValue};
use rdf_types::{
    interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
    Interpretation, IriVocabularyMut, ReverseLiteralInterpretation, Term, Vocabulary,
};

use crate::Error;

use super::{graph::SerializeGraph, property::SerializeProperty};

pub struct SerializeNode<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    result: Node<V::Iri, V::BlankId>,
}

impl<'a, V: Vocabulary, I> SerializeNode<'a, V, I> {
    pub fn new(
        vocabulary: &'a mut V,
        interpretation: &'a mut I,
        id: Option<json_ld_core::Id<V::Iri, V::BlankId>>,
    ) -> Self {
        let result = match id {
            Some(id) => Node::with_id(id),
            None => Node::new(),
        };

        Self {
            vocabulary,
            interpretation,
            result,
        }
    }
}

impl<'a, V: Vocabulary, I: Interpretation> linked_data::SubjectVisitor<V, I>
    for SerializeNode<'a, V, I>
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
    type Ok = Node<V::Iri, V::BlankId>;
    type Error = Error;

    fn predicate<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
    where
        L: ?Sized + Interpret<V, I>,
        T: ?Sized + linked_data::LinkedDataPredicateObjects<V, I>,
    {
        let prop = match predicate
            .lexical_representation(self.vocabulary, self.interpretation)
            .map(CowRdfTerm::into_owned)
        {
            Some(Term::Id(id)) => json_ld_core::Id::Valid(id),
            _ => return Err(Error::InvalidPredicate),
        };

        let serializer = SerializeProperty::new(self.vocabulary, self.interpretation);

        let objects = value.visit_objects(serializer)?;
        self.result.properties_mut().set(prop, objects);

        Ok(())
    }

    fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + linked_data::LinkedDataGraph<V, I>,
    {
        let serializer = SerializeGraph::new(self.vocabulary, self.interpretation);

        let graph = value.visit_graph(serializer)?;
        self.result.set_graph(Some(graph));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}
