use std::convert::TryFrom;
use std::fmt;

use rdf_syntax::{Id, Iri, IriBuf};

use super::JsonLdTerm;
use crate::{JsonLdLenient, syntax::Keyword};

/// Object type.
///
/// This is the value of a `@type` entry.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type {
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

impl Type {
	/// Turns this type into an IRI if possible.
	pub fn into_iri(self) -> Result<IriBuf, Self> {
		match self {
			Type::Iri(id) => Ok(id),
			typ => Err(typ),
		}
	}

	/// Returns a reference to the type IRI if any.
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			Type::Iri(id) => Some(id),
			_ => None,
		}
	}

	/// Returns a JSON-LD string representation of the type.
	pub fn as_str(&self) -> &str {
		match self {
			Type::Id => "@id",
			Type::Json => "@json",
			Type::None => "@none",
			Type::Vocab => "@vocab",
			Type::Iri(id) => id.as_ref(),
		}
	}
}

impl From<Type> for JsonLdTerm {
	fn from(t: Type) -> JsonLdTerm {
		match t {
			Type::Id => JsonLdTerm::Keyword(Keyword::Id),
			Type::Json => JsonLdTerm::Keyword(Keyword::Json),
			Type::None => JsonLdTerm::Keyword(Keyword::None),
			Type::Vocab => JsonLdTerm::Keyword(Keyword::Vocab),
			Type::Iri(id) => JsonLdTerm::Id(JsonLdLenient::Valid(Id::Iri(id))),
		}
	}
}

impl TryFrom<JsonLdTerm> for Type {
	type Error = JsonLdTerm;

	fn try_from(term: JsonLdTerm) -> Result<Type, JsonLdTerm> {
		match term {
			JsonLdTerm::Keyword(Keyword::Id) => Ok(Type::Id),
			JsonLdTerm::Keyword(Keyword::Json) => Ok(Type::Json),
			JsonLdTerm::Keyword(Keyword::None) => Ok(Type::None),
			JsonLdTerm::Keyword(Keyword::Vocab) => Ok(Type::Vocab),
			JsonLdTerm::Id(JsonLdLenient::Valid(Id::Iri(id))) => Ok(Type::Iri(id)),
			term => Err(term),
		}
	}
}

impl fmt::Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
