use contextual::DisplayWithContext;
use json_ld_context_processing::syntax::MalformedIri;
use rdf_types::BlankIdVocabulary;
use std::fmt;

#[derive(Debug)]
pub enum Warning<B> {
	MalformedIri(String),
	EmptyTerm,
	BlankNodeIdProperty(B),
	MalformedLanguageTag(String, langtag::Error),
}

impl<B> From<MalformedIri> for Warning<B> {
	fn from(MalformedIri(s): MalformedIri) -> Self {
		Self::MalformedIri(s)
	}
}

impl<B: fmt::Display> fmt::Display for Warning<B> {
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

impl<B, N: BlankIdVocabulary<BlankId = B>> DisplayWithContext<N> for Warning<B> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::MalformedIri(s) => write!(f, "malformed IRI `{}`", s),
			Self::EmptyTerm => write!(f, "empty term"),
			Self::BlankNodeIdProperty(b) => {
				write!(
					f,
					"blank node identifier `{}` used as property",
					vocabulary.blank_id(b).unwrap()
				)
			}
			Self::MalformedLanguageTag(t, e) => write!(f, "invalid language tag `{}`: {}", t, e),
		}
	}
}
