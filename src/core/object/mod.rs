//! Nodes, lists and values.
use crate::object::typ::TypeRef;
use crate::syntax::Keyword;
use crate::{Id, Indexed, LenientLangTag};
use educe::Educe;
use iref::Iri;
use json_syntax::Number;
use std::hash::Hash;

pub mod list;
mod mapped_eq;
pub mod node;
mod typ;
pub mod value;

pub use list::ListObject;
pub use mapped_eq::MappedEq;
pub use node::{Graph, IndexedNode, NodeObject, Nodes};
pub use value::{LiteralValue, ValueObject};

/// Abstract object.
pub trait AnyObject {
	fn as_ref(&self) -> Ref;

	#[inline]
	fn id(&self) -> Option<&Id> {
		match self.as_ref() {
			Ref::Node(n) => n.id.as_ref(),
			_ => None,
		}
	}

	#[inline]
	fn language(&self) -> Option<&LenientLangTag> {
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
pub enum Ref<'a> {
	/// Value object.
	Value(&'a ValueObject),

	/// Node object.
	Node(&'a NodeObject),

	/// List object.
	List(&'a ListObject),
}

/// Indexed object.
pub type IndexedObject = Indexed<Object>;

/// Object.
///
/// JSON-LD connects together multiple kinds of data objects.
/// Objects may be nodes, values or lists of objects.
///
/// You can get an `Object` by expanding a JSON-LD document using the
/// expansion algorithm or by converting an already expanded JSON document
/// using [`TryFromJson`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Object {
	/// Value object.
	Value(ValueObject),

	/// Node object.
	Node(Box<NodeObject>),

	/// List object.
	List(ListObject),
}

impl Object {
	/// Creates a `null` value object.
	#[inline(always)]
	pub fn null() -> Self {
		Self::Value(ValueObject::null())
	}

	/// Creates a new node object from a node.
	#[inline(always)]
	pub fn node(n: NodeObject) -> Self {
		Self::Node(Box::new(n))
	}

	/// Identifier of the object, if it is a node object.
	#[inline(always)]
	pub fn id(&self) -> Option<&Id> {
		match self {
			Object::Node(n) => n.id.as_ref(),
			_ => None,
		}
	}

	// /// Use the given `generator` to assign an identifier to all nodes that
	// /// don't have one.
	// pub fn identify_all(&mut self, generator: &mut impl Generator) {
	// 	match self {
	// 		Object::Node(n) => n.identify_all_with(vocabulary, generator),
	// 		Object::List(l) => {
	// 			for object in l {
	// 				object.identify_all_with(vocabulary, generator)
	// 			}
	// 		}
	// 		_ => (),
	// 	}
	// }

	// /// Puts this object literals into canonical form using the given
	// /// `buffer`.
	// ///
	// /// The buffer is used to compute the canonical form of numbers.
	// pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
	// 	match self {
	// 		Self::List(l) => l.canonicalize_with(buffer),
	// 		Self::Node(n) => n.canonicalize_with(buffer),
	// 		Self::Value(v) => v.canonicalize_with(buffer),
	// 	}
	// }

	// /// Puts this object literals into canonical form.
	// pub fn canonicalize(&mut self) {
	// 	let mut buffer = ryu_js::Buffer::new();
	// 	self.canonicalize_with(&mut buffer)
	// }

	/// Returns an iterator over the types of the object.
	pub fn types(&self) -> Types {
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
	pub fn as_iri(&self) -> Option<&Iri> {
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
	pub fn as_value(&self) -> Option<&ValueObject> {
		match self {
			Self::Value(v) => Some(v),
			_ => None,
		}
	}

	/// Returns this object as a mutable value, if it is one.
	#[inline(always)]
	pub fn as_value_mut(&mut self) -> Option<&mut ValueObject> {
		match self {
			Self::Value(v) => Some(v),
			_ => None,
		}
	}

	/// Converts this object as a value, if it is one.
	#[inline(always)]
	pub fn into_value(self) -> Option<ValueObject> {
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
	pub fn as_node(&self) -> Option<&NodeObject> {
		match self {
			Self::Node(n) => Some(n),
			_ => None,
		}
	}

	/// Returns this object as a mutable node, if it is one.
	#[inline(always)]
	pub fn as_node_mut(&mut self) -> Option<&mut NodeObject> {
		match self {
			Self::Node(n) => Some(n),
			_ => None,
		}
	}

	/// Converts this object into a node, if it is one.
	#[inline(always)]
	pub fn into_node(self) -> Option<NodeObject> {
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
	pub fn as_list(&self) -> Option<&ListObject> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}

	/// Returns this object as a mutable list, if it is one.
	#[inline(always)]
	pub fn as_list_mut(&mut self) -> Option<&mut ListObject> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}

	/// Converts this object into a list, if it is one.
	#[inline(always)]
	pub fn into_list(self) -> Option<ListObject> {
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
	pub fn as_str(&self) -> Option<&str> {
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
	pub fn language(&self) -> Option<&LenientLangTag> {
		match self {
			Object::Value(value) => value.language(),
			_ => None,
		}
	}

	/// Equivalence operator.
	///
	/// Equivalence is different from equality for anonymous objects:
	/// List objects and anonymous node objects have an implicit unlabeled blank nodes and thus never equivalent.
	pub fn equivalent(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Value(a), Self::Value(b)) => a == b,
			(Self::Node(a), Self::Node(b)) => a.equivalent(b),
			_ => false,
		}
	}

	/// Returns an iterator over the entries of JSON representation of the
	/// object.
	pub fn entries(&self) -> Entries {
		match self {
			Self::Value(value) => Entries::Value(value.entries()),
			Self::List(list) => Entries::List(Some(list.entry())),
			Self::Node(node) => Entries::Node(node.entries()),
		}
	}
}

// impl Relabel for Object {
// 	fn relabel_with<N: Vocabulary<Iri = T, BlankId = B>, G: Generator<N>>(
// 		&mut self,
// 		vocabulary: &mut N,
// 		generator: &mut G,
// 		relabeling: &mut hashbrown::HashMap<B, Subject>,
// 	) where
// 		T: Clone + Eq + Hash,
// 		B: Clone + Eq + Hash,
// 	{
// 		match self {
// 			Self::Node(n) => n.relabel_with(vocabulary, generator, relabeling),
// 			Self::List(l) => l.relabel_with(vocabulary, generator, relabeling),
// 			Self::Value(_) => (),
// 		}
// 	}
// }

impl Indexed<Object> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}

	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_node(self) -> Option<Indexed<NodeObject>> {
		let (object, index) = self.into_parts();
		object.into_node().map(|node| Indexed::new(node, index))
	}

	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_value(self) -> Option<Indexed<ValueObject>> {
		let (object, index) = self.into_parts();
		object.into_value().map(|value| Indexed::new(value, index))
	}

	/// Converts this indexed object into an indexed list, if it is one.
	#[inline(always)]
	pub fn into_indexed_list(self) -> Option<Indexed<ListObject>> {
		let (object, index) = self.into_parts();
		object.into_list().map(|list| Indexed::new(list, index))
	}

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self) -> Result<Graph, Self> {
		let (obj, index) = self.into_parts();
		match obj {
			Object::Node(n) => match n.into_unnamed_graph() {
				Ok(g) => Ok(g),
				Err(n) => Err(Indexed::new(Object::node(n), index)),
			},
			obj => Err(Indexed::new(obj, index)),
		}
	}

	pub fn entries(&self) -> IndexedEntries {
		IndexedEntries {
			index: self.index(),
			inner: self.inner().entries(),
		}
	}
}

#[derive(Educe)]
#[educe(Clone)]
pub enum Entries<'a> {
	Value(value::Entries<'a>),
	List(Option<&'a [IndexedObject]>),
	Node(node::Entries<'a>),
}

impl<'a> Iterator for Entries<'a> {
	type Item = EntryRef<'a>;

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
			.or_else(|| self.inner.next().map(IndexedEntryRef::Object))
	}
}

impl<'a> ExactSizeIterator for IndexedEntries<'a> {}

#[derive(Educe, PartialEq, Eq)]
#[educe(Clone, Copy)]
pub enum EntryKeyRef<'a> {
	Value(value::EntryKey),
	List,
	Node(node::EntryKeyRef<'a>),
}

impl<'a> EntryKeyRef<'a> {
	pub fn into_keyword(self) -> Option<Keyword> {
		match self {
			Self::Value(e) => Some(e.into_keyword()),
			Self::List => Some(Keyword::List),
			Self::Node(e) => e.into_keyword(),
		}
	}

	pub fn as_keyword(&self) -> Option<Keyword> {
		self.into_keyword()
	}

	pub fn into_str(self) -> &'a str {
		match self {
			Self::Value(e) => e.into_str(),
			Self::List => "@list",
			Self::Node(e) => e.into_str(),
		}
	}

	pub fn as_str(self) -> &'a str {
		self.into_str()
	}
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryValueRef<'a> {
	Value(value::EntryRef<'a>),
	List(&'a [IndexedObject]),
	Node(node::EntryValueRef<'a>),
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum EntryRef<'a> {
	Value(value::EntryRef<'a>),
	List(&'a [IndexedObject]),
	Node(node::EntryRef<'a>),
}

impl<'a> EntryRef<'a> {
	pub fn into_key(self) -> EntryKeyRef<'a> {
		match self {
			Self::Value(e) => EntryKeyRef::Value(e.key()),
			Self::List(_) => EntryKeyRef::List,
			Self::Node(e) => EntryKeyRef::Node(e.key()),
		}
	}

	pub fn key(&self) -> EntryKeyRef<'a> {
		self.into_key()
	}

	pub fn into_value(self) -> EntryValueRef<'a> {
		match self {
			Self::Value(v) => EntryValueRef::Value(v),
			Self::List(v) => EntryValueRef::List(v),
			Self::Node(e) => EntryValueRef::Node(e.value()),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (EntryKeyRef<'a>, EntryValueRef<'a>) {
		match self {
			Self::Value(e) => (EntryKeyRef::Value(e.key()), EntryValueRef::Value(e)),
			Self::List(e) => (EntryKeyRef::List, EntryValueRef::List(e)),
			Self::Node(e) => {
				let (k, v) = e.into_key_value();
				(EntryKeyRef::Node(k), EntryValueRef::Node(v))
			}
		}
	}

	pub fn as_key_value(&self) -> (EntryKeyRef<'a>, EntryValueRef<'a>) {
		self.into_key_value()
	}
}

#[derive(Educe, PartialEq, Eq)]
#[educe(Clone, Copy)]
pub enum IndexedEntryKeyRef<'a> {
	Index,
	Object(EntryKeyRef<'a>),
}

impl<'a> IndexedEntryKeyRef<'a> {
	pub fn into_keyword(self) -> Option<Keyword> {
		match self {
			Self::Index => Some(Keyword::Index),
			Self::Object(e) => e.into_keyword(),
		}
	}

	pub fn as_keyword(&self) -> Option<Keyword> {
		self.into_keyword()
	}

	pub fn into_str(self) -> &'a str {
		match self {
			Self::Index => "@index",
			Self::Object(e) => e.into_str(),
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
	Object(EntryValueRef<'a>),
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum IndexedEntryRef<'a> {
	Index(&'a str),
	Object(EntryRef<'a>),
}

impl<'a> IndexedEntryRef<'a> {
	pub fn into_key(self) -> IndexedEntryKeyRef<'a> {
		match self {
			Self::Index(_) => IndexedEntryKeyRef::Index,
			Self::Object(e) => IndexedEntryKeyRef::Object(e.key()),
		}
	}

	pub fn key(&self) -> IndexedEntryKeyRef<'a> {
		self.into_key()
	}

	pub fn into_value(self) -> IndexedEntryValueRef<'a> {
		match self {
			Self::Index(v) => IndexedEntryValueRef::Index(v),
			Self::Object(e) => IndexedEntryValueRef::Object(e.value()),
		}
	}

	pub fn value(&self) -> IndexedEntryValueRef<'a> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (IndexedEntryKeyRef<'a>, IndexedEntryValueRef<'a>) {
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

	pub fn as_key_value(&self) -> (IndexedEntryKeyRef<'a>, IndexedEntryValueRef<'a>) {
		self.into_key_value()
	}
}

// /// Invalid expanded JSON object error.
// ///
// /// This can be raised when trying to directly convert a JSON value into an
// /// expanded JSON-LD object without using the expansion algorithm.
// #[derive(Debug)]
// pub enum InvalidExpandedJson {
// 	InvalidObject,
// 	InvalidList,
// 	InvalidIndex,
// 	InvalidId,
// 	InvalidValueType,
// 	InvalidLiteral,
// 	InvalidLanguage,
// 	InvalidDirection,
// 	NotExpanded,
// 	UnexpectedEntry,
// 	DuplicateKey(json_syntax::object::Key),
// 	Unexpected(json_syntax::Kind, json_syntax::Kind),
// }

// impl InvalidExpandedJson {
// 	pub fn duplicate_key(
// 		json_syntax::object::Duplicate(a, _): json_syntax::object::Duplicate<
// 			json_syntax::object::Entry,
// 		>,
// 	) -> Self {
// 		InvalidExpandedJson::DuplicateKey(a.key)
// 	}
// }

impl AnyObject for Object {
	#[inline(always)]
	fn as_ref(&self) -> Ref {
		match self {
			Object::Value(value) => Ref::Value(value),
			Object::Node(node) => Ref::Node(node),
			Object::List(list) => Ref::List(list),
		}
	}
}

impl From<ValueObject> for Object {
	#[inline(always)]
	fn from(value: ValueObject) -> Self {
		Self::Value(value)
	}
}

impl From<NodeObject> for Object {
	#[inline(always)]
	fn from(node: NodeObject) -> Self {
		Self::node(node)
	}
}

/// Iterator through the types of an object.
pub enum Types<'a> {
	Value(Option<value::ValueTypeRef<'a>>),
	Node(std::slice::Iter<'a, Id>),
	List,
}

impl<'a> Iterator for Types<'a> {
	type Item = TypeRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Value(ty) => ty.take().map(TypeRef::from_value_type),
			Self::Node(tys) => tys.next().map(TypeRef::from_reference),
			Self::List => None,
		}
	}
}

/// Iterator through indexed objects.
pub struct Objects<'a>(Option<std::slice::Iter<'a, IndexedObject>>);

impl<'a> Objects<'a> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, IndexedObject>>) -> Self {
		Self(inner)
	}
}

impl<'a> Iterator for Objects<'a> {
	type Item = &'a IndexedObject;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a IndexedObject> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}
