use linked_data::LinkedDataDeserializer;
use rdf_syntax::{Quad, RDF_TYPE, pattern::CanonicalQuadPattern};

use crate::{Indexed, NodeObject};

use super::{deserialize_object, deserialize_object_ref};

pub fn deserialize_node_object<R, D>(
	deserializer: &mut D,
	subject: &R,
	graph: Option<&R>,
) -> Result<NodeObject, D::Error>
where
	R: Clone + PartialEq,
	D: LinkedDataDeserializer<R>,
{
	let mut result = NodeObject::new_with_id(
		deserialize_resource_id_opt(deserializer, subject)?.map(Into::into),
	);

	// Extract graph.
	if graph.is_none() {
		while let Some(Quad(graph_subject, _, _, _)) =
			deserializer.peek_quad(CanonicalQuadPattern::from_graph(Some(subject)))?
		{
			let graph_subject = graph_subject.into_owned();

			result
				.graph
				.get_or_insert_default()
				.insert(Indexed::unindexed(deserialize_object(
					deserializer,
					&graph_subject,
					Some(subject),
				)?));
		}
	}

	// Extract properties.
	while let Some(Quad(_, property, object, _)) =
		deserializer.deserialize_quad(Quad(Some(subject), None, None, Some(graph)))?
	{
		let property = deserialize_resource_id(deserializer, &property)?;

		if property == RDF_TYPE {
			result
				.types
				.get_or_insert_default()
				.push(deserialize_resource_id(deserializer, &object)?.into());
		}

		result.properties.insert(
			property,
			Indexed::unindexed(deserialize_object_ref(deserializer, &object, graph)?),
		);
	}

	Ok(result)
}

pub fn deserialize_node_object_ref<R, D>(
	deserializer: &mut D,
	subject: &R,
) -> Result<NodeObject, D::Error>
where
	R: ToOwned,
	D: LinkedDataDeserializer<R>,
{
	Ok(NodeObject::new_with_id(
		deserialize_resource_id_opt(deserializer, subject)?.map(Into::into),
	))
}

fn deserialize_resource_id_opt<R, D>(
	deserializer: &mut D,
	subject: &R,
) -> Result<Option<rdf_syntax::Id>, D::Error>
where
	R: ToOwned,
	D: LinkedDataDeserializer<R>,
{
	let mut id = None;

	for term in deserializer.terms_of(subject) {
		if let Ok(i) = term?.into_id()
			&& let Some(other) = id.replace(i.into_owned())
				&& Some(other) != id {
					return Err(linked_data::de::Error::custom("ambiguous id"));
				}
	}

	Ok(id)
}

fn deserialize_resource_id<R, D>(
	deserializer: &mut D,
	subject: &R,
) -> Result<rdf_syntax::Id, D::Error>
where
	R: ToOwned,
	D: LinkedDataDeserializer<R>,
{
	deserialize_resource_id_opt(deserializer, subject)?
		.ok_or_else(|| linked_data::de::Error::custom("missing required id"))
}
