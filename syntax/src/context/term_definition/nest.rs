use locspan_derive::StrippedPartialEq;
use std::hash::Hash;

use crate::is_keyword;

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Nest {
	Nest,

	/// Must not be a keyword.
	Term(#[stripped] String),
}

impl Nest {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Nest => "@nest",
			Self::Term(t) => &t,
		}
	}

	pub fn into_string(self) -> String {
		match self {
			Self::Nest => "@nest".to_string(),
			Self::Term(t) => t,
		}
	}
}

impl Hash for Nest {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum NestRef<'a> {
	Nest,
	Term(&'a str),
}

impl<'a> std::hash::Hash for NestRef<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_str().hash(state)
	}
}

impl<'a> NestRef<'a> {
	pub fn to_owned(self) -> Nest {
		match self {
			Self::Nest => Nest::Nest,
			Self::Term(t) => Nest::Term(t.to_owned()),
		}
	}

	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Nest => "@nest",
			Self::Term(t) => t,
		}
	}
}

impl<'a> From<&'a Nest> for NestRef<'a> {
	fn from(n: &'a Nest) -> Self {
		match n {
			Nest::Nest => Self::Nest,
			Nest::Term(t) => Self::Term(t),
		}
	}
}
