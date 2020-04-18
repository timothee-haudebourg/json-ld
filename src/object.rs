use std::collections::HashSet;
use std::hash::Hash;
use std::fmt;
use iref::Iri;
use json::JsonValue;
use crate::{Id, Key, Keyword, Value, Node, Literal, pp::PrettyPrint, AsJson};

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

	fn add_to_json(&self, obj: &mut json::object::Object) {
		if let Some(index) = &self.index {
			obj.insert(Keyword::Index.into(), index.as_json())
		}
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

impl<T: Id> AsJson for Object<T> {
	fn as_json(&self) -> JsonValue {
		let mut obj = json::object::Object::new();

		match self {
			Object::Value(value, data) => {
				match value {
					Value::Literal(ref lit, ref types) => {
						match lit {
							Literal::Null => {
								obj.insert(Keyword::Value.into(), JsonValue::Null)
							},
							Literal::Boolean(b) => {
								obj.insert(Keyword::Value.into(), b.as_json())
							},
							Literal::Number(n) => {
								obj.insert(Keyword::Value.into(), JsonValue::Number(n.clone()))
							},
							Literal::String { data, language, direction } => {
								obj.insert(Keyword::Value.into(), data.as_json());

								if let Some(language) = language {
									obj.insert(Keyword::Language.into(), language.as_json())
								}

								if let Some(direction) = direction {
									obj.insert(Keyword::Direction.into(), direction.as_json())
								}
							},
							Literal::Json(json) => {
								obj.insert(Keyword::Value.into(), json.clone())
							},
							Literal::Ref(id) => {
								obj.insert(Keyword::Value.into(), id.as_json())
							}
						}

						if !types.is_empty() {
							if types.len() == 1 {
								obj.insert(Keyword::Type.into(), types.iter().next().unwrap().as_json())
							} else {
								obj.insert(Keyword::Type.into(), types.as_json())
							}
						}
					},
					Value::List(items) => {
						obj.insert(Keyword::List.into(), items.as_json())
					}
				}

				data.add_to_json(&mut obj);
			},
			Object::Node(node, data) => {
				if let Some(id) = &node.id {
					obj.insert(Keyword::Id.into(), id.as_json())
				}

				if !node.types.is_empty() {
					obj.insert(Keyword::Type.into(), node.types.as_json())
				}

				if let Some(graph) = &node.graph {
					obj.insert(Keyword::Graph.into(), graph.as_json())
				}

				if let Some(included) = &node.included {
					obj.insert(Keyword::Included.into(), included.as_json())
				}

				data.add_to_json(&mut obj);

				if !node.reverse_properties.is_empty() {
					let mut reverse = json::object::Object::new();
					for (key, value) in &node.reverse_properties {
						reverse.insert(key.as_str(), value.as_json())
					}

					obj.insert(Keyword::Reverse.into(), JsonValue::Object(reverse))
				}

				for (key, value) in &node.properties {
					obj.insert(key.as_str(), value.as_json())
				}
			}
		}

		JsonValue::Object(obj)
	}
}
