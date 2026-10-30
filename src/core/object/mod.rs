//! Nodes, lists and values.
use btree_indexmap::BTreeIndexMultiSet;
use educe::Educe;
use json_syntax::JsonNumber;
use langtag::LangTag;
use rdf_syntax::Id;
use rdf_syntax::Iri;
use std::hash::Hash;

use crate::syntax::JsonLdKeyword;
use crate::{JsonLdIndexed, JsonLdLenient, JsonLdVisit};

pub mod list;
pub mod node;
mod typ;
pub mod value;

pub use list::JsonLdList;
pub use node::{JsonLdGraph, JsonLdIndexedNode, JsonLdNode, JsonLdNodes};
pub use value::{JsonLdLiteralValue, JsonLdValue};

use typ::JsonLdObjectTypeRef;

/// Abstract object.
pub trait AnyJsonLdObject {
	fn as_ref(&self) -> JsonLdObjectRef<'_>;

	#[inline]
	fn id(&self) -> Option<&JsonLdLenient<Id>> {
		match self.as_ref() {
			JsonLdObjectRef::Node(n) => n.id.as_ref(),
			_ => None,
		}
	}

	#[inline]
	fn language(&self) -> Option<JsonLdLenient<&LangTag>> {
		match self.as_ref() {
			JsonLdObjectRef::Value(value) => value.language(),
			_ => None,
		}
	}

	#[inline]
	fn is_value(&self) -> bool {
		matches!(self.as_ref(), JsonLdObjectRef::Value(_))
	}

	#[inline]
	fn is_node(&self) -> bool {
		matches!(self.as_ref(), JsonLdObjectRef::Node(_))
	}

	#[inline]
	fn is_graph(&self) -> bool {
		match self.as_ref() {
			JsonLdObjectRef::Node(n) => n.is_graph(),
			_ => false,
		}
	}

	#[inline]
	fn is_list(&self) -> bool {
		matches!(self.as_ref(), JsonLdObjectRef::List(_))
	}
}

