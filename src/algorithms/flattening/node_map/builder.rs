use std::collections::HashMap;

use btree_indexmap::BTreeIndexSet;
use rdf_syntax::{BlankId, BlankIdBuf, Generator, Id};

use crate::{
	Indexed, IndexedObject, Lenient, NodeObject, Object, algorithms::JsonLdLocationStack,
	object::ListObject, syntax::Keyword,
};

use super::{NodeMap, NodeMapExtendError};

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
	// pub fn next(&mut self) -> Id {
	// 	self.generator.next(self.vocabulary)
	// }

	/// Extends the `NodeMap` with the given `element` of an expanded JSON-LD document.
	///
	/// `location` tracks the current position within the expanded document.
	pub fn extend_node_map(
		&mut self,
		element: &IndexedObject,
		active_graph: Option<&Lenient<Id>>,
		location: JsonLdLocationStack<'_>,
	) -> Result<IndexedObject, NodeMapExtendError> {
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
				let list_loc = location.object_value(Keyword::List);

				for (i, item) in list.iter().enumerate() {
					flat_list.push(self.extend_node_map(
						item,
						active_graph,
						list_loc.array_index(i),
					)?);
				}

				Ok(Indexed::new(
					Object::List(ListObject::new(flat_list)),
					element.index().map(ToOwned::to_owned),
				))
			}
			Object::Node(node) => {
				let flat_node =
					self.extend_node_map_from_node(node, element.index(), active_graph, location)?;
				Ok(flat_node.map_inner(Object::node))
			}
		}
	}

	pub fn extend_node_map_from_node(
		&mut self,
		node: &NodeObject,
		index: Option<&str>,
		active_graph: Option<&Lenient<Id>>,
		location: JsonLdLocationStack<'_>,
	) -> Result<Indexed<NodeObject>, NodeMapExtendError> {
		let id = self.substitution.assign_node_id(node.id.as_ref());

		{
			// The `@index` value is at `location[ObjectValue("@index")]`.
			let index_location = index.map(|_| location.object_value(Keyword::Index).build());
			let flat_node = self.result.graph_mut(active_graph).unwrap().declare_node(
				id.clone(),
				index.map(|s| (s, index_location.unwrap_or_default())),
			)?;

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

			let graph_loc = location.object_value(Keyword::Graph);
			let mut flat_graph = BTreeIndexSet::new();
			for (i, object) in graph_entry.iter().enumerate() {
				let flat_object =
					self.extend_node_map(object, Some(&id), graph_loc.array_index(i))?;
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
			let included_loc = location.object_value(Keyword::Included);
			for (i, inode) in included_entry.iter().enumerate() {
				self.extend_node_map_from_node(
					inode.inner(),
					inode.index(),
					active_graph,
					included_loc.array_index(i),
				)?;
			}
		}

		for (property, objects) in node.properties() {
			let prop_loc = location.object_value(property.as_str());
			let mut flat_objects = Vec::new();
			for (i, object) in objects.iter().enumerate() {
				let flat_object =
					self.extend_node_map(object, active_graph, prop_loc.array_index(i))?;
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
			let reverse_loc = location.object_value(Keyword::Reverse);
			for (property, nodes) in reverse_properties.iter() {
				let prop_loc = reverse_loc.object_value(property.as_str());
				for (i, subject) in nodes.iter().enumerate() {
					let flat_subject = self.extend_node_map_from_node(
						subject.inner(),
						subject.index(),
						active_graph,
						prop_loc.array_index(i),
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
	id_map: HashMap<BlankIdBuf, Id>,
}

impl<G: Generator> Substitution<G> {
	pub fn assign(&mut self, blank_id: &BlankId) -> Id {
		match self.id_map.get(blank_id) {
			Some(id) => id.clone(),
			None => {
				let id = self.generator.next_id();
				self.id_map.insert(blank_id.to_owned(), id.clone());
				id
			}
		}
	}

	pub fn assign_node_id(&mut self, r: Option<&Lenient<Id>>) -> Lenient<Id> {
		match r {
			Some(Lenient::Valid(Id::BlankId(id))) => self.assign(id).into(),
			Some(r) => r.clone(),
			None => self.generator.next_id().into(),
		}
	}
}
