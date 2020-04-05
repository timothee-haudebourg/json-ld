use std::hash::{Hash, Hasher};
use iref::Iri;
use json::JsonValue;
use crate::{util, Id, Key,  Direction};

#[derive(PartialEq)]
pub enum Literal<T: Id> {
	Null,
	Boolean(bool),
	Number(json::number::Number),
	String {
		data: String,
		language: Option<String>,
		direction: Option<Direction>
	},
	Ref(Key<T>),
	Json(JsonValue),
}

// this is needed because `json::number::Number` incorrectly forgot to implement `Eq`.
impl<T: Id> Eq for Literal<T> { }

impl<T: Id> Hash for Literal<T> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		use Literal::*;
		match self {
			Null => (),
			Boolean(b) => b.hash(hasher),
			Number(n) => util::hash_json_number(n, hasher),
			String { data, language, direction } => {
				data.hash(hasher);
				language.hash(hasher);
				direction.hash(hasher);
			},
			Ref(r) => r.hash(hasher),
			Json(json) => util::hash_json(json, hasher),
		}
	}
}

impl<T: Id> Literal<T> {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Literal::String { data, .. } => Some(data.as_str()),
			Literal::Ref(r) => {
				match r.iri() {
					Some(iri) => Some(iri.into_str()),
					None => None
				}
			},
			_ => None
		}
	}
}
