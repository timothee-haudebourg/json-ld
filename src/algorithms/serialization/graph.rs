use std::collections::{BTreeMap, HashMap, HashSet};

use linked_data::{
	de::LinkedDataSubjectDeserializer, DeserializeLinkedData, LinkedDataDeserializer,
};
use rdf_types::{pattern::CanonicalQuadPattern, Quad, Term};

use crate::{algorithms::serialization::object::ProtoNodeObject, object::Graph, Id, NodeObject};

use super::ProtoObject;

pub struct ProtoGraph(HashMap<Id, ProtoNodeObject>);

impl ProtoGraph {
	fn find_list_heads(&self) -> Vec<Id> {
		let mut usages: HashMap<&Id, usize> = HashMap::new();
		let mut head_candidates = HashSet::new();

		for (id, object) in &self.0 {
			let is_list = object.is_list();

			if object.is_list() {
				head_candidates.insert(id);
			}

			for b in object.referenced_nodes() {
				let count = usages.entry(b).or_default();
				*count += 1;

				if is_list {
					head_candidates.remove(b);
				}
			}
		}

		head_candidates
			.into_iter()
			.cloned()
			.filter(|id| {
				let object = self.0.get(id).unwrap();
				object.validate_list(&self.0, &usages)
			})
			.collect()
	}

	fn resolve(mut self) -> Graph {
		let mut list_objects = BTreeMap::new();

		for id in self.find_list_heads() {
			let object = self.0.remove(&id).unwrap();
			list_objects.insert(id, object.into_list(&mut self.0));
		}

		let mut result = Graph::new();

		// ...

		result
	}
}

impl DeserializeLinkedData for ProtoGraph {
	fn deserialize_rdf<D>(mut deserializer: D, graph: Option<&Term>) -> Result<Self, D::Error>
	where
		D: LinkedDataDeserializer<Term>,
	{
		let mut result = HashMap::new();

		while let Some(Quad(subject, _, _, _)) =
			deserializer.peek_quad(CanonicalQuadPattern::graph_any(graph))?
		{
			let subject = subject.clone();
			let object: ProtoObject = DeserializeLinkedData::deserialize_rdf(
				LinkedDataSubjectDeserializer::new(&mut deserializer, subject),
				graph,
			)?;

			if let ProtoObject::Node(node) = object {
				result.insert(node.id.clone(), node);
			}
		}

		Ok(Self(result))
	}
}
