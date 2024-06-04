use super::{InvalidExpandedJson, Traverse, TryFromJson, TryFromJsonObject};
use crate::{object, utils, Id, Indexed, IndexedObject, Object, Objects, Relabel, Term};
use contextual::{IntoRefWithContext, WithContext};
use educe::Educe;
use indexmap::IndexSet;
use iref::IriBuf;
use json_ld_syntax::{IntoJson, IntoJsonWithContext, Keyword};
use rdf_types::{BlankIdBuf, Generator, Subject, Vocabulary, VocabularyMut};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

pub mod multiset;
pub mod properties;
pub mod reverse_properties;

pub use multiset::Multiset;
pub use properties::Properties;
pub use reverse_properties::ReverseProperties;

pub type Graph<T, B> = IndexSet<IndexedObject<T, B>>;

pub type Included<T, B> = IndexSet<IndexedNode<T, B>>;

pub type IndexedNode<T = IriBuf, B = BlankIdBuf> = Indexed<Node<T, B>>;

/// Node object.
///
/// A node object represents zero or more properties of a node in the graph serialized by a JSON-LD document.
/// A node is defined by its identifier (`@id` field), types, properties and reverse properties.
/// In addition, a node may represent a graph (`@graph field`) and includes nodes
/// (`@included` field).
// NOTE it may be better to use BTreeSet instead of HashSet to have some ordering?
//      in which case the Json bound should be lifted.
#[derive(Educe, Debug, Clone)]
#[educe(Eq(bound = "T: Eq + Hash, B: Eq + Hash"))]
pub struct Node<T = IriBuf, B = BlankIdBuf> {
	/// Identifier.
	///
	/// This is the `@id` field.
	pub id: Option<Id<T, B>>,

	/// Types.
	///
	/// This is the `@type` field.
	pub types: Option<Vec<Id<T, B>>>,

	/// Associated graph.
	///
	/// This is the `@graph` field.
	pub graph: Option<Graph<T, B>>,

	/// Included nodes.
	///
	/// This is the `@included` field.
	pub included: Option<Included<T, B>>,

	/// Properties.
	///
	/// Any non-keyword field.
	pub properties: Properties<T, B>,

	/// Reverse properties.
	///
	/// This is the `@reverse` field.
	pub reverse_properties: Option<ReverseProperties<T, B>>,
}

impl<T, B> Default for Node<T, B> {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl<T, B> Node<T, B> {
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
	pub fn with_id(id: Id<T, B>) -> Self {
		Self {
			id: Some(id),
			types: None,
			graph: None,
			included: None,
			properties: Properties::new(),
			reverse_properties: None,
		}
	}

	/// Creates a new graph node.
	pub fn new_graph(id: Id<T, B>, graph: Graph<T, B>) -> Self {
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
	pub fn reverse_properties_mut_or_default(&mut self) -> &mut ReverseProperties<T, B> {
		self.reverse_properties
			.get_or_insert_with(ReverseProperties::default)
	}

	/// Returns a mutable reference to the included nodes.
	///
	/// If no `@included` entry is present, one is created.
	#[inline(always)]
	pub fn included_mut_or_default(&mut self) -> &mut Included<T, B> {
		self.included.get_or_insert_with(Included::default)
	}

	/// Assigns an identifier to this node and every other node included in this
	/// one using the given `generator`.
	pub fn identify_all_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&mut self,
		vocabulary: &mut V,
		generator: &mut G,
	) where
		T: Eq + Hash,
		B: Eq + Hash,
	{
		if self.id.is_none() {
			self.id = Some(generator.next(vocabulary).into())
		}

		if let Some(graph) = self.graph_mut() {
			*graph = std::mem::take(graph)
				.into_iter()
				.map(|mut o| {
					o.identify_all_with(vocabulary, generator);
					o
				})
				.collect();
		}

		if let Some(included) = self.included_mut() {
			*included = std::mem::take(included)
				.into_iter()
				.map(|mut n| {
					n.identify_all_with(vocabulary, generator);
					n
				})
				.collect();
		}

		for (_, objects) in self.properties_mut() {
			for object in objects {
				object.identify_all_with(vocabulary, generator);
			}
		}

		if let Some(reverse_properties) = self.reverse_properties_mut() {
			for (_, nodes) in reverse_properties.iter_mut() {
				for node in nodes {
					node.identify_all_with(vocabulary, generator);
				}
			}
		}
	}

