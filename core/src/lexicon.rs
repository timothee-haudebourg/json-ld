use crate::{Id, IntoId, ValidId};
use iref::{AsIri, Iri, IriBuf};
use std::fmt;
use std::hash::Hash;

/// Lexicon identifier.
///
/// # Example
/// The following example builds a lexicon from a statically known vocabulary, defined as an
/// `enum` type. It uses the [`iref-enum`](https://crates.io/crates/iref-enum)
/// crate to automatically derive the conversion of the from/into IRIs.
/// ```
/// # use json_ld_core as json_ld;
/// use iref_enum::*;
/// use json_ld::Lexicon;
///
/// /// Vocabulary used in the implementation.
/// #[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
/// #[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
/// #[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
/// #[iri_prefix("vocab" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
/// pub enum Vocab {
///     #[iri("rdfs:comment")] Comment,
///
///     #[iri("manifest:name")] Name,
///     #[iri("manifest:entries")] Entries,
///     #[iri("manifest:action")] Action,
///     #[iri("manifest:result")] Result,
///
///     #[iri("vocab:PositiveEvaluationTest")] PositiveEvalTest,
///     #[iri("vocab:NegativeEvaluationTest")] NegativeEvalTest,
///     #[iri("vocab:option")] Option,
///     #[iri("vocab:specVersion")] SpecVersion,
///     #[iri("vocab:processingMode")] ProcessingMode,
///     #[iri("vocab:expandContext")] ExpandContext,
///     #[iri("vocab:base")] Base
/// }
///
/// /// A fully functional identifier type.
/// pub type Id = Lexicon<Vocab>;
///
/// fn handle_node(node: &json_ld::Node<Id>) {
///   for name in node.get(Vocab::Name) { // <- note that we can directly use `Vocab` here.
///     println!("node name: {}", name.as_value().unwrap().as_str().unwrap());
///   }
/// }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub enum Lexicon<V> {
	/// Identifier from the known vocabulary.
	Id(V),

	/// Any other IRI outside of the vocabulary.
	Iri(IriBuf),
}

impl<V> Lexicon<V> {
	pub fn as_str(&self) -> &str
	where
		V: AsRef<str>,
	{
		match self {
			Self::Id(i) => i.as_ref(),
			Self::Iri(i) => i.as_str(),
		}
	}

	pub fn as_iri(&self) -> Iri
	where
		V: AsIri,
	{
		match self {
			Self::Id(i) => i.as_iri(),
			Self::Iri(i) => i.as_iri(),
		}
	}
}

impl<V: AsIri> AsIri for Lexicon<V> {
	fn as_iri(&self) -> Iri {
		self.as_iri()
	}
}

#[allow(clippy::derive_hash_xor_eq)]
impl<V: Hash> Hash for Lexicon<V> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Id(i) => i.hash(state),
			Self::Iri(i) => i.hash(state),
		}
	}
}

impl<T, B> IntoId<Lexicon<T>, B> for T {
	type Id = Id<Lexicon<T>, B>;

	fn to_ref(self) -> Self::Id {
		Id::Valid(ValidId::Iri(Lexicon::Id(self)))
	}
}

impl<V: fmt::Display> fmt::Display for Lexicon<V> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Lexicon::Id(id) => id.fmt(f),
			Lexicon::Iri(iri) => iri.fmt(f),
		}
	}
}
