use crate::object::{InvalidExpandedJson, TryFromJson};
use crate::Term;
use contextual::{AsRefWithContext, DisplayWithContext};
use iref::{Iri, IriBuf};
use locspan::Meta;
use locspan_derive::*;
use rdf_types::{BlankId, BlankIdBuf, InvalidBlankId, Vocabulary, VocabularyMut};
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
pub enum Reference<I = IriBuf, B = BlankIdBuf> {
	Valid(ValidReference<I, B>),

	/// Invalid reference.
	Invalid(#[stripped] String),
}

impl<I, B, M> TryFromJson<I, B, M> for Reference<I, B> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<I, B>,
		Meta(value, meta): locspan::Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, locspan::Meta<InvalidExpandedJson<M>, M>> {
		match value {
			json_syntax::Value::String(s) => match Iri::new(s.as_str()) {
				Ok(iri) => Ok(Meta(
					Self::Valid(ValidReference::Id(vocabulary.insert(iri))),
					meta,
				)),
				Err(_) => match BlankId::new(s.as_str()) {
					Ok(blank_id) => Ok(Meta(
						Self::Valid(ValidReference::Blank(vocabulary.insert_blank_id(blank_id))),
						meta,
					)),
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
			Ok(iri) => Self::Valid(ValidReference::Id(iri.into())),
			Err((_, s)) => match BlankIdBuf::new(s) {
				Ok(blank) => Self::Valid(ValidReference::Blank(blank.into())),
				Err(InvalidBlankId(s)) => Self::Invalid(s),
			},
		}
	}
}

impl<I, B> Reference<I, B> {
	pub fn id(id: I) -> Self {
		Self::Valid(ValidReference::Id(id))
	}

	pub fn blank(b: B) -> Self {
		Self::Valid(ValidReference::Blank(b))
	}

	pub fn from_string_in(vocabulary: &mut impl VocabularyMut<I, B>, s: String) -> Self {
		match Iri::new(&s) {
			Ok(iri) => Self::Valid(ValidReference::Id(vocabulary.insert(iri))),
			Err(_) => match BlankId::new(&s) {
				Ok(blank) => Self::Valid(ValidReference::Blank(vocabulary.insert_blank_id(blank))),
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
			Self::Valid(ValidReference::Blank(b)) => Some(b),
			_ => None,
		}
	}

	/// If the reference is a node identifier, returns the node IRI.
	///
	/// Returns `None` if it is a blank node reference.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Reference::Valid(ValidReference::Id(k)) => Some(k),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn into_term(self) -> Term<I, B> {
		Term::Ref(self)
	}

	pub fn as_ref(&self) -> Ref<I, B> {
		match self {
			Self::Valid(ValidReference::Id(t)) => Ref::Id(t),
			Self::Valid(ValidReference::Blank(id)) => Ref::Blank(id),
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
			Reference::Valid(ValidReference::Id(id)) => id.as_ref(),
			Reference::Valid(ValidReference::Blank(id)) => id.as_ref(),
			Reference::Invalid(id) => id.as_str(),
		}
	}
}

impl<T, B, N: Vocabulary<T, B>> AsRefWithContext<str, N> for Reference<T, B> {
	fn as_ref_with<'a>(&'a self, vocabulary: &'a N) -> &'a str {
		match self {
			Reference::Valid(ValidReference::Id(id)) => vocabulary.iri(id).unwrap().into_str(),
			Reference::Valid(ValidReference::Blank(id)) => {
				vocabulary.blank_id(id).unwrap().as_str()
			}
			Reference::Invalid(id) => id.as_str(),
		}
	}
}

impl<I: PartialEq, B> PartialEq<I> for Reference<I, B> {
	fn eq(&self, other: &I) -> bool {
		match self {
			Reference::Valid(ValidReference::Id(id)) => id == other,
			_ => false,
		}
	}
}

impl<T: PartialEq<str>, B: PartialEq<str>> PartialEq<str> for Reference<T, B> {
	fn eq(&self, other: &str) -> bool {
		match self {
			Reference::Valid(ValidReference::Id(iri)) => iri == other,
			Reference::Valid(ValidReference::Blank(blank)) => blank == other,
			Reference::Invalid(id) => id == other,
		}
	}
}

impl<'a, T, B> From<&'a Reference<T, B>> for Reference<&'a T, &'a B> {
	fn from(r: &'a Reference<T, B>) -> Reference<&'a T, &'a B> {
		match r {
			Reference::Valid(ValidReference::Id(id)) => Reference::Valid(ValidReference::Id(id)),
			Reference::Valid(ValidReference::Blank(id)) => {
				Reference::Valid(ValidReference::Blank(id))
			}
			Reference::Invalid(id) => Reference::Invalid(id.clone()),
		}
	}
}

impl<T, B> From<T> for Reference<T, B> {
	#[inline(always)]
	fn from(id: T) -> Reference<T, B> {
		Reference::Valid(ValidReference::Id(id))
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

impl<T: fmt::Display, B: fmt::Display> fmt::Display for Reference<T, B> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Reference::Valid(id) => id.fmt(f),
			Reference::Invalid(id) => id.fmt(f),
		}
	}
}

impl<T, B, N: Vocabulary<T, B>> DisplayWithContext<N> for Reference<T, B> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		match self {
			Reference::Valid(id) => id.fmt_with(vocabulary, f),
			Reference::Invalid(id) => id.fmt(f),
		}
	}
}

impl<T: fmt::Debug, B: fmt::Debug> fmt::Debug for Reference<T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Reference::Valid(id) => write!(f, "Reference::Valid({:?})", id),
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
		Reference::Valid(ValidReference::Id(self))
	}
}

/// Valid node reference.
///
/// ## Layout
///
/// The memory layout of a valid node reference is designed to match
/// the layout of a `Reference`.
#[derive(
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	StrippedPartialEq,
	StrippedEq,
	StrippedHash,
	Debug,
)]
#[stripped(T, B)]
pub enum ValidReference<T = IriBuf, B = BlankIdBuf> {
	Id(#[stripped] T),
	Blank(#[stripped] B),
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
		Reference::Valid(r)
	}
}

impl<T, B> TryFrom<Reference<T, B>> for ValidReference<T, B> {
	type Error = String;

	fn try_from(r: Reference<T, B>) -> Result<Self, Self::Error> {
		match r {
			Reference::Valid(r) => Ok(r),
			Reference::Invalid(id) => Err(id),
		}
	}
}

impl<'a, T, B> TryFrom<&'a Reference<T, B>> for &'a ValidReference<T, B> {
	type Error = &'a String;

	fn try_from(r: &'a Reference<T, B>) -> Result<Self, Self::Error> {
		match r {
			Reference::Valid(r) => Ok(r),
			Reference::Invalid(id) => Err(id),
		}
	}
}

impl<'a, T, B> TryFrom<&'a mut Reference<T, B>> for &'a mut ValidReference<T, B> {
	type Error = &'a mut String;

	fn try_from(r: &'a mut Reference<T, B>) -> Result<Self, Self::Error> {
		match r {
			Reference::Valid(r) => Ok(r),
			Reference::Invalid(id) => Err(id),
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

impl<T, B, N: Vocabulary<T, B>> DisplayWithContext<N> for ValidReference<T, B> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		match self {
			Self::Id(i) => vocabulary.iri(i).unwrap().fmt(f),
			Self::Blank(b) => vocabulary.blank_id(b).unwrap().fmt(f),
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