	/// Assigns an identifier to this node and every other node included in this one using the given `generator`.
	pub fn identify_all<G: Generator>(&mut self, generator: &mut G)
	where
		T: Eq + Hash,
		B: Eq + Hash,
		(): Vocabulary<Iri = T, BlankId = B>,
	{
		self.identify_all_with(&mut (), generator)
	}

	/// Puts this node object literals into canonical form using the given
	/// `buffer`.
	///
	/// The buffer is used to compute the canonical form of numbers.
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		for (_, objects) in self.properties_mut() {
			for object in objects {
				object.canonicalize_with(buffer)
			}
		}

		if let Some(reverse_properties) = self.reverse_properties_mut() {
			for (_, nodes) in reverse_properties.iter_mut() {
				for node in nodes {
					node.canonicalize_with(buffer)
				}
			}
		}
	}

	/// Puts this node object literals into canonical form.
	pub fn canonicalize(&mut self) {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer)
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
	pub fn types(&self) -> &[Id<T, B>] {
		match self.types.as_ref() {
			Some(entry) => entry,
			None => &[],
		}
	}

	/// Returns a mutable reference to the node's types.
	#[inline(always)]
	pub fn types_mut(&mut self) -> &mut [Id<T, B>] {
		match self.types.as_mut() {
			Some(entry) => entry,
			None => &mut [],
		}
	}

	pub fn types_mut_or_default(&mut self) -> &mut Vec<Id<T, B>> {
		self.types.get_or_insert_with(Vec::new)
	}

	pub fn types_mut_or_insert(&mut self, value: Vec<Id<T, B>>) -> &mut Vec<Id<T, B>> {
		self.types.get_or_insert(value)
	}

	pub fn types_mut_or_insert_with(
		&mut self,
		f: impl FnOnce() -> Vec<Id<T, B>>,
	) -> &mut Vec<Id<T, B>> {
		self.types.get_or_insert_with(f)
	}

	/// Checks if the node has the given type.
	#[inline]
	pub fn has_type<U>(&self, ty: &U) -> bool
	where
		Id<T, B>: PartialEq<U>,
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
	pub fn graph(&self) -> Option<&Graph<T, B>> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_mut(&mut self) -> Option<&mut Graph<T, B>> {
		self.graph.as_mut()
	}

	/// If the node is a graph object, get the graph.
	#[inline(always)]
	pub fn graph_entry(&self) -> Option<&Graph<T, B>> {
		self.graph.as_ref()
	}

	/// If the node is a graph object, get the mutable graph.
	#[inline(always)]
	pub fn graph_entry_mut(&mut self) -> Option<&mut Graph<T, B>> {
		self.graph.as_mut()
	}

	/// Set the graph.
	#[inline(always)]
	pub fn set_graph_entry(&mut self, graph: Option<Graph<T, B>>) {
		self.graph = graph
	}

