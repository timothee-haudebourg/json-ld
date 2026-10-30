use rdf_syntax::{Id, Iri, IriBuf};
use std::fmt;

use crate::{JsonLdLenient, syntax::Keyword};

/// Identifier, keyword or `@null`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdTerm {
	/// `@null` value.
	Null,

	/// Node identifier.
	Id(JsonLdLenient<Id>),

	/// Keyword.
	Keyword(Keyword),
}

impl JsonLdTerm {
	/// Checks if this term is `@null`.
	pub fn is_null(&self) -> bool {
		matches!(self, JsonLdTerm::Null)
	}

	/// Turns this term into an IRI if possible.
	///
	/// If it is not an IRI, returns the term itself.
	pub fn into_iri(self) -> Result<IriBuf, Self> {
		match self {
			JsonLdTerm::Id(JsonLdLenient::Valid(Id::Iri(id))) => Ok(id),
			term => Err(term),
		}
	}

	/// Checks if this term is a keyword.
	pub fn is_keyword(&self) -> bool {
		matches!(self, JsonLdTerm::Keyword(_))
	}

	/// Returns a reference to the IRI representation of the term, if any.
	pub fn as_iri(&self) -> Option<&Iri> {
		match self {
			JsonLdTerm::Id(p) => p.as_iri(),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			JsonLdTerm::Id(p) => p.as_str(),
			JsonLdTerm::Keyword(k) => k.into_str(),
			JsonLdTerm::Null => "",
		}
	}
}

impl From<IriBuf> for JsonLdTerm {
	fn from(id: IriBuf) -> JsonLdTerm {
		JsonLdTerm::Id(JsonLdLenient::Valid(Id::Iri(id)))
	}
}

impl From<JsonLdLenient<Id>> for JsonLdTerm {
	fn from(prop: JsonLdLenient<Id>) -> JsonLdTerm {
		JsonLdTerm::Id(prop)
	}
}

impl fmt::Display for JsonLdTerm {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}
