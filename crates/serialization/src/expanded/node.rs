use iref::Iri;
use json_ld_core::{object::node::Multiset, Indexed, Node, Object};
use linked_data::{CowRdfTerm, LinkedDataResource};
use rdf_types::{
	interpretation::{
		ReverseBlankIdInterpretation, ReverseIriInterpretation, ReverseLiteralInterpretation,
	},
	vocabulary::{IriVocabulary, IriVocabularyMut},
	Interpretation, Term, Vocabulary, RDF_TYPE,
};
use std::hash::Hash;

use crate::Error;

use super::{
	graph::SerializeGraph,
	property::{SerializeProperty, SerializeReverseProperty},
};

/// Serialize the given Linked-Data value into a JSON-LD node object using a
/// custom vocabulary and interpretation.
pub fn serialize_node_with<I: Interpretation, V: Vocabulary, T>(
	vocabulary: &mut V,
	interpretation: &mut I,
	value: &T,
) -> Result<Node<V::Iri, V::BlankId>, Error>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
	T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
{
	let id = match value
		.lexical_representation(vocabulary, interpretation)
		.map(CowRdfTerm::into_owned)
	{
		Some(Term::Literal(_)) => return Err(Error::InvalidNode),
		Some(Term::Id(id)) => Some(json_ld_core::Id::Valid(id)),
		None => None,
	};

	let serializer = SerializeNode::new(vocabulary, interpretation, id);

	value.visit_subject(serializer)
}

pub struct SerializeNode<'a, I, V: Vocabulary> {
	vocabulary: &'a mut V,
	interpretation: &'a mut I,
	result: Node<V::Iri, V::BlankId>,
}

impl<'a, I, V: Vocabulary> SerializeNode<'a, I, V> {
	pub fn new(
		vocabulary: &'a mut V,
		interpretation: &'a mut I,
		id: Option<json_ld_core::Id<V::Iri, V::BlankId>>,
	) -> Self {
		let result = match id {
			Some(id) => Node::with_id(id),
			None => Node::new(),
		};

		Self {
			vocabulary,
			interpretation,
			result,
		}
	}
}

impl<'a, I: Interpretation, V: Vocabulary> linked_data::SubjectVisitor<I, V>
	for SerializeNode<'a, I, V>
where
	V: IriVocabularyMut,
	V::Iri: Clone + Eq + Hash,
	V::BlankId: Clone + Eq + Hash,
	I: ReverseIriInterpretation<Iri = V::Iri>
		+ ReverseBlankIdInterpretation<BlankId = V::BlankId>
		+ ReverseLiteralInterpretation<Literal = V::Literal>,
{
	type Ok = Node<V::Iri, V::BlankId>;
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
			Some(Term::Id(id)) => json_ld_core::Id::Valid(id),
			_ => return Err(Error::InvalidPredicate),
		};

		let serializer = SerializeProperty::new(self.vocabulary, self.interpretation);

		let objects = value.visit_objects(serializer)?;

		if is_iri(self.vocabulary, &prop, RDF_TYPE) {
			let mut non_iri_objects = Multiset::new();

			for obj in objects {
				match into_type_value(obj) {
					Ok(ty) => self.result.types_mut_or_default().push(ty),
					Err(obj) => {
						non_iri_objects.insert(obj);
					}
				}
			}

			if !non_iri_objects.is_empty() {
				self.result.properties_mut().set(prop, non_iri_objects);
			}
		} else {
			self.result.properties_mut().set(prop, objects);
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
		self.result
			.reverse_properties_mut_or_default()
			.set(prop, objects);

		Ok(())
	}

	fn include<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + LinkedDataResource<I, V> + linked_data::LinkedDataSubject<I, V>,
	{
		let node = serialize_node_with(self.vocabulary, self.interpretation, value)?;

		self.result
			.included_mut_or_default()
			.insert(Indexed::none(node));
		Ok(())
	}

	fn graph<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + linked_data::LinkedDataGraph<I, V>,
	{
		let serializer = SerializeGraph::new(self.vocabulary, self.interpretation);

		let graph = value.visit_graph(serializer)?;
		self.result.graph = Some(graph);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(self.result)
	}
}

pub(crate) fn into_type_value<I, B>(
	obj: Indexed<Object<I, B>>,
) -> Result<json_ld_core::Id<I, B>, Indexed<Object<I, B>>> {
	match obj.index() {
		Some(_) => Err(obj),
		None => match obj.into_inner() {
			Object::Node(node) => {
				if node.is_empty() && node.id.is_some() {
					Ok(node.id.unwrap())
				} else {
					Err(Indexed::none(Object::Node(node)))
				}
			}
			obj => Err(Indexed::none(obj)),
		},
	}
}

pub(crate) fn is_iri<V, B>(vocabulary: &V, id: &json_ld_core::Id<V::Iri, B>, iri: &Iri) -> bool
where
	V: IriVocabulary,
{
	match id {
		json_ld_core::Id::Valid(rdf_types::Id::Iri(i)) => match vocabulary.iri(i) {
			Some(i) => i == iri,
			None => false,
		},
		_ => false,
	}
}
