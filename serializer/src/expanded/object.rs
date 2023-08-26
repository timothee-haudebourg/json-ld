use std::hash::Hash;

use json_ld_core::{
    object::{node::Properties, Graph, List},
    rdf::{RDF_FIRST, RDF_REST},
    Indexed, IndexedObject, Node, Object,
};
use locspan::Meta;
use rdf_types::{Id, IriVocabularyMut, Term, Vocabulary};
use serde_ld::LexicalRepresentation;

use crate::Error;

use super::{
    graph::SerializeGraph,
    list::{SerializeListFirst, SerializeListRest},
    node::SerializeNode,
    property::SerializeProperty,
    value::literal_to_value,
};

pub fn serialize_object<V: Vocabulary, I, T>(
    vocabulary: &mut V,
    interpretation: &mut I,
    value: &T,
) -> Result<Object<V::Iri, V::BlankId>, Error>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
    T: ?Sized + LexicalRepresentation<V, I> + serde_ld::SerializeSubject<V, I>,
{
    match value.lexical_representation(interpretation, vocabulary) {
        Some(Term::Literal(lit)) => {
            let value = literal_to_value(vocabulary, lit);
            Ok(Object::Value(value))
        }
        Some(Term::Id(id)) => {
            let serializer = SerializeNode::new(
                vocabulary,
                interpretation,
                Some(json_ld_core::Id::Valid(id)),
            );

            Ok(Object::node(value.serialize_subject(serializer)?))
        }
        None => {
            let serializer = SerializeObject::new(vocabulary, interpretation);

            value.serialize_subject(serializer)
        }
    }
}

pub struct SerializeObject<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    properties: Properties<V::Iri, V::BlankId>,
    graph: Option<Graph<V::Iri, V::BlankId>>,
    first: Option<Object<V::Iri, V::BlankId>>,
    rest: Option<Vec<IndexedObject<V::Iri, V::BlankId>>>,
}

impl<'a, V: Vocabulary, I> SerializeObject<'a, V, I> {
    pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
        Self {
            vocabulary,
            interpretation,
            properties: Properties::new(),
            graph: None,
            first: None,
            rest: None,
        }
    }
}

impl<'a, V: Vocabulary, I> serde_ld::SubjectSerializer<V, I> for SerializeObject<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = Object<V::Iri, V::BlankId>;
    type Error = Error;

    fn insert<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
    where
        L: ?Sized + LexicalRepresentation<V, I>,
        T: ?Sized + serde_ld::SerializePredicate<V, I>,
    {
        let prop = match predicate.lexical_representation(self.interpretation, self.vocabulary) {
            Some(Term::Id(id)) => {
                if let Id::Iri(iri) = &id {
                    let iri = self.vocabulary.iri(iri).unwrap();
                    if iri == RDF_FIRST {
                        let serializer =
                            SerializeListFirst::new(self.vocabulary, self.interpretation);
                        self.first = value.serialize_predicate(serializer)?;
                    } else if iri == RDF_REST {
                        let serializer =
                            SerializeListRest::new(self.vocabulary, self.interpretation);
                        self.rest = Some(value.serialize_predicate(serializer)?);
                    }
                }

                json_ld_core::Id::Valid(id)
            }
            _ => return Err(Error::InvalidPredicate),
        };

        let serializer = SerializeProperty::new(self.vocabulary, self.interpretation);

        let objects = value.serialize_predicate(serializer)?;
        self.properties.set(prop, objects);

        Ok(())
    }

    fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_ld::SerializeGraph<V, I>,
    {
        let serializer = SerializeGraph::new(self.vocabulary, self.interpretation);
        self.graph = Some(value.serialize_graph(serializer)?);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        if self.first.is_some()
            && self.rest.is_some()
            && self.properties.is_empty()
            && self.graph.is_none()
        {
            let mut items = self.rest.unwrap();
            items.push(Meta::none(Indexed::none(self.first.unwrap())));
            items.reverse();
            Ok(Object::List(List::new(items)))
        } else {
            if let Some(item) = self.first {
                let iri = self.vocabulary.insert(RDF_FIRST);
                self.properties
                    .insert(json_ld_core::Id::Valid(Id::Iri(iri)), Indexed::none(item))
            }

            if let Some(rest) = self.rest {
                let iri = self.vocabulary.insert(RDF_REST);
                self.properties.insert(
                    json_ld_core::Id::Valid(Id::Iri(iri)),
                    Indexed::none(Object::List(List::new(rest))),
                )
            }

            let mut node = Node::new();
            *node.properties_mut() = self.properties;

            Ok(Object::node(node))
        }
    }
}
