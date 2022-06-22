use iref::{IriBuf, IriRefBuf};
use langtag::LanguageTagBuf;
use rdf_types::BlankIdBuf;
use locspan::Loc;
use crate::{
	Keyword,
	Nullable
};

mod reference;

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

use crate::Direction;

/// Context definition.
pub struct ContextDefinition<S, P> {
	pub base: Option<Loc<Nullable<IriRefBuf>, S, P>>,
	pub import: Option<Loc<IriRefBuf, S, P>>,
	pub language: Option<Loc<Nullable<LanguageTagBuf>, S, P>>,
	pub direction: Option<Loc<Direction, S, P>>,
	pub propagate: Option<Loc<bool, S, P>>,
	pub protected: Option<Loc<bool, S, P>>,
	pub type_: Option<Loc<ContextType<S, P>, S, P>>,
	pub version: Option<Loc<Version, S, P>>,
	pub vocab: Option<Loc<Nullable<Vocab<S, P>>, S, P>>,
	pub bindings: Vec<TermBinding<S, P>>
}

/// Term binding.
pub struct TermBinding<S, P> {
	term: Loc<String, S, P>,
	definition: Nullable<TermDefinition<S, P>>
}

/// Term definition.
pub enum TermDefinition<S, P> {
	Iri(IriBuf),
	CompactIri(CompactIri<S, P>),
	Blank(BlankIdBuf),
	Expanded(ExpandedTermDefinition<S, P>)
}

/// Expanded term definition.
pub struct ExpandedTermDefinition<S, P> {
	pub id: Option<Loc<Nullable<Id<S, P>>, S, P>>,
	pub type_: Option<Loc<Nullable<TermDefinitionType<S, P>>, S, P>>,
	pub context: Option<Box<Loc<ContextDefinition<S, P>, S, P>>>,
	pub reverse: Option<Loc<Reverse<S, P>, S, P>>,
	pub index: Option<Loc<Index<S, P>, S, P>>,
	pub language: Option<Loc<Nullable<LanguageTagBuf>, S, P>>,
	pub container: Option<Loc<Nullable<Container>, S, P>>,
	pub nest: Option<Loc<Nest, S, P>>,
	pub prefix: Option<Loc<bool, S, P>>,
	pub propagate: Option<Loc<bool, S, P>>,
	pub protected: Option<Loc<bool, S, P>>
}

pub enum Nest {
	Nest,
	Term(String)
}

/// Container value.
#[derive(Clone, Copy)]
pub enum Container {
	List,
	Set,
	Language,
	Index,
	Id,
	Graph,
	Type,
	SetIndex,
	SetId,
	SetGraph,
	SetType,
	SetLanguage,
	GraphId,
	GraphIndex,
	GraphIdSet,
	GraphIndexSet
}

pub enum Index<S, P> {
	Iri(IriBuf),
	CompactIri(CompactIri<S, P>),
	Term(String),
}

pub enum Id<S, P> {
	Iri(IriBuf),
	Blank(BlankIdBuf),
	CompactIri(CompactIri<S, P>),
	Term(String),
	Keyword(Keyword)
}

pub enum Reverse<S, P> {
	Iri(IriBuf),
	Blank(BlankIdBuf),
	CompactIri(CompactIri<S, P>),
	Term(String)
}

pub enum TermDefinitionType<S, P> {
	Iri(IriBuf),
	CompactIri(CompactIri<S, P>),
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

pub enum Vocab<S, P> {
	IriRef(IriRefBuf),
	CompactIri(CompactIri<S, P>),
	Blank(BlankIdBuf),
	Term(String)
}

pub struct CompactIri<S, P> {
	pub prefix: Option<Loc<String, S, P>>,
	pub suffix: Loc<String, S, P>
}