use std::fmt;

/// Value that can be null.
///
/// The `Option` type is used in this crate to indicate values that
/// may or may not be defined.
/// Sometimes however,
/// value can be explicitly defined as `null`,
/// hence the need for this type.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum JsonLdNullable<T> {
	/// Null value.
	Null,

	/// Some other value.
	Some(T),
}

impl<T> JsonLdNullable<T> {
	/// Checks if the value is `null`.
	#[inline(always)]
	pub fn is_null(&self) -> bool {
		matches!(self, JsonLdNullable::Null)
	}

	/// Checks if the value is not `null`.
	#[inline(always)]
	pub fn is_some(&self) -> bool {
		matches!(self, JsonLdNullable::Some(_))
	}

	/// Unwraps a non-null value.
	///
	/// Panics if the value is `null`.
	#[inline(always)]
	pub fn unwrap(self) -> T {
		match self {
			JsonLdNullable::Some(t) => t,
			JsonLdNullable::Null => panic!("cannot unwrap null"),
		}
	}

	/// Returns a nullabl reference to the inner value.
	#[inline(always)]
	pub fn as_ref(&self) -> JsonLdNullable<&T> {
		match self {
			JsonLdNullable::Null => JsonLdNullable::Null,
			JsonLdNullable::Some(t) => JsonLdNullable::Some(t),
		}
	}

	pub fn as_deref(&self) -> JsonLdNullable<&T::Target>
	where
		T: std::ops::Deref,
	{
		match self {
			Self::Null => JsonLdNullable::Null,
			Self::Some(t) => JsonLdNullable::Some(t),
		}
	}

	/// Transform into an `Option` value.
	#[inline(always)]
	pub fn option(self) -> Option<T> {
		match self {
			JsonLdNullable::Null => None,
			JsonLdNullable::Some(t) => Some(t),
		}
	}

	/// Map the inner value using the given function.
	#[inline(always)]
	pub fn map<F, U>(self, f: F) -> JsonLdNullable<U>
	where
		F: FnOnce(T) -> U,
	{
		match self {
			JsonLdNullable::Null => JsonLdNullable::Null,
			JsonLdNullable::Some(t) => JsonLdNullable::Some(f(t)),
		}
	}

	pub fn cast<U>(self) -> JsonLdNullable<U>
	where
		T: Into<U>,
	{
		match self {
			Self::Null => JsonLdNullable::Null,
			Self::Some(t) => JsonLdNullable::Some(t.into()),
		}
	}

	pub fn unwrap_or(self, default: T) -> T {
		match self {
			Self::Null => default,
			Self::Some(t) => t,
		}
	}

	pub fn unwrap_or_default(self) -> T
	where
		T: Default,
	{
		match self {
			Self::Null => T::default(),
			Self::Some(t) => t,
		}
	}
}

impl<T> From<T> for JsonLdNullable<T> {
	fn from(value: T) -> Self {
		Self::Some(value)
	}
}

impl<T> From<Option<T>> for JsonLdNullable<T> {
	fn from(value: Option<T>) -> Self {
		match value {
			Some(t) => Self::Some(t),
			None => Self::Null,
		}
	}
}

impl<T: Clone> JsonLdNullable<&T> {
	/// Clone the referenced inner value.
	#[inline(always)]
	pub fn cloned(&self) -> JsonLdNullable<T> {
		match self {
			JsonLdNullable::Null => JsonLdNullable::Null,
			JsonLdNullable::Some(t) => JsonLdNullable::Some((*t).clone()),
		}
	}
}

impl<T: Copy> JsonLdNullable<&T> {
	/// Copy the referenced inner value.
	#[inline(always)]
	pub fn copied(&self) -> JsonLdNullable<T> {
		match self {
			JsonLdNullable::Null => JsonLdNullable::Null,
			JsonLdNullable::Some(t) => JsonLdNullable::Some(**t),
		}
	}
}

impl<T: fmt::Display> fmt::Display for JsonLdNullable<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt(f),
		}
	}
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for JsonLdNullable<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Null => serializer.serialize_none(),
			Self::Some(t) => serializer.serialize_some(t),
		}
	}
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for JsonLdNullable<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Ok(Option::<T>::deserialize(deserializer)?.into())
	}
}

/// Deserializes an `Option<JsonLdNullable<T>>` field, distinguishing between
/// a missing field (`None`) and an explicit `null` (`Some(JsonLdNullable::Null)`).
///
/// Use with `#[serde(default, deserialize_with = "JsonLdNullable::optional")]`.
#[cfg(feature = "serde")]
impl<T> JsonLdNullable<T> {
	pub fn optional<'de, D>(deserializer: D) -> Result<Option<JsonLdNullable<T>>, D::Error>
	where
		T: serde::Deserialize<'de>,
		D: serde::Deserializer<'de>,
	{
		<JsonLdNullable<T> as serde::Deserialize>::deserialize(deserializer).map(Some)
	}
}
