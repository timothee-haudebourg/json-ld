use std::fmt;

use crate::{context::definition::KeyOrKeyword, Keyword};

pub enum Expandable {
	Keyword(Keyword),
	String(String),
}

pub enum ExpandableRef<'a> {
	/// Keyword.
	Keyword(Keyword),

	/// Other term.
	String(&'a str),
}

// impl<'a> From<KeyOrKeywordRef<'a>> for ExpandableRef<'a> {
// 	fn from(k: KeyOrKeywordRef<'a>) -> Self {
// 		match k {
// 			KeyOrKeywordRef::Keyword(k) => Self::Keyword(k),
// 			KeyOrKeywordRef::Key(k) => Self::String(k.as_str()),
// 		}
// 	}
// }

impl<'a> From<&'a KeyOrKeyword> for ExpandableRef<'a> {
	fn from(k: &'a KeyOrKeyword) -> Self {
		match k {
			KeyOrKeyword::Keyword(k) => Self::Keyword(*k),
			KeyOrKeyword::Key(k) => Self::String(k.as_str()),
		}
	}
}

impl<'a> From<&'a str> for ExpandableRef<'a> {
	fn from(s: &'a str) -> Self {
		match Keyword::try_from(s) {
			Ok(k) => Self::Keyword(k),
			Err(_) => Self::String(s),
		}
	}
}

impl<'a> fmt::Display for ExpandableRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Keyword(k) => k.fmt(f),
			Self::String(s) => s.fmt(f),
		}
	}
}
