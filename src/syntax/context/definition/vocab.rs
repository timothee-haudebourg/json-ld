use crate::syntax::{JsonLdCompactIri, JsonLdExpandableRef};
use rdf_syntax::BlankId;
use rdf_syntax::Iri;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct JsonLdVocab(String);

impl JsonLdVocab {
	pub fn as_iri(&self) -> Option<&Iri> {
		Iri::new(&self.0).ok()
	}

	pub fn as_compact_iri(&self) -> Option<&JsonLdCompactIri> {
		JsonLdCompactIri::new(&self.0).ok()
	}

	pub fn as_blank_id(&self) -> Option<&BlankId> {
		BlankId::new(&self.0).ok()
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

impl From<String> for JsonLdVocab {
	fn from(s: String) -> Self {
		Self(s)
	}
}

impl<'a> From<&'a JsonLdVocab> for JsonLdExpandableRef<'a> {
	fn from(v: &'a JsonLdVocab) -> Self {
		JsonLdExpandableRef::String(&v.0)
	}
}
