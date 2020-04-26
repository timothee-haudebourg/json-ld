use std::hash::Hash;
use iref::{Iri, IriBuf};
use json::JsonValue;
use crate::{
	syntax::TermLike,
	util
};

/// Unique identifier types.
///
/// While JSON-LD uses [Internationalized Resource Identifiers (IRIs)](https://en.wikipedia.org/wiki/Internationalized_resource_identifier)
/// to uniquely identify each node,
/// this crate does not imposes the internal representation of identifiers.
///
/// Whatever type you choose, it must implement this trait to usure that:
///  - there is a low cost bijection with IRIs,
///  - it can be cloned ([`Clone`]),
///  - it can be compared ([`PartialEq`], [`Eq`]),
///  - it can be hashed ([`Hash`]).
pub trait Id: Clone + PartialEq + Eq + Hash {
	fn from_iri(iri: Iri) -> Self;

	fn as_iri(&self) -> Iri;
}

impl Id for IriBuf {
	fn from_iri(iri: Iri) -> IriBuf {
		iri.into()
	}

	fn as_iri(&self) -> Iri {
		self.as_iri()
	}
}

impl<T: Id> TermLike for T {
	fn as_str(&self) -> &str {
		self.as_iri().into_str()
	}

	fn as_iri(&self) -> Option<Iri> {
		Some(self.as_iri())
	}
}

impl<T: Id> util::AsJson for T {
	fn as_json(&self) -> JsonValue {
		self.as_iri().as_str().into()
	}
}
