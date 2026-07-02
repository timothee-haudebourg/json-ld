use crate::object::{ObjectMut, ObjectRef};
use crate::syntax::Keyword;
use crate::{Indexed, IndexedObject, Lenient, Object, Objects, Term, VisitJsonLd, object};
use btree_indexmap::{BTreeIndexSet, Comparable};
use educe::Educe;
use rdf_syntax::Id;
use rdf_syntax::Iri;
use std::convert::TryFrom;
use std::hash::Hash;

pub mod properties;
pub mod reverse_properties;

pub use properties::Properties;
pub use reverse_properties::ReverseProperties;

pub type Graph = BTreeIndexSet<IndexedObject>;

pub type Included = BTreeIndexSet<IndexedNode>;

pub type IndexedNode = Indexed<NodeObject>;

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
pub struct NodeObject {
	/// Identifier.
	///
	/// This is the `@id` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@id", default, skip_serializing_if = "Option::is_none")
	)]
	pub id: Option<Lenient<Id>>,

	/// Types.
	///
	/// This is the `@type` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@type", default, skip_serializing_if = "Option::is_none")
	)]
	pub types: Option<Vec<Lenient<Id>>>,

	/// Associated graph.
	///
	/// This is the `@graph` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@graph", default, skip_serializing_if = "Option::is_none")
	)]
	pub graph: Option<Graph>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@included", default, skip_serializing_if = "Option::is_none")
	)]
	pub included: Option<Included>,

	/// Properties.
	///
	/// Any non-keyword field.
	#[cfg_attr(feature = "serde", serde(flatten))]
	pub properties: Properties,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")
	)]
	pub reverse_properties: Option<ReverseProperties>,
}

impl Default for NodeObject {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl NodeObject {
	/// Creates a new empty node.
	#[inline(always)]
	pub fn new() -> Self {
		Self::new_with_id(None)
	}

	/// Creates a new empty node.
	#[inline(always)]
	pub fn new_with_id(id: Option<Lenient<Id>>) -> Self {
		Self {
			id,
			types: None,
			graph: None,
			included: None,
			properties: Properties::new(),
			reverse_properties: None,
		}
	}

	/// Creates a new graph node.
	pub fn new_graph(id: Lenient<Id>, graph: Graph) -> Self {
		Self {
			id: Some(id),
			types: None,
			graph: Some(graph),
			included: None,
			properties: Properties::new(),
			reverse_properties: None,
		}
	}

	/// Returns a mutable reference to the reverse properties of the node.
	///
	/// If no `@reverse` entry is present, one is created.
	#[inline(always)]
	pub fn reverse_properties_mut_or_default(&mut self) -> &mut ReverseProperties {
		self.reverse_properties
			.get_or_insert_with(ReverseProperties::default)
	}

