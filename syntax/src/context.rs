use iref::{IriBuf, IriRefBuf};
use rdf_types::BlankIdBuf;
use locspan::{Loc, Location};
use indexmap::IndexMap;
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
pub enum Context<S, P> {
	Null,
	IriRef(IriRefBuf),
	Definition(ContextDefinition<S, P>)
}

/// Context definition.
pub struct ContextDefinition<S, P> {
	pub base: Option<Loc<Nullable<IriRefBuf>, S, P>>,
	pub import: Option<Loc<IriRefBuf, S, P>>,
	pub language: Option<Loc<Nullable<LenientLanguageTagBuf>, S, P>>,
	pub direction: Option<Loc<Nullable<Direction>, S, P>>,
	pub propagate: Option<Loc<bool, S, P>>,
	pub protected: Option<Loc<bool, S, P>>,
	pub type_: Option<Loc<ContextType<S, P>, S, P>>,
	pub version: Option<Loc<Version, S, P>>,
	pub vocab: Option<Loc<Nullable<Vocab>, S, P>>,
	pub bindings: IndexMap<Key, TermBinding<S, P>>
}

/// Term binding.
pub struct TermBinding<S, P> {
	key_location: Location<S, P>,
	definition: Loc<Nullable<TermDefinition<S, P>>, S, P>
}

/// Term definition.
pub enum TermDefinition<S, P> {
	Iri(IriBuf),
	CompactIri(CompactIriBuf),
	Blank(BlankIdBuf),
	Expanded(ExpandedTermDefinition<S, P>)
}

/// Expanded term definition.
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Nest {
	Nest,
	Term(String)
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Index {
	Iri(IriBuf),
	CompactIri(CompactIriBuf),
	Term(String),
}

pub enum Id {
	Iri(IriBuf),
	Blank(BlankIdBuf),
	CompactIri(CompactIriBuf),
	Term(String),
	Keyword(Keyword)
}

pub enum TermDefinitionType {
	Iri(IriBuf),
	CompactIri(CompactIriBuf),
	Term(String),
	Keyword(TypeKeyword)
}

/// Subset of keyword acceptable for as value for the `@type` entry
/// of an expanded term definition.
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub enum Version {
	V1_1
}

pub struct Import;

#[derive(Clone, Copy)]
pub struct ContextType<S, P> {
	pub container: Loc<TypeContainer, S, P>,
	pub protected: Option<Loc<bool, S, P>>
}

#[derive(Clone, Copy)]
pub enum TypeContainer {
	Set
}

pub enum Vocab {
	IriRef(IriRefBuf),
	CompactIri(CompactIriBuf),
	Blank(BlankIdBuf),
	Term(String)
}