use linked_data::{LinkedData, LinkedDataGraph, LinkedDataResource, LinkedDataSubject};
use locspan::{Meta, Stripped};
use rdf_types::{Interpretation, IriVocabularyMut, LanguageTagVocabularyMut, Vocabulary};

use crate::ExpandedDocument;

mod object;

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<I, V>
	for ExpandedDocument<T, B, M>
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
		for Stripped(Meta(object, _)) in self {
			visitor.subject(object.inner())?;
		}

		visitor.end()
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<I, V>
	for ExpandedDocument<T, B, M>
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
