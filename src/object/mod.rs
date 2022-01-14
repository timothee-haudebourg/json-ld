//! Nodes, lists and values.

pub mod node;
mod typ;
pub mod value;

use crate::{
	lang::LenientLanguageTag,
	syntax::Keyword,
	util::{AsJson, JsonFrom},
	Id, Indexed, Reference,
};
use generic_json::{JsonClone, JsonHash};
use iref::{Iri, IriBuf};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub use node::{Node, Nodes};
pub use typ::{Type, TypeRef};
pub use value::{Literal, LiteralString, Value};

pub trait Any<J: JsonHash, T: Id> {
	fn as_ref(&self) -> Ref<J, T>;

	#[inline]
	fn id<'a>(&'a self) -> Option<&Reference<T>>
	where
		J: 'a,
	{
		match self.as_ref() {
			Ref::Node(n) => n.id.as_ref(),
			_ => None,
		}
	}

	#[inline]
	fn language<'a>(&'a self) -> Option<LenientLanguageTag>
	where
		J: 'a,
		T: 'a,
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
pub enum Ref<'a, J: JsonHash, T: Id> {
	/// Value object.
	Value(&'a Value<J, T>),

	/// Node object.
	Node(&'a Node<J, T>),

	/// List object.
	List(&'a [Indexed<Object<J, T>>]),
}

/// Object.
///
/// JSON-LD connects together multiple kinds of data objects.
/// Objects may be nodes, values or lists of objects.
#[derive(PartialEq, Eq)]
pub enum Object<J: JsonHash, T: Id = IriBuf> {
	/// Value object.
	Value(Value<J, T>),

	/// Node object.
	Node(Node<J, T>),

	/// List object.
	List(Vec<Indexed<Self>>),
}

impl<J: JsonHash, T: Id> Object<J, T> {
	/// Identifier of the object, if it is a node object.
	#[inline(always)]
	pub fn id(&self) -> Option<&Reference<T>> {
		match self {
			Object::Node(n) => n.id.as_ref(),
			_ => None,
		}
	}

	pub fn types(&self) -> Types<T> {
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
	pub fn as_iri(&self) -> Option<Iri> {
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
	pub fn as_value(&self) -> Option<&Value<J, T>> {
		match self {
			Self::Value(v) => Some(v),
			_ => None,
		}
	}

	/// Converts this object as a value, if it is one.
	#[inline(always)]
	pub fn into_value(self) -> Option<Value<J, T>> {
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
	pub fn as_node(&self) -> Option<&Node<J, T>> {
		match self {
			Self::Node(n) => Some(n),
			_ => None,
		}
	}

	/// Converts this object into a node, if it is one.
	#[inline(always)]
	pub fn into_node(self) -> Option<Node<J, T>> {
		match self {
			Self::Node(n) => Some(n),
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
	pub fn as_list(&self) -> Option<&[Indexed<Self>]> {
		match self {
			Self::List(l) => Some(l.as_slice()),
			_ => None,
		}
	}

	/// Converts this object into a list, if it is one.
	#[inline(always)]
	pub fn into_list(self) -> Option<Vec<Indexed<Self>>> {
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
	pub fn as_number(&self) -> Option<&J::Number> {
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

	// pub fn blank_node_substitution(&self, other: &Self, initial_substitution: HashMap<BlankId, BlankId>) -> Option<HashMap<BlankId, BlankId>> {
	// 	match (self, other) {
	// 		(Self::Value(a), Self::Value(b)) if a == b => Some(initial_substitution),
	// 		(Self::Node(a), Self::Node(b)) => a.blank_node_substitution(b, initial_substitution),
	// 		(Self::List(a), Self::List(b)) => {
	// 			panic!("TODO")
	// 		},
	// 		_ => None
	// 	}
	// }

	/// Equivalence operator.
	///
	/// Equivalence is different from equality for anonymous objects:
	/// List objects and anonymous node objects have an implicit unlabeled blank nodes and thus never equivalent.
	pub fn equivalent(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Value(a), Self::Value(b)) => a == b,
			(Self::Node(a), Self::Node(b)) => a == b,
			_ => false,
		}
	}
}

impl<J: JsonHash, T: Id> Indexed<Object<J, T>> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}
}

impl<J: JsonHash, T: Id> Hash for Object<J, T> {
	#[inline]
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Self::Value(v) => v.hash(h),
			Self::Node(n) => n.hash(h),
			Self::List(l) => l.hash(h),
		}
	}
}

impl<J: JsonHash, T: Id> Indexed<Object<J, T>> {
	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_node(self) -> Option<Indexed<Node<J, T>>> {
		let (object, index) = self.into_parts();
		object.into_node().map(|node| Indexed::new(node, index))
	}

	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_value(self) -> Option<Indexed<Value<J, T>>> {
		let (object, index) = self.into_parts();
		object.into_value().map(|value| Indexed::new(value, index))
	}

	/// Converts this indexed object into an indexed list, if it is one.
	#[inline(always)]
	pub fn into_indexed_list(self) -> Option<Indexed<Vec<Self>>> {
		let (object, index) = self.into_parts();
		object.into_list().map(|list| Indexed::new(list, index))
	}

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self) -> Result<HashSet<Self>, Self> {
		let (obj, index) = self.into_parts();
		match obj {
			Object::Node(n) => match n.into_unnamed_graph() {
				Ok(g) => Ok(g),
				Err(n) => Err(Indexed::new(Object::Node(n), index)),
			},
			obj => Err(Indexed::new(obj, index)),
		}
	}
}

impl<J: JsonHash, T: Id> Any<J, T> for Object<J, T> {
	#[inline(always)]
	fn as_ref(&self) -> Ref<J, T> {
		match self {
			Object::Value(value) => Ref::Value(value),
			Object::Node(node) => Ref::Node(node),
			Object::List(list) => Ref::List(list.as_ref()),
		}
	}
}

impl<J: JsonHash, T: Id> From<Value<J, T>> for Object<J, T> {
	#[inline(always)]
	fn from(value: Value<J, T>) -> Self {
		Self::Value(value)
	}
}

impl<J: JsonHash, T: Id> From<Node<J, T>> for Object<J, T> {
	#[inline(always)]
	fn from(node: Node<J, T>) -> Self {
		Self::Node(node)
	}
}

impl<J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K> for Object<J, T> {
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		match self {
			Object::Value(v) => v.as_json_with(meta),
			Object::Node(n) => n.as_json_with(meta),
			Object::List(items) => {
				let mut obj = K::Object::default();
				obj.insert(
					K::new_key(Keyword::List.into_str(), meta(None)),
					items.as_json_with(meta.clone()),
				);
				K::object(obj, meta(None))
			}
		}
	}
}

impl<J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K>
	for HashSet<Indexed<Object<J, T>>>
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

/// Iterator through the types of an object.
pub enum Types<'a, T> {
	Value(Option<value::TypeRef<'a, T>>),
	Node(std::slice::Iter<'a, Reference<T>>),
	List,
}

impl<'a, T> Iterator for Types<'a, T> {
	type Item = TypeRef<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Value(ty) => ty.take().map(TypeRef::from_value_type),
			Self::Node(tys) => tys.next().map(TypeRef::from_reference),
			Self::List => None,
		}
	}
}

/// Iterator through indexed objects.
pub struct Objects<'a, J: JsonHash, T: Id>(Option<std::slice::Iter<'a, Indexed<Object<J, T>>>>);

impl<'a, J: JsonHash, T: Id> Objects<'a, J, T> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, Indexed<Object<J, T>>>>) -> Self {
		Self(inner)
	}
}

impl<'a, J: JsonHash, T: Id> Iterator for Objects<'a, J, T> {
	type Item = &'a Indexed<Object<J, T>>;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a Indexed<Object<J, T>>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}
