use crate::{object, ExpandedDocument, Id, Indexed, Node, Object, Reference};
use generic_json::JsonHash;
use smallvec::SmallVec;

/// JSON-LD Quad.
///
/// This is different from an RDF Quad since the object (last element) is a JSON-LD object.
/// A JSON-LD Quad can correspond to multiple RDF Quads.
///
/// ## Example
///
/// ```
/// # #[async_std::main]
/// # async fn main() {
/// # use serde_json::Value;
/// # use json_ld::{context, Document, NoLoader};
/// # let doc: Value = serde_json::from_str(r#"{"@context": {"name": "http://xmlns.com/foaf/0.1/name"},"@id": "https://www.rust-lang.org","name": "Rust Programming Language"}"#,).unwrap();
/// # let mut loader = NoLoader::<Value>::new();
/// # let expanded_doc = doc.expand::<context::Json<Value>, _>(&mut loader).await.unwrap();
/// for json_ld::quad::QuadRef(graph, subject, property, object) in expanded_doc.quads() {
///   // ...
/// }
/// # }
/// ```
pub struct QuadRef<'a, J: JsonHash, T: Id>(
	pub Option<&'a Reference<T>>,
	pub &'a Reference<T>,
	pub PropertyRef<'a, T>,
	pub ObjectRef<'a, J, T>,
);

pub enum PropertyRef<'a, T: Id> {
	Type,
	Ref(&'a Reference<T>),
}

pub enum ObjectRef<'a, J: JsonHash, T: Id> {
	Object(&'a Object<J, T>),
	Node(&'a Node<J, T>),
	Ref(&'a Reference<T>),
}

impl<J: JsonHash, T: Id> ExpandedDocument<J, T> {
	pub fn quads(&self) -> Quads<J, T> {
		let mut stack = SmallVec::new();
		stack.push(QuadsFrame::IndexedObjectSet(None, self.iter()));
		Quads { stack }
	}
}

const STACK_LEN: usize = 6;

pub struct Quads<'a, J: JsonHash, T: Id> {
	stack: SmallVec<[QuadsFrame<'a, J, T>; STACK_LEN]>,
}

enum QuadsFrame<'a, J: JsonHash, T: Id> {
	IndexedObjectSet(
		Option<&'a Reference<T>>,
		std::collections::hash_set::Iter<'a, Indexed<Object<J, T>>>,
	),
	IndexedObjectSlice(
		Option<&'a Reference<T>>,
		std::slice::Iter<'a, Indexed<Object<J, T>>>,
	),

	NodeTypes(
		Option<&'a Reference<T>>,
		&'a Reference<T>,
		std::slice::Iter<'a, Reference<T>>,
	),
	NodeProperties(
		Option<&'a Reference<T>>,
		&'a Reference<T>,
		object::node::properties::Iter<'a, J, T>,
	),
	NodeReverseProperties(
		Option<&'a Reference<T>>,
		&'a Node<J, T>,
		object::node::reverse_properties::Iter<'a, J, T>,
	),

	NodePropertyObjects(
		Option<&'a Reference<T>>,
		&'a Reference<T>,
		&'a Reference<T>,
		std::slice::Iter<'a, Indexed<Object<J, T>>>,
	),
	NodeReversePropertySubjects(
		Option<&'a Reference<T>>,
		&'a Node<J, T>,
		&'a Reference<T>,
		std::slice::Iter<'a, Indexed<Node<J, T>>>,
	),
}

impl<'a, J: JsonHash, T: Id> Quads<'a, J, T> {
	fn push_object(&mut self, graph: Option<&'a Reference<T>>, object: &'a Indexed<Object<J, T>>) {
		match object.inner() {
			Object::Node(node) => self.push_node(graph, node),
			Object::List(objects) => self
				.stack
				.push(QuadsFrame::IndexedObjectSlice(graph, objects.iter())),
			Object::Value(_) => (),
		}
	}

	fn push_node(&mut self, graph: Option<&'a Reference<T>>, node: &'a Node<J, T>) {
		if let Some(id) = node.id() {
			self.stack.push(QuadsFrame::NodeReverseProperties(
				graph,
				node,
				node.reverse_properties().iter(),
			));
			self.stack.push(QuadsFrame::NodeProperties(
				graph,
				id,
				node.properties().iter(),
			));
			self.stack
				.push(QuadsFrame::NodeTypes(graph, id, node.types().iter()));
		}
	}
}

impl<'a, J: JsonHash, T: Id> Iterator for Quads<'a, J, T> {
	type Item = QuadRef<'a, J, T>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(last) = self.stack.last_mut() {
			match last {
				QuadsFrame::IndexedObjectSet(graph, objects) => {
					let graph = *graph;
					match objects.next() {
						Some(object) => self.push_object(graph, object),
						None => {
							self.stack.pop();
						}
					}
				}
				QuadsFrame::IndexedObjectSlice(graph, objects) => {
					let graph = *graph;
					match objects.next() {
						Some(object) => self.push_object(graph, object),
						None => {
							self.stack.pop();
						}
					}
				}
				QuadsFrame::NodeTypes(graph, subject, types) => {
					let (graph, subject) = (*graph, *subject);
					match types.next() {
						Some(ty) => {
							return Some(QuadRef(
								graph,
								subject,
								PropertyRef::Type,
								ObjectRef::Ref(ty),
							))
						}
						None => {
							self.stack.pop();
						}
					}
				}
				QuadsFrame::NodeProperties(graph, subject, properties) => {
					let (graph, subject) = (*graph, *subject);
					match properties.next() {
						Some((property, objects)) => {
							self.stack.push(QuadsFrame::NodePropertyObjects(
								graph,
								subject,
								property,
								objects.iter(),
							))
						}
						None => {
							self.stack.pop();
						}
					}
				}
				QuadsFrame::NodeReverseProperties(graph, object, reverse_properties) => {
					let (graph, object) = (*graph, *object);
					match reverse_properties.next() {
						Some((property, subjects)) => {
							self.stack.push(QuadsFrame::NodeReversePropertySubjects(
								graph,
								object,
								property,
								subjects.iter(),
							))
						}
						None => {
							self.stack.pop();
						}
					}
				}
				QuadsFrame::NodePropertyObjects(graph, subject, property, objects) => {
					let (graph, subject, property) = (*graph, *subject, *property);
					match objects.next() {
						Some(object) => {
							self.push_object(graph, object);
							return Some(QuadRef(
								graph,
								subject,
								PropertyRef::Ref(property),
								ObjectRef::Object(object),
							));
						}
						None => {
							self.stack.pop();
						}
					}
				}
				QuadsFrame::NodeReversePropertySubjects(graph, object, property, subjects) => {
					let (graph, object, property) = (*graph, *object, *property);
					match subjects.next() {
						Some(subject) => {
							self.push_node(graph, subject.inner());
							if let Some(id) = subject.id() {
								return Some(QuadRef(
									graph,
									id,
									PropertyRef::Ref(property),
									ObjectRef::Node(object),
								));
							}
						}
						None => {
							self.stack.pop();
						}
					}
				}
			}
		}

		None
	}
}
