use langtag::{LanguageTag, LanguageTagBuf};
use locspan_derive::StrippedPartialEq;
use std::fmt;

/// Language tag buffer that may not be well-formed.
#[derive(Clone, PartialEq, StrippedPartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum LenientLanguageTagBuf {
	WellFormed(#[stripped] LanguageTagBuf),
	Malformed(#[stripped] String),
}

impl LenientLanguageTagBuf {
	pub fn new(s: String) -> (Self, Option<langtag::Error>) {
		match LanguageTagBuf::new(s.into_bytes()) {
			Ok(lang) => (Self::WellFormed(lang), None),
			Err((err, lang)) => (
				Self::Malformed(unsafe { String::from_utf8_unchecked(lang) }),
				Some(err),
			),
		}
	}

	pub fn is_well_formed(&self) -> bool {
		matches!(self, Self::WellFormed(_))
	}

	pub fn as_ref(&self) -> LenientLanguageTag<'_> {
		match self {
			Self::WellFormed(tag) => LenientLanguageTag::WellFormed(tag.as_ref()),
			Self::Malformed(tag) => LenientLanguageTag::Malformed(tag.as_ref()),
		}
	}

	pub fn as_language_tag(&self) -> Option<LanguageTag<'_>> {
		match self {
			Self::WellFormed(tag) => Some(tag.as_ref()),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::WellFormed(tag) => tag.as_str(),
			Self::Malformed(tag) => tag.as_str(),
		}
	}
}

impl From<LanguageTagBuf> for LenientLanguageTagBuf {
	fn from(tag: LanguageTagBuf) -> Self {
		Self::WellFormed(tag)
	}
}

impl From<String> for LenientLanguageTagBuf {
	fn from(tag: String) -> Self {
		Self::Malformed(tag)
	}
}

impl fmt::Display for LenientLanguageTagBuf {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::WellFormed(tag) => tag.fmt(f),
			Self::Malformed(tag) => tag.fmt(f),
		}
	}
}

/// Language tag that may not be well-formed.
#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, Hash, Debug)]
pub enum LenientLanguageTag<'a> {
	WellFormed(#[stripped] LanguageTag<'a>),
	Malformed(#[stripped] &'a str),
}

impl<'a> LenientLanguageTag<'a> {
	pub fn is_well_formed(&self) -> bool {
		matches!(self, Self::WellFormed(_))
	}

	pub fn as_language_tag(&self) -> Option<LanguageTag<'a>> {
		match self {
			Self::WellFormed(tag) => Some(*tag),
			_ => None,
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Self::WellFormed(tag) => tag.as_str(),
			Self::Malformed(tag) => tag,
		}
	}

	pub fn to_owned(self) -> LenientLanguageTagBuf {
		match self {
			Self::WellFormed(tag) => LenientLanguageTagBuf::WellFormed(tag.cloned()),
			Self::Malformed(tag) => LenientLanguageTagBuf::Malformed(tag.to_string()),
		}
	}
}

impl<'a> fmt::Display for LenientLanguageTag<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::WellFormed(tag) => tag.fmt(f),
			Self::Malformed(tag) => tag.fmt(f),
		}
	}
}
