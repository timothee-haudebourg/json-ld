use iref::{IriBuf, IriRefBuf};
use rdf_types::BlankIdBuf;
use locspan::{Loc, Location};
use locspan_derive::StrippedPartialEq;
use indexmap::IndexMap;
use derivative::Derivative;
use crate::{
	Keyword,
	Container,
	Nullable,
	CompactIriBuf,
	Direction,
	LenientLanguageTagBuf
};

mod key;
mod reference;

pub use key::*;
pub use reference::*;

/// Context entry.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(S, P)]
pub enum ContextEntry<S, P> {
	One(Loc<Context<S, P>, S, P>),
	Many(Vec<Loc<Context<S, P>, S, P>>)
}

impl<S, P> ContextEntry<S, P> {
	pub fn as_slice(&self) -> &[Loc<Context<S, P>, S, P>] {
		match self {
			Self::One(c) => std::slice::from_ref(c),
			Self::Many(list) => list
		}
	}
}

/// Context.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(S, P)]
pub enum Context<S, P> {
	Null,
	IriRef(#[stripped] IriRefBuf),
	Definition(ContextDefinition<S, P>)
}

/// Context definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(S, P)]
pub struct ContextDefinition<S, P> {
	#[stripped_option_deref]
	pub base: Option<Loc<Nullable<IriRefBuf>, S, P>>,
	#[stripped_option_deref]
	pub import: Option<Loc<IriRefBuf, S, P>>,
	pub language: Option<Loc<Nullable<LenientLanguageTagBuf>, S, P>>,
	pub direction: Option<Loc<Nullable<Direction>, S, P>>,
	pub propagate: Option<Loc<bool, S, P>>,
	pub protected: Option<Loc<bool, S, P>>,
	pub type_: Option<Loc<ContextType<S, P>, S, P>>,
	pub version: Option<Loc<Version, S, P>>,
	pub vocab: Option<Loc<Nullable<Vocab>, S, P>>,
	pub bindings: Bindings<S, P>
}

/// Context bindings.
#[derive(PartialEq, Eq, Clone, Derivative)]
#[derivative(Default(bound=""))]
pub struct Bindings<S, P>(IndexMap<Key, TermBinding<S, P>>);

impl<S, P> Bindings<S, P> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn get(&self, key: &Key) -> Option<&TermBinding<S, P>> {
		self.0.get(key)
	}

	pub fn iter(&self) -> indexmap::map::Iter<Key, TermBinding<S, P>> {
		self.0.iter()
	}
}

impl<S, P> locspan::StrippedPartialEq for Bindings<S, P> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.len() == other.len() && self.iter().all(|(key, a)| {
			other.get(key).map(|b| a.stripped_eq(b)).unwrap_or(false)
		})
	}
}

/// Term binding.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(S, P)]
pub struct TermBinding<S, P> {
	#[stripped_ignore]
	key_location: Location<S, P>,
	definition: Loc<Nullable<TermDefinition<S, P>>, S, P>
}

/// Term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(S, P)]
pub enum TermDefinition<S, P> {
	Iri(#[stripped] IriBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Blank(#[stripped] BlankIdBuf),
	Expanded(ExpandedTermDefinition<S, P>)
}

/// Expanded term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(S, P)]
pub struct ExpandedTermDefinition<S, P> {
	pub id: Option<Loc<Nullable<Id>, S, P>>,
	pub type_: Option<Loc<Nullable<TermDefinitionType>, S, P>>,
	pub context: Option<Box<Loc<ContextEntry<S, P>, S, P>>>,
	pub reverse: Option<Loc<Key, S, P>>,
	pub index: Option<Loc<Index, S, P>>,
	pub language: Option<Loc<Nullable<LenientLanguageTagBuf>, S, P>>,
	pub direction: Option<Loc<Nullable<Direction>, S, P>>,
	pub container: Option<Loc<Nullable<Container>, S, P>>,
	pub nest: Option<Loc<Nest, S, P>>,
	pub prefix: Option<Loc<bool, S, P>>,
	pub propagate: Option<Loc<bool, S, P>>,
	pub protected: Option<Loc<bool, S, P>>
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nest {
	Nest,
	Term(#[stripped] String)
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Index {
	Iri(#[stripped] IriBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Term(#[stripped] String),
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Id {
	Iri(#[stripped] IriBuf),
	Blank(#[stripped] BlankIdBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Term(#[stripped] String),
	Keyword(Keyword)
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TermDefinitionType {
	Iri(#[stripped] IriBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Term(#[stripped] String),
	Keyword(TypeKeyword)
}

/// Subset of keyword acceptable for as value for the `@type` entry
/// of an expanded term definition.
#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeKeyword {
	Id,
	Json,
	None,
	Vocab
}

impl From<TypeKeyword> for Keyword {
	fn from(k: TypeKeyword) -> Self {
		match k {
			TypeKeyword::Id => Self::Id,
			TypeKeyword::Json => Self::Json,
			TypeKeyword::None => Self::None,
			TypeKeyword::Vocab => Self::Vocab
		}
	}
}

/// Version number.
/// 
/// The only allowed value is a number with the value `1.1`.
#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Version {
	V1_1
}

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Import;

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
#[stripped_ignore(S, P)]
pub struct ContextType<S, P> {
	pub container: Loc<TypeContainer, S, P>,
	pub protected: Option<Loc<bool, S, P>>
}

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeContainer {
	Set
}

#[derive(Clone, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Vocab {
	IriRef(#[stripped] IriRefBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Blank(#[stripped] BlankIdBuf),
	Term(#[stripped] String)
}