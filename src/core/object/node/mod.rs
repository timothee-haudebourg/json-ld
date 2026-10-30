use crate::object::{JsonLdObjectMut, JsonLdObjectRef, Objects};
use crate::syntax::Keyword;
use crate::{
	IndexedJsonLdObject, JsonLdIndexed, JsonLdLenient, JsonLdObject, JsonLdTerm, JsonLdVisit,
	object,
};
use btree_indexmap::{BTreeIndexMultiSet, BTreeIndexSet, Comparable};
use educe::Educe;
use rdf_syntax::Id;
use rdf_syntax::Iri;
use std::convert::TryFrom;
use std::hash::Hash;

pub mod properties;
pub mod reverse_properties;

pub use properties::JsonLdProperties;
pub use reverse_properties::JsonLdReverseProperties;

pub type JsonLdGraph = BTreeIndexSet<IndexedJsonLdObject>;

pub type JsonLdIncluded = BTreeIndexSet<IndexedJsonLdNode>;

pub type IndexedJsonLdNode = JsonLdIndexed<JsonLdNode>;

/// Node object.
///
/// A node object represents zero or more properties of a node in the graph serialized by a JSON-LD document.
/// A node is defined by its identifier (`@id` field), types, properties and reverse properties.
/// In addition, a node may represent a graph (`@graph field`) and includes nodes
/// (`@included` field).
// NOTE it may be better to use BTreeSet instead of HashSet to have some ordering?
//      in which case the Json bound should be lifted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonLdNode {
	/// Identifier.
	///
	/// This is the `@id` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@id", default, skip_serializing_if = "Option::is_none")
	)]
	pub id: Option<JsonLdLenient<Id>>,

	/// Types.
	///
	/// This is the `@type` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@type", default, skip_serializing_if = "Option::is_none")
	)]
	pub types: Option<BTreeIndexMultiSet<JsonLdLenient<Id>>>,

	/// Associated graph.
	///
	/// This is the `@graph` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@graph", default, skip_serializing_if = "Option::is_none")
	)]
	pub graph: Option<JsonLdGraph>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@included", default, skip_serializing_if = "Option::is_none")
	)]
	pub included: Option<JsonLdIncluded>,

	/// Properties.
	///
	/// Any non-keyword field.
	#[cfg_attr(feature = "serde", serde(flatten))]
	pub properties: JsonLdProperties,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")
	)]
	pub reverse_properties: Option<JsonLdReverseProperties>,
}

impl Default for JsonLdNode {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl JsonLdNode {
	/// Creates a new empty node.
	#[inline(always)]
	pub fn new() -> Self {
		Self::new_with_id(None)
	}

	/// Creates a new empty node.
	#[inline(always)]
	pub fn new_with_id(id: Option<JsonLdLenient<Id>>) -> Self {
		Self {
			id,
			types: None,
			graph: None,
			included: None,
			properties: JsonLdProperties::new(),
			reverse_properties: None,
		}
	}

	/// Creates a new graph node.
	pub fn new_graph(id: JsonLdLenient<Id>, graph: JsonLdGraph) -> Self {
		Self {
			id: Some(id),
			types: None,
			graph: Some(graph),
			included: None,
			properties: JsonLdProperties::new(),
			reverse_properties: None,
		}
	}

	/// Returns a mutable reference to the reverse properties of the node.
	///
	/// If no `@reverse` entry is present, one is created.
	#[inline(always)]
	pub fn reverse_properties_mut_or_default(&mut self) -> &mut JsonLdReverseProperties {
		self.reverse_properties
			.get_or_insert_with(JsonLdReverseProperties::default)
	}

	/// Returns a mutable reference to the included nodes.
	///
	/// If no `@included` entry is present, one is created.
	#[inline(always)]
	pub fn included_mut_or_default(&mut self) -> &mut JsonLdIncluded {
		self.included.get_or_insert_with(JsonLdIncluded::default)
	}

	// /// Assigns an identifier to this node and every other node included in this
	// /// one using the given `generator`.
	// pub fn identify_all_with(&mut self, generator: &mut impl Generator) {
	// 	if self.id.is_none() {
	// 		self.id = Some(generator.next(vocabulary).into())
	// 	}

	// 	if let Some(graph) = self.graph_mut() {
	// 		*graph = std::mem::take(graph)
	// 			.into_iter()
	// 			.map(|mut o| {
	// 				o.identify_all_with(vocabulary, generator);
	// 				o
	// 			})
	// 			.collect();
	// 	}

