use super::{InvalidExpandedJson, Traverse, TryFromJson, TryFromJsonObject};
use crate::{
	id, object, utils, Indexed, IndexedObject, Object, Objects, Reference, StrippedIndexedObject,
	Term, ToReference,
};
use contextual::{IntoRefWithContext, WithContext};
use derivative::Derivative;
use iref::IriBuf;
use json_ld_syntax::{Entry, Keyword};
use locspan::{BorrowStripped, Meta, Stripped, StrippedEq, StrippedPartialEq};
use rdf_types::{BlankIdBuf, Vocabulary, VocabularyMut};
use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

pub mod multiset;
pub mod properties;
pub mod reverse_properties;

pub use multiset::Multiset;
pub use properties::Properties;
pub use reverse_properties::ReverseProperties;

pub type Graph<T, B, M> = HashSet<StrippedIndexedObject<T, B, M>>;

/// Node parts.
pub struct Parts<T = IriBuf, B = BlankIdBuf, M = ()> {
	/// Identifier.
	///
	/// This is the `@id` field.
	pub id: Option<Entry<Reference<T, B>, M>>,

	/// Types.
	///
	/// This is the `@type` field.
	pub types: Option<TypeEntry<T, B, M>>,

	/// Associated graph.
	///
	/// This is the `@graph` field.
	pub graph: Option<GraphEntry<T, B, M>>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	pub included: Option<IncludedEntry<T, B, M>>,

	/// Properties.
	///
	/// Any non-keyword field.
	pub properties: Properties<T, B, M>,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	pub reverse_properties: Option<Entry<ReverseProperties<T, B, M>, M>>,
}

pub type IndexedNode<T, B, M> = Meta<Indexed<Node<T, B, M>, M>, M>;

/// Indexed node, without regard for its metadata.
pub type StrippedIndexedNode<T, B, M> = Stripped<IndexedNode<T, B, M>>;

/// Node object.
///
/// A node object represents zero or more properties of a node in the graph serialized by a JSON-LD document.
/// A node is defined by its identifier (`@id` field), types, properties and reverse properties.
/// In addition, a node may represent a graph (`@graph field`) and includes nodes
/// (`@included` field).
// NOTE it may be better to use BTreeSet instead of HashSet to have some ordering?
//      in which case the Json bound should be lifted.
#[derive(Derivative, Clone)]
#[derivative(Eq(bound = "T: Eq + Hash, B: Eq + Hash, M: Eq"))]
pub struct Node<T = IriBuf, B = BlankIdBuf, M = ()> {
	/// Identifier.
	///
	/// This is the `@id` field.
	pub(crate) id: Option<Entry<Reference<T, B>, M>>,

	/// Types.
	///
	/// This is the `@type` field.
	pub(crate) types: Option<TypeEntry<T, B, M>>,

	/// Associated graph.
	///
	/// This is the `@graph` field.
	pub(crate) graph: Option<GraphEntry<T, B, M>>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	pub(crate) included: Option<IncludedEntry<T, B, M>>,

	/// Properties.
	///
	/// Any non-keyword field.
	pub(crate) properties: Properties<T, B, M>,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	pub(crate) reverse_properties: Option<Entry<ReverseProperties<T, B, M>, M>>,
}

impl<T, B, M> Default for Node<T, B, M> {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl<T, B, M> Node<T, B, M> {
	/// Creates a new empty node.
	#[inline(always)]
	pub fn new() -> Self {
		Self {
			id: None,
			types: None,
			graph: None,
			included: None,
			properties: Properties::new(),
			reverse_properties: None,
		}
	}

	/// Creates a new empty node with the given id.
	#[inline(always)]
	pub fn with_id(id: Entry<Reference<T, B>, M>) -> Self {
		Self {
			id: Some(id),
			types: None,
			graph: None,
			included: None,
			properties: Properties::new(),
			reverse_properties: None,
		}
	}

	pub fn from_parts(parts: Parts<T, B, M>) -> Self {
		Self {
			id: parts.id,
			types: parts.types,
			graph: parts.graph,
			included: parts.included,
			properties: parts.properties,
			reverse_properties: parts.reverse_properties,
		}
	}

	pub fn into_parts(self) -> Parts<T, B, M> {
		Parts {
			id: self.id,
			types: self.types,
			graph: self.graph,
			included: self.included,
			properties: self.properties,
			reverse_properties: self.reverse_properties,
		}
	}

