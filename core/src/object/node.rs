use crate::{id, object, utils, Id, Indexed, Object, Objects, Reference, Term, ToReference};
use iref::{Iri, IriBuf};
use json_ld_syntax::Keyword;
use locspan::{Stripped, BorrowStripped};
use locspan_derive::*;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

pub mod properties;
pub mod reverse_properties;

pub use properties::Properties;
pub use reverse_properties::ReverseProperties;

/// Node parts.
pub struct Parts<T: Id = IriBuf, M=()> {
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
	pub graph: Option<HashSet<Stripped<Indexed<Object<T, M>>>>>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	pub included: Option<HashSet<Stripped<Indexed<Node<T, M>>>>>,

	/// Properties.
	///
	/// Any non-keyword field.
	pub properties: Properties<T, M>,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	pub reverse_properties: ReverseProperties<T, M>,
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
#[derive(StrippedPartialEq, StrippedEq)]
#[stripped_ignore(M)]
#[stripped(T)]
pub struct Node<T: Id = IriBuf, M=()> {
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
	#[stripped]
	pub(crate) graph: Option<HashSet<Stripped<Indexed<Object<T, M>>>>>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	#[stripped]
	pub(crate) included: Option<HashSet<Stripped<Indexed<Self>>>>,

	/// Properties.
	///
	/// Any non-keyword field.
	pub(crate) properties: Properties<T, M>,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	pub(crate) reverse_properties: ReverseProperties<T, M>,
}

impl<T: Id, M> Default for Node<T, M> {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Id, M> Node<T, M> {
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

	pub fn from_parts(parts: Parts<T, M>) -> Self {
		Self {
			id: parts.id,
			types: parts.types,
			graph: parts.graph,
			included: parts.included,
			properties: parts.properties,
			reverse_properties: parts.reverse_properties,
		}
	}

	pub fn into_parts(self) -> Parts<T, M> {
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
	/// # let node: json_ld::Node<serde_json::Value> = json_ld::Node::new();
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

	/// Assigns an identifier to this node and every other node included in this one using the given `generator`.
	pub fn identify_all<G: id::Generator<T>>(&mut self, generator: &mut G) {
		if self.id.is_none() {
			self.id = Some(generator.next().into())
		}

		for (_, objects) in self.properties_mut() {
			for object in objects {
				object.identify_all(generator);
			}
		}

		for (_, nodes) in self.reverse_properties_mut() {
			for node in nodes {
				node.identify_all(generator);
			}
		}
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
	pub fn graph(&self) -> Option<&HashSet<Stripped<Indexed<Object<T, M>>>>> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_mut(&mut self) -> Option<&mut HashSet<Stripped<Indexed<Object<T, M>>>>> {
		self.graph.as_mut()
	}

	/// Set the graph.
	#[inline(always)]
	pub fn set_graph(&mut self, graph: Option<HashSet<Stripped<Indexed<Object<T, M>>>>>) {
		self.graph = graph
	}

	/// Get the set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included(&self) -> Option<&HashSet<Stripped<Indexed<Self>>>> {
		self.included.as_ref()
	}

	/// Get the mutable set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_mut(&mut self) -> Option<&mut HashSet<Stripped<Indexed<Self>>>> {
		self.included.as_mut()
	}

	/// Set the set of nodes included by the node.
	#[inline(always)]
	pub fn set_included(&mut self, included: Option<HashSet<Stripped<Indexed<Self>>>>) {
		self.included = included
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn properties(&self) -> &Properties<T, M> {
		&self.properties
	}

	/// Returns a mutable reference to the properties of the node.
	#[inline(always)]
	pub fn properties_mut(&mut self) -> &mut Properties<T, M> {
		&mut self.properties
	}

	/// Returns a reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties(&self) -> &ReverseProperties<T, M> {
		&self.reverse_properties
	}

