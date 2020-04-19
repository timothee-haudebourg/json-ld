use std::convert::TryFrom;
use std::fmt;
use json::JsonValue;
use crate::{Id, Keyword, Term, Property, AsJson};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum NodeType<T: Id> {
	Id,
	JSON,
	None,
	Vocab,
	Prop(Property<T>),
	Unknown(String)
}

impl<T: Id> NodeType<T> {
	pub fn as_str(&self) -> &str {
		match self {
			NodeType::Id => "@id",
			NodeType::JSON => "@json",
			NodeType::None => "@none",
			NodeType::Vocab => "@vocab",
			NodeType::Prop(p) => p.as_str(),
			NodeType::Unknown(s) => s.as_str()
		}
	}
}

impl<T: Id> TryFrom<Term<T>> for NodeType<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<NodeType<T>, Term<T>> {
		match term {
			Term::Keyword(Keyword::Id) => Ok(NodeType::Id),
			Term::Keyword(Keyword::JSON) => Ok(NodeType::JSON),
			Term::Keyword(Keyword::None) => Ok(NodeType::None),
			Term::Keyword(Keyword::Vocab) => Ok(NodeType::Vocab),
			Term::Prop(prop) => Ok(NodeType::Prop(prop)),
			Term::Unknown(name) => {
				Ok(NodeType::Unknown(name))
			},
			term => Err(term)
		}
	}
}

impl<T: Id> AsJson for NodeType<T> {
	fn as_json(&self) -> JsonValue {
		self.as_str().into()
	}
}

impl<T: Id> fmt::Display for NodeType<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ValueType<T: Id> {
	Id,
	JSON,
	None,
	Vocab,
	Prop(Property<T>)
}

impl<T: Id> ValueType<T> {
	pub fn as_str(&self) -> &str {
		match self {
			ValueType::Id => "@id",
			ValueType::JSON => "@json",
			ValueType::None => "@none",
			ValueType::Vocab => "@vocab",
			ValueType::Prop(p) => p.as_str()
		}
	}
}

impl<T: Id> TryFrom<Term<T>> for ValueType<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<ValueType<T>, Term<T>> {
		match term {
			Term::Keyword(Keyword::Id) => Ok(ValueType::Id),
			Term::Keyword(Keyword::JSON) => Ok(ValueType::JSON),
			Term::Keyword(Keyword::None) => Ok(ValueType::None),
			Term::Keyword(Keyword::Vocab) => Ok(ValueType::Vocab),
			Term::Prop(prop) => Ok(ValueType::Prop(prop)),
			term => Err(term)
		}
	}
}

impl<T: Id> AsJson for ValueType<T> {
	fn as_json(&self) -> JsonValue {
		self.as_str().into()
	}
}

impl<T: Id> fmt::Display for ValueType<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
