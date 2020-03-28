#[macro_use]
extern crate log;
extern crate json;
extern crate iref;

mod error;
mod keyword;
mod direction;
mod container;
pub mod context;
pub mod expansion;
mod pp;

use std::fmt;
use std::collections::HashMap;
use iref::{Iri, IriBuf};
pub use error::*;
pub use keyword::*;
pub use direction::*;
pub use container::*;
pub use expansion::expand;
pub use pp::*;

use json::JsonValue;

pub(crate) fn as_array(json: &JsonValue) -> &[JsonValue] {
	match json {
		JsonValue::Array(ary) => ary,
		_ => unsafe { std::mem::transmute::<&JsonValue, &[JsonValue; 1]>(json) as &[JsonValue] }
	}
}

pub trait Id: Clone + PartialEq + Eq + fmt::Display {
	fn from_iri(iri: Iri) -> Self;

	fn from_blank_id(id: &str) -> Self;

	fn iri(&self) -> Option<Iri>;
}

#[derive(Clone, PartialEq, Eq)]
pub enum Key<T: Id> {
	Id(T),
	Keyword(Keyword)
}

impl<T: Id> Key<T> {
	pub fn is_keyword(&self) -> bool {
		match self {
			Key::Keyword(_) => true,
			_ => false
		}
	}

	pub fn iri(&self) -> Option<Iri> {
		match self {
			Key::Id(k) => k.iri(),
			_ => None
		}
	}
}

impl<T: Id> fmt::Display for Key<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Key::Id(id) => id.fmt(f),
			Key::Keyword(kw) => kw.into_str().fmt(f)
		}
	}
}

#[derive(Clone, PartialEq, Eq)]
pub enum DefaultKey {
	Iri(IriBuf),
	Blank(String)
}

impl fmt::Display for DefaultKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			DefaultKey::Iri(iri) => iri.fmt(f),
			DefaultKey::Blank(name) => name.fmt(f)
		}
	}
}

impl Id for DefaultKey {
	fn from_iri(iri: Iri) -> DefaultKey {
		DefaultKey::Iri(iri.into())
	}

	fn from_blank_id(id: &str) -> DefaultKey {
		DefaultKey::Blank(id.to_string())
	}

	fn iri(&self) -> Option<Iri> {
		match self {
			DefaultKey::Iri(iri) => Some(iri.as_iri()),
			DefaultKey::Blank(_) => None
		}
	}
}

pub struct Literal<T: Id> {
	pub typ: Option<Key<T>>,
	pub index: Option<Key<T>>,
	pub direction: Option<Direction>,
	pub language: Option<String>,
	pub value: JsonValue,
}

impl<T: Id> Literal<T> {
	pub fn new(lit: JsonValue) -> Literal<T> {
		Literal {
			typ: None,
			index: None,
			language: None,
			direction: None,
			value: lit
		}
	}
}

pub enum Value<T: Id> {
	Literal(Literal<T>),
	Ref(Key<T>),
	List(Vec<Object<T>>)
}

pub enum Object<T: Id> {
	Value(Value<T>),
	Node(Node<T>)
}

pub struct Node<T: Id> {
	pub id: Option<Key<T>>,
	pub types: Vec<Key<T>>,
	pub graph: Option<Vec<Object<T>>>,
	pub included: Option<Vec<Object<T>>>,
	pub language: Option<String>,
	pub direction: Option<Direction>,
	pub expanded_property: Option<Key<T>>,
	pub properties: HashMap<Key<T>, Vec<Object<T>>>
}

impl<T: Id> Node<T> {
	fn new() -> Node<T> {
		Node {
			id: None,
			types: Vec::new(),
			graph: None,
			included: None,
			language: None,
			direction: None,
			expanded_property: None,
			properties: HashMap::new()
		}
	}
}
