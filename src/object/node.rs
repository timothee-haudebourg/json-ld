use crate::{
	object,
	syntax::{Keyword, Term},
	util, Id, Indexed, Object, Objects, Reference, ToReference,
};
use cc_traits::MapInsert;
use generic_json::{JsonClone, JsonHash};
use iref::{Iri, IriBuf};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

pub mod properties;
pub mod reverse_properties;

pub use properties::Properties;
pub use reverse_properties::ReverseProperties;

/// Node parts.
pub struct Parts<J: JsonHash, T: Id = IriBuf> {
	/// Identifier.
	///
	/// This is the `@id` field.
	pub id: Option<Reference<T>>,

	/// Types.
	///
	/// This is the `@type` field.
	pub types: Vec<Reference<T>>,

	/// Associated graph.
	///
	/// This is the `@graph` field.
	pub graph: Option<HashSet<Indexed<Object<J, T>>>>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	pub included: Option<HashSet<Indexed<Node<J, T>>>>,

	/// Properties.
	///
	/// Any non-keyword field.
	pub properties: Properties<J, T>,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	pub reverse_properties: ReverseProperties<J, T>,
}

/// Node object.
///
/// A node object represents zero or more properties of a node in the graph serialized by a JSON-LD document.
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
	pub(crate) properties: Properties<J, T>,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	pub(crate) reverse_properties: ReverseProperties<J, T>,
}

impl<J: JsonHash, T: Id> Default for Node<J, T> {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl<J: JsonHash, T: Id> Node<J, T> {
	/// Creates a new empty node.
	#[inline(always)]
	pub fn new() -> Self {
		Self {
			id: None,
			types: Vec::new(),
			graph: None,
			included: None,
			properties: Properties::new(),
			reverse_properties: ReverseProperties::new(),
		}
	}

	/// Creates a new empty node with the given id.
	#[inline(always)]
	pub fn with_id(id: Reference<T>) -> Self {
		Self {
			id: Some(id),
			types: Vec::new(),
			graph: None,
			included: None,
			properties: Properties::new(),
			reverse_properties: ReverseProperties::new(),
		}
	}

	pub fn from_parts(parts: Parts<J, T>) -> Self {
		Self {
			id: parts.id,
			types: parts.types,
			graph: parts.graph,
			included: parts.included,
			properties: parts.properties,
			reverse_properties: parts.reverse_properties,
		}
	}

	pub fn into_parts(self) -> Parts<J, T> {
		Parts {
			id: self.id,
			types: self.types,
			graph: self.graph,
			included: self.included,
			properties: self.properties,
			reverse_properties: self.reverse_properties,
		}
	}

	/// Checks if the node object has the given term as key.
	///
	/// # Example
	/// ```
	/// # use json_ld::syntax::{Term, Keyword};
	/// # let node: json_ld::Node<ijson::IValue> = json_ld::Node::new();
	///
	/// // Checks if the JSON object representation of the node has an `@id` key.
	/// if node.has_key(&Term::Keyword(Keyword::Id)) {
	///   // ...
	/// }
	/// ```
	#[inline(always)]
	pub fn has_key(&self, key: &Term<T>) -> bool {
		match key {
			Term::Keyword(Keyword::Id) => self.id.is_some(),
			Term::Keyword(Keyword::Type) => !self.types.is_empty(),
			Term::Keyword(Keyword::Graph) => self.graph.is_some(),
			Term::Keyword(Keyword::Included) => self.included.is_some(),
			Term::Keyword(Keyword::Reverse) => !self.reverse_properties.is_empty(),
			Term::Ref(prop) => self.properties.contains(prop),
			_ => false,
		}
	}

	/// Get the identifier of the node.
	///
	/// This correspond to the `@id` field of the JSON object.
	#[inline(always)]
	pub fn id(&self) -> Option<&Reference<T>> {
		self.id.as_ref()
	}

	/// Sets the idntifier of this node.
	#[inline(always)]
	pub fn set_id(&mut self, id: Option<Reference<T>>) {
		self.id = id
	}

