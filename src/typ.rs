use std::convert::TryFrom;
use std::fmt;
use json::JsonValue;
use crate::{Id, Keyword, Key, Property, AsJson};

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

impl<T: Id> TryFrom<Key<T>> for NodeType<T> {
	type Error = Key<T>;

	fn try_from(term: Key<T>) -> Result<NodeType<T>, Key<T>> {
		match term {
			Key::Keyword(Keyword::Id) => Ok(NodeType::Id),
			Key::Keyword(Keyword::JSON) => Ok(NodeType::JSON),
			Key::Keyword(Keyword::None) => Ok(NodeType::None),
			Key::Keyword(Keyword::Vocab) => Ok(NodeType::Vocab),
			Key::Prop(prop) => Ok(NodeType::Prop(prop)),
			Key::Unknown(name) => {
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

impl<T: Id> TryFrom<Key<T>> for ValueType<T> {
	type Error = Key<T>;

	fn try_from(term: Key<T>) -> Result<ValueType<T>, Key<T>> {
		match term {
			Key::Keyword(Keyword::Id) => Ok(ValueType::Id),
			Key::Keyword(Keyword::JSON) => Ok(ValueType::JSON),
			Key::Keyword(Keyword::None) => Ok(ValueType::None),
			Key::Keyword(Keyword::Vocab) => Ok(ValueType::Vocab),
			Key::Prop(prop) => Ok(ValueType::Prop(prop)),
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
