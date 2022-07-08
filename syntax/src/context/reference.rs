use crate::{CompactIri, Container, ExpandableRef, Keyword, LenientLanguageTag};
use derivative::Derivative;
use iref::{Iri, IriRef};
use locspan::{Meta, StrippedPartialEq};
use rdf_types::BlankId;
use std::fmt;

use super::*;

pub trait AnyContextEntry: Sized + StrippedPartialEq + Clone + Send + Sync {
	type Metadata: Clone + Send + Sync;

	type Definition: AnyContextDefinition<Self> + Send + Sync;
	type Definitions<'a>: Iterator<Item = Meta<ContextRef<'a, Self::Definition>, Self::Metadata>>
		+ Send
		+ Sync
	where
		Self: 'a;

	fn as_entry_ref(
		&self,
	) -> ContextEntryRef<Self::Metadata, Self::Definition, Self::Definitions<'_>>;
}

impl<M: Clone + Send + Sync> AnyContextEntry for ContextEntry<M> {
	type Metadata = M;

	type Definition = ContextDefinition<M>;
	type Definitions<'a> = ManyContexts<'a, M> where M: 'a;

	fn as_entry_ref(&self) -> ContextEntryRef<M> {
		self.into()
	}
}

/// Reference to a context entry.
pub enum ContextEntryRef<'a, M, D = ContextDefinition<M>, C = ManyContexts<'a, M>> {
	One(Meta<ContextRef<'a, D>, M>),
	Many(C),
}

impl<'a, M: Clone, D, C: Iterator<Item = Meta<ContextRef<'a, D>, M>>> IntoIterator
	for ContextEntryRef<'a, M, D, C>
{
	type Item = Meta<ContextRef<'a, D>, M>;
	type IntoIter = ContextEntryIter<'a, M, D, C>;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::One(i) => ContextEntryIter::One(Some(i)),
			Self::Many(m) => ContextEntryIter::Many(m),
		}
	}
}

impl<'a, M: Clone> From<&'a ContextEntry<M>> for ContextEntryRef<'a, M> {
	fn from(e: &'a ContextEntry<M>) -> Self {
		match e {
			ContextEntry::One(c) => Self::One(c.borrow_value().cast()),
			ContextEntry::Many(m) => Self::Many(ManyContexts(m.iter())),
		}
	}
}

pub struct ManyContexts<'a, M>(std::slice::Iter<'a, Meta<Context<M>, M>>);

impl<'a, M: Clone> Iterator for ManyContexts<'a, M> {
	type Item = Meta<ContextRef<'a, ContextDefinition<M>>, M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|c| c.borrow_value().cast())
	}
}

pub enum ContextEntryIter<'a, M, D = ContextDefinition<M>, C = ManyContexts<'a, M>> {
	One(Option<Meta<ContextRef<'a, D>, M>>),
	Many(C),
}

impl<'a, M, D, C: Iterator<Item = Meta<ContextRef<'a, D>, M>>> Iterator
	for ContextEntryIter<'a, M, D, C>
{
	type Item = Meta<ContextRef<'a, D>, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(i) => i.take(),
			Self::Many(m) => m.next(),
		}
	}
}

/// Reference to context.
pub enum ContextRef<'a, D> {
	Null,
	IriRef(IriRef<'a>),
	Definition(&'a D),
}

impl<'a, M> From<&'a Context<M>> for ContextRef<'a, ContextDefinition<M>> {
	fn from(c: &'a Context<M>) -> Self {
		match c {
			Context::Null => ContextRef::Null,
			Context::IriRef(i) => ContextRef::IriRef(i.as_iri_ref()),
			Context::Definition(d) => ContextRef::Definition(d),
		}
	}
}

pub trait AnyContextDefinition<C: AnyContextEntry>: Sized {
	type Bindings<'a>: Iterator<Item = (KeyRef<'a>, TermBindingRef<'a, C>)> + Send + Sync
	where
		Self: 'a,
		C: 'a,
		C::Metadata: 'a;

	fn base(&self) -> Option<Meta<Nullable<IriRef>, C::Metadata>>;
	fn import(&self) -> Option<Meta<IriRef, C::Metadata>>;
	fn language(&self) -> Option<Meta<Nullable<LenientLanguageTag>, C::Metadata>>;
	fn direction(&self) -> Option<Meta<Nullable<Direction>, C::Metadata>>;
	fn propagate(&self) -> Option<Meta<bool, C::Metadata>>;
	fn protected(&self) -> Option<Meta<bool, C::Metadata>>;
	fn type_(&self) -> Option<Meta<ContextType<C::Metadata>, C::Metadata>>;
	fn version(&self) -> Option<Meta<Version, C::Metadata>>;
	fn vocab(&self) -> Option<Meta<Nullable<VocabRef>, C::Metadata>>;
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
				_ => None,
			},
			KeyOrKeyword::Key(k) => self.get_binding(k).map(EntryRef::Definition),
		}
	}
}

