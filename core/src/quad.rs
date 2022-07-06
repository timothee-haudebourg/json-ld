use crate::{
	flattening::NodeMap, object, ExpandedDocument, FlattenedDocument, Id, Indexed, Node, Object,
	Reference,
};
use smallvec::SmallVec;
use locspan::Stripped;

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
pub struct QuadRef<'a, T: Id, M>(
	pub Option<&'a Reference<T>>,
	pub &'a Reference<T>,
	pub PropertyRef<'a, T>,
	pub ObjectRef<'a, T, M>,
);

pub enum PropertyRef<'a, T: Id> {
	Type,
	Ref(&'a Reference<T>),
}

pub enum ObjectRef<'a, T: Id, M> {
	Object(&'a Object<T, M>),
	Node(&'a Node<T, M>),
	Ref(&'a Reference<T>),
}

impl<T: Id, M> ExpandedDocument<T, M> {
	pub fn quads(&self) -> Quads<T, M> {
		let mut stack = SmallVec::new();
		stack.push(QuadsFrame::IndexedObjectSet(None, self.iter()));
		Quads { stack }
	}
}

impl<T: Id, M> FlattenedDocument<T, M> {
	pub fn quads(&self) -> Quads<T, M> {
		let mut stack = SmallVec::new();
		stack.push(QuadsFrame::IndexedNodeSlice(None, self.iter()));
		Quads { stack }
	}
}

impl<T: Id, M> NodeMap<T, M> {
	pub fn quads(&self) -> Quads<T, M> {
		let mut stack = SmallVec::new();

		for (id, graph) in self {
			stack.push(QuadsFrame::NodeMapGraph(id, graph.nodes()));
		}

		Quads { stack }
	}
}

const STACK_LEN: usize = 6;

pub struct Quads<'a, T: Id, M> {
	stack: SmallVec<[QuadsFrame<'a, T, M>; STACK_LEN]>,
}

enum QuadsFrame<'a, T: Id, M> {
	NodeMapGraph(
		Option<&'a Reference<T>>,
		crate::flattening::NodeMapGraphNodes<'a, T, M>,
	),
	IndexedObjectSet(
		Option<&'a Reference<T>>,
		std::collections::hash_set::Iter<'a, Stripped<Indexed<Object<T, M>>>>,
	),
	IndexedNodeSet(
		Option<&'a Reference<T>>,
		std::collections::hash_set::Iter<'a, Stripped<Indexed<Node<T, M>>>>,
	),
	IndexedObjectSlice(
		Option<&'a Reference<T>>,
		std::slice::Iter<'a, Indexed<Object<T, M>>>,
	),
	IndexedNodeSlice(
		Option<&'a Reference<T>>,
		std::slice::Iter<'a, Indexed<Node<T, M>>>,
	),
	NodeTypes(
		Option<&'a Reference<T>>,
		&'a Reference<T>,
		std::slice::Iter<'a, Reference<T>>,
	),
	NodeProperties(
		Option<&'a Reference<T>>,
		&'a Reference<T>,
		object::node::properties::Iter<'a, T, M>,
	),
	NodeReverseProperties(
		Option<&'a Reference<T>>,
		&'a Node<T, M>,
		object::node::reverse_properties::Iter<'a, T, M>,
	),
	NodePropertyObjects(
		Option<&'a Reference<T>>,
		&'a Reference<T>,
		&'a Reference<T>,
		std::slice::Iter<'a, Indexed<Object<T, M>>>,
	),
	NodeReversePropertySubjects(
		Option<&'a Reference<T>>,
		&'a Node<T, M>,
		&'a Reference<T>,
		std::slice::Iter<'a, Indexed<Node<T, M>>>,
	),
}

impl<'a, T: Id, M> Quads<'a, T, M> {
	fn push_object(&mut self, graph: Option<&'a Reference<T>>, object: &'a Indexed<Object<T, M>>) {
		match object.inner() {
			Object::Node(node) => self.push_node(graph, node),
			Object::List(objects) => self
				.stack
				.push(QuadsFrame::IndexedObjectSlice(graph, objects.iter())),
			Object::Value(_) => (),
		}
	}

	fn push_node(&mut self, graph: Option<&'a Reference<T>>, node: &'a Node<T, M>) {
		if let Some(id) = node.id() {
			if let Some(graph) = node.graph() {
				self.stack
					.push(QuadsFrame::IndexedObjectSet(Some(id), graph.iter()))
			}

			if let Some(included) = node.included() {
				self.stack
					.push(QuadsFrame::IndexedNodeSet(graph, included.iter()))
			}

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

impl<'a, T: Id, M> Iterator for Quads<'a, T, M> {
	type Item = QuadRef<'a, T, M>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(last) = self.stack.last_mut() {
			match last {
				QuadsFrame::NodeMapGraph(graph, nodes) => {
					let graph = *graph;
					match nodes.next() {
						Some(node) => self.push_node(graph, node),
						None => {
							self.stack.pop();
						}
					}
				}
				QuadsFrame::IndexedObjectSet(graph, objects) => {
					let graph = *graph;
					match objects.next() {
						Some(object) => self.push_object(graph, object),
						None => {
							self.stack.pop();
						}
					}
				}
				QuadsFrame::IndexedNodeSet(graph, nodes) => {
					let graph = *graph;
					match nodes.next() {
						Some(node) => self.push_node(graph, node),
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
				QuadsFrame::IndexedNodeSlice(graph, nodes) => {
					let graph = *graph;
					match nodes.next() {
						Some(node) => self.push_node(graph, node),
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
								ObjectRef::<T, M>::Ref(ty),
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
