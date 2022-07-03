use std::fmt;
use iref::{Iri, IriRef};
use rdf_types::BlankId;
use locspan::{Meta, StrippedPartialEq};
use derivative::Derivative;
use crate::{Keyword, Container, CompactIri, LenientLanguageTag, ExpandableRef};

use super::*;

pub trait AnyContextEntry: Sized + StrippedPartialEq + Clone + Send + Sync {
	type Source: Clone + Send + Sync;
	type Span: Clone + Send + Sync;

	type Definition: AnyContextDefinition<Self> + Send + Sync;
	type Definitions<'a>: Iterator<Item=Loc<ContextRef<'a, Self::Definition>, Self::Source, Self::Span>> + Send + Sync where Self: 'a;

	fn as_entry_ref(&self) -> ContextEntryRef<Self::Source, Self::Span, Self::Definition, Self::Definitions<'_>>;
}

impl<S: Clone + Send + Sync, P: Clone + Send + Sync> AnyContextEntry for ContextEntry<S, P> {
	type Source = S;
	type Span = P;

	type Definition = ContextDefinition<S, P>;
	type Definitions<'a> = ManyContexts<'a, S, P> where S: 'a, P: 'a;

	fn as_entry_ref(&self) -> ContextEntryRef<S, P> {
		self.into()
	}
}

/// Reference to a context entry.
pub enum ContextEntryRef<'a, S, P, D=ContextDefinition<S, P>, M=ManyContexts<'a, S, P>> {
	One(Loc<ContextRef<'a, D>, S, P>),
	Many(M)
}

impl<'a, S: Clone, P: Clone, D, M: Iterator<Item=Loc<ContextRef<'a, D>, S, P>>> IntoIterator for ContextEntryRef<'a, S, P, D, M> {
	type Item = Loc<ContextRef<'a, D>, S, P>;
	type IntoIter = ContextEntryIter<'a, S, P, D, M>;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::One(i) => ContextEntryIter::One(Some(i)),
			Self::Many(m) => ContextEntryIter::Many(m)
		}
	}
}

impl<'a, S: Clone, P: Clone> From<&'a ContextEntry<S, P>> for ContextEntryRef<'a, S, P> {
	fn from(e: &'a ContextEntry<S, P>) -> Self {
		match e {
			ContextEntry::One(c) => Self::One(c.borrow_value().cast()),
			ContextEntry::Many(m) => Self::Many(ManyContexts(m.iter()))
		}
	}
}

pub struct ManyContexts<'a, S, P>(std::slice::Iter<'a, Loc<Context<S, P>, S, P>>);

impl<'a, S: Clone, P: Clone> Iterator for ManyContexts<'a, S, P> {
	type Item = Loc<ContextRef<'a, ContextDefinition<S, P>>, S, P>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|c| c.borrow_value().cast())
	}
}

pub enum ContextEntryIter<'a, S, P, D=ContextDefinition<S, P>, M=ManyContexts<'a, S, P>> {
	One(Option<Loc<ContextRef<'a, D>, S, P>>),
	Many(M)
}

impl<'a, S, P, D, M: Iterator<Item=Loc<ContextRef<'a, D>, S, P>>> Iterator for ContextEntryIter<'a, S, P, D, M> {
	type Item = Loc<ContextRef<'a, D>, S, P>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(i) => i.take(),
			Self::Many(m) => m.next()
		}
	}
}

/// Reference to context.
pub enum ContextRef<'a, D> {
	Null,
	IriRef(IriRef<'a>),
	Definition(&'a D)
}

impl<'a, S, P> From<&'a Context<S, P>> for ContextRef<'a, ContextDefinition<S, P>> {
	fn from(c: &'a Context<S, P>) -> Self {
		match c {
			Context::Null => ContextRef::Null,
			Context::IriRef(i) => ContextRef::IriRef(i.as_iri_ref()),
			Context::Definition(d) => ContextRef::Definition(d)
		}
	}
}

pub trait AnyContextDefinition<C: AnyContextEntry>: Sized {
	type Bindings<'a>: Iterator<Item=(KeyRef<'a>, TermBindingRef<'a, C>)> + Send + Sync where Self: 'a, C: 'a, C::Source: 'a, C::Span: 'a;

