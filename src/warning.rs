use crate::BlankId;

/// Warning that can occur during JSON-LD documents processing.
#[derive(Clone)]
pub enum Warning {
	/// The use of empty terms is not allowed as not all
	/// programming languages are able to handle empty JSON keys.
	EmptyKey,

	/// The use of blank node identifiers to label properties is obsolete,
	/// and may be removed in a future version of JSON-LD.
	BlankNodeIdProperty(BlankId),
	KeywordLikeTerm(String),
	KeywordLikeValue(String),
	MalformedLanguageTag(String, langtag::Error),
	MalformedIri(String)
}