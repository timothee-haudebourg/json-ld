use std::convert::TryFrom;
use std::fmt;

use rdf_syntax::{Id, Iri, IriBuf};

use super::JsonLdTerm;
use crate::{JsonLdLenient, syntax::JsonLdKeyword};

/// Object type.
///
/// This is the value of a `@type` entry.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum JsonLdType {
	/// `@id`.
	///
	/// The value must be interpreted as an IRI.
	Id,

	/// `@json`.
	///
	/// The value is an arbitrary JSON value.
	Json,

	/// `@none`
	None,

	/// `@vocab`
	Vocab,

	/// IRI type.
	Iri(IriBuf),
}

impl JsonLdType {
	/// Turns this type into an IRI if possible.
	pub fn into_iri(self) -> Result<IriBuf, Self> {
		match self {
			JsonLdType::Iri(id) => Ok(id),
			typ => Err(typ),
		}
	}

	/// Returns a reference to the type IRI if any.
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			JsonLdType::Iri(id) => Some(id),
			_ => None,
		}
	}

	/// Returns a JSON-LD string representation of the type.
	pub fn as_str(&self) -> &str {
		match self {
			JsonLdType::Id => "@id",
			JsonLdType::Json => "@json",
			JsonLdType::None => "@none",
			JsonLdType::Vocab => "@vocab",
			JsonLdType::Iri(id) => id.as_ref(),
		}
	}
}

impl From<JsonLdType> for JsonLdTerm {
	fn from(t: JsonLdType) -> JsonLdTerm {
		match t {
			JsonLdType::Id => JsonLdTerm::Keyword(JsonLdKeyword::Id),
			JsonLdType::Json => JsonLdTerm::Keyword(JsonLdKeyword::Json),
			JsonLdType::None => JsonLdTerm::Keyword(JsonLdKeyword::None),
			JsonLdType::Vocab => JsonLdTerm::Keyword(JsonLdKeyword::Vocab),
			JsonLdType::Iri(id) => JsonLdTerm::Id(JsonLdLenient::Valid(Id::Iri(id))),
		}
	}
}

impl TryFrom<JsonLdTerm> for JsonLdType {
	type Error = JsonLdTerm;

	fn try_from(term: JsonLdTerm) -> Result<JsonLdType, JsonLdTerm> {
		match term {
			JsonLdTerm::Keyword(JsonLdKeyword::Id) => Ok(JsonLdType::Id),
			JsonLdTerm::Keyword(JsonLdKeyword::Json) => Ok(JsonLdType::Json),
			JsonLdTerm::Keyword(JsonLdKeyword::None) => Ok(JsonLdType::None),
			JsonLdTerm::Keyword(JsonLdKeyword::Vocab) => Ok(JsonLdType::Vocab),
			JsonLdTerm::Id(JsonLdLenient::Valid(Id::Iri(id))) => Ok(JsonLdType::Iri(id)),
			term => Err(term),
		}
	}
}

impl fmt::Display for JsonLdType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
