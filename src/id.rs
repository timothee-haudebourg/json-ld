use crate::syntax::TermLike;
use generic_json::JsonBuild;
use iref::{AsIri, Iri, IriBuf};
use std::hash::Hash;

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
///
/// # Using `enum` types
/// If you know in advance which IRIs will be used by your implementation,
/// one possibility is to use a `enum` type as identifier.
/// This can be done throught the use of the [`Lexicon`](`crate::Lexicon`) type along with the
/// [`iref-enum`](https://crates.io/crates/iref-enum) crate:
/// ```
/// use iref_enum::*;
/// use json_ld::Lexicon;
/// use ijson::IValue;
///
/// /// Vocabulary used in the implementation.
/// #[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
/// #[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
/// #[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
/// #[iri_prefix("vocab" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
/// pub enum Vocab {
///   #[iri("rdfs:comment")] Comment,
///
///   #[iri("manifest:name")] Name,
///   #[iri("manifest:entries")] Entries,
///   #[iri("manifest:action")] Action,
///   #[iri("manifest:result")] Result,
///
///   #[iri("vocab:PositiveEvaluationTest")] PositiveEvalTest,
///   #[iri("vocab:NegativeEvaluationTest")] NegativeEvalTest,
///   #[iri("vocab:option")] Option,
///   #[iri("vocab:specVersion")] SpecVersion,
///   #[iri("vocab:processingMode")] ProcessingMode,
///   #[iri("vocab:expandContext")] ExpandContext,
///   #[iri("vocab:base")] Base
/// }
///
/// /// A fully functional identifier type.
/// pub type Id = Lexicon<Vocab>;
///
/// fn handle_node(node: &json_ld::Node<IValue, Id>) {
///   for name in node.get(Vocab::Name) { // <- note that we can directly use `Vocab` here.
///     println!("node name: {}", name.as_str().unwrap());
///   }
/// }
/// ```
pub trait Id: AsIri + Clone + PartialEq + Eq + Hash {
	/// Create an identifier from its IRI.
	fn from_iri(iri: Iri) -> Self;

	fn as_json<K: JsonBuild>(&self, meta: K::MetaData) -> K {
		K::string(self.as_iri().as_str().into(), meta)
	}
}

impl Id for IriBuf {
	fn from_iri(iri: Iri) -> IriBuf {
		iri.into()
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

// impl<J: JsonClone, K: util::JsonFrom<J>, T: Id> util::AsJson<J, K> for T {
// 	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
// 		self.as_iri().as_str().as_json_with(meta)
// 	}
// }
