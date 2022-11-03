use crate::object::{InvalidExpandedJson, TryFromJson};
use crate::Term;
use contextual::{AsRefWithContext, DisplayWithContext, WithContext};
use iref::{Iri, IriBuf};
use json_ld_syntax::IntoJsonWithContextMeta;
use locspan::Meta;
use locspan_derive::*;
use rdf_types::{
	BlankId, BlankIdBuf, BlankIdVocabulary, InvalidBlankId, IriVocabulary, Vocabulary,
	VocabularyMut,
};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;

pub use rdf_types::MetaGenerator as Generator;

pub type ValidVocabularyId<V> =
	ValidId<<V as IriVocabulary>::Iri, <V as BlankIdVocabulary>::BlankId>;

pub type MetaValidVocabularyId<V, M> = Meta<ValidVocabularyId<V>, M>;

pub type VocabularyId<V> = Id<<V as IriVocabulary>::Iri, <V as BlankIdVocabulary>::BlankId>;

pub type MetaVocabularyId<V, M> = Meta<VocabularyId<V>, M>;

/// Node identifier.
///
/// Used to reference a node across a document or to a remote document.
/// It can be an identifier (IRI), a blank node identifier for local blank nodes
/// or an invalid reference (a string that is neither an IRI nor blank node identifier).
#[derive(
	Clone, PartialEq, Eq, Hash, PartialOrd, Ord, StrippedPartialEq, StrippedEq, StrippedHash,
)]
#[locspan(stripped(I, B))]
pub enum Id<I = IriBuf, B = BlankIdBuf> {
	Valid(ValidId<I, B>),

	/// Invalid reference.
	Invalid(#[locspan(stripped)] String),
}

impl<I, B, M> TryFromJson<I, B, M> for Id<I, B> {
	fn try_from_json_in(
		vocabulary: &mut impl VocabularyMut<Iri = I, BlankId = B>,
		Meta(value, meta): locspan::Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, locspan::Meta<InvalidExpandedJson<M>, M>> {
		match value {
			json_syntax::Value::String(s) => match Iri::new(s.as_str()) {
				Ok(iri) => Ok(Meta(
					Self::Valid(ValidId::Iri(vocabulary.insert(iri))),
					meta,
				)),
				Err(_) => match BlankId::new(s.as_str()) {
					Ok(blank_id) => Ok(Meta(
						Self::Valid(ValidId::Blank(vocabulary.insert_blank_id(blank_id))),
						meta,
					)),
					Err(_) => Ok(Meta(Self::Invalid(s.to_string()), meta)),
				},
			},
			_ => Err(Meta(InvalidExpandedJson::InvalidId, meta)),
		}
	}
}

impl<I: From<IriBuf>, B: From<BlankIdBuf>> Id<I, B> {
	pub fn from_string(s: String) -> Self {
		match IriBuf::from_string(s) {
			Ok(iri) => Self::Valid(ValidId::Iri(iri.into())),
			Err((_, s)) => match BlankIdBuf::new(s) {
				Ok(blank) => Self::Valid(ValidId::Blank(blank.into())),
				Err(InvalidBlankId(s)) => Self::Invalid(s),
			},
		}
	}
}

impl<I, B> Id<I, B> {
	pub fn iri(iri: I) -> Self {
		Self::Valid(ValidId::Iri(iri))
	}

	pub fn blank(b: B) -> Self {
		Self::Valid(ValidId::Blank(b))
	}

	pub fn from_string_in(
		vocabulary: &mut impl VocabularyMut<Iri = I, BlankId = B>,
		s: String,
	) -> Self {
		match Iri::new(&s) {
			Ok(iri) => Self::Valid(ValidId::Iri(vocabulary.insert(iri))),
			Err(_) => match BlankId::new(&s) {
				Ok(blank) => Self::Valid(ValidId::Blank(vocabulary.insert_blank_id(blank))),
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
			Self::Valid(ValidId::Blank(b)) => Some(b),
			_ => None,
		}
	}

	/// If the reference is a node identifier, returns the node IRI.
	///
	/// Returns `None` if it is a blank node reference.
	#[inline(always)]
	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Id::Valid(ValidId::Iri(k)) => Some(k),
			_ => None,
		}
	}

