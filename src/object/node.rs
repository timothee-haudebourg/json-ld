use crate::{
	object,
	syntax::{Keyword, Term},
	util, Id, Indexed, Object, Reference, ToReference,
};
use cc_traits::MapInsert;
use generic_json::{JsonClone, JsonHash};
use iref::{Iri, IriBuf};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

/// A node object.
///
/// A node is defined by its identifier (`@id` field), types, properties and reverse properties.
/// In addition, a node may represent a graph (`@graph field`) and includes nodes
/// (`@included` field).
// NOTE it may be better to use BTreeSet instead of HashSet to have some ordering?
//      in which case the Json bound should be lifted.
#[derive(PartialEq, Eq)]
pub struct Node<J: JsonHash, T: Id = IriBuf> {
	/// Identifier.
	///
	/// This is the `@id` field.
	pub(crate) id: Option<Reference<T>>,

	/// Types.
	///
	/// This is the `@type` field.
	pub(crate) types: Vec<Reference<T>>,

	/// Associated graph.
	///
	/// This is the `@graph` field.
	pub(crate) graph: Option<HashSet<Indexed<Object<J, T>>>>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	pub(crate) included: Option<HashSet<Indexed<Self>>>,

	/// Properties.
	///
	/// Any non-keyword field.
	pub(crate) properties: HashMap<Reference<T>, Vec<Indexed<Object<J, T>>>>,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	pub(crate) reverse_properties: HashMap<Reference<T>, Vec<Indexed<Self>>>,
}

/// Iterator through indexed objects.
pub struct Objects<'a, J: JsonHash, T: Id>(Option<std::slice::Iter<'a, Indexed<Object<J, T>>>>);

impl<'a, J: JsonHash, T: Id> Iterator for Objects<'a, J, T> {
	type Item = &'a Indexed<Object<J, T>>;

	fn next(&mut self) -> Option<&'a Indexed<Object<J, T>>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}

impl<J: JsonHash, T: Id> Default for Node<J, T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<J: JsonHash, T: Id> Node<J, T> {
	/// Create a new empty node.
	pub fn new() -> Self {
		Self {
			id: None,
			types: Vec::new(),
			graph: None,
			included: None,
			properties: HashMap::new(),
			reverse_properties: HashMap::new(),
		}
	}

	/// Create a new empty node with the given id.
	pub fn with_id(id: Reference<T>) -> Self {
		Self {
			id: Some(id),
			types: Vec::new(),
			graph: None,
			included: None,
			properties: HashMap::new(),
			reverse_properties: HashMap::new(),
		}
	}

	/// Checks if the node object has the given term as key.
	///
	/// # Example
	/// ```
	/// # use json_ld::syntax::{Term, Keyword};
	/// # let node: json_ld::Node = json_ld::Node::new();
	///
	/// // Checks if the JSON object representation of the node has an `@id` key.
	/// if node.has_key(&Term::Keyword(Keyword::Id)) {
	///   // ...
	/// }
	/// ```
	pub fn has_key(&self, key: &Term<T>) -> bool {
		match key {
			Term::Keyword(Keyword::Id) => self.id.is_some(),
			Term::Keyword(Keyword::Type) => !self.types.is_empty(),
			Term::Keyword(Keyword::Graph) => self.graph.is_some(),
			Term::Keyword(Keyword::Included) => self.included.is_some(),
			Term::Keyword(Keyword::Reverse) => !self.reverse_properties.is_empty(),
			Term::Ref(prop) => self.properties.get(prop).is_some(),
			_ => false,
		}
	}

	/// Get the identifier of the node.
	///
	/// This correspond to the `@id` field of the JSON object.
	pub fn id(&self) -> Option<&Reference<T>> {
		self.id.as_ref()
	}

	/// Get the node's as an IRI if possible.
	///
	/// Returns the node's IRI id if any. Returns `None` otherwise.
	pub fn as_iri(&self) -> Option<Iri> {
		if let Some(id) = &self.id {
			id.as_iri()
		} else {
			None
		}
	}

