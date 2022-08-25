use crate::object::{InvalidExpandedJson, TryFromJson};
use crate::{DisplayWithNamespace, Namespace, NamespaceMut, Term};
use iref::{Iri, IriBuf};
use locspan::Meta;
use locspan_derive::*;
use rdf_types::{BlankId, BlankIdBuf, InvalidBlankId};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;

/// Node reference.
///
/// Used to reference a node across a document or to a remote document.
/// It can be an identifier (IRI), a blank node identifier for local blank nodes
/// or an invalid reference (a string that is neither an IRI nor blank node identifier).
#[derive(
	Clone, PartialEq, Eq, Hash, PartialOrd, Ord, StrippedPartialEq, StrippedEq, StrippedHash,
)]
#[stripped(I, B)]
#[repr(u8)]
pub enum Reference<I = IriBuf, B = BlankIdBuf> {
	/// Node identifier, essentially an IRI.
	Id(#[stripped] I),

	/// Blank node identifier.
	Blank(#[stripped] B),

	/// Invalid reference.
	Invalid(#[stripped] String),
}

impl<I, B, M, C> TryFromJson<I, B, M, C> for Reference<I, B> {
	fn try_from_json_in(
		namespace: &mut impl NamespaceMut<I, B>,
		Meta(value, meta): locspan::Meta<json_ld_syntax::Value<M, C>, M>,
	) -> Result<Meta<Self, M>, locspan::Meta<InvalidExpandedJson, M>> {
		match value {
			json_ld_syntax::Value::String(s) => match Iri::new(s.as_str()) {
				Ok(iri) => Ok(Meta(Self::Id(namespace.insert(iri)), meta)),
				Err(_) => match BlankId::new(s.as_str()) {
					Ok(blank_id) => {
						Ok(Meta(Self::Blank(namespace.insert_blank_id(blank_id)), meta))
					}
					Err(_) => Ok(Meta(Self::Invalid(s.to_string()), meta)),
				},
			},
			_ => Err(Meta(InvalidExpandedJson::InvalidId, meta)),
		}
	}
}

impl<I: From<IriBuf>, B: From<BlankIdBuf>> Reference<I, B> {
	pub fn from_string(s: String) -> Self {
		match IriBuf::from_string(s) {
			Ok(iri) => Self::Id(iri.into()),
			Err((_, s)) => match BlankIdBuf::new(s) {
				Ok(blank) => Self::Blank(blank.into()),
				Err(InvalidBlankId(s)) => Self::Invalid(s),
			},
		}
	}
}

impl<I, B> Reference<I, B> {
	pub fn from_string_in(namespace: &mut impl NamespaceMut<I, B>, s: String) -> Self {
		match Iri::new(&s) {
			Ok(iri) => Self::Id(namespace.insert(iri)),
			Err(_) => match BlankId::new(&s) {
				Ok(blank) => Self::Blank(namespace.insert_blank_id(blank)),
				Err(_) => Self::Invalid(s),
			},
		}
	}

	/// Checks if this is a valid reference.
	///
	/// Returns `true` is this reference is a node identifier or a blank node identifier,
	/// `false` otherwise.
	#[inline(always)]
	pub fn is_valid(&self) -> bool {
		!matches!(self, Self::Invalid(_))
	}

	pub fn into_blank(self) -> Option<B> {
		match self {
			Self::Blank(b) => Some(b),
			_ => None,
		}
	}

	/// If the reference is a node identifier, returns the node IRI.
	///
	/// Returns `None` if it is a blank node reference.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Reference::Id(k) => Some(k),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn into_term(self) -> Term<I, B> {
		Term::Ref(self)
	}

	pub fn as_ref(&self) -> Ref<I, B> {
		match self {
			Self::Id(t) => Ref::Id(t),
			Self::Blank(id) => Ref::Blank(id),
			Self::Invalid(id) => Ref::Invalid(id.as_str()),
		}
	}
}

impl<I: AsRef<str>, B: AsRef<str>> Reference<I, B> {
	/// Get a string representation of the reference.
	///
	/// This will either return a string slice of an IRI, or a blank node identifier.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			Reference::Id(id) => id.as_ref(),
			Reference::Blank(id) => id.as_ref(),
			Reference::Invalid(id) => id.as_str(),
		}
	}
}

impl<'n, T, B, N: Namespace<T, B>> crate::namespace::WithNamespace<&'n Reference<T, B>, &'n N> {
	pub fn as_str(&self) -> &'n str {
		match self.0 {
			Reference::Id(id) => self.1.iri(id).unwrap().into_str(),
			Reference::Blank(id) => self.1.blank_id(id).unwrap().as_str(),
			Reference::Invalid(id) => id.as_str(),
		}
	}
}

impl<I: PartialEq, B> PartialEq<I> for Reference<I, B> {
	fn eq(&self, other: &I) -> bool {
		match self {
			Reference::Id(id) => id == other,
			_ => false,
		}
	}
}

