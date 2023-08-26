use json_ld_core::{ExpandedDocument, Indexed, Node, Object};
use locspan::Meta;
use rdf_types::{IriVocabularyMut, Term, Vocabulary};
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

pub struct SerializeExpandedDocument<'a, V: Vocabulary, I> {
    vocabulary: &'a mut V,
    interpretation: &'a mut I,
    result: ExpandedDocument<V::Iri, V::BlankId>,
}

impl<'a, V: Vocabulary, I> SerializeExpandedDocument<'a, V, I> {
    pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
        Self {
            vocabulary,
            interpretation,
            result: ExpandedDocument::new(),
        }
    }
}

impl<'a, V: Vocabulary, I> serde_ld::Serializer<V, I> for SerializeExpandedDocument<'a, V, I>
where
    V: IriVocabularyMut,
    V::Iri: Eq + Hash,
    V::BlankId: Eq + Hash,
{
    type Ok = ExpandedDocument<V::Iri, V::BlankId>;
    type Error = Error;

    fn insert_default<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_ld::SerializeGraph<V, I>,
    {
        let serializer =
            SerializeDefaultGraph::new(self.vocabulary, self.interpretation, &mut self.result);

        value.serialize_graph(serializer)
    }

    fn insert<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_ld::LexicalRepresentation<V, I> + serde_ld::SerializeGraph<V, I>,
    {
        let mut node = match value.lexical_representation(self.interpretation, self.vocabulary) {
            Some(Term::Literal(_)) => return Err(Error::InvalidGraph),
            Some(Term::Id(id)) => Node::with_id(json_ld_core::Id::Valid(id)),
            None => Node::new(),
        };

        let serializer = SerializeGraph::new(self.vocabulary, self.interpretation);

        let graph = value.serialize_graph(serializer)?;

        node.set_graph(Some(graph));
        self.result
            .insert(Meta::none(Indexed::new(Object::node(node), None)));

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.result)
    }
}
