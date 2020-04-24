use std::hash::Hash;
use std::fmt;
use iref::{Iri, IriBuf};
use json::JsonValue;
use crate::{
	TermLike,
	util
};

pub trait Id: Clone + PartialEq + Eq + Hash + fmt::Display {
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
