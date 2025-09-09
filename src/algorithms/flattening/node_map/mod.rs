use crate::{
	ExpandedDocument, FlattenedDocument, Id, Indexed, IndexedNode, IndexedObject, NodeObject,
	Object,
};
use educe::Educe;
use rdf_types::Generator;
use std::collections::{HashMap, HashSet};

mod builder;

use builder::NodeMapBuilder;

/// Conflicting indexes error.
///
/// Raised when a single node is declared with two different indexes.
#[derive(Clone, Debug, thiserror::Error)]
#[error("Index `{defined_index}` conflicts with index `{conflicting_index}`")]
pub struct ConflictingIndexes {
	pub node_id: Id,
	pub defined_index: String,
	pub conflicting_index: String,
}

pub type Parts = (NodeMapGraph, HashMap<Id, NodeMapGraph>);

impl ExpandedDocument {
	pub fn generate_node_map_with(
		&self,
		generator: impl Generator,
	) -> Result<NodeMap, ConflictingIndexes> {
		let mut builder = NodeMapBuilder::new(generator);

		for object in self {
			builder.extend_node_map(object, None)?;
		}

		Ok(builder.end())
	}
}

/// Node identifier to node definition map.
#[derive(Educe)]
#[educe(Default)]
pub struct NodeMap {
	graphs: HashMap<Id, NodeMapGraph>,
	default_graph: NodeMapGraph,
}

impl NodeMap {
	pub fn new() -> Self {
		Self {
			graphs: HashMap::new(),
			default_graph: NodeMapGraph::new(),
		}
	}

	pub fn into_parts(self) -> Parts {
		(self.default_graph, self.graphs)
	}

	pub fn iter(&self) -> Iter {
		Iter {
			default_graph: Some(&self.default_graph),
			graphs: self.graphs.iter(),
		}
	}

	pub fn iter_named(&self) -> std::collections::hash_map::Iter<Id, NodeMapGraph> {
		self.graphs.iter()
	}

	pub fn graph(&self, id: Option<&Id>) -> Option<&NodeMapGraph> {
		match id {
			Some(id) => self.graphs.get(id),
			None => Some(&self.default_graph),
		}
	}

	pub fn graph_mut(&mut self, id: Option<&Id>) -> Option<&mut NodeMapGraph> {
		match id {
			Some(id) => self.graphs.get_mut(id),
			None => Some(&mut self.default_graph),
		}
	}

	pub fn declare_graph(&mut self, id: Id) {
		if let std::collections::hash_map::Entry::Vacant(entry) = self.graphs.entry(id) {
			entry.insert(NodeMapGraph::new());
		}
	}

	/// Merge all the graphs into a single `NodeMapGraph`.
	///
	/// The order in which graphs are merged is not defined.
	pub fn merge(self) -> NodeMapGraph {
		let mut result = self.default_graph;

		for (_, graph) in self.graphs {
			result.merge_with(graph)
		}

		result
	}

	pub fn flatten(self, ordered: bool) -> FlattenedDocument {
		let (mut default_graph, named_graphs) = self.into_parts();

		let mut named_graphs: Vec<_> = named_graphs.into_iter().collect();
		if ordered {
			named_graphs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
		}

		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).ok().unwrap();
			let mut nodes: Vec<_> = graph.into_nodes().collect();
			if ordered {
				nodes.sort_by(|a, b| {
					a.id.as_ref()
						.unwrap()
						.as_str()
						.cmp(b.id.as_ref().unwrap().as_str())
				});
			}
			entry.set_graph_entry(Some(
				nodes.into_iter().filter_map(filter_sub_graph).collect(),
			));
		}

		let mut nodes: Vec<_> = default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.collect();

		if ordered {
			nodes.sort_by(|a, b| {
				a.id.as_ref()
					.unwrap()
					.as_str()
					.cmp(b.id.as_ref().unwrap().as_str())
			});
		}

		nodes
	}

	pub fn flatten_unordered(self) -> HashSet<IndexedNode> {
		let (mut default_graph, named_graphs) = self.into_parts();

		for (graph_id, graph) in named_graphs {
			let entry = default_graph.declare_node(graph_id, None).ok().unwrap();
			entry.set_graph_entry(Some(
				graph.into_nodes().filter_map(filter_sub_graph).collect(),
			));
		}

		default_graph
			.into_nodes()
			.filter_map(filter_graph)
			.collect()
	}
}