pub enum EntryRef<'a, C: AnyContextEntry> {
	Base(Meta<Nullable<IriRef<'a>>, C::Metadata>),
	Import(Meta<IriRef<'a>, C::Metadata>),
	Language(Meta<Nullable<LenientLanguageTag<'a>>, C::Metadata>),
	Direction(Meta<Nullable<Direction>, C::Metadata>),
	Propagate(Meta<bool, C::Metadata>),
	Protected(Meta<bool, C::Metadata>),
	Type(Meta<ContextType<C::Metadata>, C::Metadata>),
	Version(Meta<Version, C::Metadata>),
	Vocab(Meta<Nullable<VocabRef<'a>>, C::Metadata>),
	Definition(TermBindingRef<'a, C>),
}

impl<M: Clone + Send + Sync> AnyContextDefinition<ContextEntry<M>> for ContextDefinition<M> {
	type Bindings<'a> = Bindings<'a, M> where M: 'a;

	fn base(&self) -> Option<Meta<Nullable<IriRef>, M>> {
		self.base
			.as_ref()
			.map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_iri_ref())))
	}

	fn import(&self) -> Option<Meta<IriRef, M>> {
		self.import.as_ref().map(|v| v.borrow_value().cast())
	}

	fn language(&self) -> Option<Meta<Nullable<LenientLanguageTag>, M>> {
		self.language
			.as_ref()
			.map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref())))
	}

	fn direction(&self) -> Option<Meta<Nullable<Direction>, M>> {
		self.direction.clone()
	}

	fn propagate(&self) -> Option<Meta<bool, M>> {
		self.propagate.clone()
	}

	fn protected(&self) -> Option<Meta<bool, M>> {
		self.protected.clone()
	}

	fn type_(&self) -> Option<Meta<ContextType<M>, M>> {
		self.type_.clone()
	}

	fn version(&self) -> Option<Meta<Version, M>> {
		self.version.clone()
	}

	fn vocab(&self) -> Option<Meta<Nullable<VocabRef>, M>> {
		self.vocab
			.as_ref()
			.map(|v| v.borrow_value().map(|v| v.as_ref().cast()))
	}

	fn bindings(&self) -> Self::Bindings<'_> {
		Bindings(self.bindings.iter())
	}

	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<ContextEntry<M>>> {
		self.bindings.get(key).map(Into::into)
	}
}

pub struct Bindings<'a, M>(indexmap::map::Iter<'a, Key, TermBinding<M>>);

impl<'a, M: Clone + Send + Sync> Iterator for Bindings<'a, M> {
	type Item = (KeyRef<'a>, TermBindingRef<'a, ContextEntry<M>>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(key, def)| (key.into(), def.into()))
	}
}

pub struct TermBindingRef<'a, C: AnyContextEntry> {
	pub key_metadata: C::Metadata,
	pub definition: Meta<Nullable<TermDefinitionRef<'a, C>>, C::Metadata>,
}

impl<'a, C: AnyContextEntry> TermBindingRef<'a, C> {
	pub fn key_metadata(&self) -> &C::Metadata {
		&self.key_metadata
	}
}

impl<'a, M: Clone + Send + Sync> From<&'a TermBinding<M>> for TermBindingRef<'a, ContextEntry<M>> {
	fn from(b: &'a TermBinding<M>) -> Self {
		Self {
			key_metadata: b.key_metadata.clone(),
			definition: b.definition.borrow_value().map(|b| b.as_ref().cast()),
		}
	}
}