	/// Get the node's id, is any, as a string slice.
	///
	/// Returns `None` if the node has no `@id` field.
	pub fn as_str(&self) -> Option<&str> {
		match self.as_iri() {
			Some(iri) => Some(iri.into_str()),
			None => None,
		}
	}

	/// Get the list of the node's types.
	///
	/// This returns a list of `Lenient` types, including malformed types that are not
	/// IRIs of blank node identifiers.
	pub fn types(&self) -> &[Reference<T>] {
		self.types.as_ref()
	}

	/// Checks if the node has the given type.
	pub fn has_type<U>(&self, ty: &U) -> bool
	where
		Reference<T>: PartialEq<U>,
	{
		for self_ty in &self.types {
			if self_ty == ty {
				return true;
			}
		}

		false
	}

	/// Tests if the node is empty.
	///
	/// It is empty is every field other than `@id` is empty.
	pub fn is_empty(&self) -> bool {
		self.types.is_empty()
			&& self.graph.is_none()
			&& self.included.is_none()
			&& self.properties.is_empty()
			&& self.reverse_properties.is_empty()
	}

	/// Tests if the node is a graph object (has a `@graph` field, and optionally an `@id` field).
	/// Note that node objects may have a @graph entry,
	/// but are not considered graph objects if they include any other entries other than `@id`.
	pub fn is_graph(&self) -> bool {
		self.graph.is_some()
			&& self.types.is_empty()
			&& self.included.is_none()
			&& self.properties.is_empty()
			&& self.reverse_properties.is_empty()
	}

	/// Tests if the node is a simple graph object (a graph object without `@id` field)
	pub fn is_simple_graph(&self) -> bool {
		self.id.is_none() && self.is_graph()
	}

	/// If the node is a graph object, get the graph.
	pub fn graph(&self) -> Option<&HashSet<Indexed<Object<J, T>>>> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	pub fn graph_mut(&mut self) -> Option<&mut HashSet<Indexed<Object<J, T>>>> {
		self.graph.as_mut()
	}

	/// Set the graph.
	pub fn set_graph(&mut self, graph: Option<HashSet<Indexed<Object<J, T>>>>) {
		self.graph = graph
	}

	/// Get the set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	pub fn included(&self) -> Option<&HashSet<Indexed<Self>>> {
		self.included.as_ref()
	}

	/// Get the mutable set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	pub fn included_mut(&mut self) -> Option<&mut HashSet<Indexed<Self>>> {
		self.included.as_mut()
	}

	/// Set the set of nodes included by the node.
	pub fn set_included(&mut self, included: Option<HashSet<Indexed<Self>>>) {
		self.included = included
	}

	/// Get all the objects associated to the node with the given property.
	pub fn get<'a, Q: ToReference<T>>(&self, prop: Q) -> Objects<J, T>
	where
		T: 'a,
	{
		match self.properties.get(prop.to_ref().borrow()) {
			Some(values) => Objects(Some(values.iter())),
			None => Objects(None),
		}
	}

	/// Get one of the objects associated to the node with the given property.
	///
	/// If multiple objects are attaced to the node with this property, there are no guaranties
	/// on which object will be returned.
	pub fn get_any<'a, Q: ToReference<T>>(&self, prop: Q) -> Option<&Indexed<Object<J, T>>>
	where
		T: 'a,
	{
		match self.properties.get(prop.to_ref().borrow()) {
			Some(values) => values.iter().next(),
			None => None,
		}
	}

	/// Associate the given object to the node through the given property.
	pub fn insert(&mut self, prop: Reference<T>, value: Indexed<Object<J, T>>) {
		if let Some(node_values) = self.properties.get_mut(&prop) {
			node_values.push(value);
		} else {
			let node_values = vec![value];
			self.properties.insert(prop, node_values);
		}
	}

	/// Associate all the given objects to the node through the given property.
	pub fn insert_all<Objects: Iterator<Item = Indexed<Object<J, T>>>>(
		&mut self,
		prop: Reference<T>,
		values: Objects,
	) {
		if let Some(node_values) = self.properties.get_mut(&prop) {
			node_values.extend(values);
		} else {
			self.properties.insert(prop, values.collect());
		}
	}

	pub fn insert_reverse(&mut self, reverse_prop: Reference<T>, reverse_value: Indexed<Self>) {
		if let Some(node_values) = self.reverse_properties.get_mut(&reverse_prop) {
			node_values.push(reverse_value);
		} else {
			let node_values = vec![reverse_value];
			self.reverse_properties.insert(reverse_prop, node_values);
		}
	}

	pub fn insert_all_reverse<Nodes: Iterator<Item = Indexed<Self>>>(
		&mut self,
		reverse_prop: Reference<T>,
		reverse_values: Nodes,
	) {
		if let Some(node_values) = self.reverse_properties.get_mut(&reverse_prop) {
			node_values.extend(reverse_values);
		} else {
			self.reverse_properties
				.insert(reverse_prop, reverse_values.collect());
		}
	}

	/// Tests if the node is an unnamed graph object.
	///
	/// Returns `true` is the only field of the object is a `@graph` field.
	/// Returns `false` otherwise.
	pub fn is_unnamed_graph(&self) -> bool {
		self.graph.is_some()
			&& self.id.is_none()
			&& self.types.is_empty()
			&& self.included.is_none()
			&& self.properties.is_empty()
			&& self.reverse_properties.is_empty()
	}

	/// Returns the node as an unnamed graph, if it is one.
	///
	/// The unnamed graph is returned as a set of indexed objects.
	/// Fails and returns itself if the node is *not* an unnamed graph.
	pub fn into_unnamed_graph(self) -> Result<HashSet<Indexed<Object<J, T>>>, Self> {
		if self.is_unnamed_graph() {
			Ok(self.graph.unwrap())
		} else {
			Err(self)
		}
	}
}

