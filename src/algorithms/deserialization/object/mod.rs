use linked_data::{ser::SerializeLinkedDataWith, LinkedDataSerializer, SerializeLinkedData};
use rdf_syntax::Term;

use crate::{Indexed, Object};

use super::RdfSerializationOptions;

mod list;
mod node;
mod value;

impl SerializeLinkedDataWith<RdfSerializationOptions> for Object {
	fn serialize_rdf_with<S>(
		&self,
		opts: RdfSerializationOptions,
		serializer: S,
		graph: Option<&Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		match self {
			Self::Node(node) => node.serialize_rdf_with(opts, serializer, graph),
			Self::List(list) => list.serialize_rdf_with(opts, serializer, graph),
			Self::Value(value) => value.serialize_rdf_with(opts, serializer, graph),
		}
	}
}

impl SerializeLinkedData for Object {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		self.serialize_rdf_with(RdfSerializationOptions::default(), serializer, graph)
	}
}

impl<T: SerializeLinkedDataWith<RdfSerializationOptions>>
	SerializeLinkedDataWith<RdfSerializationOptions> for Indexed<T>
{
	fn serialize_rdf_with<S>(
		&self,
		opts: RdfSerializationOptions,
		serializer: S,
		graph: Option<&Term>,
	) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		self.inner().serialize_rdf_with(opts, serializer, graph)
	}
}

impl<T: SerializeLinkedData> SerializeLinkedData for Indexed<T> {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		self.inner().serialize_rdf(serializer, graph)
	}
}
