use crate::{
	ConflictingIndexes, JsonLdBacktraceBuilder, JsonLdExpandedDocument, JsonLdFlattenedDocument,
	JsonLdIndexed, JsonLdIndexedNode, JsonLdIndexedObject, JsonLdLenient, JsonLdLocated,
	JsonLdNode, JsonLdObject, JsonLdSourceFile,
};
use educe::Educe;
use json_syntax::tracing::JsonBacktraceBuf;
use rdf_syntax::{Generator, Id};
use std::collections::{HashMap, HashSet};

mod builder;

use builder::NodeMapBuilder;

pub type NodeMapExtendError = Box<JsonLdLocated<ConflictingIndexes>>;

pub type Parts = (
	JsonLdNodeMapGraph,
	HashMap<JsonLdLenient<Id>, JsonLdNodeMapGraph>,
);

impl JsonLdExpandedDocument {
	pub fn generate_node_map_with(
		&self,
		generator: impl Generator,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonLdNodeMap, NodeMapExtendError> {
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
pub struct JsonLdNodeMap {
	graphs: HashMap<JsonLdLenient<Id>, JsonLdNodeMapGraph>,
	default_graph: JsonLdNodeMapGraph,
}

impl JsonLdNodeMap {
	pub fn new() -> Self {
		Self {
			graphs: HashMap::new(),
			default_graph: JsonLdNodeMapGraph::new(),
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

	pub fn iter_named(
		&self,
	) -> std::collections::hash_map::Iter<'_, JsonLdLenient<Id>, JsonLdNodeMapGraph> {
		self.graphs.iter()
	}

	pub fn graph(&self, id: Option<&JsonLdLenient<Id>>) -> Option<&JsonLdNodeMapGraph> {
		match id {
			Some(id) => self.graphs.get(id),
			None => Some(&self.default_graph),
		}
	}

	pub fn graph_mut(&mut self, id: Option<&JsonLdLenient<Id>>) -> Option<&mut JsonLdNodeMapGraph> {
		match id {
			Some(id) => self.graphs.get_mut(id),
			None => Some(&mut self.default_graph),
		}
	}

	pub fn declare_graph(&mut self, id: JsonLdLenient<Id>) {
		if let std::collections::hash_map::Entry::Vacant(entry) = self.graphs.entry(id) {
			entry.insert(JsonLdNodeMapGraph::new());
		}
	}

	/// Merge all the graphs into a single `NodeMapGraph`.
	///
	/// The order in which graphs are merged is not defined.
	pub fn merge(self) -> JsonLdNodeMapGraph {
		let mut result = self.default_graph;

		for (_, graph) in self.graphs {
			result.merge_with(graph)
		}

		result
	}

	pub fn flatten(self, ordered: bool) -> JsonLdFlattenedDocument {
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

	pub fn flatten_unordered(self) -> HashSet<JsonLdIndexedNode> {
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
	default_graph: Option<&'a JsonLdNodeMapGraph>,
	graphs: std::collections::hash_map::Iter<'a, JsonLdLenient<Id>, JsonLdNodeMapGraph>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = (Option<&'a JsonLdLenient<Id>>, &'a JsonLdNodeMapGraph);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph)),
		}
	}
}

impl<'a> IntoIterator for &'a JsonLdNodeMap {
	type Item = (Option<&'a JsonLdLenient<Id>>, &'a JsonLdNodeMapGraph);
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct IntoIter {
	default_graph: Option<JsonLdNodeMapGraph>,
	graphs: std::collections::hash_map::IntoIter<JsonLdLenient<Id>, JsonLdNodeMapGraph>,
}

impl Iterator for IntoIter {
	type Item = (Option<JsonLdLenient<Id>>, JsonLdNodeMapGraph);

	fn next(&mut self) -> Option<Self::Item> {
		match self.default_graph.take() {
			Some(default_graph) => Some((None, default_graph)),
			None => self.graphs.next().map(|(id, graph)| (Some(id), graph)),
		}
	}
}

impl IntoIterator for JsonLdNodeMap {
	type Item = (Option<JsonLdLenient<Id>>, JsonLdNodeMapGraph);
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
pub struct JsonLdNodeMapGraphEntry {
	pub node: JsonLdIndexedNode,
	/// Location of the `@index` value that was declared for this node, if known.
	pub index_location: Option<JsonBacktraceBuf<JsonLdSourceFile>>,
}

impl JsonLdNodeMapGraphEntry {
	fn new(node: JsonLdIndexedNode) -> Self {
		Self {
			node,
			index_location: None,
		}
	}
}

impl std::ops::Deref for JsonLdNodeMapGraphEntry {
	type Target = JsonLdIndexedNode;
	fn deref(&self) -> &Self::Target {
		&self.node
	}
}

impl std::ops::DerefMut for JsonLdNodeMapGraphEntry {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.node
	}
}

#[derive(Educe)]
#[educe(Default)]
pub struct JsonLdNodeMapGraph {
	nodes: HashMap<JsonLdLenient<Id>, JsonLdNodeMapGraphEntry>,
}

impl JsonLdNodeMapGraph {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new(),
		}
	}
}

pub type DeclareNodeResult<'a> = Result<&'a mut JsonLdNodeMapGraphEntry, NodeMapExtendError>;

impl JsonLdNodeMapGraph {
	pub fn contains(&self, id: &JsonLdLenient<Id>) -> bool {
		self.nodes.contains_key(id)
	}

	pub fn get(&self, id: &JsonLdLenient<Id>) -> Option<&JsonLdNodeMapGraphEntry> {
		self.nodes.get(id)
	}

	pub fn get_mut(&mut self, id: &JsonLdLenient<Id>) -> Option<&mut JsonLdNodeMapGraphEntry> {
		self.nodes.get_mut(id)
	}

	/// Declares a node in the graph.
	///
	/// `index` pairs the `@index` string with its source location in the
	/// expanded document (if known).
	pub fn declare_node(
		&mut self,
		id: JsonLdLenient<Id>,
		index: Option<(&str, JsonBacktraceBuf<JsonLdSourceFile>)>,
	) -> DeclareNodeResult<'_> {
		if let Some(entry) = self.nodes.get_mut(&id) {
			match (entry.index(), index) {
				(Some(entry_index), Some((new_index, new_location))) => {
					if entry_index != new_index {
						let defined_location = entry.index_location.clone().unwrap_or_default();
						return Err(Box::new(JsonLdLocated::new(
							ConflictingIndexes {
								node_id: id,
								defined_index: JsonLdLocated::new(
									entry_index.to_string(),
									defined_location,
								),
								conflicting_index: new_index.to_string(),
							},
							new_location,
						)));
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
				JsonLdNodeMapGraphEntry {
					node: JsonLdIndexed::new(JsonLdNode::new_with_id(Some(id.clone())), index_str),
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
	pub fn merge_node(&mut self, node: JsonLdIndexedNode) {
		let (node, index) = node.into_parts();

		if let Some(id) = &node.id {
			if let Some(entry) = self.nodes.get_mut(id) {
				if let Some(index) = index {
					entry.set_index(Some(index))
				}
			} else {
				self.nodes.insert(
					id.clone(),
					JsonLdNodeMapGraphEntry::new(JsonLdIndexed::new(
						JsonLdNode::new_with_id(Some(id.clone())),
						index,
					)),
				);
			}

			let flat_node = self.nodes.get_mut(id).unwrap();

			if let Some(types) = node.types {
				flat_node.types.get_or_insert_default().extend(types);
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
	std::collections::hash_map::Values<'a, JsonLdLenient<Id>, JsonLdNodeMapGraphEntry>;
pub type IntoNodeMapGraphNodes =
	std::collections::hash_map::IntoValues<JsonLdLenient<Id>, JsonLdNodeMapGraphEntry>;

impl IntoIterator for JsonLdNodeMapGraph {
	type Item = (JsonLdLenient<Id>, JsonLdNodeMapGraphEntry);
	type IntoIter =
		std::collections::hash_map::IntoIter<JsonLdLenient<Id>, JsonLdNodeMapGraphEntry>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.into_iter()
	}
}

impl<'a> IntoIterator for &'a JsonLdNodeMapGraph {
	type Item = (&'a JsonLdLenient<Id>, &'a JsonLdNodeMapGraphEntry);
	type IntoIter =
		std::collections::hash_map::Iter<'a, JsonLdLenient<Id>, JsonLdNodeMapGraphEntry>;

	fn into_iter(self) -> Self::IntoIter {
		self.nodes.iter()
	}
}

fn filter_graph(entry: JsonLdNodeMapGraphEntry) -> Option<JsonLdIndexedNode> {
	if entry.index().is_none() && entry.is_empty() {
		None
	} else {
		Some(entry.node)
	}
}

fn filter_sub_graph(mut entry: JsonLdNodeMapGraphEntry) -> Option<JsonLdIndexedObject> {
	if entry.index().is_none() && entry.properties().is_empty() {
		None
	} else {
		entry.set_graph_entry(None);
		entry.set_included(None);
		entry.set_reverse_properties(None);
		Some(entry.node.map_inner(JsonLdObject::node))
	}
}