	/// Get the identifier of the node.
	///
	/// This correspond to the `@id` field of the JSON object.
	#[inline(always)]
	pub fn id(&self) -> Option<&Meta<Reference<T, B>, M>> {
		self.id.as_ref().map(Entry::as_value)
	}

	/// Get the identifier of the node.
	///
	/// This correspond to the `@id` field of the JSON object.
	#[inline(always)]
	pub fn id_entry(&self) -> Option<&Entry<Reference<T, B>, M>> {
		self.id.as_ref()
	}

	/// Sets the idntifier of this node.
	#[inline(always)]
	pub fn set_id(&mut self, id: Option<Entry<Reference<T, B>, M>>) {
		self.id = id
	}

	/// Assigns an identifier to this node and every other node included in this one using the given `generator`.
	pub fn identify_all_in<N, G: id::Generator<T, B, M, N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
	) where
		M: Clone,
	{
		if self.id.is_none() {
			let value = generator.next(vocabulary);
			self.id = Some(Entry::new(value.metadata().clone(), value.cast()))
		}

		for (_, objects) in self.properties_mut() {
			for object in objects {
				object.identify_all_in(vocabulary, generator);
			}
		}

		if let Some(reverse_properties) = self.reverse_properties_mut() {
			for (_, nodes) in reverse_properties.iter_mut() {
				for node in nodes {
					node.identify_all_in(vocabulary, generator);
				}
			}
		}
	}

	/// Assigns an identifier to this node and every other node included in this one using the given `generator`.
	pub fn identify_all<G: id::Generator<T, B, M, ()>>(&mut self, generator: &mut G)
	where
		M: Clone,
	{
		self.identify_all_in(&mut (), generator)
	}

	/// Get the node's as an IRI if possible.
	///
	/// Returns the node's IRI id if any. Returns `None` otherwise.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<&T> {
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
	pub fn as_str(&self) -> Option<&str>
	where
		T: AsRef<str>,
	{
		match self.as_iri() {
			Some(iri) => Some(iri.as_ref()),
			None => None,
		}
	}

	/// Get the list of the node's types.
	#[inline(always)]
	pub fn types(&self) -> &[Meta<Reference<T, B>, M>] {
		match self.types.as_ref() {
			Some(entry) => &entry.value,
			None => &[],
		}
	}

	/// Returns a mutable reference to the node's types.
	#[inline(always)]
	pub fn types_mut(&mut self) -> &mut [Meta<Reference<T, B>, M>] {
		match self.types.as_mut() {
			Some(entry) => &mut entry.value,
			None => &mut [],
		}
	}

	pub fn type_entry_or_default(
		&mut self,
		key_metadata: M,
		value_metadata: M,
	) -> &mut TypeEntry<T, B, M> {
		self.types
			.get_or_insert_with(|| Entry::new(key_metadata, Meta(Vec::new(), value_metadata)))
	}

	pub fn type_entry_or_insert(
		&mut self,
		key_metadata: M,
		value: TypeEntryValue<T, B, M>,
	) -> &mut TypeEntry<T, B, M> {
		self.types
			.get_or_insert_with(|| Entry::new(key_metadata, value))
	}

	pub fn type_entry_or_insert_with(
		&mut self,
		f: impl FnOnce() -> TypeEntry<T, B, M>,
	) -> &mut TypeEntry<T, B, M> {
		self.types.get_or_insert_with(f)
	}

	pub fn type_entry(&self) -> Option<&TypeEntry<T, B, M>> {
		self.types.as_ref()
	}

	pub fn set_type_entry(&mut self, entry: Option<TypeEntry<T, B, M>>) {
		self.types = entry
	}

