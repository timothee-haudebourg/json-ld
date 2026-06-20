use linked_data::LinkedDataDeserializer;
use rdf_syntax::{CowGroundTerm, CowTerm};

use crate::ValueObject;

pub fn try_deserialize_value_object<R, D>(
	deserializer: &mut D,
	subject: &R,
) -> Result<Option<ValueObject>, D::Error>
where
	R: ToOwned,
	D: LinkedDataDeserializer<R>,
{
	let mut value = None;

	for term in deserializer.terms_of(subject) {
		if let CowTerm::Ground(CowGroundTerm::Literal(literal)) = term? {
			if let Some(other) = value.replace(literal.into_owned()) {
				if Some(other) != value {
					return Err(linked_data::de::Error::custom("ambiguous literal value"));
				}
			}
		}
	}

	Ok(value.map(Into::into))
}
