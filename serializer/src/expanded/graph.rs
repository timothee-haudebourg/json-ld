use std::hash::Hash;

use json_ld_core::{object::Graph, Indexed};
use linked_data::LexicalRepresentation;
use locspan::{Meta, Stripped};
use rdf_types::{IriVocabularyMut, Vocabulary};

use crate::Error;

use super::object::serialize_object;

pub struct SerializeGraph<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    result: Graph<V::Iri, V::BlankId>,
}

impl<'a, V: Vocabulary, I> SerializeGraph<'a, V, I> {
    pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
        Self {
            vocabulary,
            interpretation,
            result: Graph::new(),
        }
    }
}

impl<'a, V: Vocabulary, I> linked_data::GraphVisitor<V, I> for SerializeGraph<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = Graph<V::Iri, V::BlankId>;
    type Error = Error;

    fn subject<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + LexicalRepresentation<V, I> + linked_data::LinkedDataSubject<V, I>,
    {
        let object = serialize_object(self.vocabulary, self.interpretation, value)?;
        self.result
            .insert(Stripped(Meta::none(Indexed::new(object, None))));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}
