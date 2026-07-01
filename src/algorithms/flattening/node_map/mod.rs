use crate::{
	ExpandedDocument, FlattenedDocument, Indexed, IndexedNode, IndexedObject, Lenient, NodeObject,
	Object,
	algorithms::{JsonLdLocated, JsonLdLocationStack, JsonLdSource},
};
use educe::Educe;
use json_syntax::tracing::JsonBacktraceBuf;
use rdf_syntax::{Generator, Id};
use std::collections::{HashMap, HashSet};

mod builder;

use builder::NodeMapBuilder;

/// Conflicting indexes error.
///
/// Raised when a single node is declared with two different indexes.
#[derive(Clone, Debug, thiserror::Error)]
#[error("Index `{defined_index}` conflicts with index `{conflicting_index}`")]
pub struct ConflictingIndexes {
	pub node_id: Lenient<Id>,
	/// The previously-declared index, with its source location if known.
	pub defined_index: JsonLdLocated<String>,
	pub conflicting_index: String,
}

pub type Parts = (NodeMapGraph, HashMap<Lenient<Id>, NodeMapGraph>);

impl ExpandedDocument {
	pub fn generate_node_map_with(
		&self,
		generator: impl Generator,
		location: JsonLdLocationStack<'_>,
	) -> Result<NodeMap, JsonLdLocated<ConflictingIndexes>> {
		let mut builder = NodeMapBuilder::new(generator);

		for (i, object) in self.iter().enumerate() {
			builder.extend_node_map(object, None, location.array_index(i))?;
		}

		Ok(builder.end())
	}
}

/// Node identifier to node definition map.
#[derive(Educe)]
#[educe(Default)]
pub struct NodeMap {
	graphs: HashMap<Lenient<Id>, NodeMapGraph>,
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

	pub fn iter(&self) -> Iter<'_> {
		Iter {
			default_graph: Some(&self.default_graph),
			graphs: self.graphs.iter(),
		}
	}

	pub fn iter_named(&self) -> std::collections::hash_map::Iter<'_, Lenient<Id>, NodeMapGraph> {
		self.graphs.iter()
	}

	pub fn graph(&self, id: Option<&Lenient<Id>>) -> Option<&NodeMapGraph> {
		match id {
			Some(id) => self.graphs.get(id),
			None => Some(&self.default_graph),
		}
	}

	pub fn graph_mut(&mut self, id: Option<&Lenient<Id>>) -> Option<&mut NodeMapGraph> {
		match id {
			Some(id) => self.graphs.get_mut(id),
			None => Some(&mut self.default_graph),
		}
	}

	pub fn declare_graph(&mut self, id: Lenient<Id>) {
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
	graphs: std::collections::hash_map::Iter<'a, Lenient<Id>, NodeMapGraph>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = (Option<&'a Lenient<Id>>, &'a NodeMapGraph);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph)),
		}
	}
}

impl<'a> IntoIterator for &'a NodeMap {
	type Item = (Option<&'a Lenient<Id>>, &'a NodeMapGraph);
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter {
	default_graph: Option<NodeMapGraph>,
	graphs: std::collections::hash_map::IntoIter<Lenient<Id>, NodeMapGraph>,
}

impl Iterator for IntoIter {
	type Item = (Option<Lenient<Id>>, NodeMapGraph);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph)),
		}
	}
}

impl IntoIterator for NodeMap {
	type Item = (Option<Lenient<Id>>, NodeMapGraph);
	type IntoIter = IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			default_graph: Some(self.default_graph),
			graphs: self.graphs.into_iter(),
		}
	}
}

/// Entry in a [`NodeMapGraph`], pairing an indexed node with the optional
/// source location of its `@index` value.
pub struct NodeMapGraphEntry {
	pub node: IndexedNode,
	/// Location of the `@index` value that was declared for this node, if known.
	pub index_location: Option<JsonBacktraceBuf<JsonLdSource>>,
}

impl NodeMapGraphEntry {
	fn new(node: IndexedNode) -> Self {
		Self {
			node,
			index_location: None,
		}
	}
}

impl std::ops::Deref for NodeMapGraphEntry {
	type Target = IndexedNode;
	fn deref(&self) -> &Self::Target {
		&self.node
	}
}

impl std::ops::DerefMut for NodeMapGraphEntry {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.node
	}
}

