use std::hash::Hash;

use json_ld_core::{
    rdf::{RDF_FIRST, RDF_REST},
    Indexed, IndexedObject, Object,
};
use locspan::Meta;
use rdf_types::{Id, IriVocabularyMut, Term, Vocabulary};
use serde_ld::LexicalRepresentation;

use crate::Error;

use super::object::serialize_object;

pub struct SerializeList<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    first: Option<Object<V::Iri, V::BlankId>>,
    rest: Vec<IndexedObject<V::Iri, V::BlankId>>,
}

impl<'a, V: Vocabulary, I> SerializeList<'a, V, I> {
    pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
        Self {
            vocabulary,
            interpretation,
            first: None,
            rest: Vec::new(),
        }
    }
}

impl<'a, V: Vocabulary, I> serde_ld::SubjectSerializer<V, I> for SerializeList<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = Vec<IndexedObject<V::Iri, V::BlankId>>;
    type Error = Error;

    fn insert<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
    where
        L: ?Sized + LexicalRepresentation<V, I>,
        T: ?Sized + serde_ld::SerializePredicate<V, I>,
    {
        match predicate.lexical_representation(self.interpretation, self.vocabulary) {
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
                        self.rest = value.serialize_predicate(serializer)?;
                    }
                }

                Ok(())
            }
            _ => Err(Error::InvalidPredicate),
        }
    }

    fn graph<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_ld::SerializeGraph<V, I>,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let first = self.first.unwrap_or_else(Object::null);
        let mut result = self.rest;
        result.push(Meta::none(Indexed::none(first)));
        Ok(result)
    }
}

pub struct SerializeListFirst<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    result: Option<Object<V::Iri, V::BlankId>>,
}

impl<'a, V: Vocabulary, I> SerializeListFirst<'a, V, I> {
    pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
        Self {
            vocabulary,
            interpretation,
            result: None,
        }
    }
}

impl<'a, V: Vocabulary, I> serde_ld::PredicateSerializer<V, I> for SerializeListFirst<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = Option<Object<V::Iri, V::BlankId>>;
    type Error = Error;

    fn insert<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + LexicalRepresentation<V, I> + serde_ld::SerializeSubject<V, I>,
    {
        self.result = Some(serialize_object(
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

pub struct SerializeListRest<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    result: Vec<IndexedObject<V::Iri, V::BlankId>>,
}

impl<'a, V: Vocabulary, I> SerializeListRest<'a, V, I> {
    pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
        Self {
            vocabulary,
            interpretation,
            result: Vec::new(),
        }
    }
}

impl<'a, V: Vocabulary, I> serde_ld::PredicateSerializer<V, I> for SerializeListRest<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = Vec<IndexedObject<V::Iri, V::BlankId>>;
    type Error = Error;

    fn insert<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + LexicalRepresentation<V, I> + serde_ld::SerializeSubject<V, I>,
    {
        let serializer = SerializeList::new(self.vocabulary, self.interpretation);
        self.result = value.serialize_subject(serializer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}
