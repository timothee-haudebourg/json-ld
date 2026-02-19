use std::hash::Hash;

use json_ld_core::{
	object::{
		node::{Included, Multiset, Properties, ReverseProperties},
		Graph, List,
	},
	rdf::{RDF_FIRST, RDF_REST, RDF_TYPE},
	Indexed, IndexedObject, Node, Object,
};
use linked_data::{CowRdfTerm, LinkedDataResource};
use rdf_types::{
	interpretation::{
		ReverseBlankIdInterpretation, ReverseIriInterpretation, ReverseLiteralInterpretation,
	},
	vocabulary::IriVocabularyMut,
	Id, Interpretation, Term, Vocabulary,
};

use crate::Error;

use super::{
	graph::SerializeGraph,
	list::{SerializeListFirst, SerializeListRest},
	node::{into_type_value, is_iri, SerializeNode},
	property::{SerializeProperty, SerializeReverseProperty},
	serialize_node_with,
	value::literal_to_value,
};

/// Serialize the given Linked-Data value into a JSON-LD object using a
/// custom vocabulary and interpretation.
pub fn serialize_object_with<I, V, T>(
	vocabulary: &mut V,
	interpretation: &mut I,
	value: &T,
) -> Result<Object<V::Iri, V::BlankId>, Error>
where
	V: Vocabulary + IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
	T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
{
	match value
		.lexical_representation(vocabulary, interpretation)
		.map(CowRdfTerm::into_owned)
	{
		Some(Term::Literal(lit)) => {
			let value = literal_to_value(vocabulary, lit);
			Ok(Object::Value(value))
		}
		Some(Term::Id(id)) => {
			let serializer = SerializeNode::new(
				vocabulary,
				interpretation,
				Some(json_ld_core::Id::Valid(id)),
			);

			Ok(Object::node(value.visit_subject(serializer)?))
		}
		None => {
			let serializer = SerializeObject::new(vocabulary, interpretation);

			value.visit_subject(serializer)
		}
	}
}

pub struct SerializeObject<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	types: Vec<json_ld_core::Id<V::Iri, V::BlankId>>,
	properties: Properties<V::Iri, V::BlankId>,
	reverse_properties: ReverseProperties<V::Iri, V::BlankId>,
	included: Included<V::Iri, V::BlankId>,
	graph: Option<Graph<V::Iri, V::BlankId>>,
	first: Option<Object<V::Iri, V::BlankId>>,
	rest: Option<Vec<IndexedObject<V::Iri, V::BlankId>>>,
}

impl<'a, I, V: Vocabulary> SerializeObject<'a, I, V> {
	pub fn new(vocabulary: &'a mut V, interpretation: &'a mut I) -> Self {
		Self {
			vocabulary,
			interpretation,
			types: Vec::new(),
			properties: Properties::new(),
			reverse_properties: ReverseProperties::new(),
			included: Included::new(),
			graph: None,
			first: None,
			rest: None,
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::SubjectVisitor<I, V>
	for SerializeObject<'a, I, V>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Ok = Object<V::Iri, V::BlankId>;
	type Error = Error;

	fn predicate<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I, V>,
		T: ?Sized + linked_data::LinkedDataPredicateObjects<I, V>,
	{
		let prop = match predicate
			.lexical_representation(self.vocabulary, self.interpretation)
			.map(CowRdfTerm::into_owned)
		{
			Some(Term::Id(id)) => {
				if let Id::Iri(iri) = &id {
					let iri = self.vocabulary.iri(iri).unwrap();
					if iri == RDF_FIRST {
						let serializer =
							SerializeListFirst::new(self.vocabulary, self.interpretation);
						self.first = value.visit_objects(serializer)?;
					} else if iri == RDF_REST {
						let serializer =
							SerializeListRest::new(self.vocabulary, self.interpretation);
						self.rest = Some(value.visit_objects(serializer)?);
					}
				}

				json_ld_core::Id::Valid(id)
			}
			_ => return Err(Error::InvalidPredicate),
		};

		let serializer = SerializeProperty::new(self.vocabulary, self.interpretation);

		let objects = value.visit_objects(serializer)?;

		if is_iri(self.vocabulary, &prop, RDF_TYPE) {
			let mut non_iri_objects = Multiset::new();

			for obj in objects {
				match into_type_value(obj) {
					Ok(ty) => self.types.push(ty),
					Err(obj) => {
						non_iri_objects.insert(obj);
					}
				}
			}

			if !non_iri_objects.is_empty() {
				self.properties.set(prop, non_iri_objects);
			}
		} else {
			self.properties.set(prop, objects);
		}

		Ok(())
	}

	fn reverse_predicate<L, T>(&mut self, predicate: &L, value: &T) -> Result<(), Self::Error>
	where
		L: ?Sized + LinkedDataResource<I, V>,
		T: ?Sized + linked_data::LinkedDataPredicateObjects<I, V>,
	{
		let prop = match predicate
			.lexical_representation(self.vocabulary, self.interpretation)
			.map(CowRdfTerm::into_owned)
		{
			Some(Term::Id(id)) => json_ld_core::Id::Valid(id),
			_ => return Err(Error::InvalidPredicate),
		};

		let serializer = SerializeReverseProperty::new(self.vocabulary, self.interpretation);

		let objects = value.visit_objects(serializer)?;
		self.reverse_properties.set(prop, objects);

		Ok(())
	}

	fn include<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
	{
		let node = serialize_node_with(self.vocabulary, self.interpretation, value)?;
		self.included.insert(Indexed::none(node));
		Ok(())
	}

	fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + linked_data::LinkedDataGraph<I, V>,
	{
		let serializer = SerializeGraph::new(self.vocabulary, self.interpretation);
		self.graph = Some(value.visit_graph(serializer)?);
		Ok(())
	}

	fn end(mut self) -> Result<Self::Ok, Self::Error> {
		if self.first.is_some()
			&& self.rest.is_some()
			&& self.types.is_empty()
			&& self.properties.is_empty()
			&& self.graph.is_none()
		{
			#[allow(clippy::unnecessary_unwrap)]
			let mut items = self.rest.unwrap();
			#[allow(clippy::unnecessary_unwrap)]
			items.push(Indexed::none(self.first.unwrap()));
			items.reverse();
			Ok(Object::List(List::new(items)))
		} else {
			if let Some(item) = self.first {
				let iri = self.vocabulary.insert(RDF_FIRST);
				self.properties
					.insert(json_ld_core::Id::Valid(Id::Iri(iri)), Indexed::none(item))
			}

			if let Some(rest) = self.rest {
				let iri = self.vocabulary.insert(RDF_REST);
				self.properties.insert(
					json_ld_core::Id::Valid(Id::Iri(iri)),
					Indexed::none(Object::List(List::new(rest))),
				)
			}

			let mut node = Node::new();

			if !self.types.is_empty() {
				node.types = Some(self.types)
			}

			*node.properties_mut() = self.properties;

			if !self.reverse_properties.is_empty() {
				node.set_reverse_properties(Some(self.reverse_properties));
			}

			if !self.included.is_empty() {
				node.set_included(Some(self.included));
			}

			node.graph = self.graph;

			Ok(Object::node(node))
		}
	}
}