/// Term definition.
pub enum TermDefinitionRef<'a, C: AnyContextEntry> {
	Iri(Iri<'a>),
	CompactIri(&'a CompactIri),
	Blank(&'a BlankId),
	Expanded(ExpandedTermDefinitionRef<'a, C>),
}

impl<'a, C: AnyContextEntry> TermDefinitionRef<'a, C> {
	pub fn is_expanded(&self) -> bool {
		matches!(self, Self::Expanded(_))
	}
}

impl<'a, M: Clone + Send + Sync> From<&'a TermDefinition<M>>
	for TermDefinitionRef<'a, ContextEntry<M>>
{
	fn from(d: &'a TermDefinition<M>) -> Self {
		match d {
			TermDefinition::Iri(i) => Self::Iri(i.as_iri()),
			TermDefinition::CompactIri(c) => Self::CompactIri(c),
			TermDefinition::Blank(b) => Self::Blank(b),
			TermDefinition::Expanded(e) => Self::Expanded(e.into()),
		}
	}
}

/// Expanded term definition.
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ExpandedTermDefinitionRef<'a, C: AnyContextEntry> {
	pub id: Option<Meta<Nullable<IdRef<'a>>, C::Metadata>>,
	pub type_: Option<Meta<Nullable<TermDefinitionTypeRef<'a>>, C::Metadata>>,
	pub context: Option<Meta<&'a C, C::Metadata>>,
	pub reverse: Option<Meta<KeyRef<'a>, C::Metadata>>,
	pub index: Option<Meta<IndexRef<'a>, C::Metadata>>,
	pub language: Option<Meta<Nullable<LenientLanguageTag<'a>>, C::Metadata>>,
	pub direction: Option<Meta<Nullable<Direction>, C::Metadata>>,
	pub container: Option<Meta<Nullable<Container>, C::Metadata>>,
	pub nest: Option<Meta<NestRef<'a>, C::Metadata>>,
	pub prefix: Option<Meta<bool, C::Metadata>>,
	pub propagate: Option<Meta<bool, C::Metadata>>,
	pub protected: Option<Meta<bool, C::Metadata>>,
}

impl<'a, C: AnyContextEntry> From<Meta<Nullable<TermDefinitionRef<'a, C>>, C::Metadata>>
	for ExpandedTermDefinitionRef<'a, C>
{
	fn from(Meta(d, loc): Meta<Nullable<TermDefinitionRef<'a, C>>, C::Metadata>) -> Self {
		match d {
			Nullable::Null => {
				// If `value` is null, convert it to a map consisting of a single entry
				// whose key is @id and whose value is null.
				Self {
					id: Some(Meta(Nullable::Null, loc)),
					..Default::default()
				}
			}
			Nullable::Some(TermDefinitionRef::Iri(i)) => Self {
				id: Some(Meta(Nullable::Some(IdRef::Iri(i)), loc)),
				..Default::default()
			},
			Nullable::Some(TermDefinitionRef::CompactIri(i)) => Self {
				id: Some(Meta(Nullable::Some(IdRef::CompactIri(i)), loc)),
				..Default::default()
			},
			Nullable::Some(TermDefinitionRef::Blank(i)) => Self {
				id: Some(Meta(Nullable::Some(IdRef::Blank(i)), loc)),
				..Default::default()
			},
			Nullable::Some(TermDefinitionRef::Expanded(e)) => e,
		}
	}
}

impl<'a, M: Clone + Send + Sync> From<&'a ExpandedTermDefinition<M>>
	for ExpandedTermDefinitionRef<'a, ContextEntry<M>>
{
	fn from(d: &'a ExpandedTermDefinition<M>) -> Self {
		Self {
			id: d
				.id
				.as_ref()
				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			type_: d
				.type_
				.as_ref()
				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			context: d.context.as_ref().map(|v| v.borrow_value().cast()),
			reverse: d.reverse.as_ref().map(|v| v.borrow_value().cast()),
			index: d.index.as_ref().map(|v| v.borrow_value().cast()),
			language: d
				.language
				.as_ref()
				.map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref()))),
			direction: d.direction.clone(),
			container: d.container.clone(),
			nest: d.nest.as_ref().map(|v| v.borrow_value().cast()),
			prefix: d.prefix.clone(),
			propagate: d.propagate.clone(),
			protected: d.protected.clone(),
		}
	}
}

pub enum NestRef<'a> {
	Nest,
	Term(&'a str),
}

impl<'a> NestRef<'a> {
	pub fn to_owned(self) -> Nest {
		match self {
			Self::Nest => Nest::Nest,
			Self::Term(t) => Nest::Term(t.to_owned()),
		}
	}
}

impl<'a> From<&'a Nest> for NestRef<'a> {
	fn from(n: &'a Nest) -> Self {
		match n {
			Nest::Nest => Self::Nest,
			Nest::Term(t) => Self::Term(t),
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
			Self::Term(t) => Index::Term(t.to_owned()),
		}
	}
}

impl<'a> From<&'a Index> for IndexRef<'a> {
	fn from(i: &'a Index) -> Self {
		match i {
			Index::Iri(i) => Self::Iri(i.as_iri()),
			Index::CompactIri(c) => Self::CompactIri(c),
			Index::Term(t) => Self::Term(t),
		}
	}
}

impl<'a> From<IndexRef<'a>> for KeyRef<'a> {
	fn from(i: IndexRef<'a>) -> Self {
		match i {
			IndexRef::Iri(i) => KeyRef::Iri(i),
			IndexRef::CompactIri(i) => KeyRef::CompactIri(i),
			IndexRef::Term(t) => KeyRef::Term(t),
		}
	}
}

