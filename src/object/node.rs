use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::convert::TryFrom;
use std::fmt;
use iref::Iri;
use json::JsonValue;
use crate::{
	Id,
	Keyword,
	Term,
	Type,
	Property,
	Object,
	Indexed,
	util
};

/// A node object.
#[derive(PartialEq, Eq)]
pub struct Node<T: Id> {
	pub(crate) id: Option<Term<T>>,
	pub(crate) types: Vec<Type<T>>,
	pub(crate) included: Option<HashSet<Indexed<Object<T>>>>,
	pub(crate) expanded_property: Option<Term<T>>,
	pub(crate) properties: HashMap<Property<T>, Vec<Indexed<Object<T>>>>,
	pub(crate) reverse_properties: HashMap<Property<T>, Vec<Indexed<Object<T>>>>
}

pub struct Objects<'a, T: Id>(Option<std::slice::Iter<'a, Indexed<Object<T>>>>);

impl<'a, T: Id> Iterator for Objects<'a, T> {
	type Item = &'a Indexed<Object<T>>;

	fn next(&mut self) -> Option<&'a Indexed<Object<T>>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next()
		}
	}
}

impl<T: Id> Node<T> {
	pub fn new() -> Node<T> {
		Node {
			id: None,
			types: Vec::new(),
			included: None,
			expanded_property: None,
			properties: HashMap::new(),
			reverse_properties: HashMap::new()
		}
	}

	pub fn types(&self) -> &[Type<T>] {
		self.types.as_ref()
	}

	/// Test if the node is empty.
	///
	/// It is empty is every field except for `@id` is empty.
	pub fn is_empty(&self) -> bool {
		self.types.is_empty()
		&& self.included.is_none()
		&& self.expanded_property.is_none()
		&& self.properties.is_empty()
		&& self.reverse_properties.is_empty()
	}

	pub fn as_iri(&self) -> Option<Iri> {
		if let Some(id) = &self.id {
			id.iri()
		} else {
			None
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self.as_iri() {
			Some(iri) => Some(iri.into_str()),
			None => None
		}
	}

	pub fn get(&self, prop: &Property<T>) -> Objects<T> {
		match self.properties.get(prop) {
			Some(values) => Objects(Some(values.iter())),
			None => Objects(None)
		}
	}

	pub fn insert(&mut self, prop: Property<T>, value: Object<T>) {
		if let Some(node_values) = self.properties.get_mut(&prop) {
			node_values.push(value);
		} else {
			let mut node_values = Vec::new();
			node_values.push(value);
			self.properties.insert(prop, node_values);
		}
	}

	pub fn insert_all<Objects: Iterator<Item=Object<T>>>(&mut self, prop: Property<T>, values: Objects) {
		if let Some(node_values) = self.properties.get_mut(&prop) {
			node_values.extend(values);
		} else {
			self.properties.insert(prop, values.collect());
		}
	}

	pub fn insert_reverse(&mut self, reverse_prop: Property<T>, reverse_value: Object<T>) {
		if let Some(node_values) = self.reverse_properties.get_mut(&reverse_prop) {
			node_values.push(reverse_value);
		} else {
			let mut node_values = Vec::new();
			node_values.push(reverse_value);
			self.reverse_properties.insert(reverse_prop, node_values);
		}
	}

	pub fn insert_all_reverse<Objects: Iterator<Item=Object<T>>>(&mut self, reverse_prop: Property<T>, reverse_values: Objects) {
		if let Some(node_values) = self.reverse_properties.get_mut(&reverse_prop) {
			node_values.extend(reverse_values);
		} else {
			self.reverse_properties.insert(reverse_prop, reverse_values.collect());
		}
	}
}

impl<T: Id> Hash for Node<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.id.hash(h);
		self.types.hash(h);
		util::hash_set_opt(&self.included, h);
		self.expanded_property.hash(h);
		util::hash_map(&self.properties, h);
		util::hash_map(&self.reverse_properties, h);
	}
}

impl<T: Id> util::AsJson for Node<T> {
	fn as_json(&self) -> JsonValue {
		let mut obj = json::object::Object::new();

		if let Some(id) = &self.id {
			obj.insert(Keyword::Id.into(), id.as_json());
		}

		if !self.types.is_empty() {
			obj.insert(Keyword::Type.into(), self.types.as_json())
		}

		if let Some(included) = &self.included {
			obj.insert(Keyword::Included.into(), included.as_json())
		}

		if !self.reverse_properties.is_empty() {
			let mut reverse = json::object::Object::new();
			for (key, value) in &self.reverse_properties {
				reverse.insert(key.as_str(), value.as_json())
			}

			obj.insert(Keyword::Reverse.into(), JsonValue::Object(reverse))
		}

		for (key, value) in &self.properties {
			obj.insert(key.as_str(), value.as_json())
		}

		JsonValue::Object(obj)
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type<T: Id> {
	Id,
	JSON,
	None,
	Vocab,
	Prop(Property<T>),
	Unknown(String)
}

impl<T: Id> Type<T> {
	pub fn as_str(&self) -> &str {
		match self {
			Type::Id => "@id",
			Type::JSON => "@json",
			Type::None => "@none",
			Type::Vocab => "@vocab",
			Type::Prop(p) => p.as_str(),
			Type::Unknown(s) => s.as_str()
		}
	}
}

impl<T: Id> TryFrom<Term<T>> for Type<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<Type<T>, Term<T>> {
		match term {
			Term::Keyword(Keyword::Id) => Ok(Type::Id),
			Term::Keyword(Keyword::JSON) => Ok(Type::JSON),
			Term::Keyword(Keyword::None) => Ok(Type::None),
			Term::Keyword(Keyword::Vocab) => Ok(Type::Vocab),
			Term::Prop(prop) => Ok(Type::Prop(prop)),
			Term::Unknown(name) => {
				Ok(Type::Unknown(name))
			},
			term => Err(term)
		}
	}
}

impl<T: Id> util::AsJson for Type<T> {
	fn as_json(&self) -> JsonValue {
		self.as_str().into()
	}
}

impl<T: Id> fmt::Display for Type<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
