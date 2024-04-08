use crate::{
	flattening::NodeMap, object, ExpandedDocument, FlattenedDocument, Id, Indexed, IndexedNode,
	IndexedObject, Node, Object,
};
use smallvec::SmallVec;
use std::hash::Hash;

/// JSON-LD Quad.
///
/// This is different from an RDF Quad since the object (last element) is a JSON-LD object.
/// A JSON-LD Quad can correspond to multiple RDF Quads.
pub struct QuadRef<'a, T, B>(
	pub Option<&'a Id<T, B>>,
	pub &'a Id<T, B>,
	pub PropertyRef<'a, T, B>,
	pub ObjectRef<'a, T, B>,
);

pub enum PropertyRef<'a, T, B> {
	Type,
	Ref(&'a Id<T, B>),
}

pub enum ObjectRef<'a, T, B> {
	Object(&'a Object<T, B>),
	Node(&'a Node<T, B>),
	Ref(&'a Id<T, B>),
}

pub trait LdQuads<T, B> {
	fn quads(&self) -> Quads<T, B>;
}

impl<T, B> LdQuads<T, B> for ExpandedDocument<T, B> {
	fn quads(&self) -> Quads<T, B> {
		let mut stack = SmallVec::new();
		stack.push(QuadsFrame::IndexedObjectSet(None, self.iter()));
		Quads { stack }
	}
}

impl<T, B> LdQuads<T, B> for FlattenedDocument<T, B> {
	fn quads(&self) -> Quads<T, B> {
		let mut stack = SmallVec::new();
		stack.push(QuadsFrame::IndexedNodeSlice(None, self.iter()));
		Quads { stack }
	}
}

impl<T: Eq + Hash, B: Eq + Hash> LdQuads<T, B> for NodeMap<T, B> {
	fn quads(&self) -> Quads<T, B> {
		let mut stack = SmallVec::new();

		for (id, graph) in self {
			stack.push(QuadsFrame::NodeMapGraph(id, graph.nodes()));
		}

		Quads { stack }
	}
}

const STACK_LEN: usize = 6;

pub struct Quads<'a, T, B> {
	stack: SmallVec<[QuadsFrame<'a, T, B>; STACK_LEN]>,
}

enum QuadsFrame<'a, T, B> {
	NodeMapGraph(
		Option<&'a Id<T, B>>,
		crate::flattening::NodeMapGraphNodes<'a, T, B>,
	),
	IndexedObjectSet(
		Option<&'a Id<T, B>>,
		indexmap::set::Iter<'a, IndexedObject<T, B>>,
	),
	IndexedNodeSet(
		Option<&'a Id<T, B>>,
		indexmap::set::Iter<'a, IndexedNode<T, B>>,
	),
	IndexedObjectSlice(
		Option<&'a Id<T, B>>,
		std::slice::Iter<'a, IndexedObject<T, B>>,
	),
	IndexedNodeSlice(
		Option<&'a Id<T, B>>,
		std::slice::Iter<'a, IndexedNode<T, B>>,
	),
	NodeTypes(
		Option<&'a Id<T, B>>,
		&'a Id<T, B>,
		std::slice::Iter<'a, Id<T, B>>,
	),
	NodeProperties(
		Option<&'a Id<T, B>>,
		&'a Id<T, B>,
		object::node::properties::Iter<'a, T, B>,
	),
	NodeReverseProperties(
		Option<&'a Id<T, B>>,
		&'a Node<T, B>,
		object::node::reverse_properties::Iter<'a, T, B>,
	),
	NodePropertyObjects(
		Option<&'a Id<T, B>>,
		&'a Id<T, B>,
		&'a Id<T, B>,
		std::slice::Iter<'a, IndexedObject<T, B>>,
	),
	NodeReversePropertySubjects(
		Option<&'a Id<T, B>>,
		&'a Node<T, B>,
		&'a Id<T, B>,
		std::slice::Iter<'a, IndexedNode<T, B>>,
	),
}

impl<'a, T, B> Quads<'a, T, B> {
	fn push_object(&mut self, graph: Option<&'a Id<T, B>>, object: &'a Indexed<Object<T, B>>) {
		match object.inner() {
			Object::Node(node) => self.push_node(graph, node),
			Object::List(objects) => self
				.stack
				.push(QuadsFrame::IndexedObjectSlice(graph, objects.iter())),
			Object::Value(_) => (),
		}
	}

	fn push_node(&mut self, graph: Option<&'a Id<T, B>>, node: &'a Node<T, B>) {
		if let Some(id) = &node.id {
			if let Some(graph) = node.graph_entry() {
				self.stack
					.push(QuadsFrame::IndexedObjectSet(Some(id), graph.iter()))
			}

			if let Some(included) = node.included_entry() {
				self.stack
					.push(QuadsFrame::IndexedNodeSet(graph, included.iter()))
			}

			if let Some(reverse_properties) = node.reverse_properties_entry() {
				self.stack.push(QuadsFrame::NodeReverseProperties(
					graph,
					node,
					reverse_properties.iter(),
				));
			}

			self.stack.push(QuadsFrame::NodeProperties(
				graph,
				id,
				node.properties().iter(),
			));

			if let Some(types) = &node.types {
				self.stack
					.push(QuadsFrame::NodeTypes(graph, id, types.iter()));
			}
		}
	}
}

impl<'a, T, B> Iterator for Quads<'a, T, B> {
	type Item = QuadRef<'a, T, B>;

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
								ObjectRef::<T, B>::Ref(ty),
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
							if let Some(id) = &subject.id {
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
