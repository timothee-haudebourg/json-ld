use linked_data::{DeserializeLinkedData, LinkedDataDeserializer};
use rdf_syntax::{Quad, Term, pattern::CanonicalQuadPattern};

use crate::{ExpandedDocument, Indexed};

mod object;

use object::*;

impl<R> DeserializeLinkedData<R> for ExpandedDocument
where
	R: Clone + Ord,
{
	fn deserialize_rdf<D>(mut deserializer: D, graph: Option<&R>) -> Result<Self, D::Error>
	where
		D: LinkedDataDeserializer<R>,
	{
		let mut result = Self::new();

		while let Some(Quad(subject, _, _, _)) =
			deserializer.peek_quad(CanonicalQuadPattern::from_graph(graph))?
		{
			let subject = subject.into_owned();
			result.insert(Indexed::unindexed(deserialize_object(
				&mut deserializer,
				&subject,
				graph,
			)?));
		}

		Ok(result)
	}
}

impl<T: DeserializeLinkedData> DeserializeLinkedData for Indexed<T> {
	fn deserialize_rdf<D>(deserializer: D, graph: Option<&Term>) -> Result<Self, D::Error>
	where
		D: LinkedDataDeserializer<Term>,
	{
		T::deserialize_rdf(deserializer, graph).map(Self::unindexed)
	}
}