	/// Get the set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_entry(&self) -> Option<&Included<T, B>> {
		self.included.as_ref()
	}

	/// Get the mutable set of nodes included by this node.
	///
	/// This correspond to the `@included` field in the JSON representation.
	#[inline(always)]
	pub fn included_entry_mut(&mut self) -> Option<&mut Included<T, B>> {
		self.included.as_mut()
	}

	/// Returns a reference to the set of `@included` nodes.
	pub fn included(&self) -> Option<&Included<T, B>> {
		self.included.as_ref()
	}

	/// Returns a mutable reference to the set of `@included` nodes.
	pub fn included_mut(&mut self) -> Option<&mut Included<T, B>> {
		self.included.as_mut()
	}

	/// Set the set of nodes included by the node.
	#[inline(always)]
	pub fn set_included(&mut self, included: Option<Included<T, B>>) {
		self.included = included
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn properties(&self) -> &Properties<T, B> {
		&self.properties
	}

	/// Returns a mutable reference to the properties of the node.
	#[inline(always)]
	pub fn properties_mut(&mut self) -> &mut Properties<T, B> {
		&mut self.properties
	}

	/// Returns a reference to the properties of the node.
	#[inline(always)]
	pub fn reverse_properties(&self) -> Option<&ReverseProperties<T, B>> {
		self.reverse_properties.as_ref()
	}

	/// Returns a reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_entry(&self) -> Option<&ReverseProperties<T, B>> {
		self.reverse_properties.as_ref()
	}

	/// Returns a mutable reference to the reverse properties of the node.
	#[inline(always)]
	pub fn reverse_properties_mut(&mut self) -> Option<&mut ReverseProperties<T, B>> {
		self.reverse_properties.as_mut()
	}

	pub fn set_reverse_properties(&mut self, reverse_properties: Option<ReverseProperties<T, B>>) {
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
	pub fn into_unnamed_graph(self) -> Result<Graph<T, B>, Self> {
		if self.is_unnamed_graph() {
			Ok(self.graph.unwrap())
		} else {
			Err(self)
		}
	}

	pub fn traverse(&self) -> Traverse<T, B> {
		Traverse::new(Some(super::FragmentRef::Node(self)))
	}

	#[inline(always)]
	pub fn count(&self, f: impl FnMut(&super::FragmentRef<T, B>) -> bool) -> usize {
		self.traverse().filter(f).count()
	}

	pub fn entries(&self) -> Entries<T, B> {
		Entries {
			id: self.id.as_ref(),
			type_: self.types.as_deref(),
			graph: self.graph.as_ref(),
			included: self.included.as_ref(),
			reverse: self.reverse_properties.as_ref(),
			properties: self.properties.iter(),
		}
	}

	/// Map the identifiers present in this list (recursively).
	pub fn map_ids<U, C>(
		self,
		mut map_iri: impl FnMut(T) -> U,
		mut map_id: impl FnMut(Id<T, B>) -> Id<U, C>,
	) -> Node<U, C>
	where
		U: Eq + Hash,
		C: Eq + Hash,
	{
		self.map_ids_with(&mut map_iri, &mut map_id)
	}

	pub(crate) fn map_ids_with<U, C>(
		self,
		map_iri: &mut impl FnMut(T) -> U,
		map_id: &mut impl FnMut(Id<T, B>) -> Id<U, C>,
	) -> Node<U, C>
	where
		U: Eq + Hash,
		C: Eq + Hash,
	{
		Node {
			id: self.id.map(&mut *map_id),
			types: self
				.types
				.map(|t| t.into_iter().map(&mut *map_id).collect()),
			graph: self.graph.map(|g| {
				g.into_iter()
					.map(|o| o.map_inner(|o| o.map_ids_with(map_iri, map_id)))
					.collect()
			}),
			included: self.included.map(|i| {
				i.into_iter()
					.map(|o| o.map_inner(|o| o.map_ids_with(map_iri, map_id)))
					.collect()
			}),
			properties: self
				.properties
				.into_iter()
				.map(|(id, values)| {
					(
						map_id(id),
						values
							.into_iter()
							.map(|o| o.map_inner(|o| o.map_ids_with(map_iri, map_id)))
							.collect::<Vec<_>>(),
					)
				})
				.collect(),
			reverse_properties: self.reverse_properties.map(|r| {
				r.into_iter()
					.map(|(id, values)| {
						(
							map_id(id),
							values
								.into_iter()
								.map(|o| o.map_inner(|o| o.map_ids_with(map_iri, map_id)))
								.collect::<Vec<_>>(),
						)
					})
					.collect()
			}),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash> Node<T, B> {
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
			Term::Id(prop) => self.properties.contains(prop),
			_ => false,
		}
	}

	/// Get all the objects associated to the node with the given property.
	#[inline(always)]
	pub fn get<'a, Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(
		&self,
		prop: &Q,
	) -> Objects<T, B>
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
	pub fn get_any<'a, Q: ?Sized + Hash + indexmap::Equivalent<Id<T, B>>>(
		&self,
		prop: &Q,
	) -> Option<&IndexedObject<T, B>>
	where
		T: 'a,
	{
		self.properties.get_any(prop)
	}

	/// Associates the given object to the node through the given property.
	#[inline(always)]
	pub fn insert(&mut self, prop: Id<T, B>, value: IndexedObject<T, B>) {
		self.properties.insert(prop, value)
	}

	/// Associates all the given objects to the node through the given property.
	///
	/// If there already exists objects associated to the given reverse property,
	/// `reverse_value` is added to the list. Duplicate objects are not removed.
	#[inline(always)]
	pub fn insert_all<Objects: Iterator<Item = IndexedObject<T, B>>>(
		&mut self,
		prop: Id<T, B>,
		values: Objects,
	) {
		self.properties.insert_all(prop, values)
	}

	pub fn reverse_properties_or_insert(
		&mut self,
		props: ReverseProperties<T, B>,
	) -> &mut ReverseProperties<T, B> {
		self.reverse_properties.get_or_insert(props)
	}

	pub fn reverse_properties_or_default(&mut self) -> &mut ReverseProperties<T, B> {
		self.reverse_properties
			.get_or_insert_with(ReverseProperties::default)
	}

	pub fn reverse_properties_or_insert_with(
		&mut self,
		f: impl FnOnce() -> ReverseProperties<T, B>,
	) -> &mut ReverseProperties<T, B> {
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

impl<T, B> Relabel<T, B> for Node<T, B> {
	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: Generator<N>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
		relabeling: &mut hashbrown::HashMap<B, Subject<T, B>>,
	) where
		T: Clone + Eq + Hash,
		B: Clone + Eq + Hash,
	{
		self.id = match self.id.take() {
			Some(Id::Valid(Subject::Blank(b))) => {
				let value = relabeling
					.entry(b)
					.or_insert_with(|| generator.next(vocabulary))
					.clone();
				Some(value.into())
			}
			None => {
				let value = generator.next(vocabulary);
				Some(value.into())
			}
			id => id,
		};

		for ty in self.types_mut() {
			if let Some(b) = ty.as_blank().cloned() {
				*ty = relabeling
					.entry(b)
					.or_insert_with(|| generator.next(vocabulary))
					.clone()
					.into();
			}
		}

		if let Some(graph) = self.graph_mut() {
			*graph = std::mem::take(graph)
				.into_iter()
				.map(|mut o| {
					o.relabel_with(vocabulary, generator, relabeling);
					o
				})
				.collect();
		}

		if let Some(included) = self.included_mut() {
			*included = std::mem::take(included)
				.into_iter()
				.map(|mut n| {
					n.relabel_with(vocabulary, generator, relabeling);
					n
				})
				.collect();
		}

		for (_, objects) in self.properties_mut() {
			for object in objects {
				object.relabel_with(vocabulary, generator, relabeling);
			}
		}

		if let Some(reverse_properties) = self.reverse_properties_mut() {
			for (_, nodes) in reverse_properties.iter_mut() {
				for node in nodes {
					node.relabel_with(vocabulary, generator, relabeling);
				}
			}
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash> PartialEq for Node<T, B> {
	fn eq(&self, other: &Self) -> bool {
		self.id.eq(&other.id)
			&& multiset::compare_unordered_opt(self.types.as_deref(), other.types.as_deref())
			&& self.graph.as_ref() == other.graph.as_ref()
			&& self.included.as_ref() == other.included.as_ref()
			&& self.properties.eq(&other.properties)
			&& self.reverse_properties.eq(&other.reverse_properties)
	}
}

impl<T, B> Indexed<Node<T, B>> {
	pub fn entries(&self) -> IndexedEntries<T, B> {
		IndexedEntries {
			index: self.index(),
			inner: self.inner().entries(),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash> Indexed<Node<T, B>> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}
}

#[derive(Educe, PartialEq, Eq)]
#[educe(Clone, Copy)]
pub enum EntryKeyRef<'a, T, B> {
	Id,
	Type,
	Graph,
	Included,
	Reverse,
	Property(&'a Id<T, B>),
}

impl<'a, T, B> EntryKeyRef<'a, T, B> {
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

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> IntoRefWithContext<'a, str, N>
	for EntryKeyRef<'a, T, B>
{
	fn into_ref_with(self, vocabulary: &'a N) -> &'a str {
		match self {
			EntryKeyRef::Id => "@id",
			EntryKeyRef::Type => "@type",
			EntryKeyRef::Graph => "@graph",
			EntryKeyRef::Included => "@included",
			EntryKeyRef::Reverse => "@reverse",
			EntryKeyRef::Property(p) => p.with(vocabulary).as_str(),
		}
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryValueRef<'a, T, B> {
	Id(&'a Id<T, B>),
	Type(&'a [Id<T, B>]),
	Graph(&'a IndexSet<IndexedObject<T, B>>),
	Included(&'a IndexSet<IndexedNode<T, B>>),
	Reverse(&'a ReverseProperties<T, B>),
	Property(&'a [IndexedObject<T, B>]),
}

impl<'a, T, B> EntryValueRef<'a, T, B> {
	pub fn is_json_array(&self) -> bool {
		matches!(
			self,
			Self::Type(_) | Self::Graph(_) | Self::Included(_) | Self::Property(_)
		)
	}

	pub fn is_json_object(&self) -> bool {
		matches!(self, Self::Reverse(_))
	}

	fn sub_fragments(&self) -> SubFragments<'a, T, B> {
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

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryRef<'a, T, B> {
	Id(&'a Id<T, B>),
	Type(&'a [Id<T, B>]),
	Graph(&'a Graph<T, B>),
	Included(&'a Included<T, B>),
	Reverse(&'a ReverseProperties<T, B>),
	Property(&'a Id<T, B>, &'a [IndexedObject<T, B>]),
}

impl<'a, T, B> EntryRef<'a, T, B> {
	pub fn into_key(self) -> EntryKeyRef<'a, T, B> {
		match self {
			Self::Id(_) => EntryKeyRef::Id,
			Self::Type(_) => EntryKeyRef::Type,
			Self::Graph(_) => EntryKeyRef::Graph,
			Self::Included(_) => EntryKeyRef::Included,
			Self::Reverse(_) => EntryKeyRef::Reverse,
			Self::Property(k, _) => EntryKeyRef::Property(k),
		}
	}

	pub fn key(&self) -> EntryKeyRef<'a, T, B> {
		self.into_key()
	}

	pub fn into_value(self) -> EntryValueRef<'a, T, B> {
		match self {
			Self::Id(v) => EntryValueRef::Id(v),
			Self::Type(v) => EntryValueRef::Type(v),
			Self::Graph(v) => EntryValueRef::Graph(v),
			Self::Included(v) => EntryValueRef::Included(v),
			Self::Reverse(v) => EntryValueRef::Reverse(v),
			Self::Property(_, v) => EntryValueRef::Property(v),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a, T, B> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (EntryKeyRef<'a, T, B>, EntryValueRef<'a, T, B>) {
		match self {
			Self::Id(v) => (EntryKeyRef::Id, EntryValueRef::Id(v)),
			Self::Type(v) => (EntryKeyRef::Type, EntryValueRef::Type(v)),
			Self::Graph(v) => (EntryKeyRef::Graph, EntryValueRef::Graph(v)),
			Self::Included(v) => (EntryKeyRef::Included, EntryValueRef::Included(v)),
			Self::Reverse(v) => (EntryKeyRef::Reverse, EntryValueRef::Reverse(v)),
			Self::Property(k, v) => (EntryKeyRef::Property(k), EntryValueRef::Property(v)),
		}
	}

	pub fn as_key_value(&self) -> (EntryKeyRef<'a, T, B>, EntryValueRef<'a, T, B>) {
		match self {
			Self::Id(v) => (EntryKeyRef::Id, EntryValueRef::Id(*v)),
			Self::Type(v) => (EntryKeyRef::Type, EntryValueRef::Type(v)),
			Self::Graph(v) => (EntryKeyRef::Graph, EntryValueRef::Graph(*v)),
			Self::Included(v) => (EntryKeyRef::Included, EntryValueRef::Included(*v)),
			Self::Reverse(v) => (EntryKeyRef::Reverse, EntryValueRef::Reverse(*v)),
			Self::Property(k, v) => (EntryKeyRef::Property(*k), EntryValueRef::Property(v)),
		}
	}
}

#[derive(Educe)]
#[educe(Clone)]
pub struct Entries<'a, T, B> {
	id: Option<&'a Id<T, B>>,
	type_: Option<&'a [Id<T, B>]>,
	graph: Option<&'a Graph<T, B>>,
	included: Option<&'a Included<T, B>>,
	reverse: Option<&'a ReverseProperties<T, B>>,
	properties: properties::Iter<'a, T, B>,
}

impl<'a, T, B> Iterator for Entries<'a, T, B> {
	type Item = EntryRef<'a, T, B>;

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

impl<'a, T, B> ExactSizeIterator for Entries<'a, T, B> {}

#[derive(Educe)]
#[educe(Clone)]
pub struct IndexedEntries<'a, T, B> {
	index: Option<&'a str>,
	inner: Entries<'a, T, B>,
}

impl<'a, T, B> Iterator for IndexedEntries<'a, T, B> {
	type Item = IndexedEntryRef<'a, T, B>;

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

impl<'a, T, B> ExactSizeIterator for IndexedEntries<'a, T, B> {}

#[derive(Educe, PartialEq, Eq)]
#[educe(Clone, Copy)]
pub enum IndexedEntryKeyRef<'a, T, B> {
	Index,
	Node(EntryKeyRef<'a, T, B>),
}

impl<'a, T, B> IndexedEntryKeyRef<'a, T, B> {
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

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> IntoRefWithContext<'a, str, N>
	for IndexedEntryKeyRef<'a, T, B>
{
	fn into_ref_with(self, vocabulary: &'a N) -> &'a str {
		match self {
			IndexedEntryKeyRef::Index => "@index",
			IndexedEntryKeyRef::Node(e) => e.into_with(vocabulary).into_str(),
		}
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum IndexedEntryValueRef<'a, T, B> {
	Index(&'a str),
	Node(EntryValueRef<'a, T, B>),
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum IndexedEntryRef<'a, T, B> {
	Index(&'a str),
	Node(EntryRef<'a, T, B>),
}

impl<'a, T, B> IndexedEntryRef<'a, T, B> {
	pub fn into_key(self) -> IndexedEntryKeyRef<'a, T, B> {
		match self {
			Self::Index(_) => IndexedEntryKeyRef::Index,
			Self::Node(e) => IndexedEntryKeyRef::Node(e.key()),
		}
	}

	pub fn key(&self) -> IndexedEntryKeyRef<'a, T, B> {
		self.into_key()
	}

	pub fn into_value(self) -> IndexedEntryValueRef<'a, T, B> {
		match self {
			Self::Index(v) => IndexedEntryValueRef::Index(v),
			Self::Node(e) => IndexedEntryValueRef::Node(e.value()),
		}
	}

	pub fn value(&self) -> IndexedEntryValueRef<'a, T, B> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (IndexedEntryKeyRef<'a, T, B>, IndexedEntryValueRef<'a, T, B>) {
		match self {
			Self::Index(v) => (IndexedEntryKeyRef::Index, IndexedEntryValueRef::Index(v)),
			Self::Node(e) => {
				let (k, v) = e.into_key_value();
				(IndexedEntryKeyRef::Node(k), IndexedEntryValueRef::Node(v))
			}
		}
	}

	pub fn as_key_value(&self) -> (IndexedEntryKeyRef<'a, T, B>, IndexedEntryValueRef<'a, T, B>) {
		self.into_key_value()
	}
}

/// Node object fragment reference.
pub enum FragmentRef<'a, T, B> {
	/// Node object entry.
	Entry(EntryRef<'a, T, B>),

	/// Node object entry key.
	Key(EntryKeyRef<'a, T, B>),

	/// Node object entry value.
	Value(EntryValueRef<'a, T, B>),

	/// "@type" entry value fragment.
	TypeFragment(&'a Id<T, B>),
}

impl<'a, T, B> FragmentRef<'a, T, B> {
	pub fn into_id(self) -> Option<&'a Id<T, B>> {
		match self {
			Self::Key(EntryKeyRef::Property(id)) => Some(id),
			Self::Value(EntryValueRef::Id(id)) => Some(id),
			Self::TypeFragment(ty) => Some(ty),
			_ => None,
		}
	}

	pub fn as_id(&self) -> Option<&'a Id<T, B>> {
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

	pub fn sub_fragments(&self) -> SubFragments<'a, T, B> {
		match self {
			Self::Entry(e) => SubFragments::Entry(Some(e.key()), Some(e.value())),
			Self::Value(v) => v.sub_fragments(),
			_ => SubFragments::None,
		}
	}
}

pub enum SubFragments<'a, T, B> {
	None,
	Entry(
		Option<EntryKeyRef<'a, T, B>>,
		Option<EntryValueRef<'a, T, B>>,
	),
	Type(std::slice::Iter<'a, Id<T, B>>),
	Graph(indexmap::set::Iter<'a, IndexedObject<T, B>>),
	Included(indexmap::set::Iter<'a, IndexedNode<T, B>>),
	Reverse(reverse_properties::Iter<'a, T, B>),
	Property(std::slice::Iter<'a, IndexedObject<T, B>>),
}

impl<'a, T, B> Iterator for SubFragments<'a, T, B> {
	type Item = super::FragmentRef<'a, T, B>;

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
				.map(|t| super::FragmentRef::NodeFragment(FragmentRef::TypeFragment(t))),
			Self::Graph(g) => g.next().map(|o| super::FragmentRef::IndexedObject(o)),
			Self::Included(i) => i.next().map(|n| super::FragmentRef::IndexedNode(n)),
			Self::Reverse(r) => r
				.next()
				.map(|(_, n)| super::FragmentRef::IndexedNodeList(n)),
			Self::Property(o) => o.next().map(|o| super::FragmentRef::IndexedObject(o)),
		}
	}
}

impl<T, B> object::Any<T, B> for Node<T, B> {
	#[inline(always)]
	fn as_ref(&self) -> object::Ref<T, B> {
		object::Ref::Node(self)
	}
}

impl<T, B> TryFrom<Object<T, B>> for Node<T, B> {
	type Error = Object<T, B>;

	#[inline(always)]
	fn try_from(obj: Object<T, B>) -> Result<Node<T, B>, Object<T, B>> {
		match obj {
			Object::Node(node) => Ok(*node),
			obj => Err(obj),
		}
	}
}

impl<T: Hash, B: Hash> Hash for Node<T, B> {
	#[inline]
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.id.hash(h);
		utils::hash_set_opt(self.types.as_ref(), h);
		utils::hash_set_opt(self.graph.as_ref(), h);
		utils::hash_set_opt(self.included.as_ref(), h);
		self.properties.hash(h);
		self.reverse_properties.hash(h)
	}
}

/// Iterator through indexed nodes.
pub struct Nodes<'a, T, B>(Option<std::slice::Iter<'a, IndexedNode<T, B>>>);

impl<'a, T, B> Nodes<'a, T, B> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, IndexedNode<T, B>>>) -> Self {
		Self(inner)
	}
}

impl<'a, T, B> Iterator for Nodes<'a, T, B> {
	type Item = &'a IndexedNode<T, B>;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a IndexedNode<T, B>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash> TryFromJsonObject<T, B> for Node<T, B> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		mut object: json_syntax::Object,
	) -> Result<Self, InvalidExpandedJson> {
		let id = match object
			.remove_unique("@id")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(Id::try_from_json_in(vocabulary, entry.value)?),
			None => None,
		};

		let types = match object
			.remove_unique("@type")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(Vec::try_from_json_in(vocabulary, entry.value)?),
			None => None,
		};

		let graph = match object
			.remove_unique("@graph")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(IndexSet::try_from_json_in(vocabulary, entry.value)?),
			None => None,
		};

		let included = match object
			.remove_unique("@included")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(IndexSet::try_from_json_in(vocabulary, entry.value)?),
			None => None,
		};

		let reverse_properties = match object
			.remove_unique("@reverse")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Some(ReverseProperties::try_from_json_in(
				vocabulary,
				entry.value,
			)?),
			None => None,
		};

		let properties = Properties::try_from_json_object_in(vocabulary, object)?;

		Ok(Self {
			id,
			types,
			graph,
			included,
			reverse_properties,
			properties,
		})
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> IntoJsonWithContext<N> for Node<T, B> {
	fn into_json_with(self, vocabulary: &N) -> json_syntax::Value {
		let mut obj = json_syntax::Object::new();

		if let Some(id) = self.id {
			obj.insert("@id".into(), id.into_with(vocabulary).into_json());
		}

		if let Some(types) = self.types {
			if !types.is_empty() {
				// let value = if types.len() > 1 {
				// 	types.value.into_with(vocabulary).into_json()
				// } else {
				// 	types.value.0.into_iter().next().unwrap().into_with(vocabulary).into_json()
				// };
				let value = types.into_with(vocabulary).into_json();

				obj.insert("@type".into(), value);
			}
		}

		if let Some(graph) = self.graph {
			obj.insert("@graph".into(), graph.into_with(vocabulary).into_json());
		}

		if let Some(included) = self.included {
			obj.insert(
				"@include".into(),
				included.into_with(vocabulary).into_json(),
			);
		}

		if let Some(reverse_properties) = self.reverse_properties {
			obj.insert(
				"@reverse".into(),
				reverse_properties.into_with(vocabulary).into_json(),
			);
		}

		for (prop, objects) in self.properties {
			obj.insert(
				prop.with(vocabulary).to_string().into(),
				objects.into_json_with(vocabulary),
			);
		}

		obj.into()
	}
}
