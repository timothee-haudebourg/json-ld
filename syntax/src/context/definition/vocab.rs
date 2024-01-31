use crate::{CompactIri, ExpandableRef};
use iref::Iri;
use rdf_types::BlankId;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Vocab(String);

impl Vocab {
	pub fn as_iri(&self) -> Option<&Iri> {
		Iri::new(&self.0).ok()
	}

	pub fn as_compact_iri(&self) -> Option<&CompactIri> {
		CompactIri::new(&self.0).ok()
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

impl From<String> for Vocab {
	fn from(s: String) -> Self {
		Self(s)
	}
}

impl<'a> From<&'a Vocab> for ExpandableRef<'a> {
	fn from(v: &'a Vocab) -> Self {
		ExpandableRef::String(&v.0)
	}
}