/// Object reference.
pub enum JsonLdObjectRef<'a> {
	/// Value object.
	Value(&'a JsonLdValue),

	/// Node object.
	Node(&'a JsonLdNode),

	/// List object.
	List(&'a JsonLdList),
}

pub enum JsonLdObjectMut<'a> {
	Value(&'a mut JsonLdValue),

	Node(&'a mut JsonLdNode),

	List(&'a mut JsonLdList),
}

impl<'a> JsonLdObjectMut<'a> {
	pub fn reborrow(&mut self) -> JsonLdObjectMut<'_> {
		match self {
			Self::Value(v) => JsonLdObjectMut::Value(v),
			Self::Node(n) => JsonLdObjectMut::Node(n),
			Self::List(l) => JsonLdObjectMut::List(l),
		}
	}
}

/// Indexed object.
pub type JsonLdIndexedObject = JsonLdIndexed<JsonLdObject>;

/// Object.
///
/// JSON-LD connects together multiple kinds of data objects.
/// Objects may be nodes, values or lists of objects.
///
/// You can get a `JsonLdObject` by expanding a JSON-LD document using the
/// expansion algorithm.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum JsonLdObject {
	/// Value object.
	Value(JsonLdValue),

	/// List object.
	List(JsonLdList),

	/// Node object.
	Node(Box<JsonLdNode>),
}

impl JsonLdObject {
	/// Creates a `null` value object.
	#[inline(always)]
	pub fn null() -> Self {
		Self::Value(JsonLdValue::null())
	}

	/// Creates a new node object from a node.
	#[inline(always)]
	pub fn node(n: JsonLdNode) -> Self {
		Self::Node(Box::new(n))
	}

	/// Identifier of the object, if it is a node object.
	#[inline(always)]
	pub fn id(&self) -> Option<&JsonLdLenient<Id>> {
		match self {
			JsonLdObject::Node(n) => n.id.as_ref(),
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
	pub fn types(&self) -> JsonLdObjectTypes<'_> {
		match self {
			Self::Value(value) => JsonLdObjectTypes::Value(value.typ()),
			Self::Node(node) => {
				JsonLdObjectTypes::Node(node.types.as_ref().map(BTreeIndexMultiSet::iter))
			}
			Self::List(_) => JsonLdObjectTypes::List,
		}
	}

	/// Identifier of the object as an IRI.
	///
	/// If the object is a node identified by an IRI, returns this IRI.
	/// Returns `None` otherwise.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			JsonLdObject::Node(node) => node.as_iri(),
			_ => None,
		}
	}

	/// Tests if the object is a value.
	#[inline(always)]
	pub fn is_value(&self) -> bool {
		matches!(self, JsonLdObject::Value(_))
	}

	/// Returns this object as a value, if it is one.
	#[inline(always)]
	pub fn as_value(&self) -> Option<&JsonLdValue> {
		match self {
			Self::Value(v) => Some(v),
			_ => None,
		}
	}

	/// Returns this object as a mutable value, if it is one.
	#[inline(always)]
	pub fn as_value_mut(&mut self) -> Option<&mut JsonLdValue> {
		match self {
			Self::Value(v) => Some(v),
			_ => None,
		}
	}

	/// Converts this object as a value, if it is one.
	#[inline(always)]
	pub fn into_value(self) -> Option<JsonLdValue> {
		match self {
			Self::Value(v) => Some(v),
			_ => None,
		}
	}

	/// Tests if the object is a node.
	#[inline(always)]
	pub fn is_node(&self) -> bool {
		matches!(self, JsonLdObject::Node(_))
	}

	/// Returns this object as a node, if it is one.
	#[inline(always)]
	pub fn as_node(&self) -> Option<&JsonLdNode> {
		match self {
			Self::Node(n) => Some(n),
			_ => None,
		}
	}

	/// Returns this object as a mutable node, if it is one.
	#[inline(always)]
	pub fn as_node_mut(&mut self) -> Option<&mut JsonLdNode> {
		match self {
			Self::Node(n) => Some(n),
			_ => None,
		}
	}

	/// Converts this object into a node, if it is one.
	#[inline(always)]
	pub fn into_node(self) -> Option<JsonLdNode> {
		match self {
			Self::Node(n) => Some(*n),
			_ => None,
		}
	}

	/// Tests if the object is a graph object (a node with a `@graph` field).
	#[inline(always)]
	pub fn is_graph(&self) -> bool {
		match self {
			JsonLdObject::Node(n) => n.is_graph(),
			_ => false,
		}
	}

	/// Tests if the object is a list.
	#[inline(always)]
	pub fn is_list(&self) -> bool {
		matches!(self, JsonLdObject::List(_))
	}

	/// Returns this object as a list, if it is one.
	#[inline(always)]
	pub fn as_list(&self) -> Option<&JsonLdList> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}

	/// Returns this object as a mutable list, if it is one.
	#[inline(always)]
	pub fn as_list_mut(&mut self) -> Option<&mut JsonLdList> {
		match self {
			Self::List(l) => Some(l),
			_ => None,
		}
	}

	/// Converts this object into a list, if it is one.
	#[inline(always)]
	pub fn into_list(self) -> Option<JsonLdList> {
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
			JsonLdObject::Value(value) => value.as_str(),
			JsonLdObject::Node(node) => node.as_str(),
			_ => None,
		}
	}

	/// Get the value as a boolean, if it is.
	#[inline(always)]
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			JsonLdObject::Value(value) => value.as_bool(),
			_ => None,
		}
	}

	/// Get the value as a number, if it is.
	#[inline(always)]
	pub fn as_number(&self) -> Option<&JsonNumber> {
		match self {
			JsonLdObject::Value(value) => value.as_number(),
			_ => None,
		}
	}

	/// If the object is a language-tagged value,
	/// Return its associated language.
	#[inline(always)]
	pub fn language(&self) -> Option<JsonLdLenient<&LangTag>> {
		match self {
			JsonLdObject::Value(value) => value.language(),
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
	pub fn entries(&self) -> Entries<'_> {
		match self {
			Self::Value(value) => Entries::Value(value.entries()),
			Self::List(list) => Entries::List(Some(list.entry())),
			Self::Node(node) => Entries::Node(node.entries()),
		}
	}
}

impl JsonLdVisit for JsonLdObject {
	fn visit_with(&self, f: &mut impl FnMut(JsonLdObjectRef)) {
		match self {
			Self::Value(v) => v.visit_with(f),
			Self::List(l) => l.visit_with(f),
			Self::Node(n) => n.visit_with(f),
		}
	}

	fn visit_mut_with(&mut self, f: &mut impl FnMut(JsonLdObjectMut)) {
		match self {
			Self::Value(v) => v.visit_mut_with(f),
			Self::List(l) => l.visit_mut_with(f),
			Self::Node(n) => n.visit_mut_with(f),
		}
	}
}

