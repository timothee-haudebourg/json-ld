use crate::{object::LiteralString, util::AsAnyJson, Direction};
use derivative::Derivative;
use generic_json::{Json, JsonBuild};
use langtag::{LanguageTag, LanguageTagBuf};
use std::fmt;

/// Language tag buffer that may not be well-formed.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LenientLanguageTagBuf {
	WellFormed(LanguageTagBuf),
	Malformed(String),
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
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LenientLanguageTag<'a> {
	WellFormed(LanguageTag<'a>),
	Malformed(&'a str),
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

	pub fn cloned(&self) -> LenientLanguageTagBuf {
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

impl<'a, K: JsonBuild> AsAnyJson<K> for LenientLanguageTag<'a> {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		AsAnyJson::<K>::as_json_with(self.as_str(), meta)
	}
}

impl<K: JsonBuild> AsAnyJson<K> for LenientLanguageTagBuf {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		AsAnyJson::<K>::as_json_with(self.as_str(), meta)
	}
}

/// Language string.
///
/// A language string is a string tagged with language and reading direction information.
///
/// A valid language string is associated to either a language tag or a direction, or both.
#[derive(Derivative)]
#[derivative(
	Clone(bound = "J::String: Clone"),
	PartialEq(bound = ""),
	Eq(bound = ""),
	Hash(bound = ""),
	Debug(bound = "")
)]
pub struct LangString<J: Json> {
	/// Actual content of the string.
	data: LiteralString<J>,
	language: Option<LenientLanguageTagBuf>,
	direction: Option<Direction>,
}

/// Raised when something tried to build a language string without language tag or direction.
#[derive(Clone, Copy, Debug)]
pub struct InvalidLangString;

impl<J: Json> LangString<J> {
	/// Create a new language string.
	pub fn new(
		str: LiteralString<J>,
		language: Option<LenientLanguageTagBuf>,
		direction: Option<Direction>,
	) -> Result<Self, LiteralString<J>> {
		if language.is_some() || direction.is_some() {
			Ok(Self {
				data: str,
				language,
				direction,
			})
		} else {
			Err(str)
		}
	}

	pub fn into_parts(
		self,
	) -> (
		LiteralString<J>,
		Option<LenientLanguageTagBuf>,
		Option<Direction>,
	) {
		(self.data, self.language, self.direction)
	}

	pub fn parts(
		&self,
	) -> (
		&LiteralString<J>,
		Option<&LenientLanguageTagBuf>,
		Option<&Direction>,
	) {
		(&self.data, self.language.as_ref(), self.direction.as_ref())
	}

	/// Reference to the underlying `str`.
	#[inline(always)]
	pub fn as_string(&self) -> &LiteralString<J> {
		&self.data
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
