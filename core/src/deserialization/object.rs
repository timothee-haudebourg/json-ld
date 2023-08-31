mod list;
mod node;
mod value;

use linked_data::{
	LinkedData, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject,
};
use rdf_types::{Interpretation, IriVocabularyMut, LanguageTagVocabularyMut, Vocabulary};

use crate::Object;

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataResource<V, I>
	for Object<T, B, M>
where
	T: LinkedDataResource<V, I>,
	B: LinkedDataResource<V, I>,
	M: Clone,
	V: LanguageTagVocabularyMut,
{
	fn interpretation(
		&self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> linked_data::ResourceInterpretation<V, I> {
		match self {
			Self::Node(node) => node.interpretation(vocabulary, interpretation),
			Self::List(list) => list.interpretation(vocabulary, interpretation),
			Self::Value(value) => value.interpretation(vocabulary, interpretation),
		}
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataSubject<V, I> for Object<T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::SubjectVisitor<V, I>,
	{
		match self {
			Self::Node(node) => node.visit_subject(visitor),
			Self::List(list) => list.visit_subject(visitor),
			Self::Value(value) => value.visit_subject(visitor),
		}
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<V, I>
	for Object<T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_objects<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<V, I>,
	{
		match self {
			Self::Node(node) => node.visit_objects(visitor),
			Self::List(list) => list.visit_objects(visitor),
			Self::Value(value) => value.visit_objects(visitor),
		}
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<V, I> for Object<T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_graph<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<V, I>,
	{
		match self {
			Self::Node(node) => node.visit_graph(visitor),
			Self::List(list) => list.visit_graph(visitor),
			Self::Value(value) => value.visit_graph(visitor),
		}
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<V, I> for Object<T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<V, I>,
	{
		match self {
			Self::Node(node) => node.visit(visitor),
			Self::List(list) => list.visit(visitor),
			Self::Value(value) => value.visit(visitor),
		}
	}
}
