//! Nodes, lists and values.

pub mod node;
pub mod value;

use crate::{syntax::Keyword, util::AsJson, Id, Indexed, Reference};
use generic_json::Json;
use iref::{Iri, IriBuf};
use langtag::LanguageTag;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

pub use node::Node;
pub use value::{Literal, LiteralString, Value};

pub trait Any<J: Json, T: Id> {
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
	fn language<'a>(&'a self) -> Option<LanguageTag>
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
pub enum Ref<'a, J: Json, T: Id> {
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
pub enum Object<J: Json, T: Id = IriBuf> {
	/// Value object.
	Value(Value<J, T>),

	/// Node object.
	Node(Node<J, T>),

	/// List object.
	List(Vec<Indexed<Self>>),
}

impl<J: Json, T: Id> Object<J, T> {
	/// Identifier of the object, if it is a node object.
	pub fn id(&self) -> Option<&Reference<T>> {
		match self {
			Object::Node(n) => n.id.as_ref(),
			_ => None,
		}
	}

	/// Identifier of the object as an IRI.
	///
	/// If the object is a node identified by an IRI, returns this IRI.
	/// Returns `None` otherwise.
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Object::Node(node) => node.as_iri(),
			_ => None,
		}
	}

	/// Tests if the object is a value.
	pub fn is_value(&self) -> bool {
		matches!(self, Object::Value(_))
	}

	/// Tests if the object is a node.
	pub fn is_node(&self) -> bool {
		matches!(self, Object::Node(_))
	}

	/// Tests if the object is a graph object (a node with a `@graph` field).
	pub fn is_graph(&self) -> bool {
		match self {
			Object::Node(n) => n.is_graph(),
			_ => false,
		}
	}

	/// Tests if the object is a list.
	pub fn is_list(&self) -> bool {
		matches!(self, Object::List(_))
	}

	/// Get the object as a string.
	///
	/// If the object is a value that is a string, returns this string.
	/// If the object is a node that is identified, returns the identifier as a string.
	/// Returns `None` otherwise.
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Object::Value(value) => value.as_str(),
			Object::Node(node) => node.as_str(),
			_ => None,
		}
	}

	/// Get the value as a boolean, if it is.
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Object::Value(value) => value.as_bool(),
			_ => None,
		}
	}

	/// Get the value as a number, if it is.
	pub fn as_number(&self) -> Option<&J::Number> {
		match self {
			Object::Value(value) => value.as_number(),
			_ => None,
		}
	}

	/// If the objat is a language-tagged value,
	/// Return its associated language.
	pub fn language(&self) -> Option<LanguageTag> {
		match self {
			Object::Value(value) => value.language(),
			_ => None,
		}
	}
}

impl<J: Json, T: Id> Hash for Object<J, T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Self::Value(v) => v.hash(h),
			Self::Node(n) => n.hash(h),
			Self::List(l) => l.hash(h),
		}
	}
}

impl<J: Json, T: Id> Indexed<Object<J, T>> {
	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self: Indexed<Object<J, T>>) -> Result<HashSet<Self>, Self> {
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

impl<J: Json, T: Id> Any<J, T> for Object<J, T> {
	fn as_ref(&self) -> Ref<J, T> {
		match self {
			Object::Value(value) => Ref::Value(value),
			Object::Node(node) => Ref::Node(node),
			Object::List(list) => Ref::List(list.as_ref()),
		}
	}
}

// TODO
// impl<J: Json, T: Id> fmt::Debug for Object<J, T> {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		write!(f, "{}", self.as_json().pretty(2))
// 	}
// }

impl<J: Json, T: Id> From<Value<J, T>> for Object<J, T> {
	fn from(value: Value<J, T>) -> Self {
		Self::Value(value)
	}
}

impl<J: Json, T: Id> From<Node<J, T>> for Object<J, T> {
	fn from(node: Node<J, T>) -> Self {
		Self::Node(node)
	}
}

impl<J: Json, K: Json, T: Id> AsJson<K> for Object<J, T> {
	fn as_json_with<M>(&self, meta: M) -> K
	where
		M: Clone + Fn() -> K::MetaData,
	{
		// match self {
		// 	Object::Value(v) => v.as_json(),
		// 	Object::Node(n) => n.as_json(),
		// 	Object::List(items) => {
		// 		let mut obj = json::object::Object::new();
		// 		obj.insert(Keyword::List.into(), items.as_json());
		// 		JsonValue::Object(obj)
		// 	}
		// }
		panic!("TODO object as json")
	}
}
