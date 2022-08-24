use crate::{
	flattening::NodeMap, object, ExpandedDocument, FlattenedDocument, Indexed, Node, Object,
	Reference, StrippedIndexedNode, StrippedIndexedObject,
};
use locspan::Meta;
use smallvec::SmallVec;
use std::hash::Hash;

/// JSON-LD Quad.
///
/// This is different from an RDF Quad since the object (last element) is a JSON-LD object.
/// A JSON-LD Quad can correspond to multiple RDF Quads.
pub struct QuadRef<'a, T, B, M>(
	pub Option<&'a Reference<T, B>>,
	pub &'a Reference<T, B>,
	pub PropertyRef<'a, T, B>,
	pub ObjectRef<'a, T, B, M>,
);

pub enum PropertyRef<'a, T, B> {
	Type,
	Ref(&'a Reference<T, B>),
}

pub enum ObjectRef<'a, T, B, M> {
	Object(&'a Object<T, B, M>),
	Node(&'a Node<T, B, M>),
	Ref(&'a Reference<T, B>),
}

impl<T, B, M> ExpandedDocument<T, B, M> {
	pub fn quads(&self) -> Quads<T, B, M> {
		let mut stack = SmallVec::new();
		stack.push(QuadsFrame::IndexedObjectSet(None, self.iter()));
		Quads { stack }
	}
}

impl<T, B, M> FlattenedDocument<T, B, M> {
	pub fn quads(&self) -> Quads<T, B, M> {
		let mut stack = SmallVec::new();
		stack.push(QuadsFrame::IndexedNodeSlice(None, self.iter()));
		Quads { stack }
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> NodeMap<T, B, M> {
	pub fn quads(&self) -> Quads<T, B, M> {
		let mut stack = SmallVec::new();

		for (id, graph) in self {
			stack.push(QuadsFrame::NodeMapGraph(id, graph.nodes()));
		}

		Quads { stack }
	}
}

const STACK_LEN: usize = 6;

pub struct Quads<'a, T, B, M> {
	stack: SmallVec<[QuadsFrame<'a, T, B, M>; STACK_LEN]>,
}

enum QuadsFrame<'a, T, B, M> {
	NodeMapGraph(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		crate::flattening::NodeMapGraphNodes<'a, T, B, M>,
	),
	IndexedObjectSet(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		std::collections::hash_set::Iter<'a, StrippedIndexedObject<T, B, M>>,
	),
	IndexedNodeSet(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		std::collections::hash_set::Iter<'a, StrippedIndexedNode<T, B, M>>,
	),
	IndexedObjectSlice(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		std::slice::Iter<'a, Meta<Indexed<Object<T, B, M>>, M>>,
	),
	IndexedNodeSlice(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		std::slice::Iter<'a, Meta<Indexed<Node<T, B, M>>, M>>,
	),
	NodeTypes(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		&'a Reference<T, B>,
		std::slice::Iter<'a, Meta<Reference<T, B>, M>>,
	),
	NodeProperties(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		&'a Reference<T, B>,
		object::node::properties::Iter<'a, T, B, M>,
	),
	NodeReverseProperties(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		&'a Node<T, B, M>,
		object::node::reverse_properties::Iter<'a, T, B, M>,
	),
	NodePropertyObjects(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		&'a Reference<T, B>,
		&'a Reference<T, B>,
		std::slice::Iter<'a, StrippedIndexedObject<T, B, M>>,
	),
	NodeReversePropertySubjects(
		Option<Meta<&'a Reference<T, B>, &'a M>>,
		&'a Node<T, B, M>,
		&'a Reference<T, B>,
		std::slice::Iter<'a, StrippedIndexedNode<T, B, M>>,
	),
}

impl<'a, T, B, M> Quads<'a, T, B, M> {
	fn push_object(
		&mut self,
		graph: Option<Meta<&'a Reference<T, B>, &'a M>>,
		object: &'a Indexed<Object<T, B, M>>,
	) {
		match object.inner() {
			Object::Node(node) => self.push_node(graph, node),
			Object::List(objects) => self
				.stack
				.push(QuadsFrame::IndexedObjectSlice(graph, objects.iter())),
			Object::Value(_) => (),
		}
	}

	fn push_node(
		&mut self,
		graph: Option<Meta<&'a Reference<T, B>, &'a M>>,
		node: &'a Node<T, B, M>,
	) {
		if let Some(id) = node.id_entry() {
			if let Some(graph) = node.graph_entry() {
				self.stack.push(QuadsFrame::IndexedObjectSet(
					Some(id.value.borrow()),
					graph.iter(),
				))
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

			if let Some(types) = node.type_entry() {
				self.stack
					.push(QuadsFrame::NodeTypes(graph, id, types.iter()));
			}
		}
	}
}

impl<'a, T, B, M> Iterator for Quads<'a, T, B, M> {
	type Item = QuadRef<'a, T, B, M>;

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
								graph.map(Meta::into_value),
								subject,
								PropertyRef::Type,
								ObjectRef::<T, B, M>::Ref(ty),
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
								graph.map(Meta::into_value),
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
							if let Some(id) = subject.id_entry() {
								return Some(QuadRef(
									graph.map(Meta::into_value),
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
