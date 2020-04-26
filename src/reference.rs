use std::fmt;
use std::convert::TryFrom;
use std::borrow::Borrow;
use iref::{Iri, IriBuf};
use json::JsonValue;
use crate::{
	Id,
	BlankId,
	Lenient,
	syntax::{
		Term,
		TermLike,
	},
	util
};

/// Node reference.
///
/// Used to reference a node across a document or to a remote document.
/// It can be an identifier (IRI) or a blank node identifier for local blank nodes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Reference<T: Id = IriBuf> {
	/// Node identifier, essentially an IRI.
	Id(T),

	/// Blank node identifier.
	Blank(BlankId)
}

impl<T: Id> Reference<T> {
	/// Get a string representation of the reference.
	///
	/// This will either return a string slice of an IRI, or a blank node identifier.
	pub fn as_str(&self) -> &str {
		match self {
			Reference::Id(id) => id.as_iri().into_str(),
			Reference::Blank(id) => id.as_str()
		}
	}

	/// If the renference is a node identifier, returns the node IRI.
	///
	/// Returns `None` if it is a blank node reference.
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Reference::Id(k) => Some(k.as_iri()),
			Reference::Blank(_) => None
		}
	}
}

impl<T: Id> TermLike for Reference<T> {
	fn as_iri(&self) -> Option<Iri> {
		self.as_iri()
	}

	fn as_str(&self) -> &str {
		self.as_str()
	}
}

impl<T: Id + PartialEq> PartialEq<T> for Reference<T> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Reference::Id(id) => id == other,
			_ => false
		}
	}
}

impl<T: Id + PartialEq> PartialEq<T> for Lenient<Reference<T>> {
	fn eq(&self, other: &T) -> bool {
		match self {
			Lenient::Ok(Reference::Id(id)) => id == other,
			_ => false
		}
	}
}

impl<T: Id> From<T> for Reference<T> {
	fn from(id: T) -> Reference<T> {
		Reference::Id(id)
	}
}

impl<T: Id> TryFrom<Term<T>> for Reference<T> {
	type Error = Term<T>;

	fn try_from(term: Term<T>) -> Result<Reference<T>, Term<T>> {
		match term {
			Term::Ref(prop) => Ok(prop),
			term => Err(term)
		}
	}
}

impl<T: Id> From<BlankId> for Reference<T> {
	fn from(blank: BlankId) -> Reference<T> {
		Reference::Blank(blank)
	}
}

impl<T: Id> util::AsJson for Reference<T> {
	fn as_json(&self) -> JsonValue {
		match self {
			Reference::Id(id) => id.as_json(),
			Reference::Blank(b) => b.as_json()
		}
	}
}

impl<T: Id> fmt::Display for Reference<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Reference::Id(id) => id.as_str().fmt(f),
			Reference::Blank(b) => b.fmt(f)
		}
	}
}

/// Types that can be converted into a borrowed node reference.
///
/// This is a convenient trait is used to simplify the use of references.
/// For instance consider the [`Node::get`](crate::Node::get) method, used to get the objects associated to the
/// given reference property for a given node.
/// It essentially have the following signature:
/// ```
/// fn get(&self, id: &Reference<T>) -> Objects;
/// ```
/// However building a `Refrence` by hand can be tedious, especilly while using [`Lexicon`](crate::Lexicon) and
/// [`Vocab`](crate::Vocab). It can be as verbose as `node.get(&Reference::Id(Lexicon::Id(MyVocab::Term)))`.
/// Thanks to `ToReference` which is implemented by `Lexicon<V>` for any type `V` implementing `Vocab`,
/// it is simplified into `node.get(MyVocab::Term)` (while the first syntax remains correct) where
/// the signature of `get` becomes:
/// ```
/// fn get<R: ToReference<T>>(&self, id: R) -> Objects;
/// ```
pub trait ToReference<T: Id> {
	/// The target type of the convertion, which can be borrowed as a `Reference<T>`.
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
