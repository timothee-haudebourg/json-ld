use std::fmt;
use std::convert::TryFrom;
use iref::Iri;
use json::JsonValue;
use crate::{
	Keyword,
	Id,
	Term,
	TermLike,
	Property,
	util
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type<T> {
	Id,
	Json,
	None,
	Vocab,
	Ref(T)
}

impl<T> Type<T> {
	pub fn into_ref(self) -> Result<T, Type<T>> {
		match self {
			Type::Ref(id) => Ok(id),
			typ => Err(typ)
		}
	}
}

impl<T: TermLike> Type<T> {
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Type::Ref(id) => id.as_iri(),
			_ => None
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Type::Id => "@id",
			Type::Json => "@json",
			Type::None => "@none",
			Type::Vocab => "@vocab",
			Type::Ref(id) => id.as_str()
		}
	}
}

impl<T: TermLike> TermLike for Type<T> {
	fn as_iri(&self) -> Option<Iri> {
		self.as_iri()
	}

	fn as_str(&self) -> &str {
		self.as_str()
	}
}

impl<T: Id> TryFrom<Term<T>> for Type<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<Type<T>, Term<T>> {
		match term {
			Term::Keyword(Keyword::Id) => Ok(Type::Id),
			Term::Keyword(Keyword::Json) => Ok(Type::Json),
			Term::Keyword(Keyword::None) => Ok(Type::None),
			Term::Keyword(Keyword::Vocab) => Ok(Type::Vocab),
			Term::Prop(Property::Id(id)) => Ok(Type::Ref(id)),
			term => Err(term)
		}
	}
}

impl<T: util::AsJson> util::AsJson for Type<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Type::Id => "@id".into(),
			Type::Json => "@json".into(),
			Type::None => "@none".into(),
			Type::Vocab => "@vocab".into(),
			Type::Ref(id) => id.as_json()
		}
	}
}

impl<T: fmt::Display> fmt::Display for Type<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Type::Id => write!(f, "@id"),
			Type::Json => write!(f, "@json"),
			Type::None => write!(f, "@none"),
			Type::Vocab => write!(f, "@vocab"),
			Type::Ref(id) => id.fmt(f)
		}
	}
}
