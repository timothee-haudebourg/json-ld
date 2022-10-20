//! Nodes, lists and values.
use crate::{id, Id, Indexed, LenientLanguageTag};
use contextual::{IntoRefWithContext, WithContext};
use derivative::Derivative;
use iref::IriBuf;
use json_ld_syntax::{Entry, IntoJsonWithContextMeta, Keyword};
use json_syntax::Number;
use locspan::{BorrowStripped, Meta, Stripped, StrippedEq, StrippedHash, StrippedPartialEq};
use locspan_derive::*;
use rdf_types::{BlankIdBuf, Vocabulary, VocabularyMut};
use smallvec::SmallVec;
use std::collections::HashSet;
use std::hash::Hash;

pub mod list;
mod mapped_eq;
pub mod node;
mod typ;
pub mod value;

pub use list::List;
pub use mapped_eq::MappedEq;
pub use node::{Graph, IndexedNode, Node, Nodes, StrippedIndexedNode};
pub use typ::{Type, TypeRef};
pub use value::{Literal, LiteralString, Value};

pub trait Any<T, B, M = ()> {
	fn as_ref(&self) -> Ref<T, B, M>;

	#[inline]
	fn id_entry<'a>(&'a self) -> Option<&'a Entry<Id<T, B>, M>>
	where
		M: 'a,
	{
		match self.as_ref() {
			Ref::Node(n) => n.id_entry(),
			_ => None,
		}
	}

	#[inline]
	fn id<'a>(&'a self) -> Option<&'a Meta<Id<T, B>, M>>
	where
		M: 'a,
	{
		match self.as_ref() {
			Ref::Node(n) => n.id.as_ref().map(Entry::as_value),
			_ => None,
		}
	}

	#[inline]
	fn language<'a>(&'a self) -> Option<LenientLanguageTag>
	where
		T: 'a,
		B: 'a,
		M: 'a,
	{
		match self.as_ref() {
			Ref::Value(value) => value.language(),
			_ => None,
		}
	}

	#[inline]
	fn is_value(&self) -> bool {
		matches!(self.as_ref(), Ref::Value(_))
	}

	#[inline]
	fn is_node(&self) -> bool {
		matches!(self.as_ref(), Ref::Node(_))
	}

	#[inline]
	fn is_graph(&self) -> bool {
		match self.as_ref() {
			Ref::Node(n) => n.is_graph(),
			_ => false,
		}
	}

	#[inline]
	fn is_list(&self) -> bool {
		matches!(self.as_ref(), Ref::List(_))
	}
}

/// Object reference.
pub enum Ref<'a, T, B, M = ()> {
	/// Value object.
	Value(&'a Value<T, M>),

	/// Node object.
	Node(&'a Node<T, B, M>),

	/// List object.
	List(&'a List<T, B, M>),
}

pub type IndexedObject<T, B, M> = Meta<Indexed<Object<T, B, M>, M>, M>;

/// Indexed object, without regard for its metadata.
pub type StrippedIndexedObject<T, B, M> = Stripped<IndexedObject<T, B, M>>;

/// Object.
///
/// JSON-LD connects together multiple kinds of data objects.
/// Objects may be nodes, values or lists of objects.
#[allow(clippy::derive_hash_xor_eq)]
#[derive(Derivative, Clone, Hash, StrippedHash)]
#[derivative(
	PartialEq(bound = "T: Eq + Hash, B: Eq + Hash, M: PartialEq"),
	Eq(bound = "T: Eq + Hash, B: Eq + Hash, M: Eq")
)]
#[stripped_ignore(M)]
#[stripped(T, B)]
pub enum Object<T = IriBuf, B = BlankIdBuf, M = ()> {
	/// Value object.
	Value(Value<T, M>),

	/// Node object.
	Node(Box<Node<T, B, M>>),

	/// List object.
	List(List<T, B, M>),
}

