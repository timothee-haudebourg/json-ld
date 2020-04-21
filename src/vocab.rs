use std::fmt;
use std::hash::Hash;
use iref::{Iri, IriBuf};
use crate::Id;

pub trait Vocab: Clone + PartialEq + Eq + Hash {
	fn from_iri(iri: Iri) -> Option<Self>;

	fn as_iri(&self) -> Iri;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum VocabId<V: Vocab> {
	Id(V),
	Iri(IriBuf)
}

impl<V: Vocab> fmt::Display for VocabId<V> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			VocabId::Id(id) => id.as_iri().fmt(f),
			VocabId::Iri(iri) => iri.fmt(f)
		}
	}
}

impl<V: Vocab> Id for VocabId<V> {
	fn from_iri(iri: Iri) -> VocabId<V> {
		if let Some(v) = V::from_iri(iri) {
			VocabId::Id(v)
		} else {
			VocabId::Iri(iri.into())
		}
	}

	fn as_iri(&self) -> Iri {
		match self {
			VocabId::Id(id) => id.as_iri(),
			VocabId::Iri(iri) => iri.as_iri(),
		}
	}
}
