use linked_data::{LinkedData, LinkedDataGraph, LinkedDataResource, LinkedDataSubject};
use locspan::{Meta, Stripped};
use rdf_types::{Interpretation, IriVocabularyMut, LanguageTagVocabularyMut, Vocabulary};

use crate::ExpandedDocument;

mod object;

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<V, I>
	for ExpandedDocument<T, B, M>
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
		for Stripped(Meta(object, _)) in self {
			visitor.subject(object.inner())?;
		}

		visitor.end()
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<V, I>
	for ExpandedDocument<T, B, M>
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