impl<T, B, M> Object<T, B, M> {
	/// Creates a new node object from a node.
	#[inline(always)]
	pub fn node(n: Node<T, B, M>) -> Self {
		Self::Node(Box::new(n))
	}

	/// Identifier of the object, if it is a node object.
	#[inline(always)]
	pub fn id(&self) -> Option<&Meta<Id<T, B>, M>> {
		match self {
			Object::Node(n) => n.id.as_ref().map(Entry::as_value),
			_ => None,
		}
	}

	/// Assigns an identifier to every node included in this object using the given `generator`.
	pub fn identify_all_with<N, G: id::Generator<T, B, N, M>>(
		&mut self,
		vocabulary: &mut N,
		generator: &mut G,
	) where
		M: Clone,
	{
		match self {
			Object::Node(n) => n.identify_all_with(vocabulary, generator),
			Object::List(l) => {
				for object in l {
					object.identify_all_with(vocabulary, generator)
				}
			}
			_ => (),
		}
	}

	pub fn identify_all<G: id::Generator<T, B, (), M>>(&mut self, generator: &mut G)
	where
		M: Clone,
	{
		self.identify_all_with(&mut (), generator)
	}

	pub fn types(&self) -> Types<T, B, M> {
		match self {
			Self::Value(value) => Types::Value(value.typ()),
			Self::Node(node) => Types::Node(node.types().iter()),
			Self::List(_) => Types::List,
		}
	}

	/// Identifier of the object as an IRI.
	///
	/// If the object is a node identified by an IRI, returns this IRI.
	/// Returns `None` otherwise.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<&T> {
		match self {
			Object::Node(node) => node.as_iri(),
			_ => None,
		}
	}

	/// Tests if the object is a value.
	#[inline(always)]
	pub fn is_value(&self) -> bool {
		matches!(self, Object::Value(_))
	}

	/// Returns this object as a value, if it is one.
	#[inline(always)]
	pub fn as_value(&self) -> Option<&Value<T, M>> {
		match self {
			Self::Value(v) => Some(v),
			_ => None,
		}
	}

	/// Converts this object as a value, if it is one.
	#[inline(always)]
	pub fn into_value(self) -> Option<Value<T, M>> {
		match self {
			Self::Value(v) => Some(v),
			_ => None,
		}
	}

	/// Tests if the object is a node.
	#[inline(always)]
	pub fn is_node(&self) -> bool {
		matches!(self, Object::Node(_))
	}

	/// Returns this object as a node, if it is one.
	#[inline(always)]
	pub fn as_node(&self) -> Option<&Node<T, B, M>> {
		match self {
			Self::Node(n) => Some(n),
			_ => None,
		}
	}

	/// Converts this object into a node, if it is one.
	#[inline(always)]
	pub fn into_node(self) -> Option<Node<T, B, M>> {
		match self {
			Self::Node(n) => Some(*n),
			_ => None,
		}
	}

	/// Tests if the object is a graph object (a node with a `@graph` field).
	#[inline(always)]
	pub fn is_graph(&self) -> bool {
		match self {
			Object::Node(n) => n.is_graph(),
			_ => false,
		}
	}

	/// Tests if the object is a list.
	#[inline(always)]
	pub fn is_list(&self) -> bool {
		matches!(self, Object::List(_))
	}

	/// Returns this object as a list, if it is one.
	#[inline(always)]
	pub fn as_list(&self) -> Option<&List<T, B, M>> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}

	/// Converts this object into a list, if it is one.
	#[inline(always)]
	pub fn into_list(self) -> Option<List<T, B, M>> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}

	/// Get the object as a string.
	///
	/// If the object is a value that is a string, returns this string.
	/// If the object is a node that is identified, returns the identifier as a string.
	/// Returns `None` otherwise.
	#[inline(always)]
	pub fn as_str(&self) -> Option<&str>
	where
		T: AsRef<str>,
		B: AsRef<str>,
	{
		match self {
			Object::Value(value) => value.as_str(),
			Object::Node(node) => node.as_str(),
			_ => None,
		}
	}

	/// Get the value as a boolean, if it is.
	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Object::Value(value) => value.as_bool(),
			_ => None,
		}
	}

	/// Get the value as a number, if it is.
	#[inline(always)]
	pub fn as_number(&self) -> Option<&Number> {
		match self {
			Object::Value(value) => value.as_number(),
			_ => None,
		}
	}

	/// If the object is a language-tagged value,
	/// Return its associated language.
	#[inline(always)]
	pub fn language(&self) -> Option<LenientLanguageTag> {
		match self {
			Object::Value(value) => value.language(),
			_ => None,
		}
	}

	pub fn traverse(&self) -> Traverse<T, B, M> {
		Traverse::new(Some(FragmentRef::Object(self)))
	}

	fn sub_fragments(&self) -> ObjectSubFragments<T, B, M> {
		match self {
			Self::Value(v) => ObjectSubFragments::Value(v.entries()),
			Self::List(l) => ObjectSubFragments::List(Some(l.entry())),
			Self::Node(n) => ObjectSubFragments::Node(n.entries()),
		}
	}

	/// Equivalence operator.
	///
	/// Equivalence is different from equality for anonymous objects:
	/// List objects and anonymous node objects have an implicit unlabeled blank nodes and thus never equivalent.
	pub fn equivalent(&self, other: &Self) -> bool
	where
		T: Eq + Hash,
		B: Eq + Hash,
	{
		match (self, other) {
			(Self::Value(a), Self::Value(b)) => a.stripped() == b.stripped(),
			(Self::Node(a), Self::Node(b)) => a.equivalent(b),
			_ => false,
		}
	}

	pub fn entries(&self) -> Entries<T, B, M> {
		match self {
			Self::Value(value) => Entries::Value(value.entries()),
			Self::List(list) => Entries::List(Some(list.entry())),
			Self::Node(node) => Entries::Node(node.entries()),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedPartialEq for Object<T, B, M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Value(a), Self::Value(b)) => a.stripped_eq(b),
			(Self::Node(a), Self::Node(b)) => a.stripped_eq(b),
			(Self::List(a), Self::List(b)) => a.stripped_eq(b),
			_ => false,
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> StrippedEq for Object<T, B, M> {}

impl<T: Eq + Hash, B: Eq + Hash, M> Indexed<Object<T, B, M>, M> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}
}

