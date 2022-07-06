use rdf_types::BlankIdBuf;
use json_ld_context_processing::syntax::MalformedIri;

pub enum Warning {
	MalformedIri(String),
	EmptyTerm,
	BlankNodeIdProperty(BlankIdBuf)
}

impl From<MalformedIri> for Warning {
	fn from(MalformedIri(s): MalformedIri) -> Self {
		Self::MalformedIri(s)
	}
}