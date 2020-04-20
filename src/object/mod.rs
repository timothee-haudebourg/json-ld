pub mod value;
pub mod node;
pub mod graph;

use std::collections::HashSet;
use std::hash::Hash;
use std::fmt;
use iref::Iri;
use json::JsonValue;
use crate::{
	Id,
	Term,
	Keyword,
	Indexed,
	util::AsJson
};

pub use value::Value;
pub use node::Node;
pub use graph::Graph;

/// Object descriptor.
#[derive(PartialEq, Eq, Hash)]
pub enum Object<T: Id> {
	/// Value object.
	Value(Value<T>),

	/// Node object.
	Node(Node<T>),

	/// List object.
	List(Vec<Indexed<Object<T>>>),

	/// Graph object.
	Graph(Graph<T>)
}

impl<T: Id> Object<T> {
	pub fn id(&self) -> Option<&Term<T>> {
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

	pub fn is_list(&self) -> bool {
		match self {
			Object::List(_) => true,
			_ => false
		}
	}

	pub fn is_graph(&self) -> bool {
		match self {
			Object::Graph(_) => true,
			_ => false
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Object::Value(value) => value.as_str(),
			Object::Node(node) => node.as_str(),
			Object::Graph(graph) => graph.as_str(),
			_ => None
		}
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Object::Value(value) => value.as_iri(),
			Object::Node(node) => node.as_iri(),
			Object::Graph(graph) => graph.as_iri(),
			_ => None
		}
	}

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self) -> Result<Graph<T>, Self> {
		match self {
			Object::Graph(g) if !g.is_named() => Ok(g),
			object => Err(object)
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
			},
			Object::Graph(g) => g.as_json()
		}
	}
}