	fn base(&self) -> Option<Loc<Nullable<IriRef>, C::Source, C::Span>>;
	fn import(&self) -> Option<Loc<IriRef, C::Source, C::Span>>;
	fn language(&self) -> Option<Loc<Nullable<LenientLanguageTag>, C::Source, C::Span>>;
	fn direction(&self) -> Option<Loc<Nullable<Direction>, C::Source, C::Span>>;
	fn propagate(&self) -> Option<Loc<bool, C::Source, C::Span>>;
	fn protected(&self) -> Option<Loc<bool, C::Source, C::Span>>;
	fn type_(&self) -> Option<Loc<ContextType<C::Source, C::Span>, C::Source, C::Span>>;
	fn version(&self) -> Option<Loc<Version, C::Source, C::Span>>;
	fn vocab(&self) -> Option<Loc<Nullable<VocabRef>, C::Source, C::Span>>;
	fn bindings(&self) -> Self::Bindings<'_>;
	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<C>>;

	fn get(&self, key: &KeyOrKeyword) -> Option<EntryRef<C>> {
		match key {
			KeyOrKeyword::Keyword(k) => match k {
				Keyword::Base => self.base().map(EntryRef::Base),
				Keyword::Import => self.import().map(EntryRef::Import),
				Keyword::Language => self.language().map(EntryRef::Language),
				Keyword::Direction => self.direction().map(EntryRef::Direction),
				Keyword::Propagate => self.propagate().map(EntryRef::Propagate),
				Keyword::Protected => self.protected().map(EntryRef::Protected),
				Keyword::Type => self.type_().map(EntryRef::Type),
				Keyword::Version => self.version().map(EntryRef::Version),
				Keyword::Vocab => self.vocab().map(EntryRef::Vocab),
				_ => None
			}
			KeyOrKeyword::Key(k) => self.get_binding(k).map(EntryRef::Definition)
		}
	}
}

pub enum EntryRef<'a, C: AnyContextEntry> {
	Base(Loc<Nullable<IriRef<'a>>, C::Source, C::Span>),
	Import(Loc<IriRef<'a>, C::Source, C::Span>),
	Language(Loc<Nullable<LenientLanguageTag<'a>>, C::Source, C::Span>),
	Direction(Loc<Nullable<Direction>, C::Source, C::Span>),
	Propagate(Loc<bool, C::Source, C::Span>),
	Protected(Loc<bool, C::Source, C::Span>),
	Type(Loc<ContextType<C::Source, C::Span>, C::Source, C::Span>),
	Version(Loc<Version, C::Source, C::Span>),
	Vocab(Loc<Nullable<VocabRef<'a>>, C::Source, C::Span>),
	Definition(TermBindingRef<'a, C>)
}

impl<S: Clone + Send + Sync, P: Clone + Send + Sync> AnyContextDefinition<ContextEntry<S, P>> for ContextDefinition<S, P> {
	type Bindings<'a> = Bindings<'a, S, P> where S: 'a, P: 'a;

	fn base(&self) -> Option<Loc<Nullable<IriRef>, S, P>> {
		self.base.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_iri_ref())))
	}

	fn import(&self) -> Option<Loc<IriRef, S, P>> {
		self.import.as_ref().map(|v| v.borrow_value().cast())
	}

	fn language(&self) -> Option<Loc<Nullable<LenientLanguageTag>, S, P>> {
		self.language.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref())))
	}

	fn direction(&self) -> Option<Loc<Nullable<Direction>, S, P>> {
		self.direction.clone()
	}

	fn propagate(&self) -> Option<Loc<bool, S, P>> {
		self.propagate.clone()
	}

	fn protected(&self) -> Option<Loc<bool, S, P>> {
		self.protected.clone()
	}

	fn type_(&self) -> Option<Loc<ContextType<S, P>, S, P>> {
		self.type_.clone()
	}

	fn version(&self) -> Option<Loc<Version, S, P>> {
		self.version.clone()
	}

	fn vocab(&self) -> Option<Loc<Nullable<VocabRef>, S, P>> {
		self.vocab.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().cast()))
	}

	fn bindings(&self) -> Self::Bindings<'_> {
		Bindings(self.bindings.iter())
	}

	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<ContextEntry<S, P>>> {
		self.bindings.get(key).map(Into::into)
	}
}

pub struct Bindings<'a, S, P>(indexmap::map::Iter<'a, Key, TermBinding<S, P>>);

impl<'a, S: Clone + Send + Sync, P: Clone + Send + Sync> Iterator for Bindings<'a, S, P> {
	type Item = (KeyRef<'a>, TermBindingRef<'a, ContextEntry<S, P>>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(key, def)| (key.into(), def.into()))
	}
}

pub struct TermBindingRef<'a, C: AnyContextEntry> {
	pub key_location: Location<C::Source, C::Span>,
	pub definition: Loc<Nullable<TermDefinitionRef<'a, C>>, C::Source, C::Span>
}

