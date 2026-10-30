use json_syntax::JsonString;
use langtag::{LangTag, LangTagBuf};

use crate::{JsonLdDirection, JsonLdLenient};

/// Language string.
///
/// A language string is a string tagged with language and reading direction information.
///
/// A valid language string is associated to either a language tag or a direction, or both.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct JsonLdLangString {
	/// Actual content of the string.
	#[cfg_attr(feature = "serde", serde(rename = "@value"))]
	data: JsonString,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@language", skip_serializing_if = "Option::is_none")
	)]
	language: Option<JsonLdLenient<LangTagBuf>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@direction", skip_serializing_if = "Option::is_none")
	)]
	direction: Option<JsonLdDirection>,
}

/// Raised when something tried to build a language string without language tag or direction.
#[derive(Clone, Copy, Debug)]
pub struct InvalidJsonLdLangString;

impl JsonLdLangString {
	/// Create a new language string.
	pub fn new<T>(
		data: T,
		language: Option<JsonLdLenient<LangTagBuf>>,
		direction: Option<JsonLdDirection>,
	) -> Result<Self, T>
	where
		T: Into<JsonString>,
	{
		if language.is_some() || direction.is_some() {
			Ok(Self {
				data: data.into(),
				language,
				direction,
			})
		} else {
			Err(data)
		}
	}

	pub fn from_language(
		data: impl Into<JsonString>,
		language: impl Into<JsonLdLenient<LangTagBuf>>,
	) -> Self {
		Self {
			data: data.into(),
			language: Some(language.into()),
			direction: None,
		}
	}

	pub fn from_direction(data: impl Into<JsonString>, direction: JsonLdDirection) -> Self {
		Self {
			data: data.into(),
			language: None,
			direction: Some(direction),
		}
	}

	pub fn into_parts(
		self,
	) -> (
		JsonString,
		Option<JsonLdLenient<LangTagBuf>>,
		Option<JsonLdDirection>,
	) {
		(self.data, self.language, self.direction)
	}

	pub fn parts(
		&self,
	) -> (
		&str,
		Option<&JsonLdLenient<LangTagBuf>>,
		Option<&JsonLdDirection>,
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
	pub fn language(&self) -> Option<JsonLdLenient<&LangTag>> {
		self.language.as_ref().map(|tag| tag.as_deref())
	}

	/// Sets the associated language tag.
	///
	/// If `None` is given, the direction must be set,
	/// otherwise this function will fail with an [`InvalidLangString`] error.
	pub fn set_language(
		&mut self,
		language: Option<JsonLdLenient<LangTagBuf>>,
	) -> Result<(), InvalidJsonLdLangString> {
		if self.direction.is_some() || language.is_some() {
			self.language = language;
			Ok(())
		} else {
			Err(InvalidJsonLdLangString)
		}
	}

	/// Gets the associated direction, if any.
	#[inline(always)]
	pub fn direction(&self) -> Option<JsonLdDirection> {
		self.direction
	}

	/// Sets the associated direction.
	///
	/// If `None` is given, a language tag must be set,
	/// otherwise this function will fail with an [`InvalidLangString`] error.
	pub fn set_direction(
		&mut self,
		direction: Option<JsonLdDirection>,
	) -> Result<(), InvalidJsonLdLangString> {
		if direction.is_some() || self.language.is_some() {
			self.direction = direction;
			Ok(())
		} else {
			Err(InvalidJsonLdLangString)
		}
	}

	/// Set both the language tag and direction.
	///
	/// If both `language` and `direction` are `None`,
	/// this function will fail with an [`InvalidLangString`] error.
	pub fn set(
		&mut self,
		language: Option<JsonLdLenient<LangTagBuf>>,
		direction: Option<JsonLdDirection>,
	) -> Result<(), InvalidJsonLdLangString> {
		if direction.is_some() || language.is_some() {
			self.language = language;
			self.direction = direction;
			Ok(())
		} else {
			Err(InvalidJsonLdLangString)
		}
	}

	/// Returns a reference to this lang string as a [`LangStr`].
	pub fn as_lang_str(&self) -> JsonLdLangStr<'_> {
		JsonLdLangStr {
			data: &self.data,
			language: self.language.as_ref().map(JsonLdLenient::as_deref),
			direction: self.direction,
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for JsonLdLangString {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		#[derive(serde::Deserialize)]
		struct Value {
			#[serde(rename = "@value")]
			data: JsonString,
			#[serde(rename = "@language")]
			language: Option<JsonLdLenient<LangTagBuf>>,
			#[serde(rename = "@direction")]
			direction: Option<JsonLdDirection>,
		}

		let value = Value::deserialize(deserializer)?;

		Self::new(value.data, value.language, value.direction).map_err(serde::de::Error::custom)
	}
}

/// Language string reference.
///
/// A language string is a string tagged with language and reading direction information.
///
/// A valid language string is associated to either a language tag or a direction, or both.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct JsonLdLangStr<'a> {
	/// Actual content of the string.
	#[cfg_attr(feature = "serde", serde(rename = "@value"))]
	data: &'a str,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@language", skip_serializing_if = "Option::is_none")
	)]
	language: Option<JsonLdLenient<&'a LangTag>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@direction", skip_serializing_if = "Option::is_none")
	)]
	direction: Option<JsonLdDirection>,
}

impl<'a> JsonLdLangStr<'a> {
	/// Create a new language string reference.
	pub fn new(
		data: &'a str,
		language: Option<JsonLdLenient<&'a LangTag>>,
		direction: Option<JsonLdDirection>,
	) -> Result<Self, InvalidJsonLdLangString> {
		if language.is_some() || direction.is_some() {
			Ok(Self {
				data,
				language,
				direction,
			})
		} else {
			Err(InvalidJsonLdLangString)
		}
	}

	pub fn into_parts(
		self,
	) -> (
		&'a str,
		Option<JsonLdLenient<&'a LangTag>>,
		Option<JsonLdDirection>,
	) {
		(self.data, self.language, self.direction)
	}

	/// Reference to the underlying `str`.
	#[inline(always)]
	pub fn as_str(&self) -> &'a str {
		self.data
	}

	/// Gets the associated language tag, if any.
	#[inline(always)]
	pub fn language(&self) -> Option<JsonLdLenient<&'a LangTag>> {
		self.language
	}

	/// Gets the associated direction, if any.
	#[inline(always)]
	pub fn direction(&self) -> Option<JsonLdDirection> {
		self.direction
	}
}
