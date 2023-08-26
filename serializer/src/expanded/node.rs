use std::hash::Hash;

use json_ld_core::Node;
use rdf_types::{IriVocabularyMut, Term, Vocabulary};
use serde_ld::LexicalRepresentation;

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

impl<'a, V: Vocabulary, I> serde_ld::SubjectSerializer<V, I> for SerializeNode<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = Node<V::Iri, V::BlankId>;
    type Error = Error;

    fn insert<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
    where
        L: ?Sized + LexicalRepresentation<V, I>,
        T: ?Sized + serde_ld::SerializePredicate<V, I>,
    {
        let prop = match predicate.lexical_representation(self.interpretation, self.vocabulary) {
            Some(Term::Id(id)) => json_ld_core::Id::Valid(id),
            _ => return Err(Error::InvalidPredicate),
        };

        let serializer = SerializeProperty::new(self.vocabulary, self.interpretation);

        let objects = value.serialize_predicate(serializer)?;
        self.result.properties_mut().set(prop, objects);

        Ok(())
    }

    fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_ld::SerializeGraph<V, I>,
    {
        let serializer = SerializeGraph::new(self.vocabulary, self.interpretation);

        let graph = value.serialize_graph(serializer)?;
        self.result.set_graph(Some(graph));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}