impl<'a, C: AnyContextEntry> TermBindingRef<'a, C> {
	pub fn key_location(&self) -> &Location<C::Source, C::Span> {
		&self.key_location
	}
}

impl<'a, S: Clone + Send + Sync, P: Clone + Send + Sync> From<&'a TermBinding<S, P>> for TermBindingRef<'a, ContextEntry<S, P>> {
	fn from(b: &'a TermBinding<S, P>) -> Self {
		Self {
			key_location: b.key_location.clone(),
			definition: b.definition.borrow_value().map(|b| b.as_ref().cast())
		}
	}
}

/// Term definition.
pub enum TermDefinitionRef<'a, C: AnyContextEntry> {
	Iri(Iri<'a>),
	CompactIri(&'a CompactIri),
	Blank(&'a BlankId),
	Expanded(ExpandedTermDefinitionRef<'a, C>)
}

impl<'a, C: AnyContextEntry> TermDefinitionRef<'a, C> {
	pub fn is_expanded(&self) -> bool {
		matches!(self, Self::Expanded(_))
	}
}

impl<'a, S: Clone + Send + Sync, P: Clone + Send + Sync> From<&'a TermDefinition<S, P>> for TermDefinitionRef<'a, ContextEntry<S, P>> {
	fn from(d: &'a TermDefinition<S, P>) -> Self {
		match d {
			TermDefinition::Iri(i) => Self::Iri(i.as_iri()),
			TermDefinition::CompactIri(c) => Self::CompactIri(c),
			TermDefinition::Blank(b) => Self::Blank(b),
			TermDefinition::Expanded(e) => Self::Expanded(e.into())
		}
	}
}

/// Expanded term definition.
#[derive(Derivative)]
#[derivative(Default(bound=""))]
pub struct ExpandedTermDefinitionRef<'a, C: AnyContextEntry> {
	pub id: Option<Loc<Nullable<IdRef<'a>>, C::Source, C::Span>>,
	pub type_: Option<Loc<Nullable<TermDefinitionTypeRef<'a>>, C::Source, C::Span>>,
	pub context: Option<Loc<&'a C, C::Source, C::Span>>,
	pub reverse: Option<Loc<KeyRef<'a>, C::Source, C::Span>>,
	pub index: Option<Loc<IndexRef<'a>, C::Source, C::Span>>,
	pub language: Option<Loc<Nullable<LenientLanguageTag<'a>>, C::Source, C::Span>>,
	pub direction: Option<Loc<Nullable<Direction>, C::Source, C::Span>>,
	pub container: Option<Loc<Nullable<Container>, C::Source, C::Span>>,
	pub nest: Option<Loc<NestRef<'a>, C::Source, C::Span>>,
	pub prefix: Option<Loc<bool, C::Source, C::Span>>,
	pub propagate: Option<Loc<bool, C::Source, C::Span>>,
	pub protected: Option<Loc<bool, C::Source, C::Span>>
}

impl<'a, C: AnyContextEntry> From<Loc<Nullable<TermDefinitionRef<'a, C>>, C::Source, C::Span>> for ExpandedTermDefinitionRef<'a, C> {
	fn from(Meta(d, loc): Loc<Nullable<TermDefinitionRef<'a, C>>, C::Source, C::Span>) -> Self {
		match d {
			Nullable::Null => {
				// If `value` is null, convert it to a map consisting of a single entry
				// whose key is @id and whose value is null.
				Self { id: Some(Loc(Nullable::Null, loc)), ..Default::default() }
			},
			Nullable::Some(TermDefinitionRef::Iri(i)) => {
				Self { id: Some(Loc(Nullable::Some(IdRef::Iri(i)), loc)), ..Default::default() }
			}
			Nullable::Some(TermDefinitionRef::CompactIri(i)) => {
				Self { id: Some(Loc(Nullable::Some(IdRef::CompactIri(i)), loc)), ..Default::default() }
			}
			Nullable::Some(TermDefinitionRef::Blank(i)) => {
				Self { id: Some(Loc(Nullable::Some(IdRef::Blank(i)), loc)), ..Default::default() }
			}
			Nullable::Some(TermDefinitionRef::Expanded(e)) => {
				e
			}
		}
	}
}

