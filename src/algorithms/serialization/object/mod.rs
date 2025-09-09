use std::collections::HashMap;

use linked_data::DeserializeLinkedData;
use rdf_types::{Quad, Term, RDF_FIRST, RDF_LIST, RDF_NIL, RDF_REST, RDF_TYPE};

use crate::{
	object::{node::Properties, ListObject},
	Id, Indexed, NodeObject, Object, ValueObject,
};

pub enum ProtoObject {
	Node(ProtoNodeObject),
	Value(ValueObject),
}

pub struct ProtoNodeObject {
	pub id: Id,
	pub types: Vec<Id>,
	pub properties: Properties,
}

pub enum ProtoList {
	Nil,
	Node {
		first: Indexed<Object>,
		rest: Indexed<Object>,
	},
}

impl ProtoNodeObject {
	fn new(id: rdf_types::Id) -> Self {
		Self {
			id: id.into(),
			types: Vec::new(),
			properties: Properties::default(),
		}
	}

	pub fn is_list_nil(&self) -> bool {
		(self.types.is_empty()
			|| (self.types.len() == 1 && self.types.first().unwrap() == RDF_LIST))
			&& self.properties.is_empty()
			&& self.id.as_iri() == Some(RDF_NIL)
	}

	pub fn is_list_node(&self) -> bool {
		self.types.len() == 1
			&& self.properties.len() == 2
			&& self.types.first().unwrap() == RDF_LIST
			&& self.properties.count(RDF_FIRST) == 1
			&& self.properties.count(RDF_REST) == 1
	}

	pub fn is_list(&self) -> bool {
		self.is_list_nil() || self.is_list_node()
	}

	pub fn validate_list(
		&self,
		objects: &HashMap<Id, ProtoNodeObject>,
		usages: &HashMap<&Id, usize>,
	) -> bool {
		// We can't use a list object in more that one place.
		if !self.is_list() || *usages.get(&self.id).unwrap() > 1 {
			return false;
		}

		let Object::Node(rest) = self.properties.get_any(RDF_REST).unwrap().inner() else {
			return false;
		};

		let Some(rest) = objects.get(rest.id.as_ref().unwrap()) else {
			return false;
		};

		rest.validate_list(objects, usages)
	}

	pub fn into_list(mut self, objects: &mut HashMap<Id, ProtoNodeObject>) -> ListObject {
		let mut list = self.into_reverse_list(objects);
		list.entry_mut().reverse();
		list
	}

	pub fn into_reverse_list(mut self, objects: &mut HashMap<Id, ProtoNodeObject>) -> ListObject {
		if self.is_list_nil() {
			return ListObject::new(Vec::new());
		}

		let first = self
			.properties
			.remove(RDF_REST)
			.unwrap()
			.into_iter()
			.next()
			.unwrap();

		let rest_id = self
			.properties
			.remove(RDF_REST)
			.unwrap()
			.into_iter()
			.next()
			.unwrap()
			.into_inner()
			.into_node()
			.unwrap()
			.id
			.unwrap();

		let rest = objects.remove(&rest_id).unwrap();

		let mut list = rest.into_reverse_list(objects);

		list.push(first);

		list
	}

	// pub fn as_list(&self) -> Option<ProtoList> {
	// 	if self.is_list_nil() {
	// 		Some(ProtoList::Nil)
	// 	} else if self.is_list_node() {
	// 		Some(ProtoList::Node {
	// 			first: self.properties.get_any(RDF_FIRST).unwrap().clone(),
	// 			rest: self.properties.get_any(RDF_REST).unwrap().clone(),
	// 		})
	// 	} else {
	// 		None
	// 	}
	// }

	pub fn referenced_nodes(&self) -> impl Iterator<Item = &Id> {
		self.types.iter().chain(
			self.properties
				.iter()
				.flat_map(|(_, objects)| objects.iter().filter_map(|o| o.id())),
		)
	}
}

impl DeserializeLinkedData for ProtoObject {
	fn deserialize_rdf<D>(mut deserializer: D, graph: Option<&Term>) -> Result<Self, D::Error>
	where
		D: linked_data::LinkedDataDeserializer,
	{
		let subject = match deserializer.deserialize_resource(graph)? {
			Some(subject) => {
				if let Some(other) = deserializer.deserialize_resource(graph)? {
					return Err(linked_data::de::Error::unexpected_resource(other));
				}

				subject
			}
			None => return Err(linked_data::de::Error::missing_resource()),
		};

		match subject.into_id() {
			Ok(id) => {
				let mut result = ProtoNodeObject::new(id.clone());
				let subject: Term = id.into();

				// Extract type.
				let ty_prop = Term::iri(RDF_TYPE.to_owned());
				while let Some(Quad(_, _, ty, _)) = deserializer.deserialize_quad(Quad(
					Some(&subject),
					Some(&ty_prop),
					None,
					Some(graph),
				))? {
					if let Ok(ty) = ty.into_id() {
						result.types.push(ty.into());
					}
				}

				// Extract properties.
				while let Some(Quad(_, property, object, _)) =
					deserializer.deserialize_quad(Quad(Some(&subject), None, None, Some(graph)))?
				{
					if let Ok(property) = property.into_id() {
						let value = match object.into_id() {
							Ok(id) => Object::node(NodeObject::new_with_id(Some(id.into()))),
							Err(literal) => Object::Value(literal.into()),
						};

						result
							.properties
							.insert(property, Indexed::unindexed(value));
					}
				}

				Ok(Self::Node(result))
			}
			Err(literal) => Ok(Self::Value(literal.into())),
		}
	}
}
