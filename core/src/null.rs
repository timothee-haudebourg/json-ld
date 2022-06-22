use crate::utils::{AsJson, JsonFrom};
use generic_json::{Json, JsonClone};

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

impl<J: JsonClone, K: JsonFrom<J>, T: AsJson<J, K>> AsJson<J, K> for Nullable<T> {
	#[inline(always)]
	fn as_json_with(
		&self,
		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
	) -> K {
		match self {
			Nullable::Null => K::null(meta(None)),
			Nullable::Some(t) => t.as_json_with(meta),
		}
	}
}

impl<T: PartialEq> PartialEq<T> for Nullable<T> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Nullable::Null => false,
			Nullable::Some(t) => t == other,
		}
	}
}