	/// Returns a mutable reference to the included nodes.
	///
	/// If no `@included` entry is present, one is created.
	#[inline(always)]
	pub fn included_mut_or_default(&mut self) -> &mut Included {
		self.included.get_or_insert_with(Included::default)
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

	/// Get the list of the node's types.
	#[inline(always)]
	pub fn types(&self) -> &[Lenient<Id>] {
		match self.types.as_ref() {
			Some(entry) => entry,
			None => &[],
		}
	}

	/// Returns a mutable reference to the node's types.
	#[inline(always)]
	pub fn types_mut(&mut self) -> &mut [Lenient<Id>] {
		match self.types.as_mut() {
			Some(entry) => entry,
			None => &mut [],
		}
	}

	pub fn types_mut_or_default(&mut self) -> &mut Vec<Lenient<Id>> {
		self.types.get_or_insert_with(Vec::new)
	}

	pub fn types_mut_or_insert(&mut self, value: Vec<Lenient<Id>>) -> &mut Vec<Lenient<Id>> {
		self.types.get_or_insert(value)
	}

	pub fn types_mut_or_insert_with(
		&mut self,
		f: impl FnOnce() -> Vec<Lenient<Id>>,
	) -> &mut Vec<Lenient<Id>> {
		self.types.get_or_insert_with(f)
	}

	/// Checks if the node has the given type.
	#[inline]
	pub fn has_type<U>(&self, ty: &U) -> bool
	where
		Lenient<Id>: PartialEq<U>,
	{
		for self_ty in self.types() {
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
	pub fn graph(&self) -> Option<&Graph> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_mut(&mut self) -> Option<&mut Graph> {
		self.graph.as_mut()
	}

	/// If the node is a graph object, get the graph.
	#[inline(always)]
	pub fn graph_entry(&self) -> Option<&Graph> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_entry_mut(&mut self) -> Option<&mut Graph> {
		self.graph.as_mut()
	}

	/// Set the graph.
	#[inline(always)]
	pub fn set_graph_entry(&mut self, graph: Option<Graph>) {
		self.graph = graph
	}

	/// Get the set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_entry(&self) -> Option<&Included> {
		self.included.as_ref()
	}

	/// Get the mutable set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_entry_mut(&mut self) -> Option<&mut Included> {
		self.included.as_mut()
	}

	/// Returns a reference to the set of `@included` nodes.
	pub fn included(&self) -> Option<&Included> {
		self.included.as_ref()
	}

	/// Returns a mutable reference to the set of `@included` nodes.
	pub fn included_mut(&mut self) -> Option<&mut Included> {
		self.included.as_mut()
	}

	/// Set the set of nodes included by the node.
	#[inline(always)]
	pub fn set_included(&mut self, included: Option<Included>) {
		self.included = included
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn properties(&self) -> &Properties {
		&self.properties
	}

	/// Returns a mutable reference to the properties of the node.
	#[inline(always)]
	pub fn properties_mut(&mut self) -> &mut Properties {
		&mut self.properties
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn reverse_properties(&self) -> Option<&ReverseProperties> {
		self.reverse_properties.as_ref()
	}

	/// Returns a reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_entry(&self) -> Option<&ReverseProperties> {
		self.reverse_properties.as_ref()
	}

	/// Returns a mutable reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_mut(&mut self) -> Option<&mut ReverseProperties> {
		self.reverse_properties.as_mut()
	}

	pub fn set_reverse_properties(&mut self, reverse_properties: Option<ReverseProperties>) {
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
	pub fn into_unnamed_graph(self) -> Result<Graph, Self> {
		if self.is_unnamed_graph() {
			Ok(self.graph.unwrap())
		} else {
			Err(self)
		}
	}

	pub fn entries(&self) -> Entries<'_> {
		Entries {
			id: self.id.as_ref(),
			type_: self.types.as_deref(),
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
	/// use json_ld::{NodeObject, Term};
	/// use json_ld::syntax::Keyword;
	///
	/// let node = NodeObject::new();
	///
	/// // Checks if the JSON object representation of the node has an `@id` key.
	/// if node.has_key(&Term::Keyword(Keyword::Id)) {
	///   // ...
	/// }
	/// ```
	#[inline(always)]
	pub fn has_key(&self, key: &Term) -> bool {
		match key {
			Term::Keyword(Keyword::Id) => self.id.is_some(),
			Term::Keyword(Keyword::Type) => self.types.is_some(),
			Term::Keyword(Keyword::Graph) => self.graph.is_some(),
			Term::Keyword(Keyword::Included) => self.included.is_some(),
			Term::Keyword(Keyword::Reverse) => self.reverse_properties.is_some(),
			Term::Id(prop) => self.properties.contains(prop),
			_ => false,
		}
	}

	/// Get all the objects associated to the node with the given property.
	#[inline(always)]
	pub fn get<Q>(&self, prop: &Q) -> Objects<'_>
	where
		Q: ?Sized + Hash + Comparable<Lenient<Id>>,
	{
		self.properties.get(prop)
	}

	/// Get one of the objects associated to the node with the given property.
	///
	/// If multiple objects are attached to the node with this property, there are no guaranties
	/// on which object will be returned.
	#[inline(always)]
	pub fn get_any<Q>(&self, prop: &Q) -> Option<&IndexedObject>
	where
		Q: ?Sized + Hash + Comparable<Lenient<Id>>,
	{
		self.properties.get_any(prop)
	}

	/// Associates the given object to the node through the given property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Lenient<Id>, value: IndexedObject) {
		self.properties.insert(prop, value)
	}

	/// Associates all the given objects to the node through the given property.
	///
	/// If there already exists objects associated to the given reverse property,
	/// `reverse_value` is added to the list. Duplicate objects are not removed.
	#[inline(always)]
	pub fn insert_all<Objects: IntoIterator<Item = IndexedObject>>(
		&mut self,
		prop: Lenient<Id>,
		values: Objects,
	) {
		self.properties.insert_all(prop, values)
	}

	pub fn reverse_properties_or_insert(
		&mut self,
		props: ReverseProperties,
	) -> &mut ReverseProperties {
		self.reverse_properties.get_or_insert(props)
	}

	pub fn reverse_properties_or_default(&mut self) -> &mut ReverseProperties {
		self.reverse_properties
			.get_or_insert_with(ReverseProperties::default)
	}

	pub fn reverse_properties_or_insert_with(
		&mut self,
		f: impl FnOnce() -> ReverseProperties,
	) -> &mut ReverseProperties {
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

impl VisitJsonLd for NodeObject {
	fn visit_with(&self, f: &mut impl FnMut(ObjectRef)) {
		f(ObjectRef::Node(self));
		self.graph.visit_with(f);
		self.included.visit_with(f);
		self.properties.visit_with(f);
		self.reverse_properties.visit_with(f);
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(ObjectMut)) {
		f(ObjectMut::Node(self));
		self.graph.visit_mut_with(f);
		self.included.visit_mut_with(f);
		self.properties.visit_mut_with(f);
		self.reverse_properties.visit_mut_with(f);
	}
}

impl Indexed<NodeObject> {
	pub fn entries(&self) -> IndexedEntries<'_> {
		IndexedEntries {
			index: self.index(),
			inner: self.inner().entries(),
		}
	}
}

impl Indexed<NodeObject> {
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
	Property(&'a Lenient<Id>),
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
	Id(&'a Lenient<Id>),
	Type(&'a [Lenient<Id>]),
	Graph(&'a BTreeIndexSet<IndexedObject>),
	Included(&'a BTreeIndexSet<IndexedNode>),
	Reverse(&'a ReverseProperties),
	Property(&'a [IndexedObject]),
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
	Id(&'a Lenient<Id>),
	Type(&'a [Lenient<Id>]),
	Graph(&'a Graph),
	Included(&'a Included),
	Reverse(&'a ReverseProperties),
	Property(&'a Lenient<Id>, &'a [IndexedObject]),
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
	id: Option<&'a Lenient<Id>>,
	type_: Option<&'a [Lenient<Id>]>,
	graph: Option<&'a Graph>,
	included: Option<&'a Included>,
	reverse: Option<&'a ReverseProperties>,
	properties: properties::Iter<'a>,
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

impl object::AnyObject for NodeObject {
	#[inline(always)]
	fn as_ref(&self) -> object::ObjectRef<'_> {
		object::ObjectRef::Node(self)
	}
}

impl TryFrom<Object> for NodeObject {
	type Error = Object;

	#[inline(always)]
	fn try_from(obj: Object) -> Result<NodeObject, Object> {
		match obj {
			Object::Node(node) => Ok(*node),
			obj => Err(obj),
		}
	}
}

/// Iterator through indexed nodes.
pub struct Nodes<'a>(Option<std::slice::Iter<'a, IndexedNode>>);

impl<'a> Nodes<'a> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, IndexedNode>>) -> Self {
		Self(inner)
	}
}

impl<'a> Iterator for Nodes<'a> {
	type Item = &'a IndexedNode;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a IndexedNode> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}