	// 	if let Some(included) = self.included_mut() {
	// 		*included = std::mem::take(included)
	// 			.into_iter()
	// 			.map(|mut n| {
	// 				n.identify_all_with(vocabulary, generator);
	// 				n
	// 			})
	// 			.collect();
	// 	}

	// 	for (_, objects) in self.properties_mut() {
	// 		for object in objects {
	// 			object.identify_all_with(vocabulary, generator);
	// 		}
	// 	}

	// 	if let Some(reverse_properties) = self.reverse_properties_mut() {
	// 		for (_, nodes) in reverse_properties.iter_mut() {
	// 			for node in nodes {
	// 				node.identify_all_with(vocabulary, generator);
	// 			}
	// 		}
	// 	}
	// }

	// /// Assigns an identifier to this node and every other node included in this one using the given `generator`.
	// pub fn identify_all<G: Generator>(&mut self, generator: &mut G)
	// where
	// 	T: Eq + Hash,
	// 	B: Eq + Hash,
	// 	(): Vocabulary<Iri = T, BlankId = B>,
	// {
	// 	self.identify_all_with(&mut (), generator)
	// }

	// /// Puts this node object literals into canonical form using the given
	// /// `buffer`.
	// ///
	// /// The buffer is used to compute the canonical form of numbers.
	// pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
	// 	for (_, objects) in self.properties_mut() {
	// 		for object in objects {
	// 			object.canonicalize_with(buffer)
	// 		}
	// 	}

	// 	if let Some(reverse_properties) = self.reverse_properties_mut() {
	// 		for (_, nodes) in reverse_properties.iter_mut() {
	// 			for node in nodes {
	// 				node.canonicalize_with(buffer)
	// 			}
	// 		}
	// 	}
	// }

	// /// Puts this node object literals into canonical form.
	// pub fn canonicalize(&mut self) {
	// 	let mut buffer = ryu_js::Buffer::new();
	// 	self.canonicalize_with(&mut buffer)
	// }

