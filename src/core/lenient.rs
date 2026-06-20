use std::{convert::Infallible, fmt, ops::Deref, str::FromStr};

/// String-like value that can be valid or invalid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Lenient<T> {
	Valid(T),
	Invalid(String),
}

impl<T> Lenient<T> {
	pub fn new(value: &str) -> (Self, Option<T::Err>)
	where
		T: FromStr,
	{
		match T::from_str(value) {
			Ok(t) => (Self::Valid(t), None),
			Err(e) => (Self::Invalid(value.to_owned()), Some(e)),
		}
	}

	pub fn from_string(value: String) -> (Self, Option<T::Err>)
	where
		T: FromStr,
	{
		match T::from_str(&value) {
			Ok(t) => (Self::Valid(t), None),
			Err(e) => (Self::Invalid(value), Some(e)),
		}
	}

	/// Checks if this is a valid reference.
	///
	/// Returns `true` is this reference is a node identifier or a blank node identifier,
	/// `false` otherwise.
	#[inline(always)]
	pub fn is_valid(&self) -> bool {
		matches!(self, Self::Valid(_))
	}

	pub fn is_valid_and(&self, f: impl FnOnce(&T) -> bool) -> bool {
		match self {
			Self::Valid(t) => f(t),
			Self::Invalid(_) => false,
		}
	}

	pub fn is_invalid(&self) -> bool {
		matches!(self, Self::Invalid(_))
	}

	pub fn as_deref(&self) -> LenientRef<'_, &T::Target>
	where
		T: Deref,
	{
		match self {
			Self::Valid(t) => LenientRef::Valid(t),
			Self::Invalid(s) => LenientRef::Invalid(s),
		}
	}

	pub fn as_str(&self) -> &str
	where
		T: AsRef<str>,
	{
		match self {
			Self::Valid(t) => t.as_ref(),
			Self::Invalid(s) => s,
		}
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Lenient<U> {
		match self {
			Self::Valid(id) => Lenient::Valid(f(id)),
			Self::Invalid(id) => Lenient::Invalid(id),
		}
	}
}

impl<T: AsRef<str>> AsRef<str> for Lenient<T> {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<T: fmt::Display> fmt::Display for Lenient<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Valid(t) => t.fmt(f),
			Self::Invalid(s) => s.fmt(f),
		}
	}
}

impl<T: FromStr> FromStr for Lenient<T> {
	type Err = Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match T::from_str(s) {
			Ok(t) => Self::Valid(t),
			Err(_) => Self::Invalid(s.to_owned()),
		})
	}
}

impl<T> From<T> for Lenient<T> {
	fn from(value: T) -> Self {
		Self::Valid(value)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum LenientRef<'a, T> {
	Valid(T),
	Invalid(&'a str),
}
