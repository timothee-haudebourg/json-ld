use linked_data::{DeserializeLinkedData, LinkedDataDeserializer};
use rdf_syntax::{Quad, Term};

use crate::{JsonLdIndexed, JsonLdExpandedDocument};

mod object;

use object::*;

impl<R> DeserializeLinkedData<R> for JsonLdExpandedDocument
where
	R: Clone + Ord,
{
	fn deserialize_rdf<D>(mut deserializer: D, graph: Option<&R>) -> Result<Self, D::Error>
	where
		D: LinkedDataDeserializer<R>,
	{
		let mut result = Self::new();

		while let Some(Quad(subject, _, _, _)) =
			deserializer.peek_quad(Quad(None, None, None, Some(graph)))?
		{
			let subject = subject.clone();
			result.insert(JsonLdIndexed::unindexed(deserialize_object(
				&mut deserializer,
				&subject,
				graph,
			)?));
		}

		Ok(result)
	}
}

impl<T: DeserializeLinkedData> DeserializeLinkedData for JsonLdIndexed<T> {
	fn deserialize_rdf<D>(deserializer: D, graph: Option<&Term>) -> Result<Self, D::Error>
	where
		D: LinkedDataDeserializer<Term>,
	{
		T::deserialize_rdf(deserializer, graph).map(Self::unindexed)
	}
}
