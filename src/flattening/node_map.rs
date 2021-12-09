use crate::{id, ExpandedDocument, Id, Indexed, Node, Object, Reference};
use derivative::Derivative;
use generic_json::{JsonClone, JsonHash};
use std::collections::{HashMap, HashSet};
use super::Namespace;

#[derive(Clone)]
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct ConflictingIndexes<T: Id> {
	pub node_id: Reference<T>,
	pub defined_index: String,
	pub conflicting_index: String
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct NodeMap<J: JsonHash, T: Id> {
	graphs: HashMap<Reference<T>, NodeMapGraph<J, T>>,
	default_graph: NodeMapGraph<J, T>,
}

impl<J: JsonHash, T: Id> NodeMap<J, T> {
	pub fn new() -> Self {
		Self {
			graphs: HashMap::new(),
			default_graph: NodeMapGraph::new(),
		}
	}

	pub fn into_parts(self) -> (NodeMapGraph<J, T>, HashMap<Reference<T>, NodeMapGraph<J, T>>) {
		(self.default_graph, self.graphs)
	}

	pub fn graph(&self, id: Option<&Reference<T>>) -> Option<&NodeMapGraph<J, T>> {
		match id {
			Some(id) => self.graphs.get(id),
			None => Some(&self.default_graph),
		}
	}

	pub fn graph_mut(&mut self, id: Option<&Reference<T>>) -> Option<&mut NodeMapGraph<J, T>> {
		match id {
			Some(id) => self.graphs.get_mut(id),
			None => Some(&mut self.default_graph),
		}
	}

	/// Merge all the graphs into a single `NodeMapGraph`.
	///
	/// The order in which graphs are merged is not defined.
	pub fn merge(self) -> NodeMapGraph<J, T> {
		let mut result = self.default_graph;

		for (_, graph) in self.graphs {
			result.merge_with(graph)
		}

		result
	}

	pub fn iter(&self) -> Iter<J, T> {
		Iter {
			default_graph: Some(&self.default_graph),
			graphs: self.graphs.iter()
		}
	}

	pub fn iter_named(&self) -> std::collections::hash_map::Iter<Reference<T>, NodeMapGraph<J, T>> {
		self.graphs.iter()
	}
}

pub struct Iter<'a, J: JsonHash, T: Id> {
	default_graph: Option<&'a NodeMapGraph<J, T>>,
	graphs: std::collections::hash_map::Iter<'a, Reference<T>, NodeMapGraph<J, T>>
}

impl<'a, J: JsonHash, T: Id> Iterator for Iter<'a, J, T> {
	type Item = (Option<&'a Reference<T>>, &'a NodeMapGraph<J, T>);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph))
		}
	}
}

pub struct IntoIter<J: JsonHash, T: Id> {
	default_graph: Option<NodeMapGraph<J, T>>,
	graphs: std::collections::hash_map::IntoIter<Reference<T>, NodeMapGraph<J, T>>
}

impl<J: JsonHash, T: Id> Iterator for IntoIter<J, T> {
	type Item = (Option<Reference<T>>, NodeMapGraph<J, T>);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph))
		}
	}
}