impl<'a, S: Clone + Send + Sync, P: Clone + Send + Sync> From<&'a ExpandedTermDefinition<S, P>> for ExpandedTermDefinitionRef<'a, ContextEntry<S, P>> {
	fn from(d: &'a ExpandedTermDefinition<S, P>) -> Self {
		Self {
			id: d.id.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			type_: d.type_.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			context: d.context.as_ref().map(|v| v.borrow_value().cast()),
			reverse: d.reverse.as_ref().map(|v| v.borrow_value().cast()),
			index: d.index.as_ref().map(|v| v.borrow_value().cast()),
			language: d.language.as_ref().map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref()))),
			direction: d.direction.clone(),
			container: d.container.clone(),
			nest: d.nest.as_ref().map(|v| v.borrow_value().cast()),
			prefix: d.prefix.clone(),
			propagate: d.propagate.clone(),
			protected: d.protected.clone()
		}
	}
}

pub enum NestRef<'a> {
	Nest,
	Term(&'a str)
}

impl<'a> NestRef<'a> {
	pub fn to_owned(self) -> Nest {
		match self {
			Self::Nest => Nest::Nest,
			Self::Term(t) => Nest::Term(t.to_owned())
		}
	}
}

impl<'a> From<&'a Nest> for NestRef<'a> {
	fn from(n: &'a Nest) -> Self {
		match n {
			Nest::Nest => Self::Nest,
			Nest::Term(t) => Self::Term(t)
		}
	}
}

#[derive(Clone, Copy)]
pub enum IndexRef<'a> {
	Iri(Iri<'a>),
	CompactIri(&'a CompactIri),
	Term(&'a str),
}

impl<'a> IndexRef<'a> {
	pub fn to_owned(self) -> Index {
		match self {
			Self::Iri(i) => Index::Iri(i.to_owned()),
			Self::CompactIri(i) => Index::CompactIri(i.to_owned()),
			Self::Term(t) => Index::Term(t.to_owned())
		}
	}
}

impl<'a> From<&'a Index> for IndexRef<'a> {
	fn from(i: &'a Index) -> Self {
		match i {
			Index::Iri(i) => Self::Iri(i.as_iri()),
			Index::CompactIri(c) => Self::CompactIri(c),
			Index::Term(t) => Self::Term(t)
		}
	}
}

impl<'a> From<IndexRef<'a>> for KeyRef<'a> {
	fn from(i: IndexRef<'a>) -> Self {
		match i {
			IndexRef::Iri(i) => KeyRef::Iri(i),
			IndexRef::CompactIri(i) => KeyRef::CompactIri(i),
			IndexRef::Term(t) => KeyRef::Term(t)
		}
	}
}

#[derive(Clone, Copy)]
pub enum IdRef<'a> {
	Iri(Iri<'a>),
	Blank(&'a BlankId),
	CompactIri(&'a CompactIri),
	Term(&'a str),
	Keyword(Keyword)
}

impl<'a> IdRef<'a> {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Iri(i) => i.as_str(),
			Self::Blank(i) => i.as_str(),
			Self::CompactIri(i) => i.as_str(),
			Self::Term(t) => t,
			Self::Keyword(k) => k.into_str()
		}
	}

	pub fn is_keyword(&self) -> bool {
		matches!(self, Self::Keyword(_))
	}

	pub fn is_keyword_like(&self) -> bool {
		crate::is_keyword_like(self.as_str())
	}
}

impl<'a> From<IdRef<'a>> for KeyOrKeywordRef<'a> {
	fn from(i: IdRef<'a>) -> Self {
		match i {
			IdRef::Iri(i) => Self::Key(KeyRef::Iri(i)),
			IdRef::Blank(i) => Self::Key(KeyRef::Blank(i)),
			IdRef::CompactIri(i) => Self::Key(KeyRef::CompactIri(i)),
			IdRef::Term(t) => Self::Key(KeyRef::Term(t)),
			IdRef::Keyword(k) => Self::Keyword(k)
		}
	}
}

impl<'a> From<IdRef<'a>> for ExpandableRef<'a> {
	fn from(i: IdRef<'a>) -> Self {
		match i {
			IdRef::Iri(i) => Self::Key(KeyRef::Iri(i)),
			IdRef::Blank(i) => Self::Key(KeyRef::Blank(i)),
			IdRef::CompactIri(i) => Self::Key(KeyRef::CompactIri(i)),
			IdRef::Term(t) => Self::Key(KeyRef::Term(t)),
			IdRef::Keyword(k) => Self::Keyword(k)
		}
	}
}

impl<'a> fmt::Display for IdRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(i) => i.fmt(f),
			Self::Blank(i) => i.fmt(f),
			Self::CompactIri(i) => i.fmt(f),
			Self::Term(t) => t.fmt(f),
			Self::Keyword(k) => k.fmt(f)
		}
	}
}

