use std::{convert::Infallible, fmt, ops::Deref, str::FromStr};

pub trait Validate {
	type Invalid;
}

/// String-like value that can be valid or invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Lenient<T: Validate> {
	Valid(T),
	Invalid(T::Invalid),
}

impl<T: Validate> Lenient<T> {
	pub fn new(value: &str) -> (Self, Option<T::Err>)
	where
		T: FromStr,
		T::Invalid: From<String>,
	{
		match T::from_str(value) {
			Ok(t) => (Self::Valid(t), None),
			Err(e) => (Self::Invalid(T::Invalid::from(value.to_owned())), Some(e)),
		}
	}

	pub fn from_string(value: String) -> (Self, Option<T::Err>)
	where
		T: FromStr,
		T::Invalid: From<String>,
	{
		match T::from_str(&value) {
			Ok(t) => (Self::Valid(t), None),
			Err(e) => (Self::Invalid(T::Invalid::from(value)), Some(e)),
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

	pub fn as_valid(&self) -> Option<&T> {
		match self {
			Self::Valid(t) => Some(t),
			Self::Invalid(_) => None,
		}
	}

	pub fn into_valid(self) -> Result<T, T::Invalid> {
		match self {
			Self::Valid(t) => Ok(t),
			Self::Invalid(e) => Err(e),
		}
	}

	pub fn is_invalid(&self) -> bool {
		matches!(self, Self::Invalid(_))
	}

	pub fn as_invalid(&self) -> Option<&T::Invalid> {
		match self {
			Self::Valid(_) => None,
			Self::Invalid(t) => Some(t),
		}
	}

	pub fn as_deref<'a>(&'a self) -> Lenient<&'a T::Target>
	where
		T: Deref,
		T::Invalid: Deref,
		&'a T::Target: Validate<Invalid = &'a <T::Invalid as Deref>::Target>,
	{
		match self {
			Self::Valid(t) => Lenient::Valid(t),
			Self::Invalid(s) => Lenient::Invalid(s.deref()),
		}
	}

	pub fn as_str(&self) -> &str
	where
		T: AsRef<str>,
		T::Invalid: AsRef<str>,
	{
		match self {
			Self::Valid(t) => t.as_ref(),
			Self::Invalid(s) => s.as_ref(),
		}
	}

	pub fn map(self, f: impl FnOnce(T) -> T) -> Lenient<T> {
		match self {
			Self::Valid(id) => Lenient::Valid(f(id)),
			Self::Invalid(id) => Lenient::Invalid(id),
		}
	}
}

impl<'a, T, U> Lenient<&'a T>
where
	&'a T: Validate<Invalid = &'a U>,
	T: ?Sized + ToOwned,
	U: 'a + ?Sized + ToOwned,
	T::Owned: Validate<Invalid = U::Owned>,
{
	pub fn into_owned(self) -> Lenient<T::Owned> {
		match self {
			Self::Valid(t) => Lenient::Valid(t.to_owned()),
			Self::Invalid(i) => Lenient::Invalid(i.to_owned()),
		}
	}
}

impl<T> AsRef<str> for Lenient<T>
where
	T: Validate + AsRef<str>,
	T::Invalid: AsRef<str>,
{
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<T> fmt::Display for Lenient<T>
where
	T: Validate + fmt::Display,
	T::Invalid: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Valid(t) => t.fmt(f),
			Self::Invalid(s) => s.fmt(f),
		}
	}
}

impl<T> FromStr for Lenient<T>
where
	T: Validate + FromStr,
	T::Invalid: From<String>,
{
	type Err = Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match T::from_str(s) {
			Ok(t) => Self::Valid(t),
			Err(_) => Self::Invalid(s.to_owned().into()),
		})
	}
}

impl<T: Validate> From<T> for Lenient<T> {
	fn from(value: T) -> Self {
		Self::Valid(value)
	}
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize))]
// #[cfg_attr(feature = "serde", serde(untagged))]
// pub enum LenientRef<'a, T> {
// 	Valid(T),
// 	Invalid(&'a str),
// }

// impl<'a, T: Copy> LenientRef<'a, T> {
// 	/// Returns `true` if the value is valid.
// 	#[inline(always)]
// 	pub fn is_valid(self) -> bool {
// 		matches!(self, Self::Valid(_))
// 	}

// 	/// Returns the valid value, or `None` if invalid.
// 	#[inline(always)]
// 	pub fn ok(self) -> Option<T> {
// 		match self {
// 			Self::Valid(t) => Some(t),
// 			Self::Invalid(_) => None,
// 		}
// 	}
// }

// impl<'a, T: AsRef<str>> LenientRef<'a, T> {
// 	/// Returns the string representation of the value.
// 	pub fn as_str(&self) -> &str {
// 		match self {
// 			Self::Valid(t) => t.as_ref(),
// 			Self::Invalid(s) => s,
// 		}
// 	}
// }

// impl<'a, T: ToOwned + ?Sized> LenientRef<'a, &'a T> {
// 	/// Converts to an owned [`Lenient`] value.
// 	pub fn to_owned(self) -> Lenient<T::Owned> {
// 		match self {
// 			Self::Valid(t) => Lenient::Valid(t.to_owned()),
// 			Self::Invalid(s) => Lenient::Invalid(s.to_owned()),
// 		}
// 	}
// }

// impl<'a, T: AsRef<str>> AsRef<str> for LenientRef<'a, T> {
// 	fn as_ref(&self) -> &str {
// 		self.as_str()
// 	}
// }

// impl<'a, T: fmt::Display> fmt::Display for LenientRef<'a, T> {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		match self {
// 			Self::Valid(t) => t.fmt(f),
// 			Self::Invalid(s) => s.fmt(f),
// 		}
// 	}
// }
