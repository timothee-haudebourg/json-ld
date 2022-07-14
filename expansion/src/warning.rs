use json_ld_context_processing::syntax::MalformedIri;
use rdf_types::BlankIdBuf;
use std::fmt;

#[derive(Debug)]
pub enum Warning {
	MalformedIri(String),
	EmptyTerm,
	BlankNodeIdProperty(BlankIdBuf),
	MalformedLanguageTag(String, langtag::Error),
}

impl From<MalformedIri> for Warning {
	fn from(MalformedIri(s): MalformedIri) -> Self {
		Self::MalformedIri(s)
	}
}

impl fmt::Display for Warning {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::MalformedIri(s) => write!(f, "malformed IRI `{}`", s),
			Self::EmptyTerm => write!(f, "empty term"),
			Self::BlankNodeIdProperty(b) => {
				write!(f, "blank node identifier `{}` used as property", b)
			}
			Self::MalformedLanguageTag(t, e) => write!(f, "invalid language tag `{}`: {}", t, e),
		}
	}
}