impl JsonLdIndexed<JsonLdObject> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}

	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_node(self) -> Option<JsonLdIndexed<JsonLdNode>> {
		let (object, index) = self.into_parts();
		object
			.into_node()
			.map(|node| JsonLdIndexed::new(node, index))
	}

	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_value(self) -> Option<JsonLdIndexed<JsonLdValue>> {
		let (object, index) = self.into_parts();
		object
			.into_value()
			.map(|value| JsonLdIndexed::new(value, index))
	}

	/// Converts this indexed object into an indexed list, if it is one.
	#[inline(always)]
	pub fn into_indexed_list(self) -> Option<JsonLdIndexed<JsonLdList>> {
		let (object, index) = self.into_parts();
		object
			.into_list()
			.map(|list| JsonLdIndexed::new(list, index))
	}

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self) -> Result<JsonLdGraph, Self> {
		let (obj, index) = self.into_parts();
		match obj {
			JsonLdObject::Node(n) => match n.into_unnamed_graph() {
				Ok(g) => Ok(g),
				Err(n) => Err(JsonLdIndexed::new(JsonLdObject::node(n), index)),
			},
			obj => Err(JsonLdIndexed::new(obj, index)),
		}
	}

	pub fn entries(&self) -> IndexedEntries<'_> {
		IndexedEntries {
			index: self.index(),
			inner: self.inner().entries(),
		}
	}
}