	/// Returns a mutable reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_mut(&mut self) -> &mut ReverseProperties<T, M> {
		&mut self.reverse_properties
	}

	/// Get all the objects associated to the node with the given property.
	#[inline(always)]
	pub fn get<'a, Q: ToReference<T>>(&self, prop: Q) -> Objects<T, M>
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
	pub fn get_any<'a, Q: ToReference<T>>(&self, prop: Q) -> Option<&Indexed<Object<T, M>>>
	where
		T: 'a,
	{
		self.properties.get_any(prop)
	}

	/// Associates the given object to the node through the given property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Reference<T>, value: Indexed<Object<T, M>>) {
		self.properties.insert(prop, value)
	}

	/// Associates all the given objects to the node through the given property.
	///
	/// If there already exists objects associated to the given reverse property,
	/// `reverse_value` is added to the list. Duplicate objects are not removed.
	#[inline(always)]
	pub fn insert_all<Objects: Iterator<Item = Indexed<Object<T, M>>>>(
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
	pub fn into_unnamed_graph(self) -> Result<HashSet<Stripped<Indexed<Object<T, M>>>>, Self> {
		if self.is_unnamed_graph() {
			Ok(self.graph.unwrap())
		} else {
			Err(self)
		}
	}

	pub fn traverse(&self) -> Traverse<T, M> {
		Traverse {
			itself: Some(self),
			current_object: None,
			graph: self.graph.as_ref().map(HashSet::iter),
			current_node: None,
			included: self.included.as_ref().map(HashSet::iter),
			properties: self.properties.traverse(),
			reverse_properties: self.reverse_properties.traverse(),
		}
	}

	/// Equivalence operator.
	///
	/// Equivalence is different from equality for anonymous objects.
	/// Anonymous node objects have an implicit unlabeled blank nodes and thus never equivalent.
	pub fn equivalent(&self, other: &Self) -> bool {
		if self.id().is_some() && other.id().is_some() {
			self.stripped() == other.stripped()
		} else {
			false
		}
	}
}

impl<T: Id, M> Indexed<Node<T, M>> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}
}

impl<T: Id, M> object::Any<T, M> for Node<T, M> {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref<T, M> {
		object::Ref::Node(self)
	}
}

impl<T: Id, M> TryFrom<Object<T, M>> for Node<T, M> {
	type Error = Object<T, M>;

	#[inline(always)]
	fn try_from(obj: Object<T, M>) -> Result<Node<T, M>, Object<T, M>> {
		match obj {
			Object::Node(node) => Ok(node),
			obj => Err(obj),
		}
	}
}

impl<T: Id, M: Hash> Hash for Node<T, M> {
	#[inline]
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.id.hash(h);
		self.types.hash(h);
		utils::hash_set_opt(&self.graph, h);
		utils::hash_set_opt(&self.included, h);
		self.properties.hash(h);
		self.reverse_properties.hash(h)
	}
}

impl<T: Id, M> locspan::StrippedHash for Node<T, M> {
	#[inline]
	fn stripped_hash<H: Hasher>(&self, h: &mut H) {
		self.id.hash(h);
		self.types.hash(h);
		utils::hash_set_opt(&self.graph, h);
		utils::hash_set_opt(&self.included, h);
		self.properties.stripped_hash(h);
		self.reverse_properties.stripped_hash(h)
	}
}

// impl<J: JsonHash + JsonClone, K: utils::JsonFrom<J>, T: Id> utils::AsJson<J, K> for Node<T, M> {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		let mut obj = <K as Json>::Object::default();

// 		if let Some(id) = &self.id {
// 			obj.insert(
// 				K::new_key(Keyword::Id.into_str(), meta(None)),
// 				id.as_json_with(meta.clone()),
// 			);
// 		}

// 		if !self.types.is_empty() {
// 			obj.insert(
// 				K::new_key(Keyword::Type.into_str(), meta(None)),
// 				self.types.as_json_with(meta.clone()),
// 			);
// 		}

