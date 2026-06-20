use linked_data::{DeserializeLinkedData, LinkedDataDeserializer};
use rdf_syntax::Term;

use crate::Object;

mod list;
mod node;
mod value;

use list::*;
use node::*;
use value::*;

impl DeserializeLinkedData for Object {
	fn deserialize_rdf<D>(mut deserializer: D, graph: Option<&Term>) -> Result<Self, D::Error>
	where
		D: LinkedDataDeserializer,
	{
		let subject = match deserializer.deserialize_resource(graph.cloned())? {
			Some(subject) => {
				if let Some(other) = deserializer.deserialize_resource(graph.cloned())? {
					return Err(linked_data::de::Error::unexpected_resource(&other));
				}

				subject
			}
			None => return Err(linked_data::de::Error::missing_resource()),
		};

		deserialize_object(&mut deserializer, &subject, graph)
	}
}

pub fn deserialize_object<R, D>(
	deserializer: &mut D,
	subject: &R,
	graph: Option<&R>,
) -> Result<Object, D::Error>
where
	R: Clone + PartialEq,
	D: LinkedDataDeserializer<R>,
{
	if let Some(value) = try_deserialize_value_object(deserializer, subject)? {
		return Ok(Object::Value(value));
	}

	if let Some(list) = try_deserialize_list_object(deserializer, subject, graph)? {
		return Ok(Object::List(list));
	}

	deserialize_node_object(deserializer, subject, graph).map(Object::node)
}

fn deserialize_object_ref<R, D>(
	deserializer: &mut D,
	subject: &R,
	graph: Option<&R>,
) -> Result<Object, D::Error>
where
	R: Clone + PartialEq,
	D: LinkedDataDeserializer<R>,
{
	if let Some(value) = try_deserialize_value_object(deserializer, subject)? {
		return Ok(Object::Value(value));
	}

	if let Some(list) = try_deserialize_list_object(deserializer, subject, graph)? {
		return Ok(Object::List(list));
	}

	deserialize_node_object_ref(deserializer, subject).map(Object::node)
}