	#[inline(always)]
	pub fn into_term(self) -> Term<I, B> {
		Term::Ref(self)
	}

	pub fn as_ref(&self) -> Ref<I, B> {
		match self {
			Self::Valid(ValidId::Iri(t)) => Ref::Iri(t),
			Self::Valid(ValidId::Blank(id)) => Ref::Blank(id),
			Self::Invalid(id) => Ref::Invalid(id.as_str()),
		}
	}
}

impl<I: AsRef<str>, B: AsRef<str>> Id<I, B> {
	/// Get a string representation of the reference.
	///
	/// This will either return a string slice of an IRI, or a blank node identifier.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		match self {
			Id::Valid(ValidId::Iri(id)) => id.as_ref(),
			Id::Valid(ValidId::Blank(id)) => id.as_ref(),
			Id::Invalid(id) => id.as_str(),
		}
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> AsRefWithContext<str, N> for Id<T, B> {
	fn as_ref_with<'a>(&'a self, vocabulary: &'a N) -> &'a str {
		match self {
			Id::Valid(ValidId::Iri(id)) => vocabulary.iri(id).unwrap().into_str(),
			Id::Valid(ValidId::Blank(id)) => vocabulary.blank_id(id).unwrap().as_str(),
			Id::Invalid(id) => id.as_str(),
		}
	}
}

impl<I: PartialEq, B> PartialEq<I> for Id<I, B> {
	fn eq(&self, other: &I) -> bool {
		match self {
			Id::Valid(ValidId::Iri(id)) => id == other,
			_ => false,
		}
	}
}

impl<T: PartialEq<str>, B: PartialEq<str>> PartialEq<str> for Id<T, B> {
	fn eq(&self, other: &str) -> bool {
		match self {
			Id::Valid(ValidId::Iri(iri)) => iri == other,
			Id::Valid(ValidId::Blank(blank)) => blank == other,
			Id::Invalid(id) => id == other,
		}
	}
}

impl<'a, T, B> From<&'a Id<T, B>> for Id<&'a T, &'a B> {
	fn from(r: &'a Id<T, B>) -> Id<&'a T, &'a B> {
		match r {
			Id::Valid(ValidId::Iri(id)) => Id::Valid(ValidId::Iri(id)),
			Id::Valid(ValidId::Blank(id)) => Id::Valid(ValidId::Blank(id)),
			Id::Invalid(id) => Id::Invalid(id.clone()),
		}
	}
}

impl<T, B> From<T> for Id<T, B> {
	#[inline(always)]
	fn from(id: T) -> Id<T, B> {
		Id::Valid(ValidId::Iri(id))
	}
}

impl<T: PartialEq, B: PartialEq> PartialEq<Term<T, B>> for Id<T, B> {
	#[inline]
	fn eq(&self, term: &Term<T, B>) -> bool {
		match term {
			Term::Ref(prop) => self == prop,
			_ => false,
		}
	}
}

impl<T: PartialEq, B: PartialEq> PartialEq<Id<T, B>> for Term<T, B> {
	#[inline]
	fn eq(&self, r: &Id<T, B>) -> bool {
		match self {
			Term::Ref(prop) => prop == r,
			_ => false,
		}
	}
}

impl<T, B> TryFrom<Term<T, B>> for Id<T, B> {
	type Error = Term<T, B>;

	#[inline]
	fn try_from(term: Term<T, B>) -> Result<Id<T, B>, Term<T, B>> {
		match term {
			Term::Ref(prop) => Ok(prop),
			term => Err(term),
		}
	}
}

impl<T: fmt::Display, B: fmt::Display> fmt::Display for Id<T, B> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Id::Valid(id) => id.fmt(f),
			Id::Invalid(id) => id.fmt(f),
		}
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> DisplayWithContext<N> for Id<T, B> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		match self {
			Id::Valid(id) => id.fmt_with(vocabulary, f),
			Id::Invalid(id) => id.fmt(f),
		}
	}
}

impl<T: fmt::Debug, B: fmt::Debug> fmt::Debug for Id<T, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Id::Valid(id) => write!(f, "Id::Valid({:?})", id),
			Id::Invalid(id) => write!(f, "Id::Invalid({:?})", id),
		}
	}
}