// 		if let Some(graph) = &self.graph {
// 			obj.insert(
// 				K::new_key(Keyword::Graph.into_str(), meta(None)),
// 				graph.as_json_with(meta.clone()),
// 			);
// 		}

// 		if let Some(included) = &self.included {
// 			obj.insert(
// 				K::new_key(Keyword::Included.into_str(), meta(None)),
// 				included.as_json_with(meta.clone()),
// 			);
// 		}

// 		if !self.reverse_properties.is_empty() {
// 			let mut reverse = <K as Json>::Object::default();
// 			for (key, value) in &self.reverse_properties {
// 				reverse.insert(
// 					K::new_key(key.as_str(), meta(None)),
// 					value.as_json_with(meta.clone()),
// 				);
// 			}

// 			obj.insert(
// 				K::new_key(Keyword::Reverse.into_str(), meta(None)),
// 				K::object(reverse, meta(None)),
// 			);
// 		}

// 		for (key, value) in &self.properties {
// 			obj.insert(
// 				K::new_key(key.as_str(), meta(None)),
// 				value.as_json_with(meta.clone()),
// 			);
// 		}

// 		K::object(obj, meta(None))
// 	}
// }

// impl<J: JsonHash + JsonClone, K: utils::JsonFrom<J>, T: Id> utils::AsJson<J, K>
// 	for HashSet<Indexed<Node<T, M>>>
// {
// 	#[inline(always)]
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		let array = self
// 			.iter()
// 			.map(|value| value.as_json_with(meta.clone()))
// 			.collect();
// 		K::array(array, meta(None))
// 	}
// }

/// Iterator through indexed nodes.
pub struct Nodes<'a, T: Id, M>(Option<std::slice::Iter<'a, Indexed<Node<T, M>>>>);

impl<'a, T: Id, M> Nodes<'a, T, M> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, Indexed<Node<T, M>>>>) -> Self {
		Self(inner)
	}
}

impl<'a, T: Id, M> Iterator for Nodes<'a, T, M> {
	type Item = &'a Indexed<Node<T, M>>;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a Indexed<Node<T, M>>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}

pub struct Traverse<'a, T: Id, M> {
	itself: Option<&'a Node<T, M>>,
	current_object: Option<Box<super::Traverse<'a, T, M>>>,
	graph: Option<std::collections::hash_set::Iter<'a, Stripped<Indexed<Object<T, M>>>>>,
	current_node: Option<Box<Self>>,
	included: Option<std::collections::hash_set::Iter<'a, Stripped<Indexed<Node<T, M>>>>>,
	properties: properties::Traverse<'a, T, M>,
	reverse_properties: reverse_properties::Traverse<'a, T, M>,
}

impl<'a, T: Id, M> Iterator for Traverse<'a, T, M> {
	type Item = crate::object::Ref<'a, T, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.itself.take() {
			Some(node) => Some(crate::object::Ref::Node(node)),
			None => match self.properties.next() {
				Some(next) => Some(next),
				None => match self.reverse_properties.next() {
					Some(next) => Some(next),
					None => loop {
						match &mut self.current_object {
							Some(object) => match object.next() {
								Some(next) => break Some(next),
								None => self.current_object = None,
							},
							None => match &mut self.graph {
								Some(graph) => match graph.next() {
									Some(object) => {
										self.current_object = Some(Box::new(object.traverse()))
									}
									None => self.graph = None,
								},
								None => match &mut self.current_node {
									Some(node) => match node.next() {
										Some(next) => break Some(next),
										None => self.current_node = None,
									},
									None => match &mut self.included {
										Some(included) => match included.next() {
											Some(node) => {
												self.current_node = Some(Box::new(node.traverse()))
											}
											None => self.included = None,
										},
										None => break None,
									},
								},
							},
						}
					},
				},
			},
		}
	}
}