impl<T: PartialEq<str>, B: PartialEq<str>> PartialEq<str> for Reference<T, B> {
	fn eq(&self, other: &str) -> bool {
		match self {
			Reference::Id(iri) => iri == other,
			Reference::Blank(blank) => blank == other,
			Reference::Invalid(id) => id == other,
		}
	}
}

impl<'a, T, B> From<&'a Reference<T, B>> for Reference<&'a T, &'a B> {
	fn from(r: &'a Reference<T, B>) -> Reference<&'a T, &'a B> {
		match r {
			Reference::Id(id) => Reference::Id(id),
			Reference::Blank(id) => Reference::Blank(id),
			Reference::Invalid(id) => Reference::Invalid(id.clone()),
		}
	}
}

impl<T, B> From<T> for Reference<T, B> {
	#[inline(always)]
	fn from(id: T) -> Reference<T, B> {
		Reference::Id(id)
	}
}

impl<T: PartialEq, B: PartialEq> PartialEq<Term<T, B>> for Reference<T, B> {
	#[inline]
	fn eq(&self, term: &Term<T, B>) -> bool {
		match term {
			Term::Ref(prop) => self == prop,
			_ => false,
		}
	}
}

impl<T: PartialEq, B: PartialEq> PartialEq<Reference<T, B>> for Term<T, B> {
	#[inline]
	fn eq(&self, r: &Reference<T, B>) -> bool {
		match self {
			Term::Ref(prop) => prop == r,
			_ => false,
		}
	}
}

impl<T, B> TryFrom<Term<T, B>> for Reference<T, B> {
	type Error = Term<T, B>;

	#[inline]
	fn try_from(term: Term<T, B>) -> Result<Reference<T, B>, Term<T, B>> {
		match term {
			Term::Ref(prop) => Ok(prop),
			term => Err(term),
		}
	}
}

// impl<J: JsonClone, K: utils::JsonFrom<J>, T: Id> utils::AsJson<J, K> for Reference<T, B> {
// 	#[inline]
// 	fn as_json_with(
// 		&self,
// 		meta: impl Clone + Fn(Option<&J::MetaData>) -> <K as Json>::MetaData,
// 	) -> K {
// 		match self {
// 			Reference::Id(id) => id.as_json(meta(None)),
// 			Reference::Blank(b) => b.as_json_with(meta(None)),
// 			Reference::Invalid(id) => id.as_json_with(meta(None)),
// 		}
// 	}
// }

// impl<K: generic_json::JsonBuild, T: Id> utils::AsAnyJson<K> for Reference<T, B> {
// 	#[inline]
// 	fn as_json_with(&self, meta: K::MetaData) -> K {
// 		match self {
// 			Reference::Id(id) => id.as_json(meta),
// 			Reference::Blank(b) => b.as_json_with(meta),
// 			Reference::Invalid(id) => id.as_json_with(meta),
// 		}
// 	}
// }

impl<T: fmt::Display, B: fmt::Display> fmt::Display for Reference<T, B> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Reference::Id(id) => id.fmt(f),
			Reference::Blank(b) => b.fmt(f),
			Reference::Invalid(id) => id.fmt(f),
		}
	}
}

impl<T, B, N: Namespace<T, B>> DisplayWithNamespace<N> for Reference<T, B> {
	fn fmt_with(&self, namespace: &N, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		match self {
			Reference::Id(i) => namespace.iri(i).unwrap().fmt(f),
			Reference::Blank(b) => namespace.blank_id(b).unwrap().fmt(f),
			Reference::Invalid(id) => id.fmt(f),
		}
	}
}

impl<T: fmt::Debug, B: fmt::Debug> fmt::Debug for Reference<T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Reference::Id(id) => write!(f, "Reference::Id({:?})", id),
			Reference::Blank(b) => write!(f, "Reference::Blank({:?})", b),
			Reference::Invalid(id) => write!(f, "Reference::Invalid({:?})", id),
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
/// fn get(&self, id: &Reference<T, B>) -> Objects;
/// ```
/// However building a `Reference` by hand can be tedious, especially while using [`Lexicon`](crate::Lexicon) and
/// [`Vocab`](crate::Vocab). It can be as verbose as `node.get(&Reference::Id(Lexicon::Id(MyVocab::Term)))`.
/// Thanks to `ToReference` which is implemented by `Lexicon<V>` for any type `V` implementing `Vocab`,
/// it is simplified into `node.get(MyVocab::Term)` (while the first syntax remains correct).
/// The signature of `get` becomes:
/// ```ignore
/// fn get<R: ToReference<T>>(self, id: R) -> Objects;
/// ```
pub trait ToReference<T, B> {
	/// The target type of the conversion, which can be borrowed as a `Reference<T, B>`.
	type Reference: Borrow<Reference<T, B>>;

	/// Convert the value into a reference.
	fn to_ref(self) -> Self::Reference;
}

impl<'a, T, B> ToReference<T, B> for &'a Reference<T, B> {
	type Reference = &'a Reference<T, B>;

	#[inline(always)]
	fn to_ref(self) -> Self::Reference {
		self
	}
}

