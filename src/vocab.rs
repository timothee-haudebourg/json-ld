use std::fmt;
use std::hash::Hash;
use std::convert::TryFrom;
use iref::{Iri, IriBuf};
use crate::{
	Id,
	Reference,
	ToReference,
	Lenient
};

pub trait Vocab: Clone + PartialEq + Eq + Hash {
	fn from_iri(iri: Iri) -> Option<Self>;

	fn as_iri(&self) -> Iri;
}

impl<T: Clone + PartialEq + Eq + Hash> Vocab for T where for<'a> T: TryFrom<Iri<'a>>, for<'a> &'a T: Into<Iri<'a>> {
	fn from_iri(iri: Iri) -> Option<Self> {
		match T::try_from(iri) {
			Ok(t) => Some(t),
			Err(_) => None
		}
	}

	fn as_iri(&self) -> Iri {
		self.into()
	}
}

impl<V: Vocab> ToReference<Lexicon<V>> for V {
	type Reference = Reference<Lexicon<V>>;

	fn to_ref(&self) -> Self::Reference {
		Reference::Id(Lexicon::Id(self.clone()))
	}
}

impl<V: Vocab> PartialEq<V> for Lenient<Reference<Lexicon<V>>> {
	fn eq(&self, other: &V) -> bool {
		match self {
			Lenient::Ok(Reference::Id(Lexicon::Id(v))) => {
				other == v
			},
			_ => false
		}
	}
}

impl<V: Vocab> PartialEq<V> for Reference<Lexicon<V>> {
	fn eq(&self, other: &V) -> bool {
		match self {
			Reference::Id(Lexicon::Id(v)) => {
				other == v
			},
			_ => false
		}
	}
}

impl<V: Vocab> PartialEq<V> for Lexicon<V> {
	fn eq(&self, other: &V) -> bool {
		match self {
			Lexicon::Id(v) => {
				other == v
			},
			_ => false
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Lexicon<V: Vocab> {
	Id(V),
	Iri(IriBuf)
}

impl<V: Vocab> fmt::Display for Lexicon<V> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Lexicon::Id(id) => id.as_iri().fmt(f),
			Lexicon::Iri(iri) => iri.fmt(f)
		}
	}
}

impl<V: Vocab> Id for Lexicon<V> {
	fn from_iri(iri: Iri) -> Lexicon<V> {
		if let Some(v) = V::from_iri(iri) {
			Lexicon::Id(v)
		} else {
			Lexicon::Iri(iri.into())
		}
	}

	fn as_iri(&self) -> Iri {
		match self {
			Lexicon::Id(id) => id.as_iri(),
			Lexicon::Iri(iri) => iri.as_iri(),
		}
	}
}
