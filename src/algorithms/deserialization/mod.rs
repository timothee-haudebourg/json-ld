use linked_data::{LinkedDataSerializer, SerializeLinkedData};
use rdf_types::Term;

use crate::{ExpandedDocument, Id};

mod object;

impl SerializeLinkedData for ExpandedDocument {
	fn serialize_rdf<S>(&self, mut serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		for object in self {
			object.serialize_rdf(&mut serializer, graph)?;
		}

		serializer.end()
	}
}

impl Id {
	pub fn serialize_rdf_term<S>(&self, serializer: &mut S) -> Result<Term, S::Error>
	where
		S: LinkedDataSerializer,
	{
		match self {
			Self::Valid(id) => Ok(id.clone().into()),
			Self::Invalid(_) => serializer.new_resource(),
		}
	}
}

impl SerializeLinkedData for Id {
	fn serialize_rdf<S>(&self, serializer: S, graph: Option<&Term>) -> Result<S::Ok, S::Error>
	where
		S: LinkedDataSerializer<Term>,
	{
		match self {
			Self::Valid(id) => id.serialize_rdf(serializer, graph),
			Self::Invalid(_) => serializer.end(),
		}
	}
}
