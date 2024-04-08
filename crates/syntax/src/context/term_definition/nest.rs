use std::hash::Hash;

use crate::is_keyword;

#[derive(Clone, PartialOrd, Ord, Debug)]
pub enum Nest {
	Nest,

	/// Must not be a keyword.
	Term(String),
}

impl Nest {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Nest => "@nest",
			Self::Term(t) => t,
		}
	}

	pub fn into_string(self) -> String {
		match self {
			Self::Nest => "@nest".to_string(),
			Self::Term(t) => t,
		}
	}
}

impl PartialEq for Nest {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Nest, Self::Nest) => true,
			(Self::Term(a), Self::Term(b)) => a == b,
			_ => false,
		}
	}
}

impl Eq for Nest {}

impl Hash for Nest {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

#[derive(Debug, thiserror::Error)]
#[error("invalid `@nest` value")]
pub struct InvalidNest(pub String);

impl TryFrom<String> for Nest {
	type Error = InvalidNest;

	fn try_from(s: String) -> Result<Self, InvalidNest> {
		if s == "@nest" {
			Ok(Self::Nest)
		} else if is_keyword(&s) {
			Err(InvalidNest(s))
		} else {
			Ok(Self::Term(s))
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Nest {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_str().serialize(serializer)
	}
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Nest {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Nest;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("`@nest` value")
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
				Nest::try_from(v).map_err(|e| E::custom(e))
			}
		}

		deserializer.deserialize_string(Visitor)
	}
}
