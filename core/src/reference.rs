use crate::{
	utils::{self, AsAnyJson},
	Term,
	TermLike,
	Id,
};
use generic_json::{Json, JsonClone};
use iref::{AsIri, Iri, IriBuf};
use rdf_types::{BlankId, BlankIdBuf};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;

/// Node reference.
///
/// Used to reference a node across a document or to a remote document.
/// It can be an identifier (IRI), a blank node identifier for local blank nodes
/// or an invalid reference (a string that is neither an IRI nor blank node identifier).
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Reference<T = IriBuf> {
	/// Node identifier, essentially an IRI.
	Id(T),

	/// Blank node identifier.
	Blank(BlankIdBuf),

	/// Invalid reference.
	Invalid(String),
}

impl<T: AsIri> Reference<T> {
	/// Checks if this is a valid reference.
	///
	/// Returns `true` is this reference is a node identifier or a blank node identifier,
	/// `false` otherwise.
	#[inline(always)]
	pub fn is_valid(&self) -> bool {
		!matches!(self, Self::Invalid(_))
	}

	/// Get a string representation of the reference.
	///
	/// This will either return a string slice of an IRI, or a blank node identifier.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			Reference::Id(id) => id.as_iri().into_str(),
			Reference::Blank(id) => id.as_str(),
			Reference::Invalid(id) => id.as_str(),
		}
	}

	/// If the reference is a node identifier, returns the node IRI.
	///
	/// Returns `None` if it is a blank node reference.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Reference::Id(k) => Some(k.as_iri()),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn into_term(self) -> Term<T> {
		Term::Ref(self)
	}

	pub fn as_ref(&self) -> Ref<T> {
		match self {
			Self::Id(t) => Ref::Id(t),
			Self::Blank(id) => Ref::Blank(id),
			Self::Invalid(id) => Ref::Invalid(id.as_str()),
		}
	}
}

impl<T: AsIri> TermLike for Reference<T> {
	#[inline(always)]
	fn as_iri(&self) -> Option<Iri> {
		self.as_iri()
	}

	#[inline(always)]
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

impl<T: AsIri> PartialEq<str> for Reference<T> {
	fn eq(&self, other: &str) -> bool {
		match self {
			Reference::Id(id) => match Iri::from_str(other) {
				Ok(iri) => id.as_iri() == iri,
				Err(_) => false,
			},
			Reference::Blank(id) => id.as_str() == other,
			Reference::Invalid(id) => id == other,
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
	#[inline(always)]
	fn from(id: T) -> Reference<T> {
		Reference::Id(id)
	}
}

impl<T: AsIri + PartialEq> PartialEq<Term<T>> for Reference<T> {
	#[inline]
	fn eq(&self, term: &Term<T>) -> bool {
		match term {
			Term::Ref(prop) => self == prop,
			_ => false,
		}
	}
}

impl<T: AsIri + PartialEq> PartialEq<Reference<T>> for Term<T> {
	#[inline]
	fn eq(&self, r: &Reference<T>) -> bool {
		match self {
			Term::Ref(prop) => prop == r,
			_ => false,
		}
	}
}

impl<T: AsIri> TryFrom<Term<T>> for Reference<T> {
	type Error = Term<T>;

	#[inline]
	fn try_from(term: Term<T>) -> Result<Reference<T>, Term<T>> {
		match term {
			Term::Ref(prop) => Ok(prop),
			term => Err(term),
		}
	}
}

impl<J: JsonClone, K: utils::JsonFrom<J>, T: Id> utils::AsJson<J, K> for Reference<T> {
	#[inline]
	fn as_json_with(
		&self,
		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
	) -> K {
		match self {
			Reference::Id(id) => id.as_json(meta(None)),
			Reference::Blank(b) => b.as_json_with(meta(None)),
			Reference::Invalid(id) => id.as_json_with(meta(None)),
		}
	}
}

impl<K: generic_json::JsonBuild, T: Id> utils::AsAnyJson<K> for Reference<T> {
	#[inline]
	fn as_json_with(&self, meta: K::MetaData) -> K {
		match self {
			Reference::Id(id) => id.as_json(meta),
			Reference::Blank(b) => b.as_json_with(meta),
			Reference::Invalid(id) => id.as_json_with(meta),
		}
	}
}

impl<T: AsIri> fmt::Display for Reference<T> {
	#[inline]
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
/// it is simplified into `node.get(MyVocab::Term)` (while the first syntax remains correct).
/// The signature of `get` becomes:
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

	#[inline(always)]
	fn to_ref(&self) -> Self::Reference {
		self
	}
}

/// Valid node reference.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ValidReference<T = IriBuf> {
	Id(T),
	Blank(BlankIdBuf),
}

impl<T: AsIri> ValidReference<T> {
	/// Get a string representation of the reference.
	///
	/// This will either return a string slice of an IRI, or a blank node identifier.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			Self::Id(id) => id.as_iri().into_str(),
			Self::Blank(id) => id.as_str(),
		}
	}