	/// Get the node's as an IRI if possible.
	///
	/// Returns the node's IRI id if any. Returns `None` otherwise.
	#[inline(always)]
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
	#[inline(always)]
	pub fn as_str(&self) -> Option<&str> {
		match self.as_iri() {
			Some(iri) => Some(iri.into_str()),
			None => None,
		}
	}

	/// Get the list of the node's types.
	#[inline(always)]
	pub fn types(&self) -> &[Reference<T>] {
		self.types.as_ref()
	}

	/// Returns a mutable reference to the node's types.
	#[inline(always)]
	pub fn types_mut(&mut self) -> &mut Vec<Reference<T>> {
		&mut self.types
	}

	/// Adds the given type `ty` to the end of the list of types of this node.
	#[inline(always)]
	pub fn add_type(&mut self, ty: Reference<T>) {
		self.types.push(ty)
	}

	/// Sets the types of this node.
	#[inline(always)]
	pub fn set_types(&mut self, types: Vec<Reference<T>>) {
		self.types = types
	}

	/// Checks if the node has the given type.
	#[inline]
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
	#[inline]
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
	#[inline]
	pub fn is_graph(&self) -> bool {
		self.graph.is_some()
			&& self.types.is_empty()
			&& self.included.is_none()
			&& self.properties.is_empty()
			&& self.reverse_properties.is_empty()
	}

	/// Tests if the node is a simple graph object (a graph object without `@id` field)
	#[inline(always)]
	pub fn is_simple_graph(&self) -> bool {
		self.id.is_none() && self.is_graph()
	}

	/// If the node is a graph object, get the graph.
	#[inline(always)]
	pub fn graph(&self) -> Option<&HashSet<Indexed<Object<J, T>>>> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_mut(&mut self) -> Option<&mut HashSet<Indexed<Object<J, T>>>> {
		self.graph.as_mut()
	}

	/// Set the graph.
	#[inline(always)]
	pub fn set_graph(&mut self, graph: Option<HashSet<Indexed<Object<J, T>>>>) {
		self.graph = graph
	}

	/// Get the set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included(&self) -> Option<&HashSet<Indexed<Self>>> {
		self.included.as_ref()
	}

	/// Get the mutable set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_mut(&mut self) -> Option<&mut HashSet<Indexed<Self>>> {
		self.included.as_mut()
	}

	/// Set the set of nodes included by the node.
	#[inline(always)]
	pub fn set_included(&mut self, included: Option<HashSet<Indexed<Self>>>) {
		self.included = included
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn properties(&self) -> &Properties<J, T> {
		&self.properties
	}

	/// Returns a mutable reference to the properties of the node.
	#[inline(always)]
	pub fn properties_mut(&mut self) -> &mut Properties<J, T> {
		&mut self.properties
	}

	/// Returns a reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties(&self) -> &ReverseProperties<J, T> {
		&self.reverse_properties
	}

	/// Returns a mutable reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_mut(&mut self) -> &mut ReverseProperties<J, T> {
		&mut self.reverse_properties
	}

	/// Get all the objects associated to the node with the given property.
	#[inline(always)]
	pub fn get<'a, Q: ToReference<T>>(&self, prop: Q) -> Objects<J, T>
	where
		T: 'a,
	{
		self.properties.get(prop)
	}

	/// Get one of the objects associated to the node with the given property.
	///
	/// If multiple objects are attached to the node with this property, there are no guaranties
	/// on which object will be returned.
	#[inline(always)]
	pub fn get_any<'a, Q: ToReference<T>>(&self, prop: Q) -> Option<&Indexed<Object<J, T>>>
	where
		T: 'a,
	{
		self.properties.get_any(prop)
	}

	/// Associates the given object to the node through the given property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Reference<T>, value: Indexed<Object<J, T>>) {
		self.properties.insert(prop, value)
	}

	/// Associates all the given objects to the node through the given property.
	///
	/// If there already exists objects associated to the given reverse property,
	/// `reverse_value` is added to the list. Duplicate objects are not removed.
	#[inline(always)]
	pub fn insert_all<Objects: Iterator<Item = Indexed<Object<J, T>>>>(
		&mut self,
		prop: Reference<T>,
		values: Objects,
	) {
		self.properties.insert_all(prop, values)
	}

	/// Associates the given node to the reverse property.
	///
	/// If there already exists nodes associated to the given reverse property,
	/// `reverse_value` is added to the list. Duplicate nodes are not removed.
	#[inline(always)]
	pub fn insert_reverse(&mut self, reverse_prop: Reference<T>, reverse_value: Indexed<Self>) {
		self.reverse_properties.insert(reverse_prop, reverse_value)
	}

	/// Associates all the given nodes to the reverse property.
	#[inline(always)]
	pub fn insert_all_reverse<Nodes: Iterator<Item = Indexed<Self>>>(
		&mut self,
		reverse_prop: Reference<T>,
		reverse_values: Nodes,
	) {
		self.reverse_properties
			.insert_all(reverse_prop, reverse_values)
	}

	/// Tests if the node is an unnamed graph object.
	///
	/// Returns `true` is the only field of the object is a `@graph` field.
	/// Returns `false` otherwise.
	#[inline]
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
	#[inline(always)]
	pub fn into_unnamed_graph(self) -> Result<HashSet<Indexed<Object<J, T>>>, Self> {
		if self.is_unnamed_graph() {
			Ok(self.graph.unwrap())
		} else {
			Err(self)
		}
	}

	/// Equivalence operator.
	///
	/// Equivalence is different from equality for anonymous objects.
	/// Anonymous node objects have an implicit unlabeled blank nodes and thus never equivalent.
	pub fn equivalent(&self, other: &Self) -> bool {
		if self.id().is_some() && other.id().is_some() {
			self == other
		} else {
			false
		}
	}
}

