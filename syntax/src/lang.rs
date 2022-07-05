use crate::Direction;
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

/// Language string.
///
/// A language string is a string tagged with language and reading direction information.
///
/// A valid language string is associated to either a language tag or a direction, or both.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct LangString {
	/// Actual content of the string.
	data: String,
	language: Option<LenientLanguageTagBuf>,
	direction: Option<Direction>,
}

/// Raised when something tried to build a language string without language tag or direction.
#[derive(Clone, Copy, Debug)]
pub struct InvalidLangString;

impl LangString {
	/// Create a new language string.
	pub fn new(
		data: String,
		language: Option<LenientLanguageTagBuf>,
		direction: Option<Direction>,
	) -> Result<Self, String> {
		if language.is_some() || direction.is_some() {
			Ok(Self {
				data,
				language,
				direction,
			})
		} else {
			Err(data)
		}
	}

	pub fn into_parts(
		self,
	) -> (
		String,
		Option<LenientLanguageTagBuf>,
		Option<Direction>,
	) {
		(self.data, self.language, self.direction)
	}

	pub fn parts(
		&self,
	) -> (
		&str,
		Option<&LenientLanguageTagBuf>,
		Option<&Direction>,
	) {
		(&self.data, self.language.as_ref(), self.direction.as_ref())
	}

	/// Reference to the underlying `str`.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		self.data.as_ref()
	}

	/// Gets the associated language tag, if any.
	#[inline(always)]
	pub fn language(&self) -> Option<LenientLanguageTag> {
		self.language.as_ref().map(|tag| tag.as_ref())
	}

	/// Sets the associated language tag.
	///
	/// If `None` is given, the direction must be set,
	/// otherwise this function will fail with an [`InvalidLangString`] error.
	pub fn set_language(
		&mut self,
		language: Option<LenientLanguageTagBuf>,
	) -> Result<(), InvalidLangString> {
		if self.direction.is_some() || language.is_some() {
			self.language = language;
			Ok(())
		} else {
			Err(InvalidLangString)
		}
	}

	/// Gets the associated direction, if any.
	#[inline(always)]
	pub fn direction(&self) -> Option<Direction> {
		self.direction
	}

	/// Sets the associated direction.
	///
	/// If `None` is given, a language tag must be set,
	/// otherwise this function will fail with an [`InvalidLangString`] error.
	pub fn set_direction(&mut self, direction: Option<Direction>) -> Result<(), InvalidLangString> {
		if direction.is_some() || self.language.is_some() {
			self.direction = direction;
			Ok(())
		} else {
			Err(InvalidLangString)
		}
	}

	/// Set both the language tag and direction.
	///
	/// If both `language` and `direction` are `None`,
	/// this function will fail with an [`InvalidLangString`] error.
	pub fn set(
		&mut self,
		language: Option<LenientLanguageTagBuf>,
		direction: Option<Direction>,
	) -> Result<(), InvalidLangString> {
		if direction.is_some() || language.is_some() {
			self.language = language;
			self.direction = direction;
			Ok(())
		} else {
			Err(InvalidLangString)
		}
	}
}
