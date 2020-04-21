use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use iref::Iri;
use json::JsonValue;
use crate::{
	Id,
	Term,
	Keyword,
	Object,
	Indexed,
	util
};
use super::node::Type;

/// A graph object.
#[derive(PartialEq, Eq)]
pub struct Graph<T: Id> {
	/// Name of the graph, if any.
	id: Option<Term<T>>,

	/// Nodes of the graph.
	nodes: HashSet<Indexed<Object<T>>>
}

/// Graph nodes iterator.
type Nodes<'a, T> = std::collections::hash_set::Iter<'a, Indexed<Object<T>>>;

impl<T: Id> Graph<T> {
	/// Create a new empty graph with the given id.
	pub fn new(id: Option<Term<T>>) -> Graph<T> {
		Graph {
			id: None,
			nodes: HashSet::new()
		}
	}

	/// Return the name of the graph, if any.
	pub fn id(&self) -> Option<&Term<T>> {
		self.id.as_ref()
	}

	/// Set the name of the graph.
	pub fn set_id(&mut self, id: Option<Term<T>>) {
		self.id = id
	}

	/// If the graph is named, and if such conversion is possible,
	/// return the name of the graph as a string.
	pub fn as_str(&self) -> Option<&str> {
		match &self.id {
			Some(id) => Some(id.as_str()),
			None => None
		}
	}

	/// If the graph is named by an IRI, returns it.
	pub fn as_iri(&self) -> Option<Iri> {
		match &self.id {
			Some(id) => id.as_iri(),
			None => None
		}
	}

	/// Checks if the graph has a name.
	pub fn is_named(&self) -> bool {
		self.id.is_some()
	}

	/// Checks if the graph has no name.
	pub fn is_unnamed(&self) -> bool {
		self.id.is_none()
	}

	/// Return an iterator to the nodes of the graph.
	pub fn nodes(&self) -> Nodes<T> {
		self.nodes.iter()
	}

	pub fn into_nodes(&self) -> HashSet<Indexed<Object<T>>> {
		self.nodes
	}

	/// Add a node to the graph.
	/// Return `true` if the node was not already in the graph,
	/// and `false` otherwise.
	pub fn insert(&mut self, node: Indexed<Object<T>>) -> bool {
		self.nodes.insert(node)
	}
}

impl<T: Id> Hash for Graph<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		self.id.hash(h);
		util::hash_set(&self.nodes, h)
	}
}

impl<T: Id> util::AsJson for Graph<T> {
	fn as_json(&self) -> JsonValue {
		let mut obj = json::object::Object::new();

		if let Some(id) = &self.id {
			obj.insert(Keyword::Id.into(), id.as_json());
		}

		obj.insert(Keyword::Graph.into(), self.nodes.as_json());

		JsonValue::Object(obj)
	}
}
