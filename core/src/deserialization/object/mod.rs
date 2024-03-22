mod list;
mod node;
mod value;

use linked_data::{
	LinkedData, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject,
};
use rdf_types::{vocabulary::IriVocabularyMut, Interpretation, Vocabulary};

use crate::Object;

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataResource<I, V> for Object<T, B>
where
	T: LinkedDataResource<I, V>,
	B: LinkedDataResource<I, V>,
{
	fn interpretation(
		&self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> linked_data::ResourceInterpretation<I, V> {
		match self {
			Self::Node(node) => node.interpretation(vocabulary, interpretation),
			Self::List(list) => list.interpretation(vocabulary, interpretation),
			Self::Value(value) => value.interpretation(vocabulary, interpretation),
		}
	}
}

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataSubject<I, V> for Object<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::SubjectVisitor<I, V>,
	{
		match self {
			Self::Node(node) => node.visit_subject(visitor),
			Self::List(list) => list.visit_subject(visitor),
			Self::Value(value) => value.visit_subject(visitor),
		}
	}
}

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Object<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_objects<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<I, V>,
	{
		match self {
			Self::Node(node) => node.visit_objects(visitor),
			Self::List(list) => list.visit_objects(visitor),
			Self::Value(value) => value.visit_objects(visitor),
		}
	}
}

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<I, V> for Object<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<I, V>,
	{
		match self {
			Self::Node(node) => node.visit_graph(visitor),
			Self::List(list) => list.visit_graph(visitor),
			Self::Value(value) => value.visit_graph(visitor),
		}
	}
}

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<I, V> for Object<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<I, V>,
	{
		match self {
			Self::Node(node) => node.visit(visitor),
			Self::List(list) => list.visit(visitor),
			Self::Value(value) => value.visit(visitor),
		}
	}
}
