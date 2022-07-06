//! Nodes, lists and values.

mod mapped_eq;
pub mod node;
mod typ;
pub mod value;

use crate::{id, Id, Indexed, LenientLanguageTag, Reference};
use iref::{Iri, IriBuf};
use std::collections::HashSet;
use std::hash::Hash;
use locspan::{Stripped, BorrowStripped};
use locspan_derive::*;
use json_number::Number;

pub use mapped_eq::MappedEq;
pub use node::{Node, Nodes};
pub use typ::{Type, TypeRef};
pub use value::{Literal, LiteralString, Value};

pub trait Any<T: Id, M=()> {
	fn as_ref(&self) -> Ref<T, M>;

	#[inline]
	fn id<'a>(&'a self) -> Option<&'a Reference<T>> where M: 'a {
		match self.as_ref() {
			Ref::Node(n) => n.id.as_ref(),
			_ => None,
		}
	}

	#[inline]
	fn language<'a>(&'a self) -> Option<LenientLanguageTag>
	where
		T: 'a,
		M: 'a
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
pub enum Ref<'a, T: Id, M=()> {
	/// Value object.
	Value(&'a Value<T, M>),

	/// Node object.
	Node(&'a Node<T, M>),

	/// List object.
	List(&'a [Indexed<Object<T, M>>]),
}

/// Object.
///
/// JSON-LD connects together multiple kinds of data objects.
/// Objects may be nodes, values or lists of objects.
#[derive(PartialEq, Eq, Hash)]
#[derive(StrippedPartialEq, StrippedEq, StrippedHash)]
#[stripped_ignore(M)]
#[stripped(T)]
pub enum Object<T: Id = IriBuf, M=()> {
	/// Value object.
	Value(Value<T, M>),

	/// Node object.
	Node(Node<T, M>),

	/// List object.
	List(Vec<Indexed<Self>>),
}

impl<T: Id, M> Object<T, M> {
	/// Identifier of the object, if it is a node object.
	#[inline(always)]
	pub fn id(&self) -> Option<&Reference<T>> {
		match self {
			Object::Node(n) => n.id.as_ref(),
			_ => None,
		}
	}

	/// Assigns an identifier to every node included in this object using the given `generator`.
	pub fn identify_all<G: id::Generator<T>>(&mut self, generator: &mut G) {
		match self {
			Object::Node(n) => n.identify_all(generator),
			Object::List(l) => {
				for object in l {
					object.identify_all(generator)
				}
			}
			_ => (),
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
	pub fn as_node(&self) -> Option<&Node<T, M>> {
		match self {
			Self::Node(n) => Some(n),
			_ => None,
		}
	}

	/// Converts this object into a node, if it is one.
	#[inline(always)]
	pub fn into_node(self) -> Option<Node<T, M>> {
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

	pub fn traverse(&self) -> Traverse<T, M> {
		match self {
			Self::List(list) => Traverse::List {
				current: None,
				list: list.iter(),
			},
			Self::Value(value) => Traverse::Value(Some(value)),
			Self::Node(node) => Traverse::Node(Box::new(node.traverse())),
		}
	}

	/// Equivalence operator.
	///
	/// Equivalence is different from equality for anonymous objects:
	/// List objects and anonymous node objects have an implicit unlabeled blank nodes and thus never equivalent.
	pub fn equivalent(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Value(a), Self::Value(b)) => a.stripped() == b.stripped(),
			(Self::Node(a), Self::Node(b)) => a.equivalent(b),
			_ => false,
		}
	}
}

impl<T: Id, M> Indexed<Object<T, M>> {
	pub fn equivalent(&self, other: &Self) -> bool {
		self.index() == other.index() && self.inner().equivalent(other.inner())
	}
}

impl<T: Id, M> Indexed<Object<T, M>> {
	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_node(self) -> Option<Indexed<Node<T, M>>> {
		let (object, index) = self.into_parts();
		object.into_node().map(|node| Indexed::new(node, index))
	}

	/// Converts this indexed object into an indexed node, if it is one.
	#[inline(always)]
	pub fn into_indexed_value(self) -> Option<Indexed<Value<T, M>>> {
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
	pub fn into_unnamed_graph(self) -> Result<HashSet<Stripped<Self>>, Self> {
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

impl<T: Id, M> Any<T, M> for Object<T, M> {
	#[inline(always)]
	fn as_ref(&self) -> Ref<T, M> {
		match self {
			Object::Value(value) => Ref::Value(value),
			Object::Node(node) => Ref::Node(node),
			Object::List(list) => Ref::List(list.as_ref()),
		}
	}
}

impl<T: Id, M> From<Value<T, M>> for Object<T, M> {
	#[inline(always)]
	fn from(value: Value<T, M>) -> Self {
		Self::Value(value)
	}
}

impl<T: Id, M> From<Node<T, M>> for Object<T, M> {
	#[inline(always)]
	fn from(node: Node<T, M>) -> Self {
		Self::Node(node)
	}
}

// impl<J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K> for Object<T, M> {
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

// impl<J: JsonHash + JsonClone, K: JsonFrom<J>, T: Id> AsJson<J, K>
// 	for HashSet<Indexed<Object<T, M>>>
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
pub struct Objects<'a, T: Id, M>(Option<std::slice::Iter<'a, Indexed<Object<T, M>>>>);

impl<'a, T: Id, M> Objects<'a, T, M> {
	#[inline(always)]
	pub(crate) fn new(inner: Option<std::slice::Iter<'a, Indexed<Object<T, M>>>>) -> Self {
		Self(inner)
	}
}

impl<'a, T: Id, M> Iterator for Objects<'a, T, M> {
	type Item = &'a Indexed<Object<T, M>>;

	#[inline(always)]
	fn next(&mut self) -> Option<&'a Indexed<Object<T, M>>> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}

pub enum Traverse<'a, T: Id, M> {
	List {
		current: Option<Box<Traverse<'a, T, M>>>,
		list: std::slice::Iter<'a, Indexed<Object<T, M>>>,
	},
	Value(Option<&'a Value<T, M>>),
	Node(Box<node::Traverse<'a, T, M>>),
}

impl<'a, T: Id, M> Iterator for Traverse<'a, T, M> {
	type Item = Ref<'a, T, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::List { current, list } => loop {
				match current {
					Some(object) => match object.next() {
						Some(next) => break Some(next),
						None => *current = None,
					},
					None => match list.next() {
						Some(object) => *current = Some(Box::new(object.traverse())),
						None => break None,
					},
				}
			},
			Self::Value(value) => value.take().map(Ref::Value),
			Self::Node(node) => node.next(),
		}
	}
}