#[derive(Clone, Copy)]
pub enum IdRef<'a> {
	Iri(Iri<'a>),
	Blank(&'a BlankId),
	CompactIri(&'a CompactIri),
	Term(&'a str),
	Keyword(Keyword),
}

impl<'a> IdRef<'a> {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Iri(i) => i.as_str(),
			Self::Blank(i) => i.as_str(),
			Self::CompactIri(i) => i.as_str(),
			Self::Term(t) => t,
			Self::Keyword(k) => k.into_str(),
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
			IdRef::Keyword(k) => Self::Keyword(k),
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
			IdRef::Keyword(k) => Self::Keyword(k),
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
			Self::Keyword(k) => k.fmt(f),
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
			Id::Keyword(k) => Self::Keyword(*k),
		}
	}
}

#[derive(Clone, Copy)]
pub enum TermDefinitionTypeRef<'a> {
	Iri(Iri<'a>),
	CompactIri(&'a CompactIri),
	Term(&'a str),
	Keyword(TypeKeyword),
}

impl<'a> From<&'a TermDefinitionType> for TermDefinitionTypeRef<'a> {
	fn from(t: &'a TermDefinitionType) -> Self {
		match t {
			TermDefinitionType::Iri(i) => Self::Iri(i.as_iri()),
			TermDefinitionType::CompactIri(c) => Self::CompactIri(c),
			TermDefinitionType::Term(t) => Self::Term(t),
			TermDefinitionType::Keyword(k) => Self::Keyword(*k),
		}
	}
}

impl<'a> From<TermDefinitionTypeRef<'a>> for KeyOrKeywordRef<'a> {
	fn from(d: TermDefinitionTypeRef<'a>) -> Self {
		match d {
			TermDefinitionTypeRef::Iri(i) => Self::Key(KeyRef::Iri(i)),
			TermDefinitionTypeRef::CompactIri(i) => Self::Key(KeyRef::CompactIri(i)),
			TermDefinitionTypeRef::Term(t) => Self::Key(KeyRef::Term(t)),
			TermDefinitionTypeRef::Keyword(k) => Self::Keyword(k.into()),
		}
	}
}

impl<'a> From<TermDefinitionTypeRef<'a>> for ExpandableRef<'a> {
	fn from(d: TermDefinitionTypeRef<'a>) -> Self {
		match d {
			TermDefinitionTypeRef::Iri(i) => Self::Key(KeyRef::Iri(i)),
			TermDefinitionTypeRef::CompactIri(i) => Self::Key(KeyRef::CompactIri(i)),
			TermDefinitionTypeRef::Term(t) => Self::Key(KeyRef::Term(t)),
			TermDefinitionTypeRef::Keyword(k) => Self::Keyword(k.into()),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyOrKeywordRef<'a> {
	Keyword(Keyword),
	Key(KeyRef<'a>),
}

impl<'a> KeyOrKeywordRef<'a> {
	pub fn to_owned(self) -> KeyOrKeyword {
		match self {
			Self::Keyword(k) => KeyOrKeyword::Keyword(k),
			Self::Key(k) => KeyOrKeyword::Key(k.to_owned()),
		}
	}
}

impl<'a> From<&'a KeyOrKeyword> for KeyOrKeywordRef<'a> {
	fn from(k: &'a KeyOrKeyword) -> Self {
		match k {
			KeyOrKeyword::Keyword(k) => Self::Keyword(*k),
			KeyOrKeyword::Key(k) => Self::Key(k.into()),
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
	Term(&'a str),
}

impl<'a> From<&'a Vocab> for VocabRef<'a> {
	fn from(v: &'a Vocab) -> Self {
		match v {
			Vocab::IriRef(i) => Self::IriRef(i.as_iri_ref()),
			Vocab::Blank(b) => Self::Blank(b),
			Vocab::CompactIri(c) => Self::CompactIri(c),
			Vocab::Term(t) => Self::Term(t),
		}
	}
}

impl<'a> From<VocabRef<'a>> for ExpandableRef<'a> {
	fn from(v: VocabRef<'a>) -> Self {
		match v {
			VocabRef::IriRef(i) => ExpandableRef::IriRef(i),
			VocabRef::Blank(i) => ExpandableRef::Key(KeyRef::Blank(i)),
			VocabRef::CompactIri(i) => ExpandableRef::Key(KeyRef::CompactIri(i)),
			VocabRef::Term(t) => ExpandableRef::Key(KeyRef::Term(t)),
		}
	}
}