impl<J: JsonHash, T: Id> IntoIterator for NodeMap<J, T> {
	type Item = (Option<Reference<T>>, NodeMapGraph<J, T>);
	type IntoIter = IntoIter<J, T>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			default_graph: Some(self.default_graph),
			graphs: self.graphs.into_iter()
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct NodeMapGraph<J: JsonHash, T: Id> {
	nodes: HashMap<Reference<T>, Indexed<Node<J, T>>>,
}

impl<J: JsonHash, T: Id> NodeMapGraph<J, T> {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
		}
	}

	pub fn contains(&self, id: &Reference<T>) -> bool {
		self.nodes.contains_key(id)
	}

	pub fn get(&self, id: &Reference<T>) -> Option<&Indexed<Node<J, T>>> {
		self.nodes.get(id)
	}

	pub fn get_mut(&mut self, id: &Reference<T>) -> Option<&mut Indexed<Node<J, T>>> {
		self.nodes.get_mut(id)
	}

	pub fn declare_node(
		&mut self,
		id: Reference<T>,
		index: Option<&str>,
	) -> Result<&mut Indexed<Node<J, T>>, ConflictingIndexes<T>> {
		if let Some(entry) = self.nodes.get_mut(&id) {
			match (entry.index(), index) {
				(Some(entry_index), Some(index)) => {
					if entry_index != index {
						return Err(ConflictingIndexes {
							node_id: id,
							defined_index: entry_index.to_string(),
							conflicting_index: index.to_string()
						});
					}
				}
				(None, Some(index)) => entry.set_index(Some(index.to_string())),
				_ => (),
			}
		} else {
			self.nodes.insert(
				id.clone(),
				Indexed::new(Node::with_id(id.clone()), index.map(|s| s.to_string())),
			);
		}

		Ok(self.nodes.get_mut(&id).unwrap())
	}

	/// Merge this graph with `other`.
	///
	/// This calls [`merge_node`](Self::merge_node) with every node of `other`.
	pub fn merge_with(&mut self, other: Self) {
		for (_, node) in other {
			self.merge_node(node)
		}
	}

	/// Merge the given `node` into the graph.
	///
	/// The `node` must has an identifier, or this function will have no effect.
	/// If there is already a node with the same identifier:
	/// - The index of `node`, if any, overrides the previously existing index.
	/// - The list of `node` types is concatenated after the preexisting types.
	/// - The graph and imported values are overridden.
	/// - Properties and reverse properties are merged.
	pub fn merge_node(&mut self, node: Indexed<Node<J, T>>) {
		let (node, index) = node.into_parts();
		let node = node.into_parts();

		if let Some(id) = &node.id {
			if let Some(entry) = self.nodes.get_mut(id) {
				if let Some(index) = index {
					entry.set_index(Some(index))
				}
			} else {
				self.nodes
					.insert(id.clone(), Indexed::new(Node::with_id(id.clone()), index));
			}

			let flat_node = self.nodes.get_mut(id).unwrap();
			flat_node.types_mut().extend(node.types.iter().cloned());
			flat_node.set_graph(node.graph);
			flat_node.set_included(node.included);
			flat_node.properties_mut().extend(node.properties);
			flat_node
				.reverse_properties_mut()
				.extend(node.reverse_properties);
		}
	}

	pub fn nodes(&self) -> impl Iterator<Item=&Indexed<Node<J, T>>> {
		self.nodes.values()
	}

	pub fn into_nodes(self) -> impl Iterator<Item=Indexed<Node<J, T>>> {
		self.nodes.into_values()
	}
}

impl<J: JsonHash, T: Id> IntoIterator for NodeMapGraph<J, T> {
	type Item = (Reference<T>, Indexed<Node<J, T>>);
	type IntoIter = std::collections::hash_map::IntoIter<Reference<T>, Indexed<Node<J, T>>>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a, J: JsonHash, T: Id> IntoIterator for &'a NodeMapGraph<J, T> {
	type Item = (&'a Reference<T>, &'a Indexed<Node<J, T>>);
	type IntoIter = std::collections::hash_map::Iter<'a, Reference<T>, Indexed<Node<J, T>>>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.iter()
	}
}

impl<J: JsonHash + JsonClone, T: Id> ExpandedDocument<J, T> {
	pub fn generate_node_map<G: id::Generator<T>>(
		&self,
		generator: G,
	) -> Result<NodeMap<J, T>, ConflictingIndexes<T>> {
		let mut node_map = NodeMap::new();
		let mut namespace: Namespace<T, G> = Namespace::new(generator);
		for object in self {
			extend_node_map(&mut namespace, &mut node_map, object, None)?;
		}
		Ok(node_map)
	}
}