impl<J: JsonHash, T: Id> object::Any<J, T> for Node<J, T> {
	fn as_ref(&self) -> object::Ref<J, T> {
		object::Ref::Node(self)
	}
}

impl<J: JsonHash, T: Id> TryFrom<Object<J, T>> for Node<J, T> {
	type Error = Object<J, T>;

	fn try_from(obj: Object<J, T>) -> Result<Node<J, T>, Object<J, T>> {
		match obj {
			Object::Node(node) => Ok(node),
			obj => Err(obj),
		}
	}
}

impl<J: JsonHash, T: Id> Hash for Node<J, T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.id.hash(h);
		self.types.hash(h);
		util::hash_set_opt(&self.graph, h);
		util::hash_set_opt(&self.included, h);
		util::hash_map(&self.properties, h);
		util::hash_map(&self.reverse_properties, h);
	}
}

impl<J: JsonHash + JsonClone, K: util::JsonFrom<J>, T: Id> util::AsJson<J, K> for Node<J, T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		let mut obj = K::Object::default();

		if let Some(id) = &self.id {
			obj.insert(K::new_key(Keyword::Id.into_str(), meta(None)), id.as_json_with(meta.clone()));
		}

		if !self.types.is_empty() {
			obj.insert(
				K::new_key(Keyword::Type.into_str(), meta(None)),
				self.types.as_json_with(meta.clone()),
			);
		}

		if let Some(graph) = &self.graph {
			obj.insert(
				K::new_key(Keyword::Graph.into_str(), meta(None)),
				graph.as_json_with(meta.clone()),
			);
		}

		if let Some(included) = &self.included {
			obj.insert(
				K::new_key(Keyword::Included.into_str(), meta(None)),
				included.as_json_with(meta.clone()),
			);
		}

		if !self.reverse_properties.is_empty() {
			let mut reverse = K::Object::default();
			for (key, value) in &self.reverse_properties {
				reverse.insert(
					K::new_key(key.as_str(), meta(None)),
					value.as_json_with(meta.clone())
				);
			}

			obj.insert(
				K::new_key(Keyword::Reverse.into_str(), meta(None)),
				K::object(reverse, meta(None)),
			);
		}

		for (key, value) in &self.properties {
			obj.insert(
				K::new_key(key.as_str(), meta(None)),
				value.as_json_with(meta.clone())
			);
		}

		K::object(obj, meta(None))
	}
}