pub struct Iter<'a> {
	default_graph: Option<&'a NodeMapGraph>,
	graphs: std::collections::hash_map::Iter<'a, Id, NodeMapGraph>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = (Option<&'a Id>, &'a NodeMapGraph);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph)),
		}
	}
}

impl<'a> IntoIterator for &'a NodeMap {
	type Item = (Option<&'a Id>, &'a NodeMapGraph);
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter {
	default_graph: Option<NodeMapGraph>,
	graphs: std::collections::hash_map::IntoIter<Id, NodeMapGraph>,
}

impl Iterator for IntoIter {
	type Item = (Option<Id>, NodeMapGraph);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph)),
		}
	}
}

impl IntoIterator for NodeMap {
	type Item = (Option<Id>, NodeMapGraph);
	type IntoIter = IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			default_graph: Some(self.default_graph),
			graphs: self.graphs.into_iter(),
		}
	}
}

#[derive(Educe)]
#[educe(Default)]
pub struct NodeMapGraph {
	nodes: HashMap<Id, IndexedNode>,
}

impl NodeMapGraph {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
		}
	}
}

pub type DeclareNodeResult<'a> = Result<&'a mut Indexed<NodeObject>, ConflictingIndexes>;

impl NodeMapGraph {
	pub fn contains(&self, id: &Id) -> bool {
		self.nodes.contains_key(id)
	}

	pub fn get(&self, id: &Id) -> Option<&IndexedNode> {
		self.nodes.get(id)
	}

	pub fn get_mut(&mut self, id: &Id) -> Option<&mut IndexedNode> {
		self.nodes.get_mut(id)
	}

	pub fn declare_node(&mut self, id: Id, index: Option<&str>) -> DeclareNodeResult {
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
				Indexed::new(
					NodeObject::new_with_id(Some(id.clone())),
					index.map(ToOwned::to_owned),
				),
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
	pub fn merge_node(&mut self, node: IndexedNode) {
		let (node, index) = node.into_parts();

		if let Some(id) = &node.id {
			if let Some(entry) = self.nodes.get_mut(id) {
				if let Some(index) = index {
					entry.set_index(Some(index))
				}
			} else {
				self.nodes.insert(
					id.clone(),
					Indexed::new(NodeObject::new_with_id(Some(id.clone())), index),
				);
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

	pub fn nodes(&self) -> NodeMapGraphNodes {
		self.nodes.values()
	}

	pub fn into_nodes(self) -> IntoNodeMapGraphNodes {
		self.nodes.into_values()
	}
}

pub type NodeMapGraphNodes<'a> = std::collections::hash_map::Values<'a, Id, IndexedNode>;
pub type IntoNodeMapGraphNodes = std::collections::hash_map::IntoValues<Id, IndexedNode>;

impl IntoIterator for NodeMapGraph {
	type Item = (Id, IndexedNode);
	type IntoIter = std::collections::hash_map::IntoIter<Id, IndexedNode>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a> IntoIterator for &'a NodeMapGraph {
	type Item = (&'a Id, &'a IndexedNode);
	type IntoIter = std::collections::hash_map::Iter<'a, Id, IndexedNode>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.iter()
	}
}

fn filter_graph(node: IndexedNode) -> Option<IndexedNode> {
	if node.index().is_none() && node.is_empty() {
		None
	} else {
		Some(node)
	}
}

fn filter_sub_graph(mut node: IndexedNode) -> Option<IndexedObject> {
	if node.index().is_none() && node.properties().is_empty() {
		None
	} else {
		node.set_graph_entry(None);
		node.set_included(None);
		node.set_reverse_properties(None);
		Some(node.map_inner(Object::node))
	}
}
