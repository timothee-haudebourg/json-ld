use super::Environment;
use crate::{object, ExpandedDocument, Id, Indexed, IndexedNode, IndexedObject, Node, Object};
use educe::Educe;
use indexmap::IndexSet;
use rdf_types::{
	vocabulary::{BlankIdVocabulary, IriVocabulary},
	Generator, Vocabulary,
};
use std::collections::HashMap;
use std::hash::Hash;

/// Conflicting indexes error.
///
/// Raised when a single node is declared with two different indexes.
#[derive(Clone, Debug, thiserror::Error)]
#[error("Index `{defined_index}` conflicts with index `{conflicting_index}`")]
pub struct ConflictingIndexes<T, B> {
	pub node_id: Id<T, B>,
	pub defined_index: String,
	pub conflicting_index: String,
}

pub type Parts<T, B> = (NodeMapGraph<T, B>, HashMap<Id<T, B>, NodeMapGraph<T, B>>);

/// Node identifier to node definition map.
#[derive(Educe)]
#[educe(Default)]
pub struct NodeMap<T, B> {
	graphs: HashMap<Id<T, B>, NodeMapGraph<T, B>>,
	default_graph: NodeMapGraph<T, B>,
}

impl<T, B> NodeMap<T, B> {
	pub fn new() -> Self {
		Self {
			graphs: HashMap::new(),
			default_graph: NodeMapGraph::new(),
		}
	}

	pub fn into_parts(self) -> Parts<T, B> {
		(self.default_graph, self.graphs)
	}

	pub fn iter(&self) -> Iter<T, B> {
		Iter {
			default_graph: Some(&self.default_graph),
			graphs: self.graphs.iter(),
		}
	}

	pub fn iter_named(&self) -> std::collections::hash_map::Iter<Id<T, B>, NodeMapGraph<T, B>> {
		self.graphs.iter()
	}
}

impl<T: Eq + Hash, B: Eq + Hash> NodeMap<T, B> {
	pub fn graph(&self, id: Option<&Id<T, B>>) -> Option<&NodeMapGraph<T, B>> {
		match id {
			Some(id) => self.graphs.get(id),
			None => Some(&self.default_graph),
		}
	}

	pub fn graph_mut(&mut self, id: Option<&Id<T, B>>) -> Option<&mut NodeMapGraph<T, B>> {
		match id {
			Some(id) => self.graphs.get_mut(id),
			None => Some(&mut self.default_graph),
		}
	}

	pub fn declare_graph(&mut self, id: Id<T, B>) {
		if let std::collections::hash_map::Entry::Vacant(entry) = self.graphs.entry(id) {
			entry.insert(NodeMapGraph::new());
		}
	}

	/// Merge all the graphs into a single `NodeMapGraph`.
	///
	/// The order in which graphs are merged is not defined.
	pub fn merge(self) -> NodeMapGraph<T, B>
	where
		T: Clone,
		B: Clone,
	{
		let mut result = self.default_graph;

		for (_, graph) in self.graphs {
			result.merge_with(graph)
		}

		result
	}
}

pub struct Iter<'a, T, B> {
	default_graph: Option<&'a NodeMapGraph<T, B>>,
	graphs: std::collections::hash_map::Iter<'a, Id<T, B>, NodeMapGraph<T, B>>,
}

impl<'a, T, B> Iterator for Iter<'a, T, B> {
	type Item = (Option<&'a Id<T, B>>, &'a NodeMapGraph<T, B>);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph)),
		}
	}
}

impl<'a, T, B> IntoIterator for &'a NodeMap<T, B> {
	type Item = (Option<&'a Id<T, B>>, &'a NodeMapGraph<T, B>);
	type IntoIter = Iter<'a, T, B>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter<T, B> {
	default_graph: Option<NodeMapGraph<T, B>>,
	graphs: std::collections::hash_map::IntoIter<Id<T, B>, NodeMapGraph<T, B>>,
}

impl<T, B> Iterator for IntoIter<T, B> {
	type Item = (Option<Id<T, B>>, NodeMapGraph<T, B>);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph)),
		}
	}
}

impl<T, B> IntoIterator for NodeMap<T, B> {
	type Item = (Option<Id<T, B>>, NodeMapGraph<T, B>);
	type IntoIter = IntoIter<T, B>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			default_graph: Some(self.default_graph),
			graphs: self.graphs.into_iter(),
		}
	}
}

#[derive(Educe)]
#[educe(Default)]
pub struct NodeMapGraph<T, B> {
	nodes: HashMap<Id<T, B>, IndexedNode<T, B>>,
}

impl<T, B> NodeMapGraph<T, B> {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
		}
	}
}

