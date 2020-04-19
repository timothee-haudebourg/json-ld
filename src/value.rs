use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use iref::Iri;
use crate::{util, Id, ValueType, Literal, Object};

#[derive(PartialEq, Eq)]
pub enum Value<T: Id> {
	Literal(Literal<T>, HashSet<ValueType<T>>),
	List(Vec<Object<T>>),
}

impl<T: Id> Hash for Value<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Value::Literal(lit, types) => {
				lit.hash(h);
				util::hash_set(types, h)
			},
			Value::List(l) => l.hash(h)
		}
	}
}

impl<T: Id> Value<T> {
	pub fn is_list(&self) -> bool {
		match self {
			Value::List(_) => true,
			_ => false
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self {
			Value::Literal(lit, _) => lit.as_str(),
			_ => None
		}
	}

	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Value::Literal(Literal::Ref(r), _) => r.iri(),
			_ => None
		}
	}
}