	/// Checks if the node has the given type.
	#[inline]
	pub fn has_type<U>(&self, ty: &U) -> bool
	where
		Reference<T, B>: PartialEq<U>,
	{
		for self_ty in self.types() {
			if self_ty.value() == ty {
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
	pub fn graph(&self) -> Option<&Meta<Graph<T, B, M>, M>> {
		self.graph.as_deref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_mut(&mut self) -> Option<&mut Meta<Graph<T, B, M>, M>> {
		self.graph.as_deref_mut()
	}

	/// If the node is a graph object, get the graph.
	#[inline(always)]
	pub fn graph_entry(&self) -> Option<&GraphEntry<T, B, M>> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_entry_mut(&mut self) -> Option<&mut GraphEntry<T, B, M>> {
		self.graph.as_mut()
	}

	/// Set the graph.
	#[inline(always)]
	pub fn set_graph(&mut self, graph: Option<GraphEntry<T, B, M>>) {
		self.graph = graph
	}

	/// Get the set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_entry(&self) -> Option<&IncludedEntry<T, B, M>> {
		self.included.as_ref()
	}

	/// Get the mutable set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_entry_mut(&mut self) -> Option<&mut IncludedEntry<T, B, M>> {
		self.included.as_mut()
	}

	/// Set the set of nodes included by the node.
	#[inline(always)]
	pub fn set_included(&mut self, included: Option<IncludedEntry<T, B, M>>) {
		self.included = included
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn properties(&self) -> &Properties<T, B, M> {
		&self.properties
	}

	/// Returns a mutable reference to the properties of the node.
	#[inline(always)]
	pub fn properties_mut(&mut self) -> &mut Properties<T, B, M> {
		&mut self.properties
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn reverse_properties(&self) -> Option<&Meta<ReverseProperties<T, B, M>, M>> {
		self.reverse_properties.as_ref().map(Entry::as_value)
	}

	/// Returns a reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_entry(&self) -> Option<&Entry<ReverseProperties<T, B, M>, M>> {
		self.reverse_properties.as_ref()
	}

	/// Returns a mutable reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_mut(&mut self) -> Option<&mut Entry<ReverseProperties<T, B, M>, M>> {
		self.reverse_properties.as_mut()
	}

	pub fn set_reverse_properties(
		&mut self,
		reverse_properties: Option<Entry<ReverseProperties<T, B, M>, M>>,
	) {
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
	#[inline(always)]
	pub fn into_unnamed_graph(self) -> Result<Entry<Graph<T, B, M>, M>, Self> {
		if self.is_unnamed_graph() {
			Ok(self.graph.unwrap())
		} else {
			Err(self)
		}
	}

	pub fn traverse(&self) -> Traverse<T, B, M> {
		Traverse::new(Some(super::FragmentRef::Node(self)))
	}

	pub fn entries(&self) -> Entries<T, B, M> {
		Entries {
			id: self.id.as_ref(),
			type_: self.types.as_ref(),
			graph: self.graph.as_ref(),
			included: self.included.as_ref(),
			reverse: self.reverse_properties.as_ref(),
			properties: self.properties.iter(),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> Node<T, B, M> {
	/// Checks if the node object has the given term as key.
	///
	/// # Example
	/// ```
	/// # use json_ld_syntax::Keyword;
	/// # use json_ld_core::Term;
	/// # let node: json_ld_core::Node = json_ld_core::Node::new();
	///
	/// // Checks if the JSON object representation of the node has an `@id` key.
	/// if node.has_key(&Term::Keyword(Keyword::Id)) {
	///   // ...
	/// }
	/// ```
	#[inline(always)]
	pub fn has_key(&self, key: &Term<T, B>) -> bool {
		match key {
			Term::Keyword(Keyword::Id) => self.id.is_some(),
			Term::Keyword(Keyword::Type) => self.types.is_some(),
			Term::Keyword(Keyword::Graph) => self.graph.is_some(),
			Term::Keyword(Keyword::Included) => self.included.is_some(),
			Term::Keyword(Keyword::Reverse) => self.reverse_properties.is_some(),
			Term::Ref(prop) => self.properties.contains(prop),
			_ => false,
		}
	}

	/// Get all the objects associated to the node with the given property.
	#[inline(always)]
	pub fn get<'a, Q: ToReference<T, B>>(&self, prop: Q) -> Objects<T, B, M>
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
	pub fn get_any<'a, Q: ToReference<T, B>>(&self, prop: Q) -> Option<&IndexedObject<T, B, M>>
	where
		T: 'a,
	{
		self.properties.get_any(prop)
	}

	/// Associates the given object to the node through the given property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Meta<Reference<T, B>, M>, value: IndexedObject<T, B, M>) {
		self.properties.insert(prop, value)
	}

	/// Associates all the given objects to the node through the given property.
	///
	/// If there already exists objects associated to the given reverse property,
	/// `reverse_value` is added to the list. Duplicate objects are not removed.
	#[inline(always)]
	pub fn insert_all<Objects: Iterator<Item = IndexedObject<T, B, M>>>(
		&mut self,
		prop: Meta<Reference<T, B>, M>,
		values: Objects,
	) {
		self.properties.insert_all(prop, values)
	}

	pub fn reverse_properties_or_insert(
		&mut self,
		key_metadata: M,
		props: Meta<ReverseProperties<T, B, M>, M>,
	) -> &mut Entry<ReverseProperties<T, B, M>, M> {
		self.reverse_properties
			.get_or_insert_with(|| Entry::new(key_metadata, props))
	}

	pub fn reverse_properties_or_default(
		&mut self,
		key_metadata: M,
		value_metadata: M,
	) -> &mut Entry<ReverseProperties<T, B, M>, M> {
		self.reverse_properties.get_or_insert_with(|| {
			Entry::new(key_metadata, Meta(ReverseProperties::new(), value_metadata))
		})
	}

	pub fn reverse_properties_or_insert_with(
		&mut self,
		f: impl FnOnce() -> Entry<ReverseProperties<T, B, M>, M>,
	) -> &mut Entry<ReverseProperties<T, B, M>, M> {
		self.reverse_properties.get_or_insert_with(f)
	}

	/// Equivalence operator.
	///
	/// Equivalence is different from equality for anonymous objects.
	/// Anonymous node objects have an implicit unlabeled blank nodes and thus never equivalent.
	pub fn equivalent(&self, other: &Self) -> bool {
		if self.id_entry().is_some() && other.id_entry().is_some() {
			self.stripped() == other.stripped()
		} else {
			false
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M: PartialEq> PartialEq for Node<T, B, M> {
	fn eq(&self, other: &Self) -> bool {
		self.id.eq(&other.id)
			&& multiset::compare_unordered_opt(
				self.types.as_ref().map(|t| t.as_slice()),
				other.types.as_ref().map(|t| t.as_slice()),
			) && self.graph.as_ref().map(Entry::as_value).map(Meta::value)
			== other.graph.as_ref().map(Entry::as_value).map(Meta::value)
			&& self.included.as_ref().map(Entry::as_value).map(Meta::value)
				== other
					.included
					.as_ref()
					.map(Entry::as_value)
					.map(Meta::value)
			&& self.properties.eq(&other.properties)
			&& self.reverse_properties.eq(&other.reverse_properties)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedPartialEq for Node<T, B, M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.id.stripped_eq(&other.id)
			&& multiset::compare_stripped_unordered_opt(
				self.types.as_ref().map(|t| t.as_slice()),
				other.types.as_ref().map(|t| t.as_slice()),
			) && self.graph.as_ref().map(Entry::as_value).map(Meta::value)
			== other.graph.as_ref().map(Entry::as_value).map(Meta::value)
			&& self.included.as_ref().map(Entry::as_value).map(Meta::value)
				== other
					.included
					.as_ref()
					.map(Entry::as_value)
					.map(Meta::value)
			&& self.properties.stripped_eq(&other.properties)
			&& self
				.reverse_properties
				.stripped_eq(&other.reverse_properties)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedEq for Node<T, B, M> {}

impl<T, B, M> Indexed<Node<T, B, M>, M> {
	pub fn entries(&self) -> IndexedEntries<T, B, M> {
		IndexedEntries {
			index: self.index(),
			inner: self.inner().entries(),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> Indexed<Node<T, B, M>, M> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}
}

#[derive(Derivative, PartialEq, Eq)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum EntryKeyRef<'a, T, B, M> {
	Id,
	Type,
	Graph,
	Included,
	Reverse,
	Property(Meta<&'a Reference<T, B>, &'a M>),
}

impl<'a, T, B, M> EntryKeyRef<'a, T, B, M> {
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

	pub fn into_str(self) -> &'a str
	where
		T: AsRef<str>,
		B: AsRef<str>,
	{
		match self {
			Self::Id => "@id",
			Self::Type => "@type",
			Self::Graph => "@graph",
			Self::Included => "@included",
			Self::Reverse => "@reverse",
			Self::Property(p) => p.as_str(),
		}
	}

	pub fn as_str(&self) -> &'a str
	where
		T: AsRef<str>,
		B: AsRef<str>,
	{
		self.into_str()
	}
}

impl<'a, T, B, N: Vocabulary<Iri=T, BlankId=B>, M> IntoRefWithContext<'a, str, N> for EntryKeyRef<'a, T, B, M> {
	fn into_ref_with(self, vocabulary: &'a N) -> &'a str {
		match self {
			EntryKeyRef::Id => "@id",
			EntryKeyRef::Type => "@type",
			EntryKeyRef::Graph => "@graph",
			EntryKeyRef::Included => "@included",
			EntryKeyRef::Reverse => "@reverse",
			EntryKeyRef::Property(p) => p.0.with(vocabulary).as_str(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum EntryValueRef<'a, T, B, M> {
	Id(Meta<&'a Reference<T, B>, &'a M>),
	Type(&'a TypeEntryValue<T, B, M>),
	Graph(&'a HashSet<StrippedIndexedObject<T, B, M>>),
	Included(&'a HashSet<StrippedIndexedNode<T, B, M>>),
	Reverse(&'a ReverseProperties<T, B, M>),
	Property(&'a [StrippedIndexedObject<T, B, M>]),
}

impl<'a, T, B, M> EntryValueRef<'a, T, B, M> {
	pub fn is_json_array(&self) -> bool {
		matches!(
			self,
			Self::Type(_) | Self::Graph(_) | Self::Included(_) | Self::Property(_)
		)
	}

	pub fn is_json_object(&self) -> bool {
		matches!(self, Self::Reverse(_))
	}

	fn sub_fragments(&self) -> SubFragments<'a, T, B, M> {
		match self {
			Self::Type(l) => SubFragments::Type(l.iter()),
			Self::Graph(g) => SubFragments::Graph(g.iter()),
			Self::Included(i) => SubFragments::Included(i.iter()),
			Self::Reverse(r) => SubFragments::Reverse(r.iter()),
			Self::Property(p) => SubFragments::Property(p.iter()),
			_ => SubFragments::None,
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum EntryRef<'a, T, B, M> {
	Id(&'a Entry<Reference<T, B>, M>),
	Type(&'a TypeEntry<T, B, M>),
	Graph(&'a GraphEntry<T, B, M>),
	Included(&'a IncludedEntry<T, B, M>),
	Reverse(&'a Entry<ReverseProperties<T, B, M>, M>),
	Property(
		Meta<&'a Reference<T, B>, &'a M>,
		&'a [StrippedIndexedObject<T, B, M>],
	),
}

impl<'a, T, B, M> EntryRef<'a, T, B, M> {
	pub fn into_key(self) -> EntryKeyRef<'a, T, B, M> {
		match self {
			Self::Id(_) => EntryKeyRef::Id,
			Self::Type(_) => EntryKeyRef::Type,
			Self::Graph(_) => EntryKeyRef::Graph,
			Self::Included(_) => EntryKeyRef::Included,
			Self::Reverse(_) => EntryKeyRef::Reverse,
			Self::Property(k, _) => EntryKeyRef::Property(k),
		}
	}

	pub fn key(&self) -> EntryKeyRef<'a, T, B, M> {
		self.into_key()
	}

	pub fn into_value(self) -> EntryValueRef<'a, T, B, M> {
		match self {
			Self::Id(v) => EntryValueRef::Id(Meta(&v.value, &v.key_metadata)),
			Self::Type(v) => EntryValueRef::Type(&v.value),
			Self::Graph(v) => EntryValueRef::Graph(v),
			Self::Included(v) => EntryValueRef::Included(v),
			Self::Reverse(v) => EntryValueRef::Reverse(v),
			Self::Property(_, v) => EntryValueRef::Property(v),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a, T, B, M> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (EntryKeyRef<'a, T, B, M>, EntryValueRef<'a, T, B, M>) {
		match self {
			Self::Id(v) => (
				EntryKeyRef::Id,
				EntryValueRef::Id(Meta(&v.value, &v.key_metadata)),
			),
			Self::Type(v) => (EntryKeyRef::Type, EntryValueRef::Type(&v.value)),
			Self::Graph(v) => (EntryKeyRef::Graph, EntryValueRef::Graph(v)),
			Self::Included(v) => (EntryKeyRef::Included, EntryValueRef::Included(v)),
			Self::Reverse(v) => (EntryKeyRef::Reverse, EntryValueRef::Reverse(v)),
			Self::Property(k, v) => (EntryKeyRef::Property(k), EntryValueRef::Property(v)),
		}
	}

	pub fn as_key_value(&self) -> (EntryKeyRef<'a, T, B, M>, EntryValueRef<'a, T, B, M>) {
		match self {
			Self::Id(v) => (
				EntryKeyRef::Id,
				EntryValueRef::Id(Meta(&v.value, &v.key_metadata)),
			),
			Self::Type(v) => (EntryKeyRef::Type, EntryValueRef::Type(&v.value)),
			Self::Graph(v) => (EntryKeyRef::Graph, EntryValueRef::Graph(v)),
			Self::Included(v) => (EntryKeyRef::Included, EntryValueRef::Included(v)),
			Self::Reverse(v) => (EntryKeyRef::Reverse, EntryValueRef::Reverse(v)),
			Self::Property(k, v) => (EntryKeyRef::Property(*k), EntryValueRef::Property(v)),
		}
	}
}

pub type TypeEntryValue<T, B, M> = Meta<Vec<Meta<Reference<T, B>, M>>, M>;
pub type TypeEntry<T, B, M> = Entry<Vec<Meta<Reference<T, B>, M>>, M>;
pub type GraphEntry<T, B, M> = Entry<Graph<T, B, M>, M>;
pub type IncludedEntry<T, B, M> = Entry<HashSet<StrippedIndexedNode<T, B, M>>, M>;

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct Entries<'a, T, B, M> {
	id: Option<&'a Entry<Reference<T, B>, M>>,
	type_: Option<&'a TypeEntry<T, B, M>>,
	graph: Option<&'a GraphEntry<T, B, M>>,
	included: Option<&'a IncludedEntry<T, B, M>>,
	reverse: Option<&'a Entry<ReverseProperties<T, B, M>, M>>,
	properties: properties::Iter<'a, T, B, M>,
}

impl<'a, T, B, M> Iterator for Entries<'a, T, B, M> {
	type Item = EntryRef<'a, T, B, M>;

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

impl<'a, T, B, M> ExactSizeIterator for Entries<'a, T, B, M> {}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct IndexedEntries<'a, T, B, M> {
	index: Option<&'a str>,
	inner: Entries<'a, T, B, M>,
}

impl<'a, T, B, M> Iterator for IndexedEntries<'a, T, B, M> {
	type Item = IndexedEntryRef<'a, T, B, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.inner.len() + if self.index.is_some() { 1 } else { 0 };
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.index
			.take()
			.map(IndexedEntryRef::Index)
			.or_else(|| self.inner.next().map(IndexedEntryRef::Node))
	}
}

impl<'a, T, B, M> ExactSizeIterator for IndexedEntries<'a, T, B, M> {}

#[derive(Derivative, PartialEq, Eq)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum IndexedEntryKeyRef<'a, T, B, M> {
	Index,
	Node(EntryKeyRef<'a, T, B, M>),
}

impl<'a, T, B, M> IndexedEntryKeyRef<'a, T, B, M> {
	pub fn into_keyword(self) -> Option<Keyword> {
		match self {
			Self::Index => Some(Keyword::Index),
			Self::Node(e) => e.into_keyword(),
		}
	}

	pub fn as_keyword(&self) -> Option<Keyword> {
		self.into_keyword()
	}

	pub fn into_str(self) -> &'a str
	where
		T: AsRef<str>,
		B: AsRef<str>,
	{
		match self {
			Self::Index => "@index",
			Self::Node(e) => e.into_str(),
		}
	}

	pub fn as_str(&self) -> &'a str
	where
		T: AsRef<str>,
		B: AsRef<str>,
	{
		self.into_str()
	}
}

impl<'a, T, B, N: Vocabulary<Iri=T, BlankId=B>, M> IntoRefWithContext<'a, str, N>
	for IndexedEntryKeyRef<'a, T, B, M>
{
	fn into_ref_with(self, vocabulary: &'a N) -> &'a str {
		match self {
			IndexedEntryKeyRef::Index => "@index",
			IndexedEntryKeyRef::Node(e) => e.into_with(vocabulary).into_str(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum IndexedEntryValueRef<'a, T, B, M> {
	Index(&'a str),
	Node(EntryValueRef<'a, T, B, M>),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum IndexedEntryRef<'a, T, B, M> {
	Index(&'a str),
	Node(EntryRef<'a, T, B, M>),
}

impl<'a, T, B, M> IndexedEntryRef<'a, T, B, M> {
	pub fn into_key(self) -> IndexedEntryKeyRef<'a, T, B, M> {
		match self {
			Self::Index(_) => IndexedEntryKeyRef::Index,
			Self::Node(e) => IndexedEntryKeyRef::Node(e.key()),
		}
	}

	pub fn key(&self) -> IndexedEntryKeyRef<'a, T, B, M> {
		self.into_key()
	}

	pub fn into_value(self) -> IndexedEntryValueRef<'a, T, B, M> {
		match self {
			Self::Index(v) => IndexedEntryValueRef::Index(v),
			Self::Node(e) => IndexedEntryValueRef::Node(e.value()),
		}
	}

	pub fn value(&self) -> IndexedEntryValueRef<'a, T, B, M> {
		self.into_value()
	}

	pub fn into_key_value(
		self,
	) -> (
		IndexedEntryKeyRef<'a, T, B, M>,
		IndexedEntryValueRef<'a, T, B, M>,
	) {
		match self {
			Self::Index(v) => (IndexedEntryKeyRef::Index, IndexedEntryValueRef::Index(v)),
			Self::Node(e) => {
				let (k, v) = e.into_key_value();
				(IndexedEntryKeyRef::Node(k), IndexedEntryValueRef::Node(v))
			}
		}
	}

	pub fn as_key_value(
		&self,
	) -> (
		IndexedEntryKeyRef<'a, T, B, M>,
		IndexedEntryValueRef<'a, T, B, M>,
	) {
		self.into_key_value()
	}
}

/// Node object fragment reference.
pub enum FragmentRef<'a, T, B, M> {
	/// Node object entry.
	Entry(EntryRef<'a, T, B, M>),

	/// Node object entry key.
	Key(EntryKeyRef<'a, T, B, M>),

	/// Node object entry value.
	Value(EntryValueRef<'a, T, B, M>),

	/// "@type" entry value fragment.
	TypeFragment(Meta<&'a Reference<T, B>, &'a M>),
}

impl<'a, T, B, M> FragmentRef<'a, T, B, M> {
	pub fn into_id(self) -> Option<Meta<&'a Reference<T, B>, &'a M>> {
		match self {
			Self::Key(EntryKeyRef::Property(id)) => Some(id),
			Self::Value(EntryValueRef::Id(id)) => Some(id),
			Self::TypeFragment(ty) => Some(ty),
			_ => None,
		}
	}

	pub fn as_id(&self) -> Option<&'a Reference<T, B>> {
		match self {
			Self::Key(EntryKeyRef::Property(id)) => Some(id),
			Self::Value(EntryValueRef::Id(id)) => Some(id),
			Self::TypeFragment(ty) => Some(ty),
			_ => None,
		}
	}

	pub fn is_json_array(&self) -> bool {
		match self {
			Self::Value(v) => v.is_json_array(),
			_ => false,
		}
	}

	pub fn is_json_object(&self) -> bool {
		match self {
			Self::Value(v) => v.is_json_object(),
			_ => false,
		}
	}

	pub fn sub_fragments(&self) -> SubFragments<'a, T, B, M> {
		match self {
			Self::Entry(e) => SubFragments::Entry(Some(e.key()), Some(e.value())),
			Self::Value(v) => v.sub_fragments(),
			_ => SubFragments::None,
		}
	}
}

pub enum SubFragments<'a, T, B, M> {
	None,
	Entry(
		Option<EntryKeyRef<'a, T, B, M>>,
		Option<EntryValueRef<'a, T, B, M>>,
	),
	Type(std::slice::Iter<'a, Meta<Reference<T, B>, M>>),
	Graph(std::collections::hash_set::Iter<'a, StrippedIndexedObject<T, B, M>>),
	Included(std::collections::hash_set::Iter<'a, StrippedIndexedNode<T, B, M>>),
	Reverse(reverse_properties::Iter<'a, T, B, M>),
	Property(std::slice::Iter<'a, StrippedIndexedObject<T, B, M>>),
}

impl<'a, T, B, M> Iterator for SubFragments<'a, T, B, M> {
	type Item = super::FragmentRef<'a, T, B, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Entry(k, v) => k
				.take()
				.map(|k| super::FragmentRef::NodeFragment(FragmentRef::Key(k)))
				.or_else(|| {
					v.take()
						.map(|v| super::FragmentRef::NodeFragment(FragmentRef::Value(v)))
				}),
			Self::Type(l) => l
				.next_back()
				.map(|t| super::FragmentRef::NodeFragment(FragmentRef::TypeFragment(t.borrow()))),
			Self::Graph(g) => g.next().map(|o| super::FragmentRef::IndexedObject(o)),
			Self::Included(i) => i.next().map(|n| super::FragmentRef::IndexedNode(n)),
			Self::Reverse(r) => r
				.next()
				.map(|(_, n)| super::FragmentRef::IndexedNodeList(n)),
			Self::Property(o) => o.next().map(|o| super::FragmentRef::IndexedObject(o)),
		}
	}
}

impl<T, B, M> object::Any<T, B, M> for Node<T, B, M> {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref<T, B, M> {
		object::Ref::Node(self)
	}
}

impl<T, B, M> TryFrom<Object<T, B, M>> for Node<T, B, M> {
	type Error = Object<T, B, M>;

	#[inline(always)]
	fn try_from(obj: Object<T, B, M>) -> Result<Node<T, B, M>, Object<T, B, M>> {
		match obj {
			Object::Node(node) => Ok(node),
			obj => Err(obj),
		}
	}
}

impl<T: Hash, B: Hash, M: Hash> Hash for Node<T, B, M> {
	#[inline]
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.id.hash(h);
		utils::hash_set_opt(self.types.as_ref().map(Entry::as_value).map(Meta::value), h);
		utils::hash_set_opt(self.graph.as_ref().map(Entry::as_value).map(Meta::value), h);
		utils::hash_set_opt(
			self.included.as_ref().map(Entry::as_value).map(Meta::value),
			h,
		);
		self.properties.hash(h);
		self.reverse_properties.hash(h)
	}
}

impl<T: Hash, B: Hash, M> locspan::StrippedHash for Node<T, B, M> {
	#[inline]
	fn stripped_hash<H: Hasher>(&self, h: &mut H) {
		self.id.stripped_hash(h);
		utils::hash_set_stripped_opt(self.types.as_ref().map(Entry::as_value).map(Meta::value), h);
		utils::hash_set_opt(self.graph.as_ref().map(Entry::as_value).map(Meta::value), h);
		utils::hash_set_opt(
			self.included.as_ref().map(Entry::as_value).map(Meta::value),
			h,
		);
		self.properties.stripped_hash(h);
		self.reverse_properties.stripped_hash(h)
	}
}

// impl<J: JsonHash + JsonClone, K: utils::JsonFrom<J>, T: Id> utils::AsJson<J, K> for Node<T, B, M> {
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
// 	for HashSet<IndexedNode<T, B, M>>
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
pub struct Nodes<'a, T, B, M>(Option<std::slice::Iter<'a, Stripped<IndexedNode<T, B, M>>>>);

impl<'a, T, B, M> Nodes<'a, T, B, M> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, Stripped<IndexedNode<T, B, M>>>>) -> Self {
		Self(inner)
	}
}

impl<'a, T, B, M> Iterator for Nodes<'a, T, B, M> {
	type Item = &'a IndexedNode<T, B, M>;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a IndexedNode<T, B, M>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next().map(|n| &n.0),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> TryFromJsonObject<T, B, M> for Node<T, B, M> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri=T, BlankId=B>,
		mut object: Meta<json_syntax::Object<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		let id = match object
			.remove_unique("@id")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(Entry::new(
				entry.key.into_metadata(),
				Reference::try_from_json_in(vocabulary, entry.value)?,
			)),
			None => None,
		};

		let types = match object
			.remove_unique("@type")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(Entry::new(
				entry.key.into_metadata(),
				Vec::try_from_json_in(vocabulary, entry.value)?,
			)),
			None => None,
		};

		let graph = match object
			.remove_unique("@graph")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(Entry::new(
				entry.key.into_metadata(),
				HashSet::try_from_json_in(vocabulary, entry.value)?,
			)),
			None => None,
		};

		let included = match object
			.remove_unique("@included")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(Entry::new(
				entry.key.into_metadata(),
				HashSet::try_from_json_in(vocabulary, entry.value)?,
			)),
			None => None,
		};

		let reverse_properties = match object
			.remove_unique("@reverse")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(Entry::new(
				entry.key.into_metadata(),
				ReverseProperties::try_from_json_in(vocabulary, entry.value)?,
			)),
			None => None,
		};

		let Meta(properties, meta) = Properties::try_from_json_object_in(vocabulary, object)?;

		Ok(Meta(
			Self {
				id,
				types,
				graph,
				included,
				reverse_properties,
				properties,
			},
			meta,
		))
	}
}