impl<'a> From<&'a Id> for IdRef<'a> {
	fn from(i: &'a Id) -> Self {
		match i {
			Id::Iri(i) => Self::Iri(i.as_iri()),
			Id::Blank(b) => Self::Blank(b),
			Id::CompactIri(c) => Self::CompactIri(c),
			Id::Term(t) => Self::Term(t),
			Id::Keyword(k) => Self::Keyword(*k)
		}
	}
}

#[derive(Clone, Copy)]
pub enum TermDefinitionTypeRef<'a> {
	Iri(Iri<'a>),
	CompactIri(&'a CompactIri),
	Term(&'a str),
	Keyword(TypeKeyword)
}

impl<'a> From<&'a TermDefinitionType> for TermDefinitionTypeRef<'a> {
	fn from(t: &'a TermDefinitionType) -> Self {
		match t {
			TermDefinitionType::Iri(i) => Self::Iri(i.as_iri()),
			TermDefinitionType::CompactIri(c) => Self::CompactIri(c),
			TermDefinitionType::Term(t) => Self::Term(t),
			TermDefinitionType::Keyword(k) => Self::Keyword(*k)
		}
	}
}

impl<'a> From<TermDefinitionTypeRef<'a>> for KeyOrKeywordRef<'a> {
	fn from(d: TermDefinitionTypeRef<'a>) -> Self {
		match d {
			TermDefinitionTypeRef::Iri(i) => Self::Key(KeyRef::Iri(i)),
			TermDefinitionTypeRef::CompactIri(i) => Self::Key(KeyRef::CompactIri(i)),
			TermDefinitionTypeRef::Term(t) => Self::Key(KeyRef::Term(t)),
			TermDefinitionTypeRef::Keyword(k) => Self::Keyword(k.into())
		}
	}
}

impl<'a> From<TermDefinitionTypeRef<'a>> for ExpandableRef<'a> {
	fn from(d: TermDefinitionTypeRef<'a>) -> Self {
		match d {
			TermDefinitionTypeRef::Iri(i) => Self::Key(KeyRef::Iri(i)),
			TermDefinitionTypeRef::CompactIri(i) => Self::Key(KeyRef::CompactIri(i)),
			TermDefinitionTypeRef::Term(t) => Self::Key(KeyRef::Term(t)),
			TermDefinitionTypeRef::Keyword(k) => Self::Keyword(k.into())
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyOrKeywordRef<'a> {
	Keyword(Keyword),
	Key(KeyRef<'a>)
}

impl<'a> KeyOrKeywordRef<'a> {
	pub fn to_owned(self) -> KeyOrKeyword {
		match self {
			Self::Keyword(k) => KeyOrKeyword::Keyword(k),
			Self::Key(k) => KeyOrKeyword::Key(k.to_owned())
		}
	}
}

impl<'a> From<&'a KeyOrKeyword> for KeyOrKeywordRef<'a> {
	fn from(k: &'a KeyOrKeyword) -> Self {
		match k {
			KeyOrKeyword::Keyword(k) => Self::Keyword(*k),
			KeyOrKeyword::Key(k) => Self::Key(k.into())
		}
	}
}

impl<'a> From<KeyRef<'a>> for KeyOrKeywordRef<'a> {
	fn from(k: KeyRef<'a>) -> Self {
		Self::Key(k)
	}
}

impl<'a> From<&'a Key> for KeyOrKeywordRef<'a> {
	fn from(k: &'a Key) -> Self {
		Self::Key(k.into())
	}
}

#[derive(Clone, Copy)]
pub enum VocabRef<'a> {
	IriRef(IriRef<'a>),
	CompactIri(&'a CompactIri),
	Blank(&'a BlankId),
	Term(&'a str)
}

impl<'a> From<&'a Vocab> for VocabRef<'a> {
	fn from(v: &'a Vocab) -> Self {
		match v {
			Vocab::IriRef(i) => Self::IriRef(i.as_iri_ref()),
			Vocab::Blank(b) => Self::Blank(b),
			Vocab::CompactIri(c) => Self::CompactIri(c),
			Vocab::Term(t) => Self::Term(t)
		}
	}
}

impl<'a> From<VocabRef<'a>> for ExpandableRef<'a> {
	fn from(v: VocabRef<'a>) -> Self {
		match v {
			VocabRef::IriRef(i) => ExpandableRef::IriRef(i),
			VocabRef::Blank(i) => ExpandableRef::Key(KeyRef::Blank(i)),
			VocabRef::CompactIri(i) => ExpandableRef::Key(KeyRef::CompactIri(i)),
			VocabRef::Term(t) => ExpandableRef::Key(KeyRef::Term(t))
		}
	}
}