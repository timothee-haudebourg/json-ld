use linked_data::{LinkedData, LinkedDataGraph, LinkedDataResource, LinkedDataSubject};
use rdf_types::{vocabulary::IriVocabularyMut, Interpretation, Vocabulary};

use crate::ExpandedDocument;

mod object;

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<I, V>
	for ExpandedDocument<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<I, V>,
	{
		for object in self {
			visitor.subject(object.inner())?;
		}

		visitor.end()
	}
}

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<I, V> for ExpandedDocument<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::Visitor<I, V>,
	{
		visitor.default_graph(self)?;
		visitor.end()
	}
}
