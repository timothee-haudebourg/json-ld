use json::JsonValue;
use crate::util::AsJson;

/// Value that can be null.
/// 
/// The `Option` type is used in the crate to design value that
/// may or may not be defined.
/// Sometimes value can be explicitelly defined as `null`,
/// hence the need of the type.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Nullable<T> {
	/// Null.
	Null,

	/// Some value.
	Some(T)
}

impl<T> Nullable<T> {
	/// Checks if the value is `null`.
	#[inline]
	pub fn is_null(&self) -> bool {
		match self {
			Nullable::Null => true,
			_ => false
		}
	}

	/// Checks if the value is not `null`.
	#[inline]
	pub fn is_some(&self) -> bool {
		match self {
			Nullable::Some(_) => true,
			_ => false
		}
	}

	/// Unwraps a non-null value.
	/// 
	/// Panics if the value is `null`.
	#[inline]
	pub fn unwrap(self) -> T {
		match self {
			Nullable::Some(t) => t,
			Nullable::Null => panic!("cannot unwrap null")
		}
	}

	/// Returns a nullabl reference to the inner value.
	#[inline]
	pub fn as_ref(&self) -> Nullable<&T> {
		match self {
			Nullable::Null => Nullable::Null,
			Nullable::Some(t) => Nullable::Some(t)
		}
	}

	/// Transform into an `Option` value.
	#[inline]
	pub fn option(self) -> Option<T> {
		match self {
			Nullable::Null => None,
			Nullable::Some(t) => Some(t)
		}
	}

	/// Map the inner value using the given function.
	#[inline]
	pub fn map<F, U>(self, f: F) -> Nullable<U> where F: FnOnce(T) -> U {
		match self {
			Nullable::Null => Nullable::Null,
			Nullable::Some(t) => Nullable::Some(f(t))
		}
	}
}

impl<'a, T: Clone> Nullable<&'a T> {
	/// Clone the referenced inner value.
	#[inline]
	pub fn cloned(&self) -> Nullable<T> {
		match self {
			Nullable::Null => Nullable::Null,
			Nullable::Some(t) => Nullable::Some((*t).clone())
		}
	}
}

impl<T: AsJson> AsJson for Nullable<T> {
	#[inline]
	fn as_json(&self) -> JsonValue {
		match self {
			Nullable::Null => JsonValue::Null,
			Nullable::Some(t) => t.as_json()
		}
	}
}

impl<T: PartialEq> PartialEq<T> for Nullable<T> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Nullable::Null => false,
			Nullable::Some(t) => t == other
		}
	}
}