impl<T, B, M> Indexed<Object<T, B, M>, M> {
	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_node(self) -> Option<Indexed<Node<T, B, M>, M>> {
		let (object, index) = self.into_parts();
		object.into_node().map(|node| Indexed::new(node, index))
	}

	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_value(self) -> Option<Indexed<Value<T, M>, M>> {
		let (object, index) = self.into_parts();
		object.into_value().map(|value| Indexed::new(value, index))
	}

	/// Converts this indexed object into an indexed list, if it is one.
	#[inline(always)]
	pub fn into_indexed_list(self) -> Option<Indexed<List<T, B, M>, M>> {
		let (object, index) = self.into_parts();
		object.into_list().map(|list| Indexed::new(list, index))
	}

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self) -> Result<Meta<Graph<T, B, M>, M>, Self> {
		let (obj, index) = self.into_parts();
		match obj {
			Object::Node(n) => match n.into_unnamed_graph() {
				Ok(g) => Ok(g.value),
				Err(n) => Err(Indexed::new(Object::node(n), index)),
			},
			obj => Err(Indexed::new(obj, index)),
		}
	}

	pub fn entries(&self) -> IndexedEntries<T, B, M> {
		IndexedEntries {
			index: self.index(),
			inner: self.inner().entries(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub enum Entries<'a, T, B, M> {
	Value(value::Entries<'a, T, M>),
	List(Option<list::EntryRef<'a, T, B, M>>),
	Node(node::Entries<'a, T, B, M>),
}

impl<'a, T, B, M> Iterator for Entries<'a, T, B, M> {
	type Item = EntryRef<'a, T, B, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = match self {
			Self::Value(v) => v.len(),
			Self::List(l) => usize::from(l.is_some()),
			Self::Node(n) => n.len(),
		};

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Value(v) => v.next().map(EntryRef::Value),
			Self::List(l) => l.take().map(EntryRef::List),
			Self::Node(n) => n.next().map(EntryRef::Node),
		}
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
		let len = self.inner.len() + usize::from(self.index.is_some());
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.index
			.take()
			.map(IndexedEntryRef::Index)
			.or_else(|| self.inner.next().map(IndexedEntryRef::Object))
	}
}

impl<'a, T, B, M> ExactSizeIterator for IndexedEntries<'a, T, B, M> {}

#[derive(Derivative, PartialEq, Eq)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum EntryKeyRef<'a, T, B, M> {
	Value(value::EntryKey),
	List(&'a M),
	Node(node::EntryKeyRef<'a, T, B, M>),
}

impl<'a, T, B, M> EntryKeyRef<'a, T, B, M> {
	pub fn into_keyword(self) -> Option<Keyword> {
		match self {
			Self::Value(e) => Some(e.into_keyword()),
			Self::List(_) => Some(Keyword::List),
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
			Self::Value(e) => e.into_str(),
			Self::List(_) => "@list",
			Self::Node(e) => e.into_str(),
		}
	}

	pub fn as_str(self) -> &'a str
	where
		T: AsRef<str>,
		B: AsRef<str>,
	{
		self.into_str()
	}
}

impl<'a, T, B, M, N: Vocabulary<Iri = T, BlankId = B>> IntoRefWithContext<'a, str, N>
	for EntryKeyRef<'a, T, B, M>
{
	fn into_ref_with(self, vocabulary: &'a N) -> &'a str {
		match self {
			EntryKeyRef::Value(e) => e.into_str(),
			EntryKeyRef::List(_) => "@list",
			EntryKeyRef::Node(e) => e.into_with(vocabulary).into_str(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum EntryValueRef<'a, T, B, M> {
	Value(value::EntryRef<'a, T, M>),
	List(list::EntryValueRef<'a, T, B, M>),
	Node(node::EntryValueRef<'a, T, B, M>),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum EntryRef<'a, T, B, M> {
	Value(value::EntryRef<'a, T, M>),
	List(list::EntryRef<'a, T, B, M>),
	Node(node::EntryRef<'a, T, B, M>),
}

impl<'a, T, B, M> EntryRef<'a, T, B, M> {
	pub fn into_key(self) -> EntryKeyRef<'a, T, B, M> {
		match self {
			Self::Value(e) => EntryKeyRef::Value(e.key()),
			Self::List(e) => EntryKeyRef::List(&e.key_metadata),
			Self::Node(e) => EntryKeyRef::Node(e.key()),
		}
	}

	pub fn key(&self) -> EntryKeyRef<'a, T, B, M> {
		self.into_key()
	}

	pub fn into_value(self) -> EntryValueRef<'a, T, B, M> {
		match self {
			Self::Value(v) => EntryValueRef::Value(v),
			Self::List(v) => EntryValueRef::List(v),
			Self::Node(e) => EntryValueRef::Node(e.value()),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a, T, B, M> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (EntryKeyRef<'a, T, B, M>, EntryValueRef<'a, T, B, M>) {
		match self {
			Self::Value(e) => (EntryKeyRef::Value(e.key()), EntryValueRef::Value(e)),
			Self::List(e) => (
				EntryKeyRef::List(&e.key_metadata),
				EntryValueRef::List(&e.value),
			),
			Self::Node(e) => {
				let (k, v) = e.into_key_value();
				(EntryKeyRef::Node(k), EntryValueRef::Node(v))
			}
		}
	}

	pub fn as_key_value(&self) -> (EntryKeyRef<'a, T, B, M>, EntryValueRef<'a, T, B, M>) {
		self.into_key_value()
	}
}

#[derive(Derivative, PartialEq, Eq)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum IndexedEntryKeyRef<'a, T, B, M> {
	Index,
	Object(EntryKeyRef<'a, T, B, M>),
}

impl<'a, T, B, M> IndexedEntryKeyRef<'a, T, B, M> {
	pub fn into_keyword(self) -> Option<Keyword> {
		match self {
			Self::Index => Some(Keyword::Index),
			Self::Object(e) => e.into_keyword(),
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
			Self::Object(e) => e.into_str(),
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

impl<'a, T, B, M, N: Vocabulary<Iri = T, BlankId = B>> IntoRefWithContext<'a, str, N>
	for IndexedEntryKeyRef<'a, T, B, M>
{
	fn into_ref_with(self, vocabulary: &'a N) -> &'a str {
		match self {
			IndexedEntryKeyRef::Index => "@value",
			IndexedEntryKeyRef::Object(e) => e.into_with(vocabulary).into_str(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum IndexedEntryValueRef<'a, T, B, M> {
	Index(&'a str),
	Object(EntryValueRef<'a, T, B, M>),
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum IndexedEntryRef<'a, T, B, M> {
	Index(&'a str),
	Object(EntryRef<'a, T, B, M>),
}

impl<'a, T, B, M> IndexedEntryRef<'a, T, B, M> {
	pub fn into_key(self) -> IndexedEntryKeyRef<'a, T, B, M> {
		match self {
			Self::Index(_) => IndexedEntryKeyRef::Index,
			Self::Object(e) => IndexedEntryKeyRef::Object(e.key()),
		}
	}

	pub fn key(&self) -> IndexedEntryKeyRef<'a, T, B, M> {
		self.into_key()
	}

	pub fn into_value(self) -> IndexedEntryValueRef<'a, T, B, M> {
		match self {
			Self::Index(v) => IndexedEntryValueRef::Index(v),
			Self::Object(e) => IndexedEntryValueRef::Object(e.value()),
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
			Self::Object(e) => {
				let (k, v) = e.into_key_value();
				(
					IndexedEntryKeyRef::Object(k),
					IndexedEntryValueRef::Object(v),
				)
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

pub trait TryFromJson<T, B, M>: Sized {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>>;
}

pub trait TryFromJsonObject<T, B, M>: Sized {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		object: Meta<json_syntax::Object<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>>;
}

impl<T, B, M, V: TryFromJson<T, B, M>> TryFromJson<T, B, M> for Stripped<V> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		let Meta(v, meta) = V::try_from_json_in(vocabulary, value)?;
		Ok(Meta(Stripped(v), meta))
	}
}

impl<T, B, M, V: TryFromJson<T, B, M>> TryFromJson<T, B, M> for Vec<Meta<V, M>> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		match value {
			json_syntax::Value::Array(items) => {
				let mut result = Vec::new();

				for item in items {
					result.push(V::try_from_json_in(vocabulary, item)?)
				}

				Ok(Meta(result, meta))
			}
			_ => Err(Meta(InvalidExpandedJson::InvalidList, meta)),
		}
	}
}

impl<T, B, M, V: StrippedEq + StrippedHash + TryFromJson<T, B, M>> TryFromJson<T, B, M>
	for HashSet<Stripped<Meta<V, M>>>
{
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		match value {
			json_syntax::Value::Array(items) => {
				let mut result = HashSet::new();

				for item in items {
					result.insert(Stripped(V::try_from_json_in(vocabulary, item)?));
				}

				Ok(Meta(result, meta))
			}
			_ => Err(Meta(InvalidExpandedJson::InvalidList, meta)),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> TryFromJson<T, B, M> for Object<T, B, M> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		match value {
			json_syntax::Value::Object(object) => {
				Self::try_from_json_object_in(vocabulary, Meta(object, meta))
			}
			_ => Err(Meta(InvalidExpandedJson::InvalidObject, meta)),
		}
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> TryFromJsonObject<T, B, M> for Object<T, B, M> {
	fn try_from_json_object_in(
		vocabulary: &mut impl VocabularyMut<Iri = T, BlankId = B>,
		Meta(mut object, meta): Meta<json_syntax::Object<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidExpandedJson<M>, M>> {
		match object
			.remove_unique("@context")
			.map_err(InvalidExpandedJson::duplicate_key)?
		{
			Some(entry) => Err(Meta(
				InvalidExpandedJson::NotExpanded,
				entry.key.into_metadata(),
			)),
			None => {
				if let Some(value_entry) = object
					.remove_unique("@value")
					.map_err(InvalidExpandedJson::duplicate_key)?
				{
					Ok(Meta(
						Self::Value(Value::try_from_json_object_in(
							vocabulary,
							object,
							value_entry,
						)?),
						meta,
					))
				} else if let Some(list_entry) = object
					.remove_unique("@list")
					.map_err(InvalidExpandedJson::duplicate_key)?
				{
					Ok(Meta(
						Self::List(List::try_from_json_object_in(
							vocabulary, object, list_entry,
						)?),
						meta,
					))
				} else {
					let Meta(node, meta) =
						Node::try_from_json_object_in(vocabulary, Meta(object, meta))?;
					Ok(Meta(Self::node(node), meta))
				}
			}
		}
	}
}

#[derive(Debug)]
pub enum InvalidExpandedJson<M> {
	InvalidObject,
	InvalidList,
	InvalidIndex,
	InvalidId,
	InvalidValueType,
	InvalidLiteral,
	InvalidLanguage,
	InvalidDirection,
	NotExpanded,
	UnexpectedEntry,
	DuplicateKey(Meta<json_syntax::object::Key, M>),
	Unexpected(json_syntax::Kind, json_syntax::Kind),
}

impl<M> InvalidExpandedJson<M> {
	pub fn duplicate_key(
		json_syntax::object::Duplicate(a, b): json_syntax::object::Duplicate<
			json_syntax::object::Entry<M>,
		>,
	) -> Meta<Self, M> {
		Meta(
			InvalidExpandedJson::DuplicateKey(a.key),
			b.key.into_metadata(),
		)
	}
}

impl<T, B, M> Any<T, B, M> for Object<T, B, M> {
	#[inline(always)]
	fn as_ref(&self) -> Ref<T, B, M> {
		match self {
			Object::Value(value) => Ref::Value(value),
			Object::Node(node) => Ref::Node(node),
			Object::List(list) => Ref::List(list),
		}
	}
}

impl<T, B, M> From<Value<T, M>> for Object<T, B, M> {
	#[inline(always)]
	fn from(value: Value<T, M>) -> Self {
		Self::Value(value)
	}
}

impl<T, B, M> From<Node<T, B, M>> for Object<T, B, M> {
	#[inline(always)]
	fn from(node: Node<T, B, M>) -> Self {
		Self::node(node)
	}
}

// impl<J: JsonHash + JsonClone, K: JsonFrom<J>, T> AsJson<J, K> for Object<T, B, M> {
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		match self {
// 			Object::Value(v) => v.as_json_with(meta),
// 			Object::Node(n) => n.as_json_with(meta),
// 			Object::List(items) => {
// 				let mut obj = <K as Json>::Object::default();
// 				obj.insert(
// 					K::new_key(Keyword::List.into_str(), meta(None)),
// 					items.as_json_with(meta.clone()),
// 				);
// 				K::object(obj, meta(None))
// 			}
// 		}
// 	}
// }

// impl<J: JsonHash + JsonClone, K: JsonFrom<J>, T> AsJson<J, K>
// 	for HashSet<Indexed<Object<T, B, M>>>
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

/// Iterator through the types of an object.
pub enum Types<'a, T, B, M> {
	Value(Option<value::TypeRef<'a, T>>),
	Node(std::slice::Iter<'a, Meta<Id<T, B>, M>>),
	List,
}

impl<'a, T, B, M> Iterator for Types<'a, T, B, M> {
	type Item = TypeRef<'a, T, B>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Value(ty) => ty.take().map(TypeRef::from_value_type),
			Self::Node(tys) => tys.next().map(|t| TypeRef::from_reference(t.value())),
			Self::List => None,
		}
	}
}

/// Iterator through indexed objects.
pub struct Objects<'a, T, B, M>(Option<std::slice::Iter<'a, Stripped<IndexedObject<T, B, M>>>>);

impl<'a, T, B, M> Objects<'a, T, B, M> {
	#[inline(always)]
	pub(crate) fn new(
		inner: Option<std::slice::Iter<'a, Stripped<IndexedObject<T, B, M>>>>,
	) -> Self {
		Self(inner)
	}
}

impl<'a, T, B, M> Iterator for Objects<'a, T, B, M> {
	type Item = &'a IndexedObject<T, B, M>;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a IndexedObject<T, B, M>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next().map(|o| &o.0),
		}
	}
}

/// JSON-LD object fragment.
pub enum FragmentRef<'a, T, B, M> {
	/// "@index" entry.
	IndexEntry(&'a str),

	/// "@index" entry key.
	IndexKey,

	/// "@index" entry value.
	IndexValue(&'a str),

	/// Object.
	Object(&'a Object<T, B, M>),

	/// Indexed object.
	IndexedObject(&'a Indexed<Object<T, B, M>, M>),

	/// Node object.
	Node(&'a Node<T, B, M>),

	/// Indexed node object.
	IndexedNode(&'a IndexedNode<T, B, M>),

	IndexedNodeList(&'a [StrippedIndexedNode<T, B, M>]),

	/// Value object fragment.
	ValueFragment(value::FragmentRef<'a, T, M>),

	/// List object fragment.
	ListFragment(list::FragmentRef<'a, T, B, M>),

	/// Node object fragment.
	NodeFragment(node::FragmentRef<'a, T, B, M>),
}

impl<'a, T, B, M> FragmentRef<'a, T, B, M> {
	pub fn into_ref(self) -> Option<Ref<'a, T, B, M>> {
		match self {
			Self::Object(o) => Some(o.as_ref()),
			Self::IndexedObject(o) => Some(o.inner().as_ref()),
			Self::Node(n) => Some(n.as_ref()),
			Self::IndexedNode(n) => Some(n.inner().as_ref()),
			_ => None,
		}
	}

	pub fn into_id(self) -> Option<Id<&'a T, &'a B>> {
		match self {
			Self::ValueFragment(i) => i.into_iri().map(Id::iri),
			Self::NodeFragment(i) => i.into_id().map(Meta::into_value).map(Into::into),
			_ => None,
		}
	}

	pub fn as_id(&self) -> Option<Id<&'a T, &'a B>> {
		match self {
			Self::ValueFragment(i) => i.as_iri().map(Id::iri),
			Self::NodeFragment(i) => i.as_id().map(Into::into),
			_ => None,
		}
	}

	pub fn is_json_array(&self) -> bool {
		match self {
			Self::IndexedNodeList(_) => true,
			Self::ValueFragment(i) => i.is_json_array(),
			Self::NodeFragment(n) => n.is_json_array(),
			_ => false,
		}
	}

	pub fn is_json_object(&self) -> bool {
		match self {
			Self::Object(_) | Self::IndexedObject(_) | Self::Node(_) | Self::IndexedNode(_) => true,
			Self::ValueFragment(i) => i.is_json_array(),
			Self::NodeFragment(i) => i.is_json_array(),
			_ => false,
		}
	}

	pub fn sub_fragments(&self) -> SubFragments<'a, T, B, M> {
		match self {
			Self::IndexEntry(v) => SubFragments::IndexEntry(Some(()), Some(v)),
			Self::Object(o) => SubFragments::Object(None, o.sub_fragments()),
			Self::IndexedObject(o) => SubFragments::Object(o.index(), o.sub_fragments()),
			Self::Node(n) => SubFragments::Object(None, ObjectSubFragments::Node(n.entries())),
			Self::IndexedNode(n) => {
				SubFragments::Object(n.index(), ObjectSubFragments::Node(n.inner().entries()))
			}
			Self::IndexedNodeList(l) => SubFragments::IndexedNodeList(l.iter()),
			Self::ValueFragment(i) => SubFragments::Value(i.sub_fragments()),
			Self::NodeFragment(i) => SubFragments::Node(i.sub_fragments()),
			_ => SubFragments::None,
		}
	}
}

pub enum ObjectSubFragments<'a, T, B, M> {
	List(Option<list::EntryRef<'a, T, B, M>>),
	Value(value::Entries<'a, T, M>),
	Node(node::Entries<'a, T, B, M>),
}

impl<'a, T, B, M> Iterator for ObjectSubFragments<'a, T, B, M> {
	type Item = FragmentRef<'a, T, B, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::List(l) => l
				.take()
				.map(|e| FragmentRef::ListFragment(list::FragmentRef::Entry(e))),
			Self::Value(e) => e
				.next_back()
				.map(|e| FragmentRef::ValueFragment(value::FragmentRef::Entry(e))),
			Self::Node(e) => e
				.next()
				.map(|e| FragmentRef::NodeFragment(node::FragmentRef::Entry(e))),
		}
	}
}

pub enum SubFragments<'a, T, B, M> {
	None,
	IndexEntry(Option<()>, Option<&'a str>),
	Object(Option<&'a str>, ObjectSubFragments<'a, T, B, M>),
	Value(value::SubFragments<'a, T, M>),
	Node(node::SubFragments<'a, T, B, M>),
	IndexedNodeList(std::slice::Iter<'a, StrippedIndexedNode<T, B, M>>),
}

impl<'a, T, B, M> Iterator for SubFragments<'a, T, B, M> {
	type Item = FragmentRef<'a, T, B, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::IndexEntry(k, v) => k
				.take()
				.map(|()| FragmentRef::IndexKey)
				.or_else(|| v.take().map(FragmentRef::IndexValue)),
			Self::Object(index, i) => match index.take() {
				Some(index) => Some(FragmentRef::IndexEntry(index)),
				None => i.next(),
			},
			Self::Value(i) => i.next().map(FragmentRef::ValueFragment),
			Self::Node(i) => i.next(),
			Self::IndexedNodeList(i) => i.next().map(|n| FragmentRef::IndexedNode(&n.0)),
		}
	}
}

pub struct Traverse<'a, T, B, M> {
	stack: SmallVec<[FragmentRef<'a, T, B, M>; 8]>,
}

impl<'a, T, B, M> Traverse<'a, T, B, M> {
	pub(crate) fn new(items: impl IntoIterator<Item = FragmentRef<'a, T, B, M>>) -> Self {
		let stack = items.into_iter().collect();
		Self { stack }
	}
}

impl<'a, T, B, M> Iterator for Traverse<'a, T, B, M> {
	type Item = FragmentRef<'a, T, B, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.stack.pop() {
			Some(item) => {
				self.stack.extend(item.sub_fragments());
				Some(item)
			}
			None => None,
		}
	}
}

impl<T, B, M: Clone, N: Vocabulary<Iri = T, BlankId = B>> IntoJsonWithContextMeta<M, N>
	for Object<T, B, M>
{
	fn into_json_meta_with(self, meta: M, vocabulary: &N) -> Meta<json_syntax::Value<M>, M> {
		match self {
			Self::Value(v) => v.into_json_meta_with(meta, vocabulary),
			Self::Node(n) => n.into_json_meta_with(meta, vocabulary),
			Self::List(l) => l.into_json_meta_with(meta, vocabulary),
		}
	}
}
