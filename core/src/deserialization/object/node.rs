use linked_data::{
	LinkedData, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject,
	ResourceInterpretation,
};
use locspan::{Meta, Stripped};
use rdf_types::{Interpretation, IriVocabularyMut, LanguageTagVocabularyMut, Vocabulary};

use crate::{rdf::RDF_TYPE, Node, StrippedIndexedNode, StrippedIndexedObject};

impl<T, B, M, V: Vocabulary, I: Interpretation> LinkedDataResource<I, V> for Node<T, B, M>
where
	T: LinkedDataResource<I, V>,
	B: LinkedDataResource<I, V>,
{
	fn interpretation(
		&self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> ResourceInterpretation<I, V> {
		match self.id() {
			Some(Meta(crate::Id::Valid(id), _)) => id.interpretation(vocabulary, interpretation),
			_ => ResourceInterpretation::Uninterpreted(None),
		}
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataSubject<I, V> for Node<T, B, M>
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
		if !self.types().is_empty() {
			visitor.predicate(RDF_TYPE, &Types(self.types()))?;
		}

		for (Meta(property, _), objects) in self.properties() {
			if let crate::Id::Valid(id) = property {
				visitor.predicate(id, &Objects(objects))?;
			}
		}

		if let Some(Meta(reverse_properties, _)) = self.reverse_properties() {
			for (Meta(property, _), nodes) in reverse_properties {
				if let crate::Id::Valid(id) = property {
					visitor.reverse_predicate(id, &Nodes(nodes))?;
				}
			}
		}

		if self.is_graph() {
			visitor.graph(self)?;
		}

		if let Some(Meta(included, _)) = self.included() {
			for Stripped(Meta(node, _)) in included {
				visitor.include(node.inner())?;
			}
		}

		visitor.end()
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Node<T, B, M>
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

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<I, V> for Node<T, B, M>
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
		match self.graph() {
			Some(g) => {
				for Stripped(Meta(object, _)) in g.iter() {
					visitor.subject(object.inner())?;
				}
			}
			None => {
				visitor.subject(self)?;
			}
		}

		visitor.end()
	}
}

impl<T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<I, V> for Node<T, B, M>
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
		if self.is_graph() {
			visitor.named_graph(self)?;
		} else {
			visitor.default_graph(self)?;
		}

		visitor.end()
	}
}

struct Types<'a, T, B, M>(&'a [Meta<crate::Id<T, B>, M>]);

impl<'a, T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Types<'a, T, B, M>
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
		for Meta(ty, _) in self.0 {
			if let crate::Id::Valid(id) = ty {
				visitor.object(id)?;
			}
		}

		visitor.end()
	}
}

struct Objects<'a, T, B, M>(&'a [StrippedIndexedObject<T, B, M>]);

impl<'a, T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Objects<'a, T, B, M>
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
		for Stripped(Meta(object, _)) in self.0 {
			visitor.object(object.inner())?;
		}

		visitor.end()
	}
}

struct Nodes<'a, T, B, M>(&'a [StrippedIndexedNode<T, B, M>]);

impl<'a, T, B, M, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Nodes<'a, T, B, M>
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
		for Stripped(Meta(node, _)) in self.0 {
			visitor.object(node.inner())?;
		}

		visitor.end()
	}
}
