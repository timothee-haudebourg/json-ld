use rdf_types::BlankIdBuf;
use std::fmt;

/// Warning that can occur during JSON-LD documents processing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Warning {
	/// Some node has an empty term.
	///
	/// The use of empty terms is not allowed as not all
	/// programming languages are able to handle empty JSON keys.
	EmptyTerm,

	/// Blank node identifier is used as property.
	///
	/// The use of blank node identifiers to label properties is obsolete,
	/// and may be removed in a future version of JSON-LD.
	BlankNodeIdProperty(BlankIdBuf),

	/// Term as the form of a keyword.
	KeywordLikeTerm(String),

	/// Value as the form of a keyword.
	KeywordLikeValue(String),

	/// Language tag is not well-formed.
	MalformedLanguageTag(String, langtag::Error),

	/// String literal is not an IRI.
	MalformedIri(String),
}

impl fmt::Display for Warning {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::EmptyTerm => write!(f, "empty term"),
			Self::BlankNodeIdProperty(id) => {
				write!(f, "blank node identifier `{}` is used as property", id)
			}
			Self::KeywordLikeTerm(term) => write!(f, "term `{}` has the form of a keyword", term),
			Self::KeywordLikeValue(value) => {
				write!(f, "value `{}` has the form of a keyword", value)
			}
			Self::MalformedLanguageTag(tag, e) => {
				write!(f, "malformed language tag `{}`: {}", tag, e)
			}
			Self::MalformedIri(value) => write!(f, "invalid IRI `{}`", value),
		}
	}
}
