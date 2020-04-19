use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use iref::Iri;
use crate::{util, Id, Term, NodeType, Property, Object};

#[derive(PartialEq, Eq)]
pub struct Node<T: Id> {
	pub(crate) id: Option<Term<T>>,
	pub(crate) types: Vec<NodeType<T>>,
	pub(crate) graph: Option<HashSet<Object<T>>>,
	pub(crate) included: Option<HashSet<Object<T>>>,
	// pub(crate) language: Option<String>,
	// pub(crate) direction: Option<Direction>,
	pub(crate) expanded_property: Option<Term<T>>,
	pub(crate) properties: HashMap<Property<T>, Vec<Object<T>>>,
	pub(crate) reverse_properties: HashMap<Property<T>, Vec<Object<T>>>
}

pub struct Objects<'a, T: Id>(Option<std::slice::Iter<'a, Object<T>>>);

impl<'a, T: Id> Iterator for Objects<'a, T> {
	type Item = &'a Object<T>;

	fn next(&mut self) -> Option<&'a Object<T>> {
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
			graph: None,
			included: None,
			// language: None,
			// direction: None,
			expanded_property: None,
			properties: HashMap::new(),
			reverse_properties: HashMap::new()
		}
	}

	pub fn types(&self) -> &[NodeType<T>] {
		self.types.as_ref()
	}

	/// Test if the node is empty.
	///
	/// It is empty is every field except for `@id` is empty.
	pub fn is_empty(&self) -> bool {
		self.types.is_empty()
		&& self.graph.is_none()
		&& self.included.is_none()
		// && self.language.is_none()
		// && self.direction.is_none()
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

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self) -> Result<HashSet<Object<T>>, Self> {
		if self.id.is_none()
			&& self.types.is_empty()
			&& self.graph.is_some()
			&& self.included.is_none()
			// && self.language.is_none()
			// && self.direction.is_none()
			&& self.expanded_property.is_none()
			&& self.properties.is_empty()
			&& self.reverse_properties.is_empty() {
			Ok(self.graph.unwrap())
		} else {
			Err(self)
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
		util::hash_set_opt(&self.graph, h);
		util::hash_set_opt(&self.included, h);
		// self.language.hash(h);
		// self.direction.hash(h);
		self.expanded_property.hash(h);
		util::hash_map(&self.properties, h);
		util::hash_map(&self.reverse_properties, h);
	}
}