	/// If the reference is a node identifier, returns the node IRI.
	///
	/// Returns `None` if it is a blank node reference.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Self::Id(k) => Some(k.as_iri()),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn into_term(self) -> Term<T> {
		Term::Ref(self.into())
	}
}

impl<T> From<ValidReference<T>> for Reference<T> {
	fn from(r: ValidReference<T>) -> Self {
		// This is safe because both types have the same internal representation over common variants.
		unsafe {
			let u = std::mem::transmute_copy(&r);
			std::mem::forget(r);
			u
		}
	}
}

impl<T> TryFrom<Reference<T>> for ValidReference<T> {
	type Error = String;

	fn try_from(r: Reference<T>) -> Result<Self, Self::Error> {
		match r {
			Reference::Id(id) => Ok(Self::Id(id)),
			Reference::Blank(id) => Ok(Self::Blank(id)),
			Reference::Invalid(id) => Err(id),
		}
	}
}

impl<'a, T> From<&'a ValidReference<T>> for &'a Reference<T> {
	fn from(r: &'a ValidReference<T>) -> Self {
		// This is safe because both types have the same internal representation over common variants.
		unsafe { std::mem::transmute(r) }
	}
}

impl<'a, T> TryFrom<&'a Reference<T>> for &'a ValidReference<T> {
	type Error = &'a String;

	fn try_from(r: &'a Reference<T>) -> Result<Self, Self::Error> {
		match r {
			Reference::Invalid(id) => Err(id),
			r => Ok({
				// This is safe because both types have the same internal representation over common variants.
				unsafe { std::mem::transmute(r) }
			}),
		}
	}
}

impl<'a, T> From<&'a mut ValidReference<T>> for &'a mut Reference<T> {
	fn from(r: &'a mut ValidReference<T>) -> Self {
		// This is safe because both types have the same internal representation over common variants.
		unsafe { std::mem::transmute(r) }
	}
}

impl<'a, T> TryFrom<&'a mut Reference<T>> for &'a mut ValidReference<T> {
	type Error = &'a mut String;

	fn try_from(r: &'a mut Reference<T>) -> Result<Self, Self::Error> {
		match r {
			Reference::Invalid(id) => Err(id),
			r => Ok({
				// This is safe because both types have the same internal representation over common variants.
				unsafe { std::mem::transmute(r) }
			}),
		}
	}
}

impl<T: AsIri> From<BlankIdBuf> for ValidReference<T> {
	#[inline(always)]
	fn from(blank: BlankIdBuf) -> Self {
		Self::Blank(blank)
	}
}

impl<T: AsIri> fmt::Display for ValidReference<T> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => id.as_iri().fmt(f),
			Self::Blank(b) => b.fmt(f),
		}
	}
}

impl<T: AsIri> crate::rdf::Display for ValidReference<T> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => write!(f, "<{}>", id.as_iri()),
			Self::Blank(b) => write!(f, "{}", b),
		}
	}
}

impl<T: AsIri> fmt::Debug for ValidReference<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => write!(f, "ValidReference::Id({})", id.as_iri()),
			Self::Blank(b) => write!(f, "ValidReference::Blank({})", b),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use static_iref::iri;

	#[test]
	fn valid_reference_into_reference() {
		let tests = [
			(
				Reference::Id(iri!("https://example.com/a")),
				ValidReference::Id(iri!("https://example.com/a")),
			),
			(
				Reference::Blank(BlankIdBuf::from_suffix("a").unwrap()),
				ValidReference::Blank(BlankIdBuf::from_suffix("a").unwrap()),
			),
		];

		for (r, valid_r) in tests {
			let result: Reference<_> = valid_r.into();
			assert_eq!(r, result)
		}
	}

	#[test]
	fn borrowed_valid_reference_into_reference() {
		let tests = [
			(
				&Reference::Id(iri!("https://example.com/a")),
				&ValidReference::Id(iri!("https://example.com/a")),
			),
			(
				&Reference::Blank(BlankIdBuf::from_suffix("a").unwrap()),
				&ValidReference::Blank(BlankIdBuf::from_suffix("a").unwrap()),
			),
		];

		for (r, valid_r) in tests {
			let result: &Reference<_> = valid_r.into();
			assert_eq!(r, result)
		}
	}
}

/// Reference to a reference.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ref<'a, T = IriBuf> {
	/// Node identifier, essentially an IRI.
	Id(&'a T),

	/// Blank node identifier.
	Blank(&'a BlankId),

	/// Invalid reference.
	Invalid(&'a str),
}
