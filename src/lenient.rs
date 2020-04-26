use std::convert::{TryFrom, TryInto};
use std::fmt;
use json::JsonValue;
use iref::Iri;
use crate::{
	syntax::TermLike,
	util
};

/// Wrapper for string data that may be malformed.
///
/// This is usually used with `Term`, or `Type` to allow for terms/types that are not IRIs or
/// blanck nodes, etc. but must be kept in the data structure.
#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Lenient<T> {
	Ok(T),
	Unknown(String)
}

impl<T> Lenient<T> {
	pub fn cast<U>(self) -> Lenient<U> where U: From<T> {
		match self {
			Lenient::Ok(t) => Lenient::Ok(t.into()),
			Lenient::Unknown(t) => Lenient::Unknown(t)
		}
	}

	pub fn try_cast<U>(self) -> Result<Lenient<U>, U::Error> where U: TryFrom<T> {
		match self {
			Lenient::Ok(t) => Ok(Lenient::Ok(t.try_into()?)),
			Lenient::Unknown(t) => Ok(Lenient::Unknown(t))
		}
	}
}

impl<T: PartialEq> PartialEq<T> for Lenient<T> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Lenient::Ok(t) => t == other,
			_ => false
		}
	}
}

impl<T: TermLike> Lenient<T> {
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Lenient::Ok(term) => term.as_iri(),
			Lenient::Unknown(_) => None
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Lenient::Ok(term) => term.as_str(),
			Lenient::Unknown(unknown) => unknown.as_str()
		}
	}
}

impl<T> From<T> for Lenient<T> {
	fn from(t: T) -> Lenient<T> {
		Lenient::Ok(t)
	}
}

impl<T: fmt::Display> fmt::Display for Lenient<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Lenient::Ok(t) => t.fmt(f),
			Lenient::Unknown(u) => u.fmt(f)
		}
	}
}

impl<T: util::AsJson> util::AsJson for Lenient<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Lenient::Ok(t) => t.as_json(),
			Lenient::Unknown(u) => u.as_json()
		}
	}
}