pub type DeclareNodeResult<'a, T, B> =
	Result<&'a mut Indexed<Node<T, B>>, ConflictingIndexes<T, B>>;

impl<T: Eq + Hash, B: Eq + Hash> NodeMapGraph<T, B> {
	pub fn contains(&self, id: &Id<T, B>) -> bool {
		self.nodes.contains_key(id)
	}

	pub fn get(&self, id: &Id<T, B>) -> Option<&IndexedNode<T, B>> {
		self.nodes.get(id)
	}

	pub fn get_mut(&mut self, id: &Id<T, B>) -> Option<&mut IndexedNode<T, B>> {
		self.nodes.get_mut(id)
	}

	pub fn declare_node(&mut self, id: Id<T, B>, index: Option<&str>) -> DeclareNodeResult<T, B>
	where
		T: Clone,
		B: Clone,
	{
		if let Some(entry) = self.nodes.get_mut(&id) {
			match (entry.index(), index) {
				(Some(entry_index), Some(index)) => {
					if entry_index != index {
						return Err(ConflictingIndexes {
							node_id: id,
							defined_index: entry_index.to_string(),
							conflicting_index: index.to_string(),
						});
					}
				}
				(None, Some(index)) => entry.set_index(Some(index.to_owned())),
				_ => (),
			}
		} else {
			self.nodes.insert(
				id.clone(),
				Indexed::new(Node::with_id(id.clone()), index.map(ToOwned::to_owned)),
			);
		}

		Ok(self.nodes.get_mut(&id).unwrap())
	}

	/// Merge this graph with `other`.
	///
	/// This calls [`merge_node`](Self::merge_node) with every node of `other`.
	pub fn merge_with(&mut self, other: Self)
	where
		T: Clone,
		B: Clone,
	{
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
	pub fn merge_node(&mut self, node: IndexedNode<T, B>)
	where
		T: Clone,
		B: Clone,
	{
		let (node, index) = node.into_parts();

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

			if let Some(types) = node.types {
				flat_node.types_mut_or_default().extend(types);
			}

			flat_node.set_graph_entry(node.graph);
			flat_node.set_included(node.included);
			flat_node.properties_mut().extend_unique(node.properties);

			if let Some(props) = node.reverse_properties {
				flat_node
					.reverse_properties_or_default()
					.extend_unique(props);
			}
		}
	}

	pub fn nodes(&self) -> NodeMapGraphNodes<T, B> {
		self.nodes.values()
	}

	pub fn into_nodes(self) -> IntoNodeMapGraphNodes<T, B> {
		self.nodes.into_values()
	}
}

pub type NodeMapGraphNodes<'a, T, B> =
	std::collections::hash_map::Values<'a, Id<T, B>, IndexedNode<T, B>>;
pub type IntoNodeMapGraphNodes<T, B> =
	std::collections::hash_map::IntoValues<Id<T, B>, IndexedNode<T, B>>;

impl<T, B> IntoIterator for NodeMapGraph<T, B> {
	type Item = (Id<T, B>, IndexedNode<T, B>);
	type IntoIter = std::collections::hash_map::IntoIter<Id<T, B>, IndexedNode<T, B>>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a, T, B> IntoIterator for &'a NodeMapGraph<T, B> {
	type Item = (&'a Id<T, B>, &'a IndexedNode<T, B>);
	type IntoIter = std::collections::hash_map::Iter<'a, Id<T, B>, IndexedNode<T, B>>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.iter()
	}
}

impl<T: Clone + Eq + Hash, B: Clone + Eq + Hash> ExpandedDocument<T, B> {
	pub fn generate_node_map_with<V: Vocabulary<Iri = T, BlankId = B>, G: Generator<V>>(
		&self,
		vocabulary: &mut V,
		generator: G,
	) -> Result<NodeMap<T, B>, ConflictingIndexes<T, B>> {
		let mut node_map: NodeMap<T, B> = NodeMap::new();
		let mut env: Environment<V, G> = Environment::new(vocabulary, generator);
		for object in self {
			extend_node_map(&mut env, &mut node_map, object, None)?;
		}
		Ok(node_map)
	}
}

pub type ExtendNodeMapResult<V> = Result<
	IndexedObject<<V as IriVocabulary>::Iri, <V as BlankIdVocabulary>::BlankId>,
	ConflictingIndexes<<V as IriVocabulary>::Iri, <V as BlankIdVocabulary>::BlankId>,
>;

