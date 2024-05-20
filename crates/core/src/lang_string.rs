use crate::{object::InvalidExpandedJson, Direction, LenientLangTag, LenientLangTagBuf};

/// Language string.
///
/// A language string is a string tagged with language and reading direction information.
///
/// A valid language string is associated to either a language tag or a direction, or both.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LangString {
	/// Actual content of the string.
	#[cfg_attr(feature = "serde", serde(rename = "@value"))]
	data: json_ld_syntax::String,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@language", skip_serializing_if = "Option::is_none")
	)]
	language: Option<LenientLangTagBuf>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@direction", skip_serializing_if = "Option::is_none")
	)]
	direction: Option<Direction>,
}

/// Raised when something tried to build a language string without language tag or direction.
#[derive(Clone, Copy, Debug)]
pub struct InvalidLangString;

impl LangString {
	/// Create a new language string.
	pub fn new(
		data: json_ld_syntax::String,
		language: Option<LenientLangTagBuf>,
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
		Option<LenientLangTagBuf>,
		Option<Direction>,
	) {
		(self.data, self.language, self.direction)
	}

	pub fn parts(&self) -> (&str, Option<&LenientLangTagBuf>, Option<&Direction>) {
		(&self.data, self.language.as_ref(), self.direction.as_ref())
	}

	/// Reference to the underlying `str`.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		self.data.as_ref()
	}

	/// Gets the associated language tag, if any.
	#[inline(always)]
	pub fn language(&self) -> Option<&LenientLangTag> {
		self.language
			.as_ref()
			.map(|tag| tag.as_lenient_lang_tag_ref())
	}

	/// Sets the associated language tag.
	///
	/// If `None` is given, the direction must be set,
	/// otherwise this function will fail with an [`InvalidLangString`] error.
	pub fn set_language(
		&mut self,
		language: Option<LenientLangTagBuf>,
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
		language: Option<LenientLangTagBuf>,
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

	/// Returns a reference to this lang string as a [`LangStr`].
	pub fn as_lang_str(&self) -> LangStr {
		LangStr {
			data: &self.data,
			language: self.language.as_deref(),
			direction: self.direction,
		}
	}

	pub(crate) fn try_from_json(
		object: json_syntax::Object,
		value: json_syntax::Value,
		language: Option<json_syntax::Value>,
		direction: Option<json_syntax::Value>,
	) -> Result<Self, InvalidExpandedJson> {
		let data = match value {
			json_syntax::Value::String(s) => s,
			v => {
				return Err(InvalidExpandedJson::Unexpected(
					v.kind(),
					json_syntax::Kind::String,
				))
			}
		};

		let language = match language {
			Some(json_syntax::Value::String(value)) => {
				let (tag, _) = LenientLangTagBuf::new(value.to_string());
				Some(tag)
			}
			Some(v) => {
				return Err(InvalidExpandedJson::Unexpected(
					v.kind(),
					json_syntax::Kind::String,
				))
			}
			None => None,
		};

		let direction = match direction {
			Some(json_syntax::Value::String(value)) => match Direction::try_from(value.as_str()) {
				Ok(direction) => Some(direction),
				Err(_) => return Err(InvalidExpandedJson::InvalidDirection),
			},
			Some(v) => {
				return Err(InvalidExpandedJson::Unexpected(
					v.kind(),
					json_syntax::Kind::String,
				))
			}
			None => None,
		};

		match object.into_iter().next() {
			None => Ok(Self::new(data, language, direction).unwrap()),
			Some(_) => Err(InvalidExpandedJson::UnexpectedEntry),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LangString {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		#[derive(serde::Deserialize)]
		struct Value {
			#[serde(rename = "@value")]
			data: json_ld_syntax::String,
			#[serde(rename = "@language")]
			language: Option<LenientLangTagBuf>,
			#[serde(rename = "@direction")]
			direction: Option<Direction>,
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
pub struct LangStr<'a> {
	/// Actual content of the string.
	#[cfg_attr(feature = "serde", serde(rename = "@value"))]
	data: &'a str,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@language", skip_serializing_if = "Option::is_none")
	)]
	language: Option<&'a LenientLangTag>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@direction", skip_serializing_if = "Option::is_none")
	)]
	direction: Option<Direction>,
}

impl<'a> LangStr<'a> {
	/// Create a new language string reference.
	pub fn new(
		data: &'a str,
		language: Option<&'a LenientLangTag>,
		direction: Option<Direction>,
	) -> Result<Self, InvalidLangString> {
		if language.is_some() || direction.is_some() {
			Ok(Self {
				data,
				language,
				direction,
			})
		} else {
			Err(InvalidLangString)
		}
	}

	pub fn into_parts(self) -> (&'a str, Option<&'a LenientLangTag>, Option<Direction>) {
		(self.data, self.language, self.direction)
	}

	/// Reference to the underlying `str`.
	#[inline(always)]
	pub fn as_str(&self) -> &'a str {
		self.data
	}

	/// Gets the associated language tag, if any.
	#[inline(always)]
	pub fn language(&self) -> Option<&'a LenientLangTag> {
		self.language
	}

	/// Gets the associated direction, if any.
	#[inline(always)]
	pub fn direction(&self) -> Option<Direction> {
		self.direction
	}
}
