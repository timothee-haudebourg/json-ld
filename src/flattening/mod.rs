//! Flattening algorithm and related types.
use std::marker::PhantomData;
use std::collections::{HashMap, HashSet};
use generic_json::{JsonHash, JsonClone};
use crate::{Id, BlankId, id, Reference, ExpandedDocument, Indexed, Object, Node};

pub struct Namespace<T, G> {
	id: PhantomData<T>,
	generator: G,
	map: HashMap<BlankId, Reference<T>>
}

impl<T, G> Namespace<T, G> {
	pub fn new(generator: G) -> Self {
		Self {
			id: PhantomData,
			generator,
			map: HashMap::new()
		}
	}
}

impl<T: Id, G: id::Generator<T>> Namespace<T, G> {
	fn assign(&mut self, blank_id: BlankId) -> Reference<T> {
		use std::collections::hash_map::Entry;
		match self.map.entry(blank_id) {
			Entry::Occupied(entry) => entry.get().clone(),
			Entry::Vacant(entry) => {
				let id = self.generator.next();
				entry.insert(id.clone());
				id
			}
		}
	}

	fn assign_node_id(&mut self, r: Option<&Reference<T>>) -> Reference<T> {
		match r {
			Some(Reference::Blank(id)) => self.assign(id.clone()),
			Some(r) => r.clone(),
			None => self.generator.next()
		}
	}
}

pub struct NodeMap<J: JsonHash, T: Id> {
	graphs: HashMap<Reference<T>, NodeMapGraph<J, T>>,
	default_graph: NodeMapGraph<J, T>
}

impl<J: JsonHash, T: Id> NodeMap<J, T> {
	pub fn new() -> Self {
		Self {
			graphs: HashMap::new(),
			default_graph: NodeMapGraph::new()
		}
	}

	pub fn graph_mut(&mut self, id: Option<&Reference<T>>) -> Option<&mut NodeMapGraph<J, T>> {
		match id {
			Some(id) => self.graphs.get_mut(id),
			None => Some(&mut self.default_graph)
		}
	}
}

pub struct NodeMapGraph<J: JsonHash, T: Id> {
	nodes: HashMap<Reference<T>, Indexed<Node<J, T>>>
}

impl<J: JsonHash, T: Id> NodeMapGraph<J, T> {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new()
		}
	}

	pub fn get_mut(&mut self, id: &Reference<T>) -> Option<&mut Indexed<Node<J, T>>> {
		self.nodes.get_mut(id)
	}

	pub fn declare_node(&mut self, id: Reference<T>, index: Option<&str>) -> Result<&mut Indexed<Node<J, T>>, ConflictingIndexes> {
		if let Some(entry) = self.nodes.get_mut(&id) {
			match (entry.index(), index) {
				(Some(entry_index), Some(index)) => {
					if entry_index != index {
						return Err(ConflictingIndexes)
					}
				},
				(None, Some(index)) => entry.set_index(Some(index.to_string())),
				_ => ()
			}
		} else {
			self.nodes.insert(id.clone(), Indexed::new(Node::with_id(id.clone()), index.map(|s| s.to_string())));
		}
		
		Ok(self.nodes.get_mut(&id).unwrap())
	}
}

pub struct ConflictingIndexes;

impl<J: JsonHash + JsonClone, T: Id> ExpandedDocument<J, T> {
	pub fn generate_node_map<G: id::Generator<T>>(&self, generator: G) -> Result<NodeMap<J, T>, ConflictingIndexes> {
		let mut node_map = NodeMap::new();
		let mut namespace: Namespace<T, G> = Namespace::new(generator);
		for object in self {
			extend_node_map(&mut namespace, &mut node_map, object, None)?;
		}
		Ok(node_map)
	}
}

/// Extends the `NodeMap` with the given `element` of an expanded JSON-LD document.
pub fn extend_node_map<J: JsonHash + JsonClone, T: Id, G: id::Generator<T>>(
	namespace: &mut Namespace<T, G>,
	node_map: &mut NodeMap<J, T>,
	element: &Indexed<Object<J, T>>,
	active_graph: Option<&Reference<T>>
) -> Result<Indexed<Object<J, T>>, ConflictingIndexes> {
	match element.inner() {
		Object::Value(value) => {
			let flat_value = value.clone();
			Ok(Indexed::new(Object::Value(flat_value), element.index().map(|s| s.to_string())))
		},
		Object::List(list) => {
			let mut flat_list = Vec::new();
			
			for item in list {
				flat_list.push(extend_node_map(namespace, node_map, item, active_graph)?);
			}

			Ok(Indexed::new(Object::List(flat_list), element.index().map(|s| s.to_string())))
		},
		Object::Node(node) => {
			let flat_node = extend_node_map_from_node(namespace, node_map, node, element.index(), active_graph)?;
			Ok(flat_node.map_inner(Object::Node))
		}
	}
}

pub fn extend_node_map_from_node<J: JsonHash + JsonClone, T: Id, G: id::Generator<T>>(
	namespace: &mut Namespace<T, G>,
	node_map: &mut NodeMap<J, T>,
	node: &Node<J, T>,
	index: Option<&str>,
	active_graph: Option<&Reference<T>>
) -> Result<Indexed<Node<J, T>>, ConflictingIndexes> {
	let id = namespace.assign_node_id(node.id());

	{
		let flat_node = node_map.graph_mut(active_graph).unwrap().declare_node(id.clone(), index)?;
		flat_node.set_types(node.types().iter().map(|ty| namespace.assign_node_id(Some(ty))).collect());
	}

	if let Some(graph) = node.graph() {
		let mut flat_graph = HashSet::new();
		for object in graph {
			let flat_object = extend_node_map(namespace, node_map, object, Some(&id))?;
			flat_graph.insert(flat_object);
		}
		
		let flat_node = node_map.graph_mut(active_graph).unwrap().get_mut(&id).unwrap();
		match flat_node.graph_mut() {
			Some(graph) => graph.extend(flat_graph),
			None => flat_node.set_graph(Some(flat_graph))
		}
	}

	if let Some(included) = node.included() {
		let mut flat_included = HashSet::new();
		for inode in included {
			let flat_inode = extend_node_map_from_node(namespace, node_map, inode.inner(), inode.index(), Some(&id))?;
			flat_included.insert(flat_inode);
		}
		
		let flat_node = node_map.graph_mut(active_graph).unwrap().get_mut(&id).unwrap();
		match flat_node.included_mut() {
			Some(nodes) => nodes.extend(flat_included),
			None => flat_node.set_included(Some(flat_included))
		}
	}

	for (property, objects) in node.properties() {
		let mut flat_objects = Vec::new();
		for object in objects {
			let flat_object = extend_node_map(namespace, node_map, object, active_graph)?;
			flat_objects.push(flat_object);
		}
		node_map.graph_mut(active_graph).unwrap().get_mut(&id).unwrap().properties_mut().insert_all(property.clone(), flat_objects)
	}

	for (property, nodes) in node.reverse_properties() {
		let mut flat_nodes = Vec::new();
		for node in nodes {
			let flat_node = extend_node_map_from_node(namespace, node_map, node.inner(), node.index(), active_graph)?;
			flat_nodes.push(flat_node);
		}
		node_map.graph_mut(active_graph).unwrap().get_mut(&id).unwrap().reverse_properties_mut().insert_all(property.clone(), flat_nodes)
	}

	Ok(Indexed::new(Node::with_id(id), None))
}