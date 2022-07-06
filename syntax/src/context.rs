use iref::{IriBuf, IriRefBuf};
use rdf_types::BlankIdBuf;
use locspan::Meta;
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
#[stripped_ignore(M)]
pub enum ContextEntry<M> {
	One(Meta<Context<M>, M>),
	Many(Vec<Meta<Context<M>, M>>)
}

impl<M> ContextEntry<M> {
	pub fn as_slice(&self) -> &[Meta<Context<M>, M>] {
		match self {
			Self::One(c) => std::slice::from_ref(c),
			Self::Many(list) => list
		}
	}
}

/// Context.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(M)]
pub enum Context<M> {
	Null,
	IriRef(#[stripped] IriRefBuf),
	Definition(ContextDefinition<M>)
}

/// Context definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(M)]
pub struct ContextDefinition<M> {
	#[stripped_option_deref]
	pub base: Option<Meta<Nullable<IriRefBuf>, M>>,
	#[stripped_option_deref]
	pub import: Option<Meta<IriRefBuf, M>>,
	pub language: Option<Meta<Nullable<LenientLanguageTagBuf>, M>>,
	pub direction: Option<Meta<Nullable<Direction>, M>>,
	pub propagate: Option<Meta<bool, M>>,
	pub protected: Option<Meta<bool, M>>,
	pub type_: Option<Meta<ContextType<M>, M>>,
	pub version: Option<Meta<Version, M>>,
	pub vocab: Option<Meta<Nullable<Vocab>, M>>,
	pub bindings: Bindings<M>
}

/// Context bindings.
#[derive(PartialEq, Eq, Clone, Derivative)]
#[derivative(Default(bound=""))]
pub struct Bindings<M>(IndexMap<Key, TermBinding<M>>);

impl<M> Bindings<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn get(&self, key: &Key) -> Option<&TermBinding<M>> {
		self.0.get(key)
	}

	pub fn iter(&self) -> indexmap::map::Iter<Key, TermBinding<M>> {
		self.0.iter()
	}
}

impl<M> locspan::StrippedPartialEq for Bindings<M> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.len() == other.len() && self.iter().all(|(key, a)| {
			other.get(key).map(|b| a.stripped_eq(b)).unwrap_or(false)
		})
	}
}

/// Term binding.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(M)]
pub struct TermBinding<M> {
	#[stripped_ignore]
	key_metadata: M,
	definition: Meta<Nullable<TermDefinition<M>>, M>
}

/// Term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(M)]
pub enum TermDefinition<M> {
	Iri(#[stripped] IriBuf),
	CompactIri(#[stripped] CompactIriBuf),
	Blank(#[stripped] BlankIdBuf),
	Expanded(ExpandedTermDefinition<M>)
}

/// Expanded term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone)]
#[stripped_ignore(M)]
pub struct ExpandedTermDefinition<M> {
	pub id: Option<Meta<Nullable<Id>, M>>,
	pub type_: Option<Meta<Nullable<TermDefinitionType>, M>>,
	pub context: Option<Box<Meta<ContextEntry<M>, M>>>,
	pub reverse: Option<Meta<Key, M>>,
	pub index: Option<Meta<Index, M>>,
	pub language: Option<Meta<Nullable<LenientLanguageTagBuf>, M>>,
	pub direction: Option<Meta<Nullable<Direction>, M>>,
	pub container: Option<Meta<Nullable<Container>, M>>,
	pub nest: Option<Meta<Nest, M>>,
	pub prefix: Option<Meta<bool, M>>,
	pub propagate: Option<Meta<bool, M>>,
	pub protected: Option<Meta<bool, M>>
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
#[stripped_ignore(M)]
pub struct ContextType<M> {
	pub container: Meta<TypeContainer, M>,
	pub protected: Option<Meta<bool, M>>
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

pub struct InvalidContextEntry;

impl<M> TryFrom<json_syntax::Value<M>> for ContextEntry<M> {
	type Error = InvalidContextEntry;

	fn try_from(value: json_syntax::Value<M>) -> Result<Self, Self::Error> {
		todo!("context entry from JSON")
	}
}