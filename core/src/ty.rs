use super::Term;
use crate::{Reference, ValidReference};
use json_ld_syntax::Keyword;
use std::convert::TryFrom;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Type<I> {
	Id,
	Json,
	None,
	Vocab,
	Ref(I),
}

impl<I> Type<I> {
	pub fn into_ref(self) -> Result<I, Type<I>> {
		match self {
			Type::Ref(id) => Ok(id),
			typ => Err(typ),
		}
	}

	pub fn map<U, F: FnOnce(&I) -> U>(&self, f: F) -> Type<U> {
		match self {
			Type::Id => Type::Id,
			Type::Json => Type::Json,
			Type::None => Type::None,
			Type::Vocab => Type::Vocab,
			Type::Ref(t) => Type::Ref(f(t)),
		}
	}
}

impl<'a, I: Clone> Type<&'a I> {
	pub fn cloned(self) -> Type<I> {
		match self {
			Type::Id => Type::Id,
			Type::Json => Type::Json,
			Type::None => Type::None,
			Type::Vocab => Type::Vocab,
			Type::Ref(t) => Type::Ref(t.clone()),
		}
	}
}

impl<I> Type<I> {
	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Type::Ref(id) => Some(id),
			_ => None,
		}
	}
}

impl<I: AsRef<str>> Type<I> {
	pub fn as_str(&self) -> &str {
		match self {
			Type::Id => "@id",
			Type::Json => "@json",
			Type::None => "@none",
			Type::Vocab => "@vocab",
			Type::Ref(id) => id.as_ref(),
		}
	}
}

impl<'a, I> From<&'a Type<I>> for Type<&'a I> {
	fn from(t: &'a Type<I>) -> Type<&'a I> {
		match t {
			Type::Id => Type::Id,
			Type::Json => Type::Json,
			Type::None => Type::None,
			Type::Vocab => Type::Vocab,
			Type::Ref(id) => Type::Ref(id),
		}
	}
}

impl<I, B> From<Type<I>> for Term<I, B> {
	fn from(t: Type<I>) -> Term<I, B> {
		match t {
			Type::Id => Term::Keyword(Keyword::Id),
			Type::Json => Term::Keyword(Keyword::Json),
			Type::None => Term::Keyword(Keyword::None),
			Type::Vocab => Term::Keyword(Keyword::Vocab),
			Type::Ref(id) => Term::Ref(Reference::Valid(ValidReference::Id(id))),
		}
	}
}

impl<I, B> TryFrom<Term<I, B>> for Type<I> {
	type Error = Term<I, B>;

	fn try_from(term: Term<I, B>) -> Result<Type<I>, Term<I, B>> {
		match term {
			Term::Keyword(Keyword::Id) => Ok(Type::Id),
			Term::Keyword(Keyword::Json) => Ok(Type::Json),
			Term::Keyword(Keyword::None) => Ok(Type::None),
			Term::Keyword(Keyword::Vocab) => Ok(Type::Vocab),
			Term::Ref(Reference::Valid(ValidReference::Id(id))) => Ok(Type::Ref(id)),
			term => Err(term),
		}
	}
}

// impl<K: JsonBuild, I: utils::AsAnyJson<K>> utils::AsAnyJson<K> for Type<I> {
// 	fn as_json_with(&self, meta: K::MetaData) -> K {
// 		match self {
// 			Type::Id => "@id".as_json_with(meta),
// 			Type::Json => "@json".as_json_with(meta),
// 			Type::None => "@none".as_json_with(meta),
// 			Type::Vocab => "@vocab".as_json_with(meta),
// 			Type::Ref(id) => id.as_json_with(meta),
// 		}
// 	}
// }

impl<I: fmt::Display> fmt::Display for Type<I> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Type::Id => write!(f, "@id"),
			Type::Json => write!(f, "@json"),
			Type::None => write!(f, "@none"),
			Type::Vocab => write!(f, "@vocab"),
			Type::Ref(id) => id.fmt(f),
		}
	}
}

// pub type NodeType<I> = Type<Reference<I>>;

// impl<I: Id> TryFrom<Term<I>> for NodeType<I> {
// 	type Error = Term<I>;

// 	fn try_from(term: Term<I>) -> Result<NodeType<I>, Term<I>> {
// 		match term {
// 			Term::Keyword(Keyword::Id) => Ok(Type::Id),
// 			Term::Keyword(Keyword::Json) => Ok(Type::Json),
// 			Term::Keyword(Keyword::None) => Ok(Type::None),
// 			Term::Keyword(Keyword::Vocab) => Ok(Type::Vocab),
// 			Term::Ref(prop) => Ok(Type::Ref(prop)),
// 			term => Err(term),
// 		}
// 	}
// }
