use crate::{object::InvalidExpandedJson, Direction, LenientLanguageTag, LenientLanguageTagBuf};
use locspan::Meta;

/// Language string.
///
/// A language string is a string tagged with language and reading direction information.
///
/// A valid language string is associated to either a language tag or a direction, or both.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct LangString {
	/// Actual content of the string.
	data: json_ld_syntax::String,
	language: Option<LenientLanguageTagBuf>,
	direction: Option<Direction>,
}

/// Raised when something tried to build a language string without language tag or direction.
#[derive(Clone, Copy, Debug)]
pub struct InvalidLangString;

impl LangString {
	/// Create a new language string.
	pub fn new(
		data: json_ld_syntax::String,
		language: Option<LenientLanguageTagBuf>,
		direction: Option<Direction>,
	) -> Result<Self, json_ld_syntax::String> {
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
		json_ld_syntax::String,
		Option<LenientLanguageTagBuf>,
		Option<Direction>,
	) {
		(self.data, self.language, self.direction)
	}

	pub fn parts(&self) -> (&str, Option<&LenientLanguageTagBuf>, Option<&Direction>) {
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

	pub(crate) fn try_from_json<M>(
		object: json_syntax::Object<M>,
		value: Meta<json_syntax::Value<M>, M>,
		language: Option<Meta<json_syntax::Value<M>, M>>,
		direction: Option<Meta<json_syntax::Value<M>, M>>,
	) -> Result<Self, Meta<InvalidExpandedJson<M>, M>> {
		let data = match value {
			Meta(json_syntax::Value::String(s), _) => s,
			Meta(v, meta) => {
				return Err(Meta(
					InvalidExpandedJson::Unexpected(v.kind(), json_syntax::Kind::String),
					meta,
				))
			}
		};

		let language = match language {
			Some(Meta(json_syntax::Value::String(value), _)) => {
				let (tag, _) = LenientLanguageTagBuf::new(value.to_string());
				Some(tag)
			}
			Some(Meta(v, meta)) => {
				return Err(Meta(
					InvalidExpandedJson::Unexpected(v.kind(), json_syntax::Kind::String),
					meta,
				))
			}
			None => None,
		};

		let direction = match direction {
			Some(Meta(json_syntax::Value::String(value), meta)) => {
				match Direction::try_from(value.as_str()) {
					Ok(direction) => Some(direction),
					Err(_) => return Err(Meta(InvalidExpandedJson::InvalidDirection, meta)),
				}
			}
			Some(Meta(v, meta)) => {
				return Err(Meta(
					InvalidExpandedJson::Unexpected(v.kind(), json_syntax::Kind::String),
					meta,
				))
			}
			None => None,
		};

		match object.into_iter().next() {
			None => Ok(Self::new(data, language, direction).unwrap()),
			Some(entry) => Err(Meta(
				InvalidExpandedJson::UnexpectedEntry,
				entry.key.into_metadata(),
			)),
		}
	}
}
