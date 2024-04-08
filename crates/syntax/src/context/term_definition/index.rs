use crate::CompactIri;
use iref::Iri;
use std::fmt;
use std::hash::Hash;

#[derive(Clone, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Index(String);

impl Index {
	pub fn as_iri(&self) -> Option<&Iri> {
		Iri::new(&self.0).ok()
	}

	pub fn as_compact_iri(&self) -> Option<&CompactIri> {
		CompactIri::new(&self.0).ok()
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

impl PartialEq for Index {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl fmt::Display for Index {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl Eq for Index {}

impl Hash for Index {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl From<String> for Index {
	fn from(s: String) -> Self {
		Self(s)
	}
}

// #[derive(Clone, Copy)]
// pub struct IndexRef<'a>(&'a str);

// impl<'a> IndexRef<'a> {
// 	pub fn to_owned(self) -> Index {
// 		Index(self.0.to_owned())
// 	}

// 	pub fn as_str(&self) -> &'a str {
// 		self.0
// 	}
// }

// impl<'a> From<&'a Index> for IndexRef<'a> {
// 	fn from(i: &'a Index) -> Self {
// 		Self(&i.0)
// 	}
// }

// impl<'a> From<IndexRef<'a>> for super::EntryKeyRef<'a> {
// 	fn from(i: IndexRef<'a>) -> Self {
// 		i.0.into()
// 	}
// }
