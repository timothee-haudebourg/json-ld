use std::fmt;

/// Value that can be null.
///
/// The `Option` type is used in this crate to indicate values that
/// may or may not be defined.
/// Sometimes however,
/// value can be explicitly defined as `null`,
/// hence the need for this type.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Nullable<T> {
	/// Null value.
	Null,

	/// Some other value.
	Some(T),
}

impl<T> Nullable<T> {
	/// Checks if the value is `null`.
	#[inline(always)]
	pub fn is_null(&self) -> bool {
		matches!(self, Nullable::Null)
	}

	/// Checks if the value is not `null`.
	#[inline(always)]
	pub fn is_some(&self) -> bool {
		matches!(self, Nullable::Some(_))
	}

	/// Unwraps a non-null value.
	///
	/// Panics if the value is `null`.
	#[inline(always)]
	pub fn unwrap(self) -> T {
		match self {
			Nullable::Some(t) => t,
			Nullable::Null => panic!("cannot unwrap null"),
		}
	}

	/// Returns a nullabl reference to the inner value.
	#[inline(always)]
	pub fn as_ref(&self) -> Nullable<&T> {
		match self {
			Nullable::Null => Nullable::Null,
			Nullable::Some(t) => Nullable::Some(t),
		}
	}

	pub fn as_deref(&self) -> Nullable<&T::Target>
	where
		T: std::ops::Deref,
	{
		match self {
			Self::Null => Nullable::Null,
			Self::Some(t) => Nullable::Some(t),
		}
	}

	/// Transform into an `Option` value.
	#[inline(always)]
	pub fn option(self) -> Option<T> {
		match self {
			Nullable::Null => None,
			Nullable::Some(t) => Some(t),
		}
	}

	/// Map the inner value using the given function.
	#[inline(always)]
	pub fn map<F, U>(self, f: F) -> Nullable<U>
	where
		F: FnOnce(T) -> U,
	{
		match self {
			Nullable::Null => Nullable::Null,
			Nullable::Some(t) => Nullable::Some(f(t)),
		}
	}

	pub fn cast<U>(self) -> Nullable<U>
	where
		T: Into<U>,
	{
		match self {
			Self::Null => Nullable::Null,
			Self::Some(t) => Nullable::Some(t.into()),
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

impl<T> From<T> for Nullable<T> {
	fn from(value: T) -> Self {
		Self::Some(value)
	}
}

impl<T> From<Option<T>> for Nullable<T> {
	fn from(value: Option<T>) -> Self {
		match value {
			Some(t) => Self::Some(t),
			None => Self::Null,
		}
	}
}

impl<T: Clone> Nullable<&T> {
	/// Clone the referenced inner value.
	#[inline(always)]
	pub fn cloned(&self) -> Nullable<T> {
		match self {
			Nullable::Null => Nullable::Null,
			Nullable::Some(t) => Nullable::Some((*t).clone()),
		}
	}
}

impl<T: fmt::Display> fmt::Display for Nullable<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt(f),
		}
	}
}

impl<T: contextual::DisplayWithContext<V>, V> contextual::DisplayWithContext<V> for Nullable<T> {
	fn fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(vocabulary, f),
		}
	}
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Nullable<T> {
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
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Nullable<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		Ok(Option::<T>::deserialize(deserializer)?.into())
	}
}

#[cfg(feature = "serde")]
impl<T> Nullable<T> {
	pub fn optional<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
	where
		T: serde::Deserialize<'de>,
		D: serde::Deserializer<'de>,
	{
		use serde::Deserialize;
		Ok(Some(Option::<T>::deserialize(deserializer)?.into()))
	}
}
