use std::{convert::Infallible, fmt, ops::Deref, str::FromStr};

pub trait JsonLdValidate {
	type Invalid;
}

/// String-like value that can be valid or invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum JsonLdLenient<T: JsonLdValidate> {
	Valid(T),
	Invalid(T::Invalid),
}

impl<T: JsonLdValidate> JsonLdLenient<T> {
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

	pub fn as_deref<'a>(&'a self) -> JsonLdLenient<&'a T::Target>
	where
		T: Deref,
		T::Invalid: Deref,
		&'a T::Target: JsonLdValidate<Invalid = &'a <T::Invalid as Deref>::Target>,
	{
		match self {
			Self::Valid(t) => JsonLdLenient::Valid(t),
			Self::Invalid(s) => JsonLdLenient::Invalid(s.deref()),
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

	pub fn map(self, f: impl FnOnce(T) -> T) -> JsonLdLenient<T> {
		match self {
			Self::Valid(id) => JsonLdLenient::Valid(f(id)),
			Self::Invalid(id) => JsonLdLenient::Invalid(id),
		}
	}
}

impl<'a, T, U> JsonLdLenient<&'a T>
where
	&'a T: JsonLdValidate<Invalid = &'a U>,
	T: ?Sized + ToOwned,
	U: 'a + ?Sized + ToOwned,
	T::Owned: JsonLdValidate<Invalid = U::Owned>,
{
	pub fn into_owned(self) -> JsonLdLenient<T::Owned> {
		match self {
			Self::Valid(t) => JsonLdLenient::Valid(t.to_owned()),
			Self::Invalid(i) => JsonLdLenient::Invalid(i.to_owned()),
		}
	}
}

impl<T> AsRef<str> for JsonLdLenient<T>
where
	T: JsonLdValidate + AsRef<str>,
	T::Invalid: AsRef<str>,
{
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<T> fmt::Display for JsonLdLenient<T>
where
	T: JsonLdValidate + fmt::Display,
	T::Invalid: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Valid(t) => t.fmt(f),
			Self::Invalid(s) => s.fmt(f),
		}
	}
}

impl<T> FromStr for JsonLdLenient<T>
where
	T: JsonLdValidate + FromStr,
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

impl<T: JsonLdValidate> From<T> for JsonLdLenient<T> {
	fn from(value: T) -> Self {
		Self::Valid(value)
	}
}

impl<T: JsonLdValidate + FromStr> From<&str> for JsonLdLenient<T>
where
	T::Invalid: From<String>,
{
	fn from(value: &str) -> Self {
		let (this, _) = Self::new(value);
		this
	}
}
