use std::fmt;

use crate::syntax::context::JsonLdKeyOrKeyword;

use super::Keyword;

pub enum JsonLdExpandable {
	Keyword(Keyword),
	String(String),
}

pub enum JsonLdExpandableRef<'a> {
	/// Keyword.
	Keyword(Keyword),

	/// Other term.
	String(&'a str),
}

impl<'a> From<&'a JsonLdKeyOrKeyword> for JsonLdExpandableRef<'a> {
	fn from(k: &'a JsonLdKeyOrKeyword) -> Self {
		match k {
			JsonLdKeyOrKeyword::Keyword(k) => Self::Keyword(*k),
			JsonLdKeyOrKeyword::Key(k) => Self::String(k.as_str()),
		}
	}
}

impl<'a> From<&'a str> for JsonLdExpandableRef<'a> {
	fn from(s: &'a str) -> Self {
		match Keyword::try_from(s) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::String(s),
		}
	}
}

impl<'a> fmt::Display for JsonLdExpandableRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Keyword(k) => k.fmt(f),
			Self::String(s) => s.fmt(f),
		}
	}
}
