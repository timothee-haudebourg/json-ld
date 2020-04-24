pub mod value;
pub mod node;

use std::collections::HashSet;
use std::hash::Hash;
use std::fmt;
use iref::Iri;
use json::JsonValue;
use crate::{
	Id,
	Lenient,
	Reference,
	Indexed,
	syntax::Keyword,
	util::AsJson
};

pub use value::{
	Literal,
	Value
};
pub use node::Node;

/// Object descriptor.
#[derive(PartialEq, Eq, Hash)]
pub enum Object<T: Id> {
	/// Value object.
	Value(Value<T>),

	/// Node object.
	Node(Node<T>),

	/// List object.
	List(Vec<Indexed<Object<T>>>),
}

impl<T: Id> Object<T> {
	pub fn id(&self) -> Option<&Lenient<Reference<T>>> {
		match self {
			Object::Node(n) => n.id.as_ref(),
			_ => None
		}
	}

	pub fn is_value(&self) -> bool {
		match self {
			Object::Value(_) => true,
			_ => false
		}
	}

	pub fn is_node(&self) -> bool {
		match self {
			Object::Node(_) => true,
			_ => false
		}
	}

	pub fn is_graph(&self) -> bool {
		match self {
			Object::Node(n) => n.is_graph(),
			_ => false
		}
	}

	pub fn is_list(&self) -> bool {
		match self {
			Object::List(_) => true,
			_ => false
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Object::Value(value) => value.as_str(),
			Object::Node(node) => node.as_str(),
			_ => None
		}
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Object::Node(node) => node.as_iri(),
			_ => None
		}
	}

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self: Indexed<Self>) -> Result<HashSet<Indexed<Object<T>>>, Indexed<Self>> {
		let (obj, index) = self.into_parts();
		match obj {
			Object::Node(n) => {
				match n.into_unnamed_graph() {
					Ok(g) => Ok(g),
					Err(n) => Err(Indexed::new(Object::Node(n), index))
				}
			},
			obj => Err(Indexed::new(obj, index))
		}
	}
}

impl<T: Id> fmt::Debug for Object<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_json().pretty(2))
	}
}

impl<T: Id> From<Value<T>> for Object<T> {
	fn from(value: Value<T>) -> Object<T> {
		Object::Value(value)
	}
}

impl<T: Id> From<Node<T>> for Object<T> {
	fn from(node: Node<T>) -> Object<T> {
		Object::Node(node)
	}
}

impl<T: Id> AsJson for Object<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Object::Value(v) => v.as_json(),
			Object::Node(n) => n.as_json(),
			Object::List(items) => {
				let mut obj = json::object::Object::new();
				obj.insert(Keyword::List.into(), items.as_json());
				JsonValue::Object(obj)
			}
		}
	}
}