#[derive(Educe)]
#[educe(Default)]
pub struct NodeMapGraph {
	nodes: HashMap<Lenient<Id>, NodeMapGraphEntry>,
}

impl NodeMapGraph {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
		}
	}
}

pub type DeclareNodeResult<'a> =
	Result<&'a mut NodeMapGraphEntry, JsonLdLocated<ConflictingIndexes>>;

impl NodeMapGraph {
	pub fn contains(&self, id: &Lenient<Id>) -> bool {
		self.nodes.contains_key(id)
	}

	pub fn get(&self, id: &Lenient<Id>) -> Option<&NodeMapGraphEntry> {
		self.nodes.get(id)
	}

	pub fn get_mut(&mut self, id: &Lenient<Id>) -> Option<&mut NodeMapGraphEntry> {
		self.nodes.get_mut(id)
	}

	/// Declares a node in the graph.
	///
	/// `index` pairs the `@index` string with its source location in the
	/// expanded document (if known).
	pub fn declare_node(
		&mut self,
		id: Lenient<Id>,
		index: Option<(&str, JsonBacktraceBuf<JsonLdSource>)>,
	) -> DeclareNodeResult<'_> {
		if let Some(entry) = self.nodes.get_mut(&id) {
			match (entry.index(), index) {
				(Some(entry_index), Some((new_index, new_location))) => {
					if entry_index != new_index {
						let defined_location = entry.index_location.clone().unwrap_or_default();
						return Err(JsonLdLocated::new(
							ConflictingIndexes {
								node_id: id,
								defined_index: JsonLdLocated::new(
									entry_index.to_string(),
									defined_location,
								),
								conflicting_index: new_index.to_string(),
							},
							new_location,
						));
					}
				}
				(None, Some((new_index, new_location))) => {
					entry.set_index(Some(new_index.to_owned()));
					entry.index_location = Some(new_location);
				}
				_ => (),
			}
		} else {
			let (index_str, index_location) = match index {
				Some((s, loc)) => (Some(s.to_owned()), Some(loc)),
				None => (None, None),
			};
			self.nodes.insert(
				id.clone(),
				NodeMapGraphEntry {
					node: Indexed::new(NodeObject::new_with_id(Some(id.clone())), index_str),
					index_location,
				},
			);
		}

		Ok(self.nodes.get_mut(&id).unwrap())
	}

	/// Merge this graph with `other`.
	///
	/// This calls [`merge_node`](Self::merge_node) with every node of `other`.
	pub fn merge_with(&mut self, other: Self) {
		for (_, entry) in other {
			self.merge_node(entry.node)
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
					NodeMapGraphEntry::new(Indexed::new(
						NodeObject::new_with_id(Some(id.clone())),
						index,
					)),
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

	pub fn nodes(&self) -> NodeMapGraphNodes<'_> {
		self.nodes.values()
	}

	pub fn into_nodes(self) -> IntoNodeMapGraphNodes {
		self.nodes.into_values()
	}
}

pub type NodeMapGraphNodes<'a> =
	std::collections::hash_map::Values<'a, Lenient<Id>, NodeMapGraphEntry>;
pub type IntoNodeMapGraphNodes =
	std::collections::hash_map::IntoValues<Lenient<Id>, NodeMapGraphEntry>;

impl IntoIterator for NodeMapGraph {
	type Item = (Lenient<Id>, NodeMapGraphEntry);
	type IntoIter = std::collections::hash_map::IntoIter<Lenient<Id>, NodeMapGraphEntry>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a> IntoIterator for &'a NodeMapGraph {
	type Item = (&'a Lenient<Id>, &'a NodeMapGraphEntry);
	type IntoIter = std::collections::hash_map::Iter<'a, Lenient<Id>, NodeMapGraphEntry>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.iter()
	}
}

fn filter_graph(entry: NodeMapGraphEntry) -> Option<IndexedNode> {
	if entry.index().is_none() && entry.is_empty() {
		None
	} else {
		Some(entry.node)
	}
}

fn filter_sub_graph(mut entry: NodeMapGraphEntry) -> Option<IndexedObject> {
	if entry.index().is_none() && entry.properties().is_empty() {
		None
	} else {
		entry.set_graph_entry(None);
		entry.set_included(None);
		entry.set_reverse_properties(None);
		Some(entry.node.map_inner(Object::node))
	}
}