/// Extends the `NodeMap` with the given `element` of an expanded JSON-LD document.
fn extend_node_map<J: JsonHash + JsonClone, T: Id, G: id::Generator<T>>(
	namespace: &mut Namespace<T, G>,
	node_map: &mut NodeMap<J, T>,
	element: &Indexed<Object<J, T>>,
	active_graph: Option<&Reference<T>>,
) -> Result<Indexed<Object<J, T>>, ConflictingIndexes<T>> {
	match element.inner() {
		Object::Value(value) => {
			let flat_value = value.clone();
			Ok(Indexed::new(
				Object::Value(flat_value),
				element.index().map(|s| s.to_string()),
			))
		}
		Object::List(list) => {
			let mut flat_list = Vec::new();

			for item in list {
				flat_list.push(extend_node_map(namespace, node_map, item, active_graph)?);
			}

			Ok(Indexed::new(
				Object::List(flat_list),
				element.index().map(|s| s.to_string()),
			))
		}
		Object::Node(node) => {
			let flat_node = extend_node_map_from_node(
				namespace,
				node_map,
				node,
				element.index(),
				active_graph,
			)?;
			Ok(flat_node.map_inner(Object::Node))
		}
	}
}

fn extend_node_map_from_node<J: JsonHash + JsonClone, T: Id, G: id::Generator<T>>(
	namespace: &mut Namespace<T, G>,
	node_map: &mut NodeMap<J, T>,
	node: &Node<J, T>,
	index: Option<&str>,
	active_graph: Option<&Reference<T>>,
) -> Result<Indexed<Node<J, T>>, ConflictingIndexes<T>> {
	let id = namespace.assign_node_id(node.id());

	{
		let flat_node = node_map
			.graph_mut(active_graph)
			.unwrap()
			.declare_node(id.clone(), index)?;
		flat_node.set_types(
			node.types()
				.iter()
				.map(|ty| namespace.assign_node_id(Some(ty)))
				.collect(),
		);
	}

	if let Some(graph) = node.graph() {
		let mut flat_graph = HashSet::new();
		for object in graph {
			let flat_object = extend_node_map(namespace, node_map, object, Some(&id))?;
			flat_graph.insert(flat_object);
		}

		let flat_node = node_map
			.graph_mut(active_graph)
			.unwrap()
			.get_mut(&id)
			.unwrap();
		match flat_node.graph_mut() {
			Some(graph) => graph.extend(flat_graph),
			None => flat_node.set_graph(Some(flat_graph)),
		}
	}

	if let Some(included) = node.included() {
		let mut flat_included = HashSet::new();
		for inode in included {
			let flat_inode = extend_node_map_from_node(
				namespace,
				node_map,
				inode.inner(),
				inode.index(),
				Some(&id),
			)?;
			flat_included.insert(flat_inode);
		}

		let flat_node = node_map
			.graph_mut(active_graph)
			.unwrap()
			.get_mut(&id)
			.unwrap();
		match flat_node.included_mut() {
			Some(nodes) => nodes.extend(flat_included),
			None => flat_node.set_included(Some(flat_included)),
		}
	}

	for (property, objects) in node.properties() {
		let mut flat_objects = Vec::new();
		for object in objects {
			let flat_object = extend_node_map(namespace, node_map, object, active_graph)?;
			flat_objects.push(flat_object);
		}
		node_map
			.graph_mut(active_graph)
			.unwrap()
			.get_mut(&id)
			.unwrap()
			.properties_mut()
			.insert_all(property.clone(), flat_objects)
	}

	for (property, nodes) in node.reverse_properties() {
		let mut flat_nodes = Vec::new();
		for node in nodes {
			let flat_node = extend_node_map_from_node(
				namespace,
				node_map,
				node.inner(),
				node.index(),
				active_graph,
			)?;
			flat_nodes.push(flat_node);
		}
		node_map
			.graph_mut(active_graph)
			.unwrap()
			.get_mut(&id)
			.unwrap()
			.reverse_properties_mut()
			.insert_all(property.clone(), flat_nodes)
	}

	Ok(Indexed::new(Node::with_id(id), None))
}
