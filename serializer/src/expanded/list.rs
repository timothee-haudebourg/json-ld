use std::hash::Hash;

use json_ld_core::{
    rdf::{RDF_FIRST, RDF_REST},
    Indexed, IndexedObject, Object,
};
use linked_data::{CowRdfTerm, Interpret, RdfLiteralValue};
use locspan::Meta;
use rdf_types::{
    interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
    Id, Interpretation, IriVocabularyMut, ReverseLiteralInterpretation, Term, Vocabulary,
};

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

impl<'a, V: Vocabulary, I: Interpretation> linked_data::SubjectVisitor<V, I>
    for SerializeList<'a, V, I>
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
    type Ok = Vec<IndexedObject<V::Iri, V::BlankId>>;
    type Error = Error;

    fn predicate<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
    where
        L: ?Sized + Interpret<V, I>,
        T: ?Sized + linked_data::LinkedDataPredicateObjects<V, I>,
    {
        let repr = predicate
            .interpret(self.vocabulary, self.interpretation)
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

    fn graph<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + linked_data::LinkedDataGraph<V, I>,
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

impl<'a, V: Vocabulary, I: Interpretation> linked_data::PredicateObjectsVisitor<V, I>
    for SerializeListFirst<'a, V, I>
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
    type Ok = Option<Object<V::Iri, V::BlankId>>;
    type Error = Error;

    fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Interpret<V, I> + linked_data::LinkedDataSubject<V, I>,
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

impl<'a, V: Vocabulary, I: Interpretation> linked_data::PredicateObjectsVisitor<V, I>
    for SerializeListRest<'a, V, I>
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
    type Ok = Vec<IndexedObject<V::Iri, V::BlankId>>;
    type Error = Error;

    fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Interpret<V, I> + linked_data::LinkedDataSubject<V, I>,
    {
        let serializer = SerializeList::new(self.vocabulary, self.interpretation);
        self.result = value.visit_subject(serializer)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}
