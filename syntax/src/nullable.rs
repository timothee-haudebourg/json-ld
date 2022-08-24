use std::fmt;

use locspan_derive::StrippedPartialEq;

/// Value that can be null.
///
/// The `Option` type is used in this crate to indicate values that
/// may or may not be defined.
/// Sometimes however,
/// value can be explicitly defined as `null`,
/// hence the need for this type.
#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
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

impl<'a, T: Clone> Nullable<&'a T> {
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
