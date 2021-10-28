use crate::{object::LiteralString, Direction};
use derivative::Derivative;
use generic_json::Json;
use langtag::{LanguageTag, LanguageTagBuf};

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
	language: Option<LanguageTagBuf>,
	direction: Option<Direction>,
}

/// Raised when something tried to build a language string without language tag or direction.
#[derive(Clone, Copy, Debug)]
pub struct InvalidLangString;

impl<J: Json> LangString<J> {
	/// Create a new language string.
	pub fn new(
		str: LiteralString<J>,
		language: Option<LanguageTagBuf>,
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
	pub fn language(&self) -> Option<LanguageTag> {
		self.language.as_ref().map(|tag| tag.as_ref())
	}

	/// Sets the associated language tag.
	///
	/// If `None` is given, the direction must be set,
	/// otherwise this function will fail with an [`InvalidLangString`] error.
	pub fn set_language(
		&mut self,
		language: Option<LanguageTagBuf>,
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
		language: Option<LanguageTagBuf>,
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