impl<J: JsonHash, T: Id> Indexed<Node<J, T>> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}
}

impl<J: JsonHash, T: Id> object::Any<J, T> for Node<J, T> {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref<J, T> {
		object::Ref::Node(self)
	}
}

impl<J: JsonHash, T: Id> TryFrom<Object<J, T>> for Node<J, T> {
	type Error = Object<J, T>;

	#[inline(always)]
	fn try_from(obj: Object<J, T>) -> Result<Node<J, T>, Object<J, T>> {
		match obj {
			Object::Node(node) => Ok(node),
			obj => Err(obj),
		}
	}
}

impl<J: JsonHash, T: Id> Hash for Node<J, T> {
	#[inline]
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.id.hash(h);
		self.types.hash(h);
		util::hash_set_opt(&self.graph, h);
		util::hash_set_opt(&self.included, h);
		self.properties.hash(h);
		self.reverse_properties.hash(h)
	}
}

impl<J: JsonHash + JsonClone, K: util::JsonFrom<J>, T: Id> util::AsJson<J, K> for Node<J, T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		let mut obj = K::Object::default();

		if let Some(id) = &self.id {
			obj.insert(
				K::new_key(Keyword::Id.into_str(), meta(None)),
				id.as_json_with(meta.clone()),
			);
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
					value.as_json_with(meta.clone()),
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
				value.as_json_with(meta.clone()),
			);
		}

		K::object(obj, meta(None))
	}
}

impl<J: JsonHash + JsonClone, K: util::JsonFrom<J>, T: Id> util::AsJson<J, K>
	for HashSet<Indexed<Node<J, T>>>
{
	#[inline(always)]
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		let array = self
			.iter()
			.map(|value| value.as_json_with(meta.clone()))
			.collect();
		K::array(array, meta(None))
	}
}

/// Iterator through indexed nodes.
pub struct Nodes<'a, J: JsonHash, T: Id>(Option<std::slice::Iter<'a, Indexed<Node<J, T>>>>);

impl<'a, J: JsonHash, T: Id> Nodes<'a, J, T> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, Indexed<Node<J, T>>>>) -> Self {
		Self(inner)
	}
}

impl<'a, J: JsonHash, T: Id> Iterator for Nodes<'a, J, T> {
	type Item = &'a Indexed<Node<J, T>>;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a Indexed<Node<J, T>>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}
