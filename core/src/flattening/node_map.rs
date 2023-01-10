use super::Environment;
use crate::{id, object, ExpandedDocument, Id, Indexed, IndexedNode, IndexedObject, Node, Object};
use derivative::Derivative;
use json_ld_syntax::Entry;
use locspan::{BorrowStripped, Meta, Stripped};
use rdf_types::{BlankIdVocabulary, IriVocabulary, Vocabulary};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Conflicting indexes error.
///
/// Raised when a single node is declared with two different indexes.
#[derive(Clone, Debug, thiserror::Error)]
#[error("Index `{defined_index}` conflicts with index `{conflicting_index}`")]
pub struct ConflictingIndexes<T, B, M> {
	pub node_id: Meta<Id<T, B>, M>,
	pub defined_index: String,
	pub conflicting_index: String,
}

pub type Parts<T, B, M> = (
	NodeMapGraph<T, B, M>,
	HashMap<Id<T, B>, NamedNodeMapGraph<T, B, M>>,
);

/// Node identifier to node definition map.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct NodeMap<T, B, M> {
	graphs: HashMap<Id<T, B>, NamedNodeMapGraph<T, B, M>>,
	default_graph: NodeMapGraph<T, B, M>,
}

impl<T, B, M> NodeMap<T, B, M> {
	pub fn new() -> Self {
		Self {
			graphs: HashMap::new(),
			default_graph: NodeMapGraph::new(),
		}
	}

	pub fn into_parts(self) -> Parts<T, B, M> {
		(self.default_graph, self.graphs)
	}

	pub fn iter(&self) -> Iter<T, B, M> {
		Iter {
			default_graph: Some(&self.default_graph),
			graphs: self.graphs.iter(),
		}
	}

	pub fn iter_named(
		&self,
	) -> std::collections::hash_map::Iter<Id<T, B>, NamedNodeMapGraph<T, B, M>> {
		self.graphs.iter()
	}
}

pub struct NamedNodeMapGraph<T, B, M> {
	id_metadata: M,
	graph: NodeMapGraph<T, B, M>,
}

impl<T, B, M> NamedNodeMapGraph<T, B, M> {
	pub fn new(id_metadata: M) -> Self {
		Self {
			id_metadata,
			graph: NodeMapGraph::new(),
		}
	}

	pub fn id_metadata(&self) -> &M {
		&self.id_metadata
	}

	pub fn as_graph(&self) -> &NodeMapGraph<T, B, M> {
		&self.graph
	}

	pub fn as_graph_mut(&mut self) -> &mut NodeMapGraph<T, B, M> {
		&mut self.graph
	}

	pub fn into_parts(self) -> (M, NodeMapGraph<T, B, M>) {
		(self.id_metadata, self.graph)
	}
}

impl<T: Eq + Hash, B: Eq + Hash, M> NodeMap<T, B, M> {
	pub fn graph(&self, id: Option<&Id<T, B>>) -> Option<&NodeMapGraph<T, B, M>> {
		match id {
			Some(id) => self.graphs.get(id).map(NamedNodeMapGraph::as_graph),
			None => Some(&self.default_graph),
		}
	}

	pub fn graph_mut(&mut self, id: Option<&Id<T, B>>) -> Option<&mut NodeMapGraph<T, B, M>> {
		match id {
			Some(id) => self.graphs.get_mut(id).map(NamedNodeMapGraph::as_graph_mut),
			None => Some(&mut self.default_graph),
		}
	}

	pub fn declare_graph(&mut self, Meta(id, meta): Meta<Id<T, B>, M>) {
		if let std::collections::hash_map::Entry::Vacant(entry) = self.graphs.entry(id) {
			entry.insert(NamedNodeMapGraph::new(meta));
		}
	}

	/// Merge all the graphs into a single `NodeMapGraph`.
	///
	/// The order in which graphs are merged is not defined.
	pub fn merge(self) -> NodeMapGraph<T, B, M>
	where
		T: Clone,
		B: Clone,
		M: Clone,
	{
		let mut result = self.default_graph;

		for (_, graph) in self.graphs {
			result.merge_with(graph.graph)
		}

		result
	}
}

