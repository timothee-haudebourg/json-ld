use langtag::InvalidLangTag;
use rdf_types::BlankIdBuf;

#[derive(Debug, thiserror::Error)]
pub enum Warning {
	#[error("keyword-like term `{0}`")]
	KeywordLikeTerm(String),

	#[error("keyword-like value `{0}`")]
	KeywordLikeValue(String),

	#[error("malformed IRI `{0}`")]
	MalformedIri(String),

	#[error("empty term")]
	EmptyTerm,

	#[error("blank node identifier `{0}` used as property")]
	BlankNodeIdProperty(BlankIdBuf),

	#[error("invalid language tag `{0}`: {1}")]
	MalformedLanguageTag(String, InvalidLangTag<String>),
}

pub fn ignore_warnings(_: Warning) {}
