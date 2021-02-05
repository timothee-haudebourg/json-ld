use std::fmt;
use std::hash::Hash;
use std::convert::TryFrom;
use iref::{Iri, IriBuf, AsIri};
use crate::{
	Id,
	Reference,
	ToReference,
	Lenient
};

pub trait Vocab: AsIri + Clone + PartialEq + Eq + Hash {
	fn from_iri(iri: Iri) -> Option<Self>;
}

impl<T: AsIri + Clone + PartialEq + Eq + Hash> Vocab for T where for<'a> T: TryFrom<Iri<'a>> {
	fn from_iri(iri: Iri) -> Option<Self> {
		match T::try_from(iri) {
			Ok(t) => Some(t),
			Err(_) => None
		}
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

/// Lexicon identifier.
///
/// A Lexicon is a special identifier type ([`Id`]) built from a [`Vocab`] (vocabulary) type.
/// While [`Vocab`] represents only a subset of all the possible IRIs, this type can also hold
/// any IRI outside of the predefined vocabulary.
/// It is a simple way to get an identifier type from a vocabulary.
///
/// # Example
/// The following example builds a lexicon from a statically known vocabulary, defined as an
/// `enum` type. It uses the [`iref-enum`](https://crates.io/crates/iref-enum)
/// crate to automatically derive the convertion of the
/// `Vocab` type from/into IRIs.
/// ```
/// use iref_enum::*;
/// use json_ld::Lexicon;
///
/// /// Vocabulary used in the implementation.
/// #[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
/// #[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
/// #[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
/// #[iri_prefix("vocab" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
/// pub enum Vocab {
/// 	#[iri("rdfs:comment")] Comment,
///
/// 	#[iri("manifest:name")] Name,
/// 	#[iri("manifest:entries")] Entries,
/// 	#[iri("manifest:action")] Action,
/// 	#[iri("manifest:result")] Result,
///
/// 	#[iri("vocab:PositiveEvaluationTest")] PositiveEvalTest,
/// 	#[iri("vocab:NegativeEvaluationTest")] NegativeEvalTest,
/// 	#[iri("vocab:option")] Option,
/// 	#[iri("vocab:specVersion")] SpecVersion,
/// 	#[iri("vocab:processingMode")] ProcessingMode,
/// 	#[iri("vocab:expandContext")] ExpandContext,
/// 	#[iri("vocab:base")] Base
/// }
///
/// /// A fully functional identifier type.
/// pub type Id = Lexicon<Vocab>;
///
/// fn handle_node(node: &json_ld::Node<Id>) {
///   for name in node.get(Vocab::Name) { // <- note that we can directly use `Vocab` here.
///   	println!("node name: {}", name.as_str().unwrap());
///   }
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Lexicon<V: Vocab> {
	/// Identifier from the known vocabulary.
	Id(V),

	/// Any other IRI outside of the vocabulary.
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

impl<V: Vocab> AsIri for Lexicon<V> {
	fn as_iri(&self) -> Iri {
		match self {
			Lexicon::Id(id) => id.as_iri(),
			Lexicon::Iri(iri) => iri.as_iri(),
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
}
