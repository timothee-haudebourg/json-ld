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

impl<T, B, M, V: Vocabulary, I: Interpretation> LinkedDataResource<I, V> for List<T, B, M> {
	fn interpretation(
		&self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> ResourceInterpretation<I, V> {
		ResourceInterpretation::Uninterpreted(None)
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataSubject<I, V> for List<T, B, M>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_subject<S>(&self, visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::SubjectVisitor<I, V>,
	{
		Rest(self.as_slice()).visit_subject(visitor)
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for List<T, B, M>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<I, V>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<I, V> for List<T, B, M>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<I, V>,
	{
		visitor.subject(self)?;
		visitor.end()
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<I, V> for List<T, B, M>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<I, V>,
	{
		visitor.default_graph(self)?;
		visitor.end()
	}
}

struct Rest<'a, T, B, M>(&'a [IndexedObject<T, B, M>]);

impl<'a, T, B, M, V: Vocabulary, I: Interpretation> LinkedDataResource<I, V> for Rest<'a, T, B, M> {
	fn interpretation(
		&self,
		_vocabulary: &mut V,
		_interpretation: &mut I,
	) -> ResourceInterpretation<I, V> {
		ResourceInterpretation::Uninterpreted(None)
	}
}

impl<'a, T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataSubject<I, V>
	for Rest<'a, T, B, M>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_subject<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::SubjectVisitor<I, V>,
	{
		if let Some((Meta(first, _), rest)) = self.0.split_first() {
			visitor.predicate(RDF_FIRST, first.inner())?;
			visitor.predicate(RDF_REST, &Rest(rest))?;
		}

		visitor.end()
	}
}

impl<'a, T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Rest<'a, T, B, M>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	M: Clone,
	V: IriVocabularyMut + LanguageTagVocabularyMut,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<I, V>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}