	/// Get the node's as an IRI if possible.
	///
	/// Returns the node's IRI id if any. Returns `None` otherwise.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<&Iri> {
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
			Some(iri) => Some(iri.as_ref()),
			None => None,
		}
	}

	// /// Get the list of the node's types.
	// #[inline(always)]
	// pub fn types(&self) -> &[Lenient<Id>] {
	// 	match self.types.as_ref() {
	// 		Some(entry) => entry,
	// 		None => &[],
	// 	}
	// }

	// /// Returns a mutable reference to the node's types.
	// #[inline(always)]
	// pub fn types_mut(&mut self) -> &mut [Lenient<Id>] {
	// 	match self.types.as_mut() {
	// 		Some(entry) => entry,
	// 		None => &mut [],
	// 	}
	// }

	// pub fn types_mut_or_default(&mut self) -> &mut Vec<Lenient<Id>> {
	// 	self.types.get_or_insert_with(Vec::new)
	// }

	/// Checks if the node has the given type.
	#[inline]
	pub fn has_type<U>(&self, ty: &U) -> bool
	where
		U: ?Sized + Comparable<JsonLdLenient<Id>>,
	{
		self.types.as_ref().is_some_and(|types| types.contains(ty))
	}

	/// Tests if the node is empty.
	///
	/// It is empty is every field other than `@id` is empty.
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.types.is_none()
			&& self.graph.is_none()
			&& self.included.is_none()
			&& self.properties.is_empty()
			&& self.reverse_properties.is_none()
	}

	/// Tests if the node is a graph object (has a `@graph` field, and optionally an `@id` field).
	/// Note that node objects may have a @graph entry,
	/// but are not considered graph objects if they include any other entries other than `@id`.
	#[inline]
	pub fn is_graph(&self) -> bool {
		self.graph.is_some()
			&& self.types.is_none()
			&& self.included.is_none()
			&& self.properties.is_empty()
			&& self.reverse_properties.is_none()
	}

	/// Tests if the node is a simple graph object (a graph object without `@id` field)
	#[inline(always)]
	pub fn is_simple_graph(&self) -> bool {
		self.id.is_none() && self.is_graph()
	}

	/// If the node is a graph object, get the graph.
	#[inline(always)]
	pub fn graph(&self) -> Option<&JsonLdGraph> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_mut(&mut self) -> Option<&mut JsonLdGraph> {
		self.graph.as_mut()
	}

	/// If the node is a graph object, get the graph.
	#[inline(always)]
	pub fn graph_entry(&self) -> Option<&JsonLdGraph> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_entry_mut(&mut self) -> Option<&mut JsonLdGraph> {
		self.graph.as_mut()
	}

	/// Set the graph.
	#[inline(always)]
	pub fn set_graph_entry(&mut self, graph: Option<JsonLdGraph>) {
		self.graph = graph
	}

	/// Get the set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_entry(&self) -> Option<&JsonLdIncluded> {
		self.included.as_ref()
	}

	/// Get the mutable set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_entry_mut(&mut self) -> Option<&mut JsonLdIncluded> {
		self.included.as_mut()
	}

	/// Returns a reference to the set of `@included` nodes.
	pub fn included(&self) -> Option<&JsonLdIncluded> {
		self.included.as_ref()
	}

	/// Returns a mutable reference to the set of `@included` nodes.
	pub fn included_mut(&mut self) -> Option<&mut JsonLdIncluded> {
		self.included.as_mut()
	}

	/// Set the set of nodes included by the node.
	#[inline(always)]
	pub fn set_included(&mut self, included: Option<JsonLdIncluded>) {
		self.included = included
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn properties(&self) -> &JsonLdProperties {
		&self.properties
	}

	/// Returns a mutable reference to the properties of the node.
	#[inline(always)]
	pub fn properties_mut(&mut self) -> &mut JsonLdProperties {
		&mut self.properties
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn reverse_properties(&self) -> Option<&JsonLdReverseProperties> {
		self.reverse_properties.as_ref()
	}

	/// Returns a reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_entry(&self) -> Option<&JsonLdReverseProperties> {
		self.reverse_properties.as_ref()
	}

	/// Returns a mutable reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_mut(&mut self) -> Option<&mut JsonLdReverseProperties> {
		self.reverse_properties.as_mut()
	}

	pub fn set_reverse_properties(&mut self, reverse_properties: Option<JsonLdReverseProperties>) {
		self.reverse_properties = reverse_properties
	}

	/// Tests if the node is an unnamed graph object.
	///
	/// Returns `true` is the only field of the object is a `@graph` field.
	/// Returns `false` otherwise.
	#[inline]
	pub fn is_unnamed_graph(&self) -> bool {
		self.graph.is_some()
			&& self.id.is_none()
			&& self.types.is_none()
			&& self.included.is_none()
			&& self.properties.is_empty()
			&& self.reverse_properties.is_none()
	}

	/// Returns the node as an unnamed graph, if it is one.
	///
	/// The unnamed graph is returned as a set of indexed objects.
	/// Fails and returns itself if the node is *not* an unnamed graph.
	#[allow(clippy::result_large_err)]
	#[inline(always)]
	pub fn into_unnamed_graph(self) -> Result<JsonLdGraph, Self> {
		if self.is_unnamed_graph() {
			Ok(self.graph.unwrap())
		} else {
			Err(self)
		}
	}

	pub fn entries(&self) -> Entries<'_> {
		Entries {
			id: self.id.as_ref(),
			type_: self.types.as_ref(),
			graph: self.graph.as_ref(),
			included: self.included.as_ref(),
			reverse: self.reverse_properties.as_ref(),
			properties: self.properties.iter(),
		}
	}

	/// Checks if the node object has the given term as key.
	///
	/// # Example
	/// ```
	/// use json_ld::{JsonLdNode, JsonLdTerm};
	/// use json_ld::syntax::Keyword;
	///
	/// let node = JsonLdNode::new();
	///
	/// // Checks if the JSON object representation of the node has an `@id` key.
	/// if node.has_key(&JsonLdTerm::Keyword(Keyword::Id)) {
	///   // ...
	/// }
	/// ```
	#[inline(always)]
	pub fn has_key(&self, key: &JsonLdTerm) -> bool {
		match key {
			JsonLdTerm::Keyword(Keyword::Id) => self.id.is_some(),
			JsonLdTerm::Keyword(Keyword::Type) => self.types.is_some(),
			JsonLdTerm::Keyword(Keyword::Graph) => self.graph.is_some(),
			JsonLdTerm::Keyword(Keyword::Included) => self.included.is_some(),
			JsonLdTerm::Keyword(Keyword::Reverse) => self.reverse_properties.is_some(),
			JsonLdTerm::Id(prop) => self.properties.contains(prop),
			_ => false,
		}
	}

	/// Get all the objects associated to the node with the given property.
	#[inline(always)]
	pub fn get<Q>(&self, prop: &Q) -> Objects<'_>
	where
		Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>,
	{
		self.properties.get(prop)
	}

	/// Get one of the objects associated to the node with the given property.
	///
	/// If multiple objects are attached to the node with this property, there are no guaranties
	/// on which object will be returned.
	#[inline(always)]
	pub fn get_any<Q>(&self, prop: &Q) -> Option<&IndexedJsonLdObject>
	where
		Q: ?Sized + Hash + Comparable<JsonLdLenient<Id>>,
	{
		self.properties.get_any(prop)
	}

	/// Associates the given object to the node through the given property.
	#[inline(always)]
	pub fn insert(&mut self, prop: JsonLdLenient<Id>, value: IndexedJsonLdObject) {
		self.properties.insert(prop, value)
	}

	/// Associates all the given objects to the node through the given property.
	///
	/// If there already exists objects associated to the given reverse property,
	/// `reverse_value` is added to the list. Duplicate objects are not removed.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = IndexedJsonLdObject>>(
		&mut self,
		prop: JsonLdLenient<Id>,
		values: Objects,
	) {
		self.properties.insert_all(prop, values)
	}

	pub fn reverse_properties_or_insert(
		&mut self,
		props: JsonLdReverseProperties,
	) -> &mut JsonLdReverseProperties {
		self.reverse_properties.get_or_insert(props)
	}

	pub fn reverse_properties_or_default(&mut self) -> &mut JsonLdReverseProperties {
		self.reverse_properties
			.get_or_insert_with(JsonLdReverseProperties::default)
	}

	pub fn reverse_properties_or_insert_with(
		&mut self,
		f: impl FnOnce() -> JsonLdReverseProperties,
	) -> &mut JsonLdReverseProperties {
		self.reverse_properties.get_or_insert_with(f)
	}

	/// Equivalence operator.
	///
	/// Equivalence is different from equality for anonymous objects.
	/// Anonymous node objects have an implicit unlabeled blank nodes and thus never equivalent.
	pub fn equivalent(&self, other: &Self) -> bool {
		if self.id.is_some() && other.id.is_some() {
			self == other
		} else {
			false
		}
	}
}