pub struct Iter<'a, T, B, M> {
	default_graph: Option<&'a NodeMapGraph<T, B, M>>,
	graphs: std::collections::hash_map::Iter<'a, Id<T, B>, NamedNodeMapGraph<T, B, M>>,
}

impl<'a, T, B, M> Iterator for Iter<'a, T, B, M> {
	type Item = (Option<Meta<&'a Id<T, B>, &'a M>>, &'a NodeMapGraph<T, B, M>);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self
				.graphs
				.next()
				.map(|(id, graph)| (Some(Meta(id, &graph.id_metadata)), &graph.graph)),
		}
	}
}

impl<'a, T, B, M> IntoIterator for &'a NodeMap<T, B, M> {
	type Item = (Option<Meta<&'a Id<T, B>, &'a M>>, &'a NodeMapGraph<T, B, M>);
	type IntoIter = Iter<'a, T, B, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter<T, B, M> {
	default_graph: Option<NodeMapGraph<T, B, M>>,
	graphs: std::collections::hash_map::IntoIter<Id<T, B>, NamedNodeMapGraph<T, B, M>>,
}

impl<T, B, M> Iterator for IntoIter<T, B, M> {
	type Item = (Option<Meta<Id<T, B>, M>>, NodeMapGraph<T, B, M>);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self
				.graphs
				.next()
				.map(|(id, graph)| (Some(Meta(id, graph.id_metadata)), graph.graph)),
		}
	}
}

impl<T, B, M> IntoIterator for NodeMap<T, B, M> {
	type Item = (Option<Meta<Id<T, B>, M>>, NodeMapGraph<T, B, M>);
	type IntoIter = IntoIter<T, B, M>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			default_graph: Some(self.default_graph),
			graphs: self.graphs.into_iter(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct NodeMapGraph<T, B, M> {
	nodes: HashMap<Id<T, B>, IndexedNode<T, B, M>>,
}

impl<T, B, M> NodeMapGraph<T, B, M> {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
		}
	}
}

pub type DeclareNodeResult<'a, T, B, M> =
	Result<&'a mut Indexed<Node<T, B, M>, M>, ConflictingIndexes<T, B, M>>;

impl<T: Eq + Hash, B: Eq + Hash, M> NodeMapGraph<T, B, M> {
	pub fn contains(&self, id: &Id<T, B>) -> bool {
		self.nodes.contains_key(id)
	}

	pub fn get(&self, id: &Id<T, B>) -> Option<&IndexedNode<T, B, M>> {
		self.nodes.get(id)
	}

	pub fn get_mut(&mut self, id: &Id<T, B>) -> Option<&mut IndexedNode<T, B, M>> {
		self.nodes.get_mut(id)
	}

