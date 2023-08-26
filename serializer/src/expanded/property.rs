use std::hash::Hash;

use indexmap::IndexSet;
use json_ld_core::{object::node::Multiset, Indexed, IndexedObject, StrippedIndexedObject};
use locspan::Meta;
use rdf_types::{IriVocabularyMut, Vocabulary};
use serde_ld::LexicalRepresentation;

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

impl<'a, V: Vocabulary, I> serde_ld::PredicateSerializer<V, I> for SerializeProperty<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = Multiset<StrippedIndexedObject<V::Iri, V::BlankId>>;
    type Error = Error;

    fn insert<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + LexicalRepresentation<V, I> + serde_ld::SerializeSubject<V, I>,
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
