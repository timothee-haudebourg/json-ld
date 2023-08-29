use std::hash::Hash;

use json_ld_core::{object::node::Multiset, Indexed, StrippedIndexedObject};
use linked_data::{Interpret, RdfLiteralValue};
use locspan::Meta;
use rdf_types::{
    interpretation::{ReverseBlankIdInterpretation, ReverseIriInterpretation},
    Interpretation, IriVocabularyMut, ReverseLiteralInterpretation, Vocabulary,
};

use crate::Error;

use super::object::serialize_object;

pub struct SerializeProperty<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    result: Multiset<StrippedIndexedObject<V::Iri, V::BlankId>>,
}

impl<'a, V: Vocabulary, I> SerializeProperty<'a, V, I> {
    pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
        Self {
            vocabulary,
            interpretation,
            result: Multiset::new(),
        }
    }
}

impl<'a, V: Vocabulary, I: Interpretation> linked_data::PredicateObjectsVisitor<V, I>
    for SerializeProperty<'a, V, I>
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
    type Ok = Multiset<StrippedIndexedObject<V::Iri, V::BlankId>>;
    type Error = Error;

    fn object<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Interpret<V, I> + linked_data::LinkedDataSubject<V, I>,
    {
        let object = serialize_object(self.vocabulary, self.interpretation, value)?;
        self.result
            .insert(locspan::Stripped(Meta::none(Indexed::none(object))));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}
