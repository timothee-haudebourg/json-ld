use std::collections::{HashSet, hash_set};
use std::hash::{Hash, Hasher};
use std::fmt;
use iref::Iri;
use crate::{util, Id, Key, Value, Node, pp::PrettyPrint};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ObjectData {
	pub index: Option<String>
}

impl ObjectData {
	pub fn new() -> ObjectData {
		ObjectData {
			index: None
		}
	}

	pub fn is_empty(&self) -> bool {
		self.index.is_none()
	}
}

#[derive(PartialEq, Eq, Hash)]
pub enum Object<T: Id> {
	Value(Value<T>, ObjectData),
	Node(Node<T>, ObjectData)
}

impl<T: Id> Object<T> {
	pub fn id(&self) -> Option<&Key<T>> {
		match self {
			Object::Node(n, _) => n.id.as_ref(),
			_ => None
		}
	}

	pub fn data(&self) -> &ObjectData {
		match self {
			Object::Node(_, ref data) => data,
			Object::Value(_, ref data) => data
		}
	}

	pub fn data_mut(&mut self) -> &mut ObjectData {
		match self {
			Object::Node(_, ref mut data) => data,
			Object::Value(_, ref mut data) => data
		}
	}

	pub fn is_list(&self) -> bool {
		match self {
			Object::Value(v, _) => v.is_list(),
			_ => false
		}
	}

	pub fn is_graph(&self) -> bool {
		match self {
			Object::Node(n, _) => n.graph.is_some(),
			_ => false
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Object::Value(value, _) => value.as_str(),
			Object::Node(node, _) => node.as_str()
		}
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Object::Value(value, _) => value.as_iri(),
			Object::Node(node, _) => node.as_iri()
		}
	}

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self) -> Result<HashSet<Object<T>>, Self> {
		match self {
			Object::Value(v, data) => Err(Object::Value(v, data)),
			Object::Node(n, data) => {
				if data.is_empty() {
					match n.into_unnamed_graph() {
						Ok(graph) => Ok(graph),
						Err(n) => Err(Object::Node(n, data))
					}
				} else {
					Err(Object::Node(n, data))
				}
			}
		}
	}
}

impl<T: Id> fmt::Debug for Object<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", PrettyPrint(self))
	}
}

impl<T: Id> From<Value<T>> for Object<T> {
	fn from(value: Value<T>) -> Object<T> {
		Object::Value(value, ObjectData::new())
	}
}

impl<T: Id> From<Node<T>> for Object<T> {
	fn from(node: Node<T>) -> Object<T> {
		Object::Node(node, ObjectData::new())
	}
}
