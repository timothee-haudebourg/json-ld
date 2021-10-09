use crate::{
	syntax::{Term, TermLike},
	util, BlankId, Id,
};
use generic_json::{Json, JsonClone};
use iref::{AsIri, Iri, IriBuf};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;

/// Node reference.
///
/// Used to reference a node across a document or to a remote document.
/// It can be an identifier (IRI) or a blank node identifier for local blank nodes.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Reference<T: AsIri = IriBuf> {
	/// Node identifier, essentially an IRI.
	Id(T),

	/// Blank node identifier.
	Blank(BlankId),

	/// Invalid reference.
	Invalid(String),
}

impl<T: AsIri> Reference<T> {
	/// Checks if this is a valid reference.
	///
	/// Returns `true` is this reference is a node identifier or a blank node identifier,
	/// `false` otherwise.
	pub fn is_valid(&self) -> bool {
		!matches!(self, Self::Invalid(_))
	}

	/// Get a string representation of the reference.
	///
	/// This will either return a string slice of an IRI, or a blank node identifier.
	pub fn as_str(&self) -> &str {
		match self {
			Reference::Id(id) => id.as_iri().into_str(),
			Reference::Blank(id) => id.as_str(),
			Reference::Invalid(id) => id.as_str(),
		}
	}

	/// If the renference is a node identifier, returns the node IRI.
	///
	/// Returns `None` if it is a blank node reference.
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Reference::Id(k) => Some(k.as_iri()),
			_ => None,
		}
	}

	pub fn into_term(self) -> Term<T> {
		Term::Ref(self)
	}
}

impl<T: AsIri> TermLike for Reference<T> {
	fn as_iri(&self) -> Option<Iri> {
		self.as_iri()
	}

	fn as_str(&self) -> &str {
		self.as_str()
	}
}

impl<T: AsIri + PartialEq> PartialEq<T> for Reference<T> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Reference::Id(id) => id == other,
			_ => false,
		}
	}
}

impl<'a, T: AsIri> From<&'a Reference<T>> for Reference<&'a T> {
	fn from(r: &'a Reference<T>) -> Reference<&'a T> {
		match r {
			Reference::Id(id) => Reference::Id(id),
			Reference::Blank(id) => Reference::Blank(id.clone()),
			Reference::Invalid(id) => Reference::Invalid(id.clone()),
		}
	}
}

impl<T: AsIri> From<T> for Reference<T> {
	fn from(id: T) -> Reference<T> {
		Reference::Id(id)
	}
}

impl<T: AsIri + PartialEq> PartialEq<Term<T>> for Reference<T> {
	fn eq(&self, term: &Term<T>) -> bool {
		match term {
			Term::Ref(prop) => self == prop,
			_ => false,
		}
	}
}

impl<T: AsIri + PartialEq> PartialEq<Reference<T>> for Term<T> {
	fn eq(&self, r: &Reference<T>) -> bool {
		match self {
			Term::Ref(prop) => prop == r,
			_ => false,
		}
	}
}

impl<T: AsIri> TryFrom<Term<T>> for Reference<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<Reference<T>, Term<T>> {
		match term {
			Term::Ref(prop) => Ok(prop),
			term => Err(term),
		}
	}
}

impl<T: AsIri> From<BlankId> for Reference<T> {
	fn from(blank: BlankId) -> Reference<T> {
		Reference::Blank(blank)
	}
}

impl<J: JsonClone, K: util::JsonFrom<J>, T: AsIri + util::AsJson<J, K>> util::AsJson<J, K>
	for Reference<T>
{
	fn as_json_with(&self, meta: impl Clone + Fn(Option<&J::MetaData>) -> K::MetaData) -> K {
		match self {
			Reference::Id(id) => id.as_json_with(meta),
			Reference::Blank(b) => b.as_json_with(meta),
			Reference::Invalid(id) => id.as_json_with(meta),
		}
	}
}

impl<T: AsIri> fmt::Display for Reference<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Reference::Id(id) => id.as_iri().fmt(f),
			Reference::Blank(b) => b.fmt(f),
			Reference::Invalid(id) => id.fmt(f),
		}
	}
}

impl<T: AsIri> fmt::Debug for Reference<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Reference::Id(id) => write!(f, "Reference::Id({})", id.as_iri()),
			Reference::Blank(b) => write!(f, "Reference::Blank({})", b),
			Reference::Invalid(id) => write!(f, "Reference::Invalid({})", id),
		}
	}
}

/// Types that can be converted into a borrowed node reference.
///
/// This is a convenient trait is used to simplify the use of references.
/// For instance consider the [`Node::get`](crate::Node::get) method, used to get the objects associated to the
/// given reference property for a given node.
/// It essentially have the following signature:
/// ```ignore
/// fn get(&self, id: &Reference<T>) -> Objects;
/// ```
/// However building a `Reference` by hand can be tedious, especially while using [`Lexicon`](crate::Lexicon) and
/// [`Vocab`](crate::Vocab). It can be as verbose as `node.get(&Reference::Id(Lexicon::Id(MyVocab::Term)))`.
/// Thanks to `ToReference` which is implemented by `Lexicon<V>` for any type `V` implementing `Vocab`,
/// it is simplified into `node.get(MyVocab::Term)` (while the first syntax remains correct) where
/// the signature of `get` becomes:
/// ```ignore
/// fn get<R: ToReference<T>>(&self, id: R) -> Objects;
/// ```
pub trait ToReference<T: Id> {
	/// The target type of the conversion, which can be borrowed as a `Reference<T>`.
	type Reference: Borrow<Reference<T>>;

	/// Convert the value into a reference.
	fn to_ref(&self) -> Self::Reference;
}

impl<'a, T: Id> ToReference<T> for &'a Reference<T> {
	type Reference = &'a Reference<T>;

	fn to_ref(&self) -> Self::Reference {
		self
	}
}