impl JsonLdVisit for JsonLdNode {
	fn visit_with(&self, f: &mut impl FnMut(JsonLdObjectRef)) {
		f(JsonLdObjectRef::Node(self));
		self.graph.visit_with(f);
		self.included.visit_with(f);
		self.properties.visit_with(f);
		self.reverse_properties.visit_with(f);
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(JsonLdObjectMut)) {
		f(JsonLdObjectMut::Node(self));
		self.graph.visit_mut_with(f);
		self.included.visit_mut_with(f);
		self.properties.visit_mut_with(f);
		self.reverse_properties.visit_mut_with(f);
	}
}

impl JsonLdIndexed<JsonLdNode> {
	pub fn entries(&self) -> IndexedEntries<'_> {
		IndexedEntries {
			index: self.index(),
			inner: self.inner().entries(),
		}
	}
}

impl JsonLdIndexed<JsonLdNode> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}
}

#[derive(Educe, PartialEq, Eq)]
#[educe(Clone, Copy)]
pub enum EntryKeyRef<'a> {
	Id,
	Type,
	Graph,
	Included,
	Reverse,
	Property(&'a JsonLdLenient<Id>),
}

impl<'a> EntryKeyRef<'a> {
	pub fn into_keyword(self) -> Option<Keyword> {
		match self {
			Self::Id => Some(Keyword::Id),
			Self::Type => Some(Keyword::Type),
			Self::Graph => Some(Keyword::Graph),
			Self::Included => Some(Keyword::Included),
			Self::Reverse => Some(Keyword::Reverse),
			Self::Property(_) => None,
		}
	}

	pub fn as_keyword(&self) -> Option<Keyword> {
		self.into_keyword()
	}

