use locspan_derive::StrippedPartialEq;
use std::hash::Hash;

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Nest {
	Nest,
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

impl From<String> for Nest {
	fn from(s: String) -> Self {
		if s == "@nest" {
			Self::Nest
		} else {
			Self::Term(s)
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
