use std::collections::HashMap;

use indexmap::IndexSet;
use rdf_types::{BlankId, BlankIdBuf, Generator};

use crate::{object::ListObject, Id, Indexed, IndexedObject, NodeObject, Object, ValidId};

use super::{ConflictingIndexes, NodeMap};

pub struct NodeMapBuilder<G> {
	substitution: Substitution<G>,
	result: NodeMap,
}

impl<G> NodeMapBuilder<G> {
	pub fn new(generator: G) -> Self {
		Self {
			substitution: Substitution {
				generator,
				id_map: HashMap::new(),
			},
			result: NodeMap::new(),
		}
	}

	pub fn end(self) -> NodeMap {
		self.result
	}
}

impl<G: Generator> NodeMapBuilder<G> {
	// #[allow(clippy::should_implement_trait)]
	// pub fn next(&mut self) -> ValidId {
	// 	self.generator.next(self.vocabulary)
	// }

	/// Extends the `NodeMap` with the given `element` of an expanded JSON-LD document.
	pub fn extend_node_map(
		&mut self,
		element: &IndexedObject,
		active_graph: Option<&Id>,
	) -> Result<IndexedObject, ConflictingIndexes> {
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
					flat_list.push(self.extend_node_map(item, active_graph)?);
				}

				Ok(Indexed::new(
					Object::List(ListObject::new(flat_list)),
					element.index().map(ToOwned::to_owned),
				))
			}
			Object::Node(node) => {
				let flat_node =
					self.extend_node_map_from_node(node, element.index(), active_graph)?;
				Ok(flat_node.map_inner(Object::node))
			}
		}
	}

	pub fn extend_node_map_from_node(
		&mut self,
		node: &NodeObject,
		index: Option<&str>,
		active_graph: Option<&Id>,
	) -> Result<Indexed<NodeObject>, ConflictingIndexes> {
		let id = self.substitution.assign_node_id(node.id.as_ref());

		{
			let flat_node = self
				.result
				.graph_mut(active_graph)
				.unwrap()
				.declare_node(id.clone(), index)?;

			if let Some(entry) = node.types.as_deref() {
				flat_node.types = Some(
					entry
						.iter()
						.map(|ty| self.substitution.assign_node_id(Some(ty)))
						.collect(),
				);
			}
		}

		if let Some(graph_entry) = node.graph_entry() {
			self.result.declare_graph(id.clone());

			let mut flat_graph = IndexSet::new();
			for object in graph_entry.iter() {
				let flat_object = self.extend_node_map(object, Some(&id))?;
				flat_graph.insert(flat_object);
			}

			let flat_node = self
				.result
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
				self.extend_node_map_from_node(inode.inner(), inode.index(), active_graph)?;
			}
		}

		for (property, objects) in node.properties() {
			let mut flat_objects = Vec::new();
			for object in objects {
				let flat_object = self.extend_node_map(object, active_graph)?;
				flat_objects.push(flat_object);
			}
			self.result
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
					let flat_subject = self.extend_node_map_from_node(
						subject.inner(),
						subject.index(),
						active_graph,
					)?;

					let subject_id = flat_subject.id.as_ref().unwrap();

					let flat_subject = self
						.result
						.graph_mut(active_graph)
						.unwrap()
						.get_mut(subject_id)
						.unwrap();

					flat_subject.properties_mut().insert_unique(
						property.clone(),
						Indexed::unindexed(Object::node(NodeObject::new_with_id(Some(id.clone())))),
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

		Ok(Indexed::new(NodeObject::new_with_id(Some(id)), None))
	}
}

struct Substitution<G> {
	generator: G,
	id_map: HashMap<BlankIdBuf, ValidId>,
}

impl<G: Generator> Substitution<G> {
	pub fn assign(&mut self, blank_id: &BlankId) -> ValidId {
		match self.id_map.get(blank_id) {
			Some(id) => id.clone(),
			None => {
				let id = self.generator.next_id();
				self.id_map.insert(blank_id.to_owned(), id.clone());
				id
			}
		}
	}

	pub fn assign_node_id(&mut self, r: Option<&Id>) -> Id {
		match r {
			Some(Id::Valid(ValidId::BlankId(id))) => self.assign(id).into(),
			Some(r) => r.clone(),
			None => self.generator.next_id().into(),
		}
	}
}
