use linked_data::{
	LinkedData, LinkedDataGraph, LinkedDataPredicateObjects, LinkedDataResource, LinkedDataSubject,
	ResourceInterpretation,
};
use rdf_types::{vocabulary::IriVocabularyMut, Interpretation, Vocabulary};

use crate::{rdf::RDF_TYPE, IndexedNode, IndexedObject, Node};

impl<T, B, V: Vocabulary, I: Interpretation> LinkedDataResource<I, V> for Node<T, B>
where
	T: LinkedDataResource<I, V>,
	B: LinkedDataResource<I, V>,
{
	fn interpretation(
		&self,
		vocabulary: &mut V,
		interpretation: &mut I,
	) -> ResourceInterpretation<I, V> {
		match &self.id {
			Some(crate::Id::Valid(id)) => id.interpretation(vocabulary, interpretation),
			_ => ResourceInterpretation::Uninterpreted(None),
		}
	}
}

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataSubject<I, V> for Node<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_subject<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::SubjectVisitor<I, V>,
	{
		if !self.types().is_empty() {
			visitor.predicate(RDF_TYPE, &Types(self.types()))?;
		}

		for (property, objects) in self.properties() {
			if let crate::Id::Valid(id) = property {
				visitor.predicate(id, &Objects(objects))?;
			}
		}

		if let Some(reverse_properties) = self.reverse_properties() {
			for (property, nodes) in reverse_properties {
				if let crate::Id::Valid(id) = property {
					visitor.reverse_predicate(id, &Nodes(nodes))?;
				}
			}
		}

		if self.is_graph() {
			visitor.graph(self)?;
		}

		if let Some(included) = self.included() {
			for node in included {
				visitor.include(node.inner())?;
			}
		}

		visitor.end()
	}
}

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Node<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<I, V>,
	{
		visitor.object(self)?;
		visitor.end()
	}
}

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataGraph<I, V> for Node<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_graph<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::GraphVisitor<I, V>,
	{
		match self.graph() {
			Some(g) => {
				for object in g.iter() {
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

impl<T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedData<I, V> for Node<T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
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

struct Types<'a, T, B>(&'a [crate::Id<T, B>]);

impl<'a, T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Types<'a, T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<I, V>,
	{
		for ty in self.0 {
			if let crate::Id::Valid(id) = ty {
				visitor.object(id)?;
			}
		}

		visitor.end()
	}
}

struct Objects<'a, T, B>(&'a [IndexedObject<T, B>]);

impl<'a, T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Objects<'a, T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<I, V>,
	{
		for object in self.0 {
			visitor.object(object.inner())?;
		}

		visitor.end()
	}
}

struct Nodes<'a, T, B>(&'a [IndexedNode<T, B>]);

impl<'a, T, B, V: Vocabulary<Iri = T>, I: Interpretation> LinkedDataPredicateObjects<I, V>
	for Nodes<'a, T, B>
where
	T: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	B: LinkedDataResource<I, V> + LinkedDataSubject<I, V>,
	V: IriVocabularyMut,
{
	fn visit_objects<S>(&self, mut visitor: S) -> Result<S::Ok, S::Error>
	where
		S: linked_data::PredicateObjectsVisitor<I, V>,
	{
		for node in self.0 {
			visitor.object(node.inner())?;
		}

		visitor.end()
	}
}