/// Extends the `NodeMap` with the given `element` of an expanded JSON-LD document.
fn extend_node_map<N: Vocabulary, G: Generator<N>>(
	env: &mut Environment<N, G>,
	node_map: &mut NodeMap<N::Iri, N::BlankId>,
	element: &IndexedObject<N::Iri, N::BlankId>,
	active_graph: Option<&Id<N::Iri, N::BlankId>>,
) -> ExtendNodeMapResult<N>
where
	N::Iri: Clone + Eq + Hash,
	N::BlankId: Clone + Eq + Hash,
{
	match element.inner() {
		Object::Value(value) => {
			let flat_value = value.clone();
			Ok(Indexed::new(
				Object::Value(flat_value),
				element.index().map(ToOwned::to_owned),
			))
		}
		Object::List(list) => {
			let mut flat_list = Vec::new();

			for item in list {
				flat_list.push(extend_node_map(env, node_map, item, active_graph)?);
			}

			Ok(Indexed::new(
				Object::List(object::List::new(flat_list)),
				element.index().map(ToOwned::to_owned),
			))
		}
		Object::Node(node) => {
			let flat_node =
				extend_node_map_from_node(env, node_map, node, element.index(), active_graph)?;
			Ok(flat_node.map_inner(Object::node))
		}
	}
}

type ExtendNodeMapFromNodeResult<T, B> = Result<Indexed<Node<T, B>>, ConflictingIndexes<T, B>>;

fn extend_node_map_from_node<N: Vocabulary, G: Generator<N>>(
	env: &mut Environment<N, G>,
	node_map: &mut NodeMap<N::Iri, N::BlankId>,
	node: &Node<N::Iri, N::BlankId>,
	index: Option<&str>,
	active_graph: Option<&Id<N::Iri, N::BlankId>>,
) -> ExtendNodeMapFromNodeResult<N::Iri, N::BlankId>
where
	N::Iri: Clone + Eq + Hash,
	N::BlankId: Clone + Eq + Hash,
{
	let id = env.assign_node_id(node.id.as_ref());

	{
		let flat_node = node_map
			.graph_mut(active_graph)
			.unwrap()
			.declare_node(id.clone(), index)?;

		if let Some(entry) = node.types.as_deref() {
			flat_node.types = Some(
				entry
					.iter()
					.map(|ty| env.assign_node_id(Some(ty)))
					.collect(),
			);
		}
	}

	if let Some(graph_entry) = node.graph_entry() {
		node_map.declare_graph(id.clone());

		let mut flat_graph = IndexSet::new();
		for object in graph_entry.iter() {
			let flat_object = extend_node_map(env, node_map, object, Some(&id))?;
			flat_graph.insert(flat_object);
		}

		let flat_node = node_map
			.graph_mut(active_graph)
			.unwrap()
			.get_mut(&id)
			.unwrap();
		match flat_node.graph_entry_mut() {
			Some(graph) => graph.extend(flat_graph),
			None => flat_node.set_graph_entry(Some(flat_graph)),
		}
	}

	if let Some(included_entry) = node.included_entry() {
		for inode in included_entry {
			extend_node_map_from_node(env, node_map, inode.inner(), inode.index(), active_graph)?;
		}
	}

	for (property, objects) in node.properties() {
		let mut flat_objects = Vec::new();
		for object in objects {
			let flat_object = extend_node_map(env, node_map, object, active_graph)?;
			flat_objects.push(flat_object);
		}
		node_map
			.graph_mut(active_graph)
			.unwrap()
			.get_mut(&id)
			.unwrap()
			.properties_mut()
			.insert_all_unique(property.clone(), flat_objects)
	}

	if let Some(reverse_properties) = node.reverse_properties_entry() {
		for (property, nodes) in reverse_properties.iter() {
			for subject in nodes {
				let flat_subject = extend_node_map_from_node(
					env,
					node_map,
					subject.inner(),
					subject.index(),
					active_graph,
				)?;

				let subject_id = flat_subject.id.as_ref().unwrap();

				let flat_subject = node_map
					.graph_mut(active_graph)
					.unwrap()
					.get_mut(subject_id)
					.unwrap();

				flat_subject.properties_mut().insert_unique(
					property.clone(),
					Indexed::none(Object::node(Node::with_id(id.clone()))),
				)
			}

			// let mut flat_nodes = Vec::new();
			// for node in nodes {
			// 	let flat_node = extend_node_map_from_node(
			// 		env,
			// 		node_map,
			// 		node.inner(),
			// 		node.index(),
			// 		active_graph,
			// 	)?;
			// 	flat_nodes.push(flat_node);
			// }

			// node_map
			// 	.graph_mut(active_graph)
			// 	.unwrap()
			// 	.get_mut(&id)
			// 	.unwrap()
			// 	.reverse_properties_mut()
			// 	.insert_all_unique(property.clone(), flat_nodes)
		}
	}

	Ok(Indexed::new(Node::with_id(id), None))
}