	pub fn declare_node(
		&mut self,
		id: Meta<Id<T, B>, M>,
		index: Option<&Entry<String, M>>,
	) -> DeclareNodeResult<T, B, M>
	where
		T: Clone,
		B: Clone,
		M: Clone,
	{
		if let Some(entry) = self.nodes.get_mut(&id) {
			match (entry.index_entry(), index) {
				(Some(entry_index), Some(index)) => {
					if entry_index.stripped() != index.stripped() {
						return Err(ConflictingIndexes {
							node_id: id,
							defined_index: entry_index.to_string(),
							conflicting_index: index.to_string(),
						});
					}
				}
				(None, Some(index)) => entry.set_index(Some(index.clone())),
				_ => (),
			}
		} else {
			self.nodes.insert(
				id.value().clone(),
				Meta(
					Indexed::new(
						Node::with_id(Entry::new(id.metadata().clone(), id.clone())),
						index.cloned(),
					),
					id.metadata().clone(),
				),
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
		M: Clone,
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
	pub fn merge_node(&mut self, Meta(node, meta): IndexedNode<T, B, M>)
	where
		T: Clone,
		B: Clone,
		M: Clone,
	{
		let (node, index) = node.into_parts();
		let node = node.into_parts();

		if let Some(id) = &node.id {
			if let Some(entry) = self.nodes.get_mut(id) {
				if let Some(index) = index {
					entry.set_index(Some(index))
				}
			} else {
				self.nodes.insert(
					id.value.value().clone(),
					Meta(Indexed::new(Node::with_id(id.clone()), index), meta),
				);
			}

			let flat_node = self.nodes.get_mut(id).unwrap();

			if let Some(types) = node.types {
				flat_node
					.type_entry_or_default(
						types.key_metadata.clone(),
						types.value.metadata().clone(),
					)
					.extend(types.value.into_value().into_iter());
			}

			flat_node.set_graph(node.graph);
			flat_node.set_included(node.included);
			flat_node
				.properties_mut()
				.extend_unique_stripped(node.properties);

			if let Some(props) = node.reverse_properties {
				flat_node
					.reverse_properties_or_default(
						props.key_metadata.clone(),
						props.value.metadata().clone(),
					)
					.extend_unique_stripped(props.value.into_value());
			}
		}
	}

	pub fn nodes(&self) -> NodeMapGraphNodes<T, B, M> {
		self.nodes.values()
	}

	pub fn into_nodes(self) -> IntoNodeMapGraphNodes<T, B, M> {
		self.nodes.into_values()
	}
}

pub type NodeMapGraphNodes<'a, T, B, M> =
	std::collections::hash_map::Values<'a, Id<T, B>, IndexedNode<T, B, M>>;
pub type IntoNodeMapGraphNodes<T, B, M> =
	std::collections::hash_map::IntoValues<Id<T, B>, IndexedNode<T, B, M>>;

impl<T, B, M> IntoIterator for NodeMapGraph<T, B, M> {
	type Item = (Id<T, B>, IndexedNode<T, B, M>);
	type IntoIter = std::collections::hash_map::IntoIter<Id<T, B>, IndexedNode<T, B, M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a, T, B, M> IntoIterator for &'a NodeMapGraph<T, B, M> {
	type Item = (&'a Id<T, B>, &'a IndexedNode<T, B, M>);
	type IntoIter = std::collections::hash_map::Iter<'a, Id<T, B>, IndexedNode<T, B, M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.iter()
	}
}

impl<T: Clone + Eq + Hash, B: Clone + Eq + Hash, M: Clone> ExpandedDocument<T, B, M> {
	pub fn generate_node_map_with<V: Vocabulary<Iri = T, BlankId = B>, G: id::Generator<V, M>>(
		&self,
		vocabulary: &mut V,
		generator: G,
	) -> Result<NodeMap<T, B, M>, ConflictingIndexes<T, B, M>> {
		let mut node_map: NodeMap<T, B, M> = NodeMap::new();
		let mut env: Environment<M, V, G> = Environment::new(vocabulary, generator);
		for object in self {
			extend_node_map(&mut env, &mut node_map, object, None)?;
		}
		Ok(node_map)
	}
}

pub type ExtendNodeMapResult<V, M> = Result<
	IndexedObject<<V as IriVocabulary>::Iri, <V as BlankIdVocabulary>::BlankId, M>,
	ConflictingIndexes<<V as IriVocabulary>::Iri, <V as BlankIdVocabulary>::BlankId, M>,
>;

/// Extends the `NodeMap` with the given `element` of an expanded JSON-LD document.
fn extend_node_map<N: Vocabulary, M: Clone, G: id::Generator<N, M>>(
	env: &mut Environment<M, N, G>,
	node_map: &mut NodeMap<N::Iri, N::BlankId, M>,
	Meta(element, meta): &IndexedObject<N::Iri, N::BlankId, M>,
	active_graph: Option<&Id<N::Iri, N::BlankId>>,
) -> ExtendNodeMapResult<N, M>
where
	N::Iri: Clone + Eq + Hash,
	N::BlankId: Clone + Eq + Hash,
{
	match element.inner() {
		Object::Value(value) => {
			let flat_value = value.clone();
			Ok(Meta(
				Indexed::new(Object::Value(flat_value), element.index_entry().cloned()),
				meta.clone(),
			))
		}
		Object::List(list) => {
			let mut flat_list = Vec::new();

			for item in list {
				flat_list.push(extend_node_map(env, node_map, item, active_graph)?);
			}

			Ok(Meta(
				Indexed::new(
					Object::List(object::List::new(
						list.entry().key_metadata.clone(),
						Meta(flat_list, list.entry().value.metadata().clone()),
					)),
					element.index_entry().cloned(),
				),
				meta.clone(),
			))
		}
		Object::Node(node) => {
			let flat_node = extend_node_map_from_node(
				env,
				node_map,
				node,
				element.index_entry(),
				active_graph,
			)?;
			Ok(Meta(flat_node.map_inner(Object::node), meta.clone()))
		}
	}
}

type ExtendNodeMapFromNodeResult<T, B, M> =
	Result<Indexed<Node<T, B, M>, M>, ConflictingIndexes<T, B, M>>;

fn extend_node_map_from_node<N: Vocabulary, M: Clone, G: id::Generator<N, M>>(
	env: &mut Environment<M, N, G>,
	node_map: &mut NodeMap<N::Iri, N::BlankId, M>,
	node: &Node<N::Iri, N::BlankId, M>,
	index: Option<&Entry<String, M>>,
	active_graph: Option<&Id<N::Iri, N::BlankId>>,
) -> ExtendNodeMapFromNodeResult<N::Iri, N::BlankId, M>
where
	N::Iri: Clone + Eq + Hash,
	N::BlankId: Clone + Eq + Hash,
{
	let id = env.assign_node_id(node.id_entry().map(Entry::as_value));

	{
		let flat_node = node_map
			.graph_mut(active_graph)
			.unwrap()
			.declare_node(id.clone(), index)?;

		if let Some(entry) = node.type_entry() {
			flat_node.set_type_entry(Some(Entry::new(
				entry.key_metadata.clone(),
				Meta(
					entry
						.value
						.iter()
						.map(|ty| env.assign_node_id(Some(ty)))
						.collect(),
					entry.value.metadata().clone(),
				),
			)));
		}
	}

	if let Some(graph_entry) = node.graph_entry() {
		node_map.declare_graph(id.clone());

		let mut flat_graph = HashSet::new();
		for object in graph_entry.iter() {
			let flat_object = extend_node_map(env, node_map, object, Some(&id))?;
			flat_graph.insert(Stripped(flat_object));
		}

		let flat_node = node_map
			.graph_mut(active_graph)
			.unwrap()
			.get_mut(&id)
			.unwrap();
		match flat_node.graph_entry_mut() {
			Some(graph) => graph.extend(flat_graph),
			None => flat_node.set_graph(Some(Entry::new(
				graph_entry.key_metadata.clone(),
				Meta(flat_graph, graph_entry.value.metadata().clone()),
			))),
		}
	}

	if let Some(included_entry) = node.included_entry() {
		for inode in included_entry.value.iter() {
			extend_node_map_from_node(
				env,
				node_map,
				inode.inner(),
				inode.index_entry(),
				active_graph,
			)?;
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
			.insert_all_unique(property.cloned(), flat_objects)
	}

	if let Some(reverse_properties) = node.reverse_properties_entry() {
		for (property, nodes) in reverse_properties.iter() {
			for subject in nodes {
				let flat_subject = extend_node_map_from_node(
					env,
					node_map,
					subject.inner(),
					subject.index_entry(),
					active_graph,
				)?;

				let subject_id = flat_subject.id_entry().unwrap();

				let flat_subject = node_map
					.graph_mut(active_graph)
					.unwrap()
					.get_mut(subject_id)
					.unwrap();

				flat_subject.properties_mut().insert_unique(
					property.cloned(),
					Meta(
						Indexed::new(
							Object::node(Node::with_id(Entry::new(
								id.metadata().clone(),
								id.clone(),
							))),
							None,
						),
						id.metadata().clone(),
					),
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

	Ok(Indexed::new(
		Node::with_id(Entry::new(id.metadata().clone(), id)),
		None,
	))
}
