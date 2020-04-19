use std::hash::Hash;
use std::fmt;
use iref::{Iri, IriBuf};

pub trait Id: Clone + PartialEq + Eq + Hash + fmt::Display {
	fn from_iri(iri: Iri) -> Self;

	fn iri(&self) -> Iri;
}

impl Id for IriBuf {
	fn from_iri(iri: Iri) -> IriBuf {
		iri.into()
	}

	fn iri(&self) -> Iri {
		self.as_iri()
	}
}