#[derive(Educe)]
#[educe(Clone)]
pub enum Entries<'a> {
	Value(value::JsonLdValueEntries<'a>),
	List(Option<&'a [JsonLdIndexedObject]>),
	Node(node::JsonLdNodeEntries<'a>),
}

impl<'a> Iterator for Entries<'a> {
	type Item = JsonLdObjectEntryRef<'a>;

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
			Self::Value(v) => v.next().map(JsonLdObjectEntryRef::Value),
			Self::List(l) => l.take().map(JsonLdObjectEntryRef::List),
			Self::Node(n) => n.next().map(JsonLdObjectEntryRef::Node),
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
	type Item = JsonLdIndexedObjectEntryRef<'a>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.inner.len() + usize::from(self.index.is_some());
		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.index
			.take()
			.map(JsonLdIndexedObjectEntryRef::Index)
			.or_else(|| self.inner.next().map(JsonLdIndexedObjectEntryRef::Object))
	}
}

impl<'a> ExactSizeIterator for IndexedEntries<'a> {}

#[derive(Educe, PartialEq, Eq)]
#[educe(Clone, Copy)]
pub enum JsonLdObjectEntryKeyRef<'a> {
	Value(value::JsonLdValueEntryKey),
	List,
	Node(node::JsonLdNodeEntryKeyRef<'a>),
}

impl<'a> JsonLdObjectEntryKeyRef<'a> {
	pub fn into_keyword(self) -> Option<JsonLdKeyword> {
		match self {
			Self::Value(e) => Some(e.into_keyword()),
			Self::List => Some(JsonLdKeyword::List),
			Self::Node(e) => e.into_keyword(),
		}
	}

	pub fn as_keyword(&self) -> Option<JsonLdKeyword> {
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
pub enum JsonLdObjectEntryValueRef<'a> {
	Value(value::JsonLdValueEntryRef<'a>),
	List(&'a [JsonLdIndexedObject]),
	Node(node::JsonLdNodeEntryValueRef<'a>),
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum JsonLdObjectEntryRef<'a> {
	Value(value::JsonLdValueEntryRef<'a>),
	List(&'a [JsonLdIndexedObject]),
	Node(node::JsonLdNodeEntryRef<'a>),
}

impl<'a> JsonLdObjectEntryRef<'a> {
	pub fn into_key(self) -> JsonLdObjectEntryKeyRef<'a> {
		match self {
			Self::Value(e) => JsonLdObjectEntryKeyRef::Value(e.key()),
			Self::List(_) => JsonLdObjectEntryKeyRef::List,
			Self::Node(e) => JsonLdObjectEntryKeyRef::Node(e.key()),
		}
	}

	pub fn key(&self) -> JsonLdObjectEntryKeyRef<'a> {
		self.into_key()
	}

	pub fn into_value(self) -> JsonLdObjectEntryValueRef<'a> {
		match self {
			Self::Value(v) => JsonLdObjectEntryValueRef::Value(v),
			Self::List(v) => JsonLdObjectEntryValueRef::List(v),
			Self::Node(e) => JsonLdObjectEntryValueRef::Node(e.value()),
		}
	}

	pub fn value(&self) -> JsonLdObjectEntryValueRef<'a> {
		self.into_value()
	}

	pub fn into_key_value(self) -> (JsonLdObjectEntryKeyRef<'a>, JsonLdObjectEntryValueRef<'a>) {
		match self {
			Self::Value(e) => (
				JsonLdObjectEntryKeyRef::Value(e.key()),
				JsonLdObjectEntryValueRef::Value(e),
			),
			Self::List(e) => (
				JsonLdObjectEntryKeyRef::List,
				JsonLdObjectEntryValueRef::List(e),
			),
			Self::Node(e) => {
				let (k, v) = e.into_key_value();
				(
					JsonLdObjectEntryKeyRef::Node(k),
					JsonLdObjectEntryValueRef::Node(v),
				)
			}
		}
	}

	pub fn as_key_value(&self) -> (JsonLdObjectEntryKeyRef<'a>, JsonLdObjectEntryValueRef<'a>) {
		self.into_key_value()
	}
}

#[derive(Educe, PartialEq, Eq)]
#[educe(Clone, Copy)]
pub enum JsonLdIndexedObjectEntryKeyRef<'a> {
	Index,
	Object(JsonLdObjectEntryKeyRef<'a>),
}

impl<'a> JsonLdIndexedObjectEntryKeyRef<'a> {
	pub fn into_keyword(self) -> Option<JsonLdKeyword> {
		match self {
			Self::Index => Some(JsonLdKeyword::Index),
			Self::Object(e) => e.into_keyword(),
		}
	}

	pub fn as_keyword(&self) -> Option<JsonLdKeyword> {
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
pub enum JsonLdIndexedObjectEntryValueRef<'a> {
	Index(&'a str),
	Object(JsonLdObjectEntryValueRef<'a>),
}

#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum JsonLdIndexedObjectEntryRef<'a> {
	Index(&'a str),
	Object(JsonLdObjectEntryRef<'a>),
}

impl<'a> JsonLdIndexedObjectEntryRef<'a> {
	pub fn into_key(self) -> JsonLdIndexedObjectEntryKeyRef<'a> {
		match self {
			Self::Index(_) => JsonLdIndexedObjectEntryKeyRef::Index,
			Self::Object(e) => JsonLdIndexedObjectEntryKeyRef::Object(e.key()),
		}
	}

	pub fn key(&self) -> JsonLdIndexedObjectEntryKeyRef<'a> {
		self.into_key()
	}

	pub fn into_value(self) -> JsonLdIndexedObjectEntryValueRef<'a> {
		match self {
			Self::Index(v) => JsonLdIndexedObjectEntryValueRef::Index(v),
			Self::Object(e) => JsonLdIndexedObjectEntryValueRef::Object(e.value()),
		}
	}

	pub fn value(&self) -> JsonLdIndexedObjectEntryValueRef<'a> {
		self.into_value()
	}

	pub fn into_key_value(
		self,
	) -> (
		JsonLdIndexedObjectEntryKeyRef<'a>,
		JsonLdIndexedObjectEntryValueRef<'a>,
	) {
		match self {
			Self::Index(v) => (
				JsonLdIndexedObjectEntryKeyRef::Index,
				JsonLdIndexedObjectEntryValueRef::Index(v),
			),
			Self::Object(e) => {
				let (k, v) = e.into_key_value();
				(
					JsonLdIndexedObjectEntryKeyRef::Object(k),
					JsonLdIndexedObjectEntryValueRef::Object(v),
				)
			}
		}
	}

	pub fn as_key_value(
		&self,
	) -> (
		JsonLdIndexedObjectEntryKeyRef<'a>,
		JsonLdIndexedObjectEntryValueRef<'a>,
	) {
		self.into_key_value()
	}
}

impl AnyJsonLdObject for JsonLdObject {
	#[inline(always)]
	fn as_ref(&self) -> JsonLdObjectRef<'_> {
		match self {
			JsonLdObject::Value(value) => JsonLdObjectRef::Value(value),
			JsonLdObject::Node(node) => JsonLdObjectRef::Node(node),
			JsonLdObject::List(list) => JsonLdObjectRef::List(list),
		}
	}
}

impl From<JsonLdValue> for JsonLdObject {
	#[inline(always)]
	fn from(value: JsonLdValue) -> Self {
		Self::Value(value)
	}
}

impl From<JsonLdNode> for JsonLdObject {
	#[inline(always)]
	fn from(node: JsonLdNode) -> Self {
		Self::node(node)
	}
}

/// Iterator through the types of an object.
pub enum JsonLdObjectTypes<'a> {
	Value(Option<value::JsonLdValueTypeRef<'a>>),
	Node(Option<btree_indexmap::multi_set::Iter<'a, JsonLdLenient<Id>>>),
	List,
}

impl<'a> Iterator for JsonLdObjectTypes<'a> {
	type Item = JsonLdObjectTypeRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Value(ty) => ty.take().map(JsonLdObjectTypeRef::from_value_type),
			Self::Node(tys) => tys.as_mut()?.next().map(JsonLdObjectTypeRef::from_id),
			Self::List => None,
		}
	}
}

/// Iterator through indexed objects.
pub struct JsonLdObjects<'a>(Option<std::slice::Iter<'a, JsonLdIndexedObject>>);

impl<'a> JsonLdObjects<'a> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, JsonLdIndexedObject>>) -> Self {
		Self(inner)
	}
}

impl<'a> Iterator for JsonLdObjects<'a> {
	type Item = &'a JsonLdIndexedObject;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a JsonLdIndexedObject> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}
