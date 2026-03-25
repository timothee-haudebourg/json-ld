use linked_data::{LinkedDataSerializer, SerializeLinkedData};
use rdf_types::Term;

use crate::{Indexed, Object};

mod list;
mod node;
mod value;

impl SerializeLinkedData for Object {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		match self {
			Self::Node(node) => node.serialize_rdf(serializer, graph),
			Self::List(list) => list.serialize_rdf(serializer, graph),
			Self::Value(value) => value.serialize_rdf(serializer, graph),
		}
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