	pub fn into_str(self) -> &'a str {
		match self {
			Self::Id => "@id",
			Self::Type => "@type",
			Self::Graph => "@graph",
			Self::Included => "@included",
			Self::Reverse => "@reverse",
			Self::Property(p) => p.as_str(),
		}
	}

	pub fn as_str(&self) -> &'a str {
		self.into_str()
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryValueRef<'a> {
	Id(&'a JsonLdLenient<Id>),
	Type(&'a BTreeIndexMultiSet<JsonLdLenient<Id>>),
	Graph(&'a BTreeIndexSet<IndexedJsonLdObject>),
	Included(&'a BTreeIndexSet<IndexedJsonLdNode>),
	Reverse(&'a JsonLdReverseProperties),
	Property(&'a [IndexedJsonLdObject]),
}

impl<'a> EntryValueRef<'a> {
	pub fn is_json_array(&self) -> bool {
		matches!(
			self,
			Self::Type(_) | Self::Graph(_) | Self::Included(_) | Self::Property(_)
		)
	}

	pub fn is_json_object(&self) -> bool {
		matches!(self, Self::Reverse(_))
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryRef<'a> {
	Id(&'a JsonLdLenient<Id>),
	Type(&'a BTreeIndexMultiSet<JsonLdLenient<Id>>),
	Graph(&'a JsonLdGraph),
	Included(&'a JsonLdIncluded),
	Reverse(&'a JsonLdReverseProperties),
	Property(&'a JsonLdLenient<Id>, &'a [IndexedJsonLdObject]),
}

impl<'a> EntryRef<'a> {
	pub fn into_key(self) -> EntryKeyRef<'a> {
		match self {
			Self::Id(_) => EntryKeyRef::Id,
			Self::Type(_) => EntryKeyRef::Type,
			Self::Graph(_) => EntryKeyRef::Graph,
			Self::Included(_) => EntryKeyRef::Included,
			Self::Reverse(_) => EntryKeyRef::Reverse,
			Self::Property(k, _) => EntryKeyRef::Property(k),
		}
	}

	pub fn key(&self) -> EntryKeyRef<'a> {
		self.into_key()
	}

	pub fn into_value(self) -> EntryValueRef<'a> {
		match self {
			Self::Id(v) => EntryValueRef::Id(v),
			Self::Type(v) => EntryValueRef::Type(v),
			Self::Graph(v) => EntryValueRef::Graph(v),
			Self::Included(v) => EntryValueRef::Included(v),
			Self::Reverse(v) => EntryValueRef::Reverse(v),
			Self::Property(_, v) => EntryValueRef::Property(v),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (EntryKeyRef<'a>, EntryValueRef<'a>) {
		match self {
			Self::Id(v) => (EntryKeyRef::Id, EntryValueRef::Id(v)),
			Self::Type(v) => (EntryKeyRef::Type, EntryValueRef::Type(v)),
			Self::Graph(v) => (EntryKeyRef::Graph, EntryValueRef::Graph(v)),
			Self::Included(v) => (EntryKeyRef::Included, EntryValueRef::Included(v)),
			Self::Reverse(v) => (EntryKeyRef::Reverse, EntryValueRef::Reverse(v)),
			Self::Property(k, v) => (EntryKeyRef::Property(k), EntryValueRef::Property(v)),
		}
	}

	pub fn as_key_value(&self) -> (EntryKeyRef<'a>, EntryValueRef<'a>) {
		match self {
			Self::Id(v) => (EntryKeyRef::Id, EntryValueRef::Id(v)),
			Self::Type(v) => (EntryKeyRef::Type, EntryValueRef::Type(v)),
			Self::Graph(v) => (EntryKeyRef::Graph, EntryValueRef::Graph(v)),
			Self::Included(v) => (EntryKeyRef::Included, EntryValueRef::Included(v)),
			Self::Reverse(v) => (EntryKeyRef::Reverse, EntryValueRef::Reverse(v)),
			Self::Property(k, v) => (EntryKeyRef::Property(k), EntryValueRef::Property(v)),
		}
	}
}

#[derive(Educe)]
#[educe(Clone)]
pub struct Entries<'a> {
	id: Option<&'a JsonLdLenient<Id>>,
	type_: Option<&'a BTreeIndexMultiSet<JsonLdLenient<Id>>>,
	graph: Option<&'a JsonLdGraph>,
	included: Option<&'a JsonLdIncluded>,
	reverse: Option<&'a JsonLdReverseProperties>,
	properties: properties::JsonLdPropertiesIter<'a>,
}

impl<'a> Iterator for Entries<'a> {
	type Item = EntryRef<'a>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = self.properties.len();

		if self.id.is_some() {
			len += 1
		}

		if self.type_.is_some() {
			len += 1
		}

		if self.graph.is_some() {
			len += 1
		}

		if self.included.is_some() {
			len += 1
		}

		if self.reverse.is_some() {
			len += 1
		}

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.id.take().map(EntryRef::Id).or_else(|| {
			self.type_.take().map(EntryRef::Type).or_else(|| {
				self.graph.take().map(EntryRef::Graph).or_else(|| {
					self.included.take().map(EntryRef::Included).or_else(|| {
						self.reverse.take().map(EntryRef::Reverse).or_else(|| {
							self.properties
								.next()
								.map(|(k, v)| EntryRef::Property(k, v))
						})
					})
				})
			})
		})
	}
}

impl<'a> ExactSizeIterator for Entries<'a> {}

#[derive(Educe)]
#[educe(Clone)]
pub struct IndexedEntries<'a> {
	index: Option<&'a str>,
	inner: Entries<'a>,
}

impl<'a> Iterator for IndexedEntries<'a> {
	type Item = IndexedEntryRef<'a>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.inner.len() + usize::from(self.index.is_some());
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.index
			.take()
			.map(IndexedEntryRef::Index)
			.or_else(|| self.inner.next().map(IndexedEntryRef::Node))
	}
}

impl<'a> ExactSizeIterator for IndexedEntries<'a> {}

#[derive(Educe, PartialEq, Eq)]
#[educe(Clone, Copy)]
pub enum IndexedEntryKeyRef<'a> {
	Index,
	Node(EntryKeyRef<'a>),
}

impl<'a> IndexedEntryKeyRef<'a> {
	pub fn into_keyword(self) -> Option<Keyword> {
		match self {
			Self::Index => Some(Keyword::Index),
			Self::Node(e) => e.into_keyword(),
		}
	}

	pub fn as_keyword(&self) -> Option<Keyword> {
		self.into_keyword()
	}

	pub fn into_str(self) -> &'a str {
		match self {
			Self::Index => "@index",
			Self::Node(e) => e.into_str(),
		}
	}

	pub fn as_str(&self) -> &'a str {
		self.into_str()
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum IndexedEntryValueRef<'a> {
	Index(&'a str),
	Node(EntryValueRef<'a>),
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum IndexedEntryRef<'a> {
	Index(&'a str),
	Node(EntryRef<'a>),
}

impl<'a> IndexedEntryRef<'a> {
	pub fn into_key(self) -> IndexedEntryKeyRef<'a> {
		match self {
			Self::Index(_) => IndexedEntryKeyRef::Index,
			Self::Node(e) => IndexedEntryKeyRef::Node(e.key()),
		}
	}

	pub fn key(&self) -> IndexedEntryKeyRef<'a> {
		self.into_key()
	}

	pub fn into_value(self) -> IndexedEntryValueRef<'a> {
		match self {
			Self::Index(v) => IndexedEntryValueRef::Index(v),
			Self::Node(e) => IndexedEntryValueRef::Node(e.value()),
		}
	}

	pub fn value(&self) -> IndexedEntryValueRef<'a> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (IndexedEntryKeyRef<'a>, IndexedEntryValueRef<'a>) {
		match self {
			Self::Index(v) => (IndexedEntryKeyRef::Index, IndexedEntryValueRef::Index(v)),
			Self::Node(e) => {
				let (k, v) = e.into_key_value();
				(IndexedEntryKeyRef::Node(k), IndexedEntryValueRef::Node(v))
			}
		}
	}

	pub fn as_key_value(&self) -> (IndexedEntryKeyRef<'a>, IndexedEntryValueRef<'a>) {
		self.into_key_value()
	}
}

impl object::AnyJsonLdObject for JsonLdNode {
	#[inline(always)]
	fn as_ref(&self) -> object::JsonLdObjectRef<'_> {
		object::JsonLdObjectRef::Node(self)
	}
}

impl TryFrom<JsonLdObject> for JsonLdNode {
	type Error = JsonLdObject;

	#[inline(always)]
	fn try_from(obj: JsonLdObject) -> Result<JsonLdNode, JsonLdObject> {
		match obj {
			JsonLdObject::Node(node) => Ok(*node),
			obj => Err(obj),
		}
	}
}

/// Iterator through indexed nodes.
pub struct Nodes<'a>(Option<std::slice::Iter<'a, IndexedJsonLdNode>>);

impl<'a> Nodes<'a> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, IndexedJsonLdNode>>) -> Self {
		Self(inner)
	}
}

impl<'a> Iterator for Nodes<'a> {
	type Item = &'a IndexedJsonLdNode;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a IndexedJsonLdNode> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}
