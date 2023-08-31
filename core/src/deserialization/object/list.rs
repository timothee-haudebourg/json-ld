use linked_data::{
	LinkedData, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject,
	ResourceInterpretation,
};
use locspan::Meta;
use rdf_types::{Interpretation, IriVocabularyMut, LanguageTagVocabularyMut, Vocabulary};

use crate::{
	object::List,
	rdf::{RDF_FIRST, RDF_REST},
	IndexedObject,
};

impl<T, B, M, V: Vocabulary, I: Interpretation> LinkedDataResource<V, I> for List<T, B, M> {
	fn interpretation(
		&self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> ResourceInterpretation<V, I> {
		ResourceInterpretation::Uninterpreted(None)
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataSubject<V, I> for List<T, B, M>
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
		Rest(self.as_slice()).visit_subject(visitor)
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<V, I>
	for List<T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<V, I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<V, I> for List<T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<V, I>,
	{
		visitor.subject(self)?;
		visitor.end()
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<V, I> for List<T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<V, I>,
	{
		visitor.default_graph(self)?;
		visitor.end()
	}
}

struct Rest<'a, T, B, M>(&'a [IndexedObject<T, B, M>]);

impl<'a, T, B, M, V: Vocabulary, I: Interpretation> LinkedDataResource<V, I> for Rest<'a, T, B, M> {
	fn interpretation(
		&self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> ResourceInterpretation<V, I> {
		ResourceInterpretation::Uninterpreted(None)
	}
}

impl<'a, T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataSubject<V, I>
	for Rest<'a, T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_subject<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::SubjectVisitor<V, I>,
	{
		if let Some((Meta(first, _), rest)) = self.0.split_first() {
			visitor.predicate(RDF_FIRST, first.inner())?;
			visitor.predicate(RDF_REST, &Rest(rest))?;
		}

		visitor.end()
	}
}

impl<'a, T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<V, I>
	for Rest<'a, T, B, M>
where
	T: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	B: LinkedDataResource<V, I> + LinkedDataSubject<V, I>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<V, I>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}