impl<T, B> ToReference<T, B> for T {
	type Reference = Reference<T, B>;

	fn to_ref(self) -> Self::Reference {
		Reference::Id(self)
	}
}

/// Valid node reference.
///
/// ## Layout
///
/// The memory layout of a valid node reference is designed to match
/// the layout of a `Reference`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(target_pointer_width = "16", repr(u8, align(8)))]
#[cfg_attr(target_pointer_width = "32", repr(u8, align(16)))]
#[cfg_attr(target_pointer_width = "64", repr(u8, align(32)))]
pub enum ValidReference<T = IriBuf, B = BlankIdBuf> {
	Id(T),
	Blank(B),
}

impl<T, B> ValidReference<T, B> {
	pub fn into_rdf_subject(self) -> rdf_types::Subject<T, B> {
		match self {
			Self::Id(t) => rdf_types::Subject::Iri(t),
			Self::Blank(b) => rdf_types::Subject::Blank(b),
		}
	}

	pub fn as_rdf_subject(&self) -> rdf_types::Subject<&T, &B> {
		match self {
			Self::Id(t) => rdf_types::Subject::Iri(t),
			Self::Blank(b) => rdf_types::Subject::Blank(b),
		}
	}
}

impl<I, B> ValidReference<I, B> {
	/// If the reference is a node identifier, returns the node IRI.
	///
	/// Returns `None` if it is a blank node reference.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Self::Id(k) => Some(k),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn into_term(self) -> Term<I, B> {
		Term::Ref(self.into())
	}
}

impl<T: AsRef<str>, B: AsRef<str>> ValidReference<T, B> {
	/// Get a string representation of the reference.
	///
	/// This will either return a string slice of an IRI, or a blank node identifier.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			Self::Id(id) => id.as_ref(),
			Self::Blank(id) => id.as_ref(),
		}
	}
}

impl<T, B> From<ValidReference<T, B>> for Reference<T, B> {
	fn from(r: ValidReference<T, B>) -> Self {
		// This is safe because both types have the same internal representation over common variants.
		unsafe {
			let u = std::mem::transmute_copy(&r);
			std::mem::forget(r);
			u
		}
	}
}

impl<T, B> TryFrom<Reference<T, B>> for ValidReference<T, B> {
	type Error = String;

	fn try_from(r: Reference<T, B>) -> Result<Self, Self::Error> {
		match r {
			Reference::Id(id) => Ok(Self::Id(id)),
			Reference::Blank(id) => Ok(Self::Blank(id)),
			Reference::Invalid(id) => Err(id),
		}
	}
}

impl<'a, T, B> From<&'a ValidReference<T, B>> for &'a Reference<T, B> {
	fn from(r: &'a ValidReference<T, B>) -> Self {
		// This is safe because both types have the same internal representation over common variants.
		unsafe { std::mem::transmute(r) }
	}
}

impl<'a, T, B> TryFrom<&'a Reference<T, B>> for &'a ValidReference<T, B> {
	type Error = &'a String;

	fn try_from(r: &'a Reference<T, B>) -> Result<Self, Self::Error> {
		match r {
			Reference::Invalid(id) => Err(id),
			r => Ok({
				// This is safe because both types have the same internal representation over common variants.
				unsafe { std::mem::transmute(r) }
			}),
		}
	}
}

impl<'a, T, B> From<&'a mut ValidReference<T, B>> for &'a mut Reference<T, B> {
	fn from(r: &'a mut ValidReference<T, B>) -> Self {
		// This is safe because both types have the same internal representation over common variants.
		unsafe { std::mem::transmute(r) }
	}
}

impl<'a, T, B> TryFrom<&'a mut Reference<T, B>> for &'a mut ValidReference<T, B> {
	type Error = &'a mut String;

	fn try_from(r: &'a mut Reference<T, B>) -> Result<Self, Self::Error> {
		match r {
			Reference::Invalid(id) => Err(id),
			r => Ok({
				// This is safe because both types have the same internal representation over common variants.
				unsafe { std::mem::transmute(r) }
			}),
		}
	}
}

impl<T: fmt::Display, B: fmt::Display> fmt::Display for ValidReference<T, B> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => id.fmt(f),
			Self::Blank(b) => b.fmt(f),
		}
	}
}

impl<T, B, N: Namespace<T, B>> DisplayWithNamespace<N> for ValidReference<T, B> {
	fn fmt_with(&self, namespace: &N, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		match self {
			Self::Id(i) => namespace.iri(i).unwrap().fmt(f),
			Self::Blank(b) => namespace.blank_id(b).unwrap().fmt(f),
		}
	}
}

impl<T: fmt::Display, B: fmt::Display> crate::rdf::Display for ValidReference<T, B> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => write!(f, "<{}>", id),
			Self::Blank(b) => write!(f, "{}", b),
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
pub enum Ref<'a, T = IriBuf, B = BlankIdBuf> {
	/// Node identifier, essentially an IRI.
	Id(&'a T),

	/// Blank node identifier.
	Blank(&'a B),

	/// Invalid reference.
	Invalid(&'a str),
}
