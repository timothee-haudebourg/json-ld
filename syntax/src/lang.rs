use langtag::{LanguageTag, LanguageTagBuf};
use std::fmt;

/// Language tag buffer that may not be well-formed.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum LenientLanguageTagBuf {
	WellFormed(LanguageTagBuf),
	Malformed(String),
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

	pub fn into_string(self) -> String {
		match self {
			Self::WellFormed(LanguageTagBuf::Normal(n)) => unsafe {
				String::from_utf8_unchecked(n.into_inner())
			},
			Self::WellFormed(LanguageTagBuf::PrivateUse(p)) => unsafe {
				String::from_utf8_unchecked(p.into_inner())
			},
			Self::WellFormed(LanguageTagBuf::Grandfathered(g)) => g.to_string(),
			Self::Malformed(s) => s,
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

#[cfg(feature = "serde")]
impl serde::Serialize for LenientLanguageTagBuf {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LenientLanguageTagBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = LenientLanguageTagBuf;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("JSON-LD version")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				self.visit_string(v.to_owned())
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(LenientLanguageTagBuf::new(v).0)
			}
		}

		deserializer.deserialize_string(Visitor)
	}
}