impl<T, B, M, N: Vocabulary<Iri = T, BlankId = B>> IntoJsonWithContextMeta<M, N> for Id<T, B> {
	fn into_json_meta_with(self, meta: M, context: &N) -> Meta<json_syntax::Value<M>, M> {
		Meta(self.into_with(context).to_string().into(), meta)
	}
}

/// Types that can be converted into a borrowed node reference.
///
/// This is a convenient trait is used to simplify the use of references.
/// For instance consider the [`Node::get`](crate::Node::get) method, used to get the objects associated to the
/// given reference property for a given node.
/// It essentially have the following signature:
/// ```ignore
/// fn get(&self, id: &Id<T, B>) -> Objects;
/// ```
/// However building a `Id` by hand can be tedious, especially while using [`Lexicon`](crate::Lexicon) and
/// [`Vocab`](crate::Vocab). It can be as verbose as `node.get(&Id::Id(Lexicon::Id(MyVocab::Term)))`.
/// Thanks to `IntoId` which is implemented by `Lexicon<V>` for any type `V` implementing `Vocab`,
/// it is simplified into `node.get(MyVocab::Term)` (while the first syntax remains correct).
/// The signature of `get` becomes:
/// ```ignore
/// fn get<R: IntoId<T>>(self, id: R) -> Objects;
/// ```
pub trait IntoId<T, B> {
	/// The target type of the conversion, which can be borrowed as a `Id<T, B>`.
	type Id: Borrow<Id<T, B>>;

	/// Convert the value into a reference.
	fn to_ref(self) -> Self::Id;
}

impl<'a, T, B> IntoId<T, B> for &'a Id<T, B> {
	type Id = &'a Id<T, B>;

	#[inline(always)]
	fn to_ref(self) -> Self::Id {
		self
	}
}

impl<T, B> IntoId<T, B> for T {
	type Id = Id<T, B>;

	fn to_ref(self) -> Self::Id {
		Id::Valid(ValidId::Iri(self))
	}
}

pub use rdf_types::Subject as ValidId;

impl<T, B> From<ValidId<T, B>> for Id<T, B> {
	fn from(r: ValidId<T, B>) -> Self {
		Id::Valid(r)
	}
}

impl<T, B> TryFrom<Id<T, B>> for ValidId<T, B> {
	type Error = String;

	fn try_from(r: Id<T, B>) -> Result<Self, Self::Error> {
		match r {
			Id::Valid(r) => Ok(r),
			Id::Invalid(id) => Err(id),
		}
	}
}

impl<'a, T, B> TryFrom<&'a Id<T, B>> for &'a ValidId<T, B> {
	type Error = &'a String;

	fn try_from(r: &'a Id<T, B>) -> Result<Self, Self::Error> {
		match r {
			Id::Valid(r) => Ok(r),
			Id::Invalid(id) => Err(id),
		}
	}
}

impl<'a, T, B> TryFrom<&'a mut Id<T, B>> for &'a mut ValidId<T, B> {
	type Error = &'a mut String;

	fn try_from(r: &'a mut Id<T, B>) -> Result<Self, Self::Error> {
		match r {
			Id::Valid(r) => Ok(r),
			Id::Invalid(id) => Err(id),
		}
	}
}

impl<T: fmt::Display, B: fmt::Display> crate::rdf::Display for ValidId<T, B> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => write!(f, "<{}>", id),
			Self::Blank(b) => write!(f, "{}", b),
		}
	}
}

/// Id to a reference.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ref<'a, T = IriBuf, B = BlankIdBuf> {
	/// Node identifier, essentially an IRI.
	Iri(&'a T),

	/// Blank node identifier.
	Blank(&'a B),

	/// Invalid reference.
	Invalid(&'a str),
}

pub trait IdentifyAll<T, B, M> {
	fn identify_all_with<N: Vocabulary<Iri = T, BlankId = B>, G: Generator<N, M>>(
		&mut self,
		vocabulary: &mut N,
		generator: G,
	) where
		M: Clone;

	fn identify_all<G: Generator<(), M>>(&mut self, generator: G)
	where
		M: Clone,
		(): Vocabulary<Iri = T, BlankId = B>;
}
