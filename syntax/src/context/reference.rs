use crate::{CompactIri, ContainerRef, ExpandableRef, Keyword, LenientLanguageTag};
use derivative::Derivative;
use iref::{Iri, IriRef};
use locspan::{Meta, StrippedPartialEq};
use rdf_types::BlankId;
use std::fmt;

use super::*;

pub trait AnyContextEntry: Sized + StrippedPartialEq + Clone + Send + Sync {
	type Metadata: Clone + Send + Sync;

	type Definition: AnyContextDefinition<ContextEntry = Self> + Send + Sync;
	type Definitions<'a>: Iterator<Item = Meta<ContextRef<'a, Self::Definition>, Self::Metadata>>
		+ Send
		+ Sync
	where
		Self: 'a;

	fn as_entry_ref(
		&self,
	) -> ContextEntryRef<Self::Metadata, Self::Definition, Self::Definitions<'_>>;
}

pub trait AnyContextEntryMut {
	fn append(&mut self, context: Self);
}

impl<M: Clone + Send + Sync> AnyContextEntry for ContextEntry<M> {
	type Metadata = M;

	type Definition = ContextDefinition<M>;
	type Definitions<'a> = ManyContexts<'a, M> where M: 'a;

	fn as_entry_ref(&self) -> ContextEntryRef<M> {
		self.into()
	}
}

impl<M> AnyContextEntryMut for ContextEntry<M> {
	fn append(&mut self, context: Self) {
		match self {
			Self::One(a) => {
				let a = unsafe { core::ptr::read(a) };

				let contexts = match context {
					Self::One(b) => vec![a, b],
					Self::Many(b) => {
						let mut contexts = vec![a];
						contexts.extend(b);
						contexts
					}
				};

				unsafe { core::ptr::write(self, Self::Many(contexts)) }
			}
			Self::Many(a) => match context {
				Self::One(b) => a.push(b),
				Self::Many(b) => a.extend(b),
			},
		}
	}
}

/// Reference to a context entry.
#[derive(Derivative)]
#[derivative(Clone(bound = "M: Clone, C: Clone"))]
pub enum ContextEntryRef<'a, M, D = ContextDefinition<M>, C = ManyContexts<'a, M>> {
	One(Meta<ContextRef<'a, D>, M>),
	Many(C),
}

impl<'a, M, D, C> ContextEntryRef<'a, M, D, C> {
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Many(_))
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::One(c) => c.is_object(),
			_ => false,
		}
	}
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

#[derive(Clone)]
pub struct ManyContexts<'a, M>(std::slice::Iter<'a, Meta<Context<M>, M>>);

impl<'a, M: Clone> Iterator for ManyContexts<'a, M> {
	type Item = Meta<ContextRef<'a, ContextDefinition<M>>, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|c| c.borrow_value().cast())
	}
}

impl<'a, M: Clone> ExactSizeIterator for ManyContexts<'a, M> {}

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
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum ContextRef<'a, D> {
	Null,
	IriRef(IriRef<'a>),
	Definition(&'a D),
}

impl<'a, D> ContextRef<'a, D> {
	pub fn is_object(&self) -> bool {
		matches!(self, Self::Definition(_))
	}
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

pub trait AnyContextDefinition: Sized {
	type ContextEntry: AnyContextEntry;

	type Bindings<'a>: Iterator<Item = (KeyRef<'a>, TermBindingRef<'a, Self::ContextEntry>)>
		+ Send
		+ Sync
	where
		Self: 'a,
		Self::ContextEntry: 'a,
		<Self::ContextEntry as AnyContextEntry>::Metadata: 'a;

	fn base(
		&self,
	) -> Option<Meta<Nullable<IriRef>, <Self::ContextEntry as AnyContextEntry>::Metadata>>;
	fn import(&self) -> Option<Meta<IriRef, <Self::ContextEntry as AnyContextEntry>::Metadata>>;
	fn language(
		&self,
	) -> Option<Meta<Nullable<LenientLanguageTag>, <Self::ContextEntry as AnyContextEntry>::Metadata>>;
	fn direction(
		&self,
	) -> Option<Meta<Nullable<Direction>, <Self::ContextEntry as AnyContextEntry>::Metadata>>;
	fn propagate(&self) -> Option<Meta<bool, <Self::ContextEntry as AnyContextEntry>::Metadata>>;
	fn protected(&self) -> Option<Meta<bool, <Self::ContextEntry as AnyContextEntry>::Metadata>>;
	fn type_(
		&self,
	) -> Option<
		Meta<
			ContextType<<Self::ContextEntry as AnyContextEntry>::Metadata>,
			<Self::ContextEntry as AnyContextEntry>::Metadata,
		>,
	>;
	fn version(&self) -> Option<Meta<Version, <Self::ContextEntry as AnyContextEntry>::Metadata>>;
	fn vocab(
		&self,
	) -> Option<Meta<Nullable<VocabRef>, <Self::ContextEntry as AnyContextEntry>::Metadata>>;
	fn bindings(&self) -> Self::Bindings<'_>;
	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<Self::ContextEntry>>;

	fn get(&self, key: &KeyOrKeyword) -> Option<ValueRef<Self::ContextEntry>> {
		match key {
			KeyOrKeyword::Keyword(k) => match k {
				Keyword::Base => self.base().map(ValueRef::Base),
				Keyword::Import => self.import().map(ValueRef::Import),
				Keyword::Language => self.language().map(ValueRef::Language),
				Keyword::Direction => self.direction().map(ValueRef::Direction),
				Keyword::Propagate => self.propagate().map(ValueRef::Propagate),
				Keyword::Protected => self.protected().map(ValueRef::Protected),
				Keyword::Type => self.type_().map(ValueRef::Type),
				Keyword::Version => self.version().map(ValueRef::Version),
				Keyword::Vocab => self.vocab().map(ValueRef::Vocab),
				_ => None,
			},
			KeyOrKeyword::Key(k) => self
				.get_binding(k)
				.map(|b| ValueRef::Definition(b.definition)),
		}
	}

	fn entries(&self) -> ContextEntries<Self::ContextEntry, Self::Bindings<'_>> {
		ContextEntries {
			base: self.base(),
			import: self.import(),
			language: self.language(),
			direction: self.direction(),
			propagate: self.propagate(),
			protected: self.protected(),
			type_: self.type_(),
			version: self.version(),
			vocab: self.vocab(),
			bindings: self.bindings(),
		}
	}
}

pub struct ContextEntries<'a, C: AnyContextEntry, B> {
	base: Option<Meta<Nullable<IriRef<'a>>, C::Metadata>>,
	import: Option<Meta<IriRef<'a>, C::Metadata>>,
	language: Option<Meta<Nullable<LenientLanguageTag<'a>>, C::Metadata>>,
	direction: Option<Meta<Nullable<Direction>, C::Metadata>>,
	propagate: Option<Meta<bool, C::Metadata>>,
	protected: Option<Meta<bool, C::Metadata>>,
	type_: Option<Meta<ContextType<C::Metadata>, C::Metadata>>,
	version: Option<Meta<Version, C::Metadata>>,
	vocab: Option<Meta<Nullable<VocabRef<'a>>, C::Metadata>>,
	bindings: B,
}

impl<'a, C: 'a + AnyContextEntry, B> Iterator for ContextEntries<'a, C, B>
where
	B: Iterator<Item = (KeyRef<'a>, TermBindingRef<'a, C>)>,
{
	type Item = EntryRef<'a, C>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = 0;

		if self.base.is_some() {
			len += 1
		}

		if self.import.is_some() {
			len += 1
		}

		if self.language.is_some() {
			len += 1
		}

		if self.direction.is_some() {
			len += 1
		}

		if self.propagate.is_some() {
			len += 1
		}

		if self.protected.is_some() {
			len += 1
		}

		if self.type_.is_some() {
			len += 1
		}

		if self.version.is_some() {
			len += 1
		}

		if self.vocab.is_some() {
			len += 1
		}

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self.base.take() {
			Some(value) => Some(EntryRef::Base(value)),
			None => match self.import.take() {
				Some(value) => Some(EntryRef::Import(value)),
				None => match self.language.take() {
					Some(value) => Some(EntryRef::Language(value)),
					None => match self.direction.take() {
						Some(value) => Some(EntryRef::Direction(value)),
						None => match self.propagate.take() {
							Some(value) => Some(EntryRef::Propagate(value)),
							None => match self.protected.take() {
								Some(value) => Some(EntryRef::Protected(value)),
								None => match self.type_.take() {
									Some(value) => Some(EntryRef::Type(value)),
									None => match self.version.take() {
										Some(value) => Some(EntryRef::Version(value)),
										None => match self.vocab.take() {
											Some(value) => Some(EntryRef::Vocab(value)),
											None => self
												.bindings
												.next()
												.map(|(k, v)| EntryRef::Definition(k, v)),
										},
									},
								},
							},
						},
					},
				},
			},
		}
	}
}

impl<'a, C: 'a + AnyContextEntry, B> ExactSizeIterator for ContextEntries<'a, C, B> where
	B: Iterator<Item = (KeyRef<'a>, TermBindingRef<'a, C>)>
{
}
pub enum ValueRef<'a, C: AnyContextEntry> {
	Base(Meta<Nullable<IriRef<'a>>, C::Metadata>),
	Import(Meta<IriRef<'a>, C::Metadata>),
	Language(Meta<Nullable<LenientLanguageTag<'a>>, C::Metadata>),
	Direction(Meta<Nullable<Direction>, C::Metadata>),
	Propagate(Meta<bool, C::Metadata>),
	Protected(Meta<bool, C::Metadata>),
	Type(Meta<ContextType<C::Metadata>, C::Metadata>),
	Version(Meta<Version, C::Metadata>),
	Vocab(Meta<Nullable<VocabRef<'a>>, C::Metadata>),
	Definition(Meta<Nullable<TermDefinitionRef<'a, C>>, C::Metadata>),
}

impl<'a, C: AnyContextEntry> ValueRef<'a, C> {
	pub fn is_object(&self) -> bool {
		match self {
			Self::Type(_) => true,
			Self::Definition(Meta(Nullable::Some(d), _)) => d.is_object(),
			_ => false,
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
	Definition(KeyRef<'a>, TermBindingRef<'a, C>),
}

pub enum ContextDefinitionKeyRef<'a> {
	Base,
	Import,
	Language,
	Direction,
	Propagate,
	Protected,
	Type,
	Version,
	Vocab,
	Definition(KeyRef<'a>),
}

impl<'a> ContextDefinitionKeyRef<'a> {
	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Base => "@base",
			Self::Import => "@import",
			Self::Language => "@language",
			Self::Direction => "@direction",
			Self::Propagate => "@propagate",
			Self::Protected => "@protected",
			Self::Type => "@type",
			Self::Version => "@version",
			Self::Vocab => "@vocab",
			Self::Definition(d) => d.as_str(),
		}
	}
}

impl<'a, C: AnyContextEntry> EntryRef<'a, C> {
	pub fn key(&self) -> ContextDefinitionKeyRef<'a> {
		match self {
			Self::Base(_) => ContextDefinitionKeyRef::Base,
			Self::Import(_) => ContextDefinitionKeyRef::Import,
			Self::Language(_) => ContextDefinitionKeyRef::Language,
			Self::Direction(_) => ContextDefinitionKeyRef::Direction,
			Self::Propagate(_) => ContextDefinitionKeyRef::Propagate,
			Self::Protected(_) => ContextDefinitionKeyRef::Protected,
			Self::Type(_) => ContextDefinitionKeyRef::Type,
			Self::Version(_) => ContextDefinitionKeyRef::Version,
			Self::Vocab(_) => ContextDefinitionKeyRef::Vocab,
			Self::Definition(key, _) => ContextDefinitionKeyRef::Definition(*key),
		}
	}

	pub fn into_value(self) -> ValueRef<'a, C> {
		match self {
			Self::Base(v) => ValueRef::Base(v),
			Self::Import(v) => ValueRef::Import(v),
			Self::Language(v) => ValueRef::Language(v),
			Self::Direction(v) => ValueRef::Direction(v),
			Self::Propagate(v) => ValueRef::Propagate(v),
			Self::Protected(v) => ValueRef::Protected(v),
			Self::Type(v) => ValueRef::Type(v),
			Self::Version(v) => ValueRef::Version(v),
			Self::Vocab(v) => ValueRef::Vocab(v),
			Self::Definition(_, b) => ValueRef::Definition(b.definition),
		}
	}

	pub fn into_pair(self) -> (ContextDefinitionKeyRef<'a>, ValueRef<'a, C>) {
		match self {
			Self::Base(v) => (ContextDefinitionKeyRef::Base, ValueRef::Base(v)),
			Self::Import(v) => (ContextDefinitionKeyRef::Import, ValueRef::Import(v)),
			Self::Language(v) => (ContextDefinitionKeyRef::Language, ValueRef::Language(v)),
			Self::Direction(v) => (ContextDefinitionKeyRef::Direction, ValueRef::Direction(v)),
			Self::Propagate(v) => (ContextDefinitionKeyRef::Propagate, ValueRef::Propagate(v)),
			Self::Protected(v) => (ContextDefinitionKeyRef::Protected, ValueRef::Protected(v)),
			Self::Type(v) => (ContextDefinitionKeyRef::Type, ValueRef::Type(v)),
			Self::Version(v) => (ContextDefinitionKeyRef::Version, ValueRef::Version(v)),
			Self::Vocab(v) => (ContextDefinitionKeyRef::Vocab, ValueRef::Vocab(v)),
			Self::Definition(key, b) => (
				ContextDefinitionKeyRef::Definition(key),
				ValueRef::Definition(b.definition),
			),
		}
	}
}

impl<M: Clone + Send + Sync> AnyContextDefinition for ContextDefinition<M> {
	type ContextEntry = ContextEntry<M>;
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

	pub fn is_object(&self) -> bool {
		self.is_expanded()
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
	pub container: Option<Meta<Nullable<ContainerRef<'a, C::Metadata>>, C::Metadata>>,
	pub nest: Option<Meta<NestRef<'a>, C::Metadata>>,
	pub prefix: Option<Meta<bool, C::Metadata>>,
	pub propagate: Option<Meta<bool, C::Metadata>>,
	pub protected: Option<Meta<bool, C::Metadata>>,
}

impl<'a, C: AnyContextEntry> ExpandedTermDefinitionRef<'a, C> {
	pub fn iter(&self) -> TermDefinitionEntries<'a, C> {
		TermDefinitionEntries {
			id: self.id.clone(),
			type_: self.type_.clone(),
			context: self.context.clone(),
			reverse: self.reverse.clone(),
			index: self.index.clone(),
			language: self.language.clone(),
			direction: self.direction.clone(),
			container: self.container.clone(),
			nest: self.nest.clone(),
			prefix: self.prefix.clone(),
			propagate: self.propagate.clone(),
			protected: self.protected.clone(),
		}
	}
}

impl<'a, C: AnyContextEntry> IntoIterator for ExpandedTermDefinitionRef<'a, C> {
	type Item = TermDefinitionEntryRef<'a, C>;
	type IntoIter = TermDefinitionEntries<'a, C>;

	fn into_iter(self) -> TermDefinitionEntries<'a, C> {
		TermDefinitionEntries {
			id: self.id,
			type_: self.type_,
			context: self.context,
			reverse: self.reverse,
			index: self.index,
			language: self.language,
			direction: self.direction,
			container: self.container,
			nest: self.nest,
			prefix: self.prefix,
			propagate: self.propagate,
			protected: self.protected,
		}
	}
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
			container: d
				.container
				.as_ref()
				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			nest: d.nest.as_ref().map(|v| v.borrow_value().cast()),
			prefix: d.prefix.clone(),
			propagate: d.propagate.clone(),
			protected: d.protected.clone(),
		}
	}
}

pub struct TermDefinitionEntries<'a, C: AnyContextEntry> {
	id: Option<Meta<Nullable<IdRef<'a>>, C::Metadata>>,
	type_: Option<Meta<Nullable<TermDefinitionTypeRef<'a>>, C::Metadata>>,
	context: Option<Meta<&'a C, C::Metadata>>,
	reverse: Option<Meta<KeyRef<'a>, C::Metadata>>,
	index: Option<Meta<IndexRef<'a>, C::Metadata>>,
	language: Option<Meta<Nullable<LenientLanguageTag<'a>>, C::Metadata>>,
	direction: Option<Meta<Nullable<Direction>, C::Metadata>>,
	container: Option<Meta<Nullable<ContainerRef<'a, C::Metadata>>, C::Metadata>>,
	nest: Option<Meta<NestRef<'a>, C::Metadata>>,
	prefix: Option<Meta<bool, C::Metadata>>,
	propagate: Option<Meta<bool, C::Metadata>>,
	protected: Option<Meta<bool, C::Metadata>>,
}

pub enum TermDefinitionEntryRef<'a, C: AnyContextEntry> {
	Id(Meta<Nullable<IdRef<'a>>, C::Metadata>),
	Type(Meta<Nullable<TermDefinitionTypeRef<'a>>, C::Metadata>),
	Context(Meta<&'a C, C::Metadata>),
	Reverse(Meta<KeyRef<'a>, C::Metadata>),
	Index(Meta<IndexRef<'a>, C::Metadata>),
	Language(Meta<Nullable<LenientLanguageTag<'a>>, C::Metadata>),
	Direction(Meta<Nullable<Direction>, C::Metadata>),
	Container(Meta<Nullable<ContainerRef<'a, C::Metadata>>, C::Metadata>),
	Nest(Meta<NestRef<'a>, C::Metadata>),
	Prefix(Meta<bool, C::Metadata>),
	Propagate(Meta<bool, C::Metadata>),
	Protected(Meta<bool, C::Metadata>),
}

impl<'a, C: AnyContextEntry> TermDefinitionEntryRef<'a, C> {
	pub fn key(&self) -> TermDefinitionKey {
		match self {
			Self::Id(_) => TermDefinitionKey::Id,
			Self::Type(_) => TermDefinitionKey::Type,
			Self::Context(_) => TermDefinitionKey::Context,
			Self::Reverse(_) => TermDefinitionKey::Reverse,
			Self::Index(_) => TermDefinitionKey::Index,
			Self::Language(_) => TermDefinitionKey::Language,
			Self::Direction(_) => TermDefinitionKey::Direction,
			Self::Container(_) => TermDefinitionKey::Container,
			Self::Nest(_) => TermDefinitionKey::Nest,
			Self::Prefix(_) => TermDefinitionKey::Prefix,
			Self::Propagate(_) => TermDefinitionKey::Propagate,
			Self::Protected(_) => TermDefinitionKey::Protected,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Context(c) => c.as_entry_ref().is_object(),
			_ => false,
		}
	}

	pub fn is_array(&self) -> bool {
		match self {
			Self::Container(Meta(Nullable::Some(c), _)) => c.is_array(),
			_ => false,
		}
	}
}

pub enum TermDefinitionKey {
	Id,
	Type,
	Context,
	Reverse,
	Index,
	Language,
	Direction,
	Container,
	Nest,
	Prefix,
	Propagate,
	Protected,
}

impl TermDefinitionKey {
	pub fn keyword(&self) -> Keyword {
		match self {
			Self::Id => Keyword::Id,
			Self::Type => Keyword::Type,
			Self::Context => Keyword::Context,
			Self::Reverse => Keyword::Reverse,
			Self::Index => Keyword::Index,
			Self::Language => Keyword::Language,
			Self::Direction => Keyword::Direction,
			Self::Container => Keyword::Container,
			Self::Nest => Keyword::Nest,
			Self::Prefix => Keyword::Prefix,
			Self::Propagate => Keyword::Propagate,
			Self::Protected => Keyword::Protected,
		}
	}

	pub fn as_str(&self) -> &'static str {
		self.keyword().into_str()
	}
}

impl<'a, C: 'a + AnyContextEntry> Iterator for TermDefinitionEntries<'a, C> {
	type Item = TermDefinitionEntryRef<'a, C>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = 0;

		if self.id.is_some() {
			len += 1
		}

		if self.type_.is_some() {
			len += 1
		}

		if self.context.is_some() {
			len += 1
		}

		if self.reverse.is_some() {
			len += 1
		}

		if self.index.is_some() {
			len += 1
		}

		if self.language.is_some() {
			len += 1
		}

		if self.direction.is_some() {
			len += 1
		}

		if self.container.is_some() {
			len += 1
		}

		if self.nest.is_some() {
			len += 1
		}

		if self.prefix.is_some() {
			len += 1
		}

		if self.propagate.is_some() {
			len += 1
		}

		if self.protected.is_some() {
			len += 1
		}

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self.id.take() {
			Some(value) => Some(TermDefinitionEntryRef::Id(value)),
			None => match self.type_.take() {
				Some(value) => Some(TermDefinitionEntryRef::Type(value)),
				None => match self.context.take() {
					Some(value) => Some(TermDefinitionEntryRef::Context(value)),
					None => match self.reverse.take() {
						Some(value) => Some(TermDefinitionEntryRef::Reverse(value)),
						None => match self.index.take() {
							Some(value) => Some(TermDefinitionEntryRef::Index(value)),
							None => match self.language.take() {
								Some(value) => Some(TermDefinitionEntryRef::Language(value)),
								None => match self.direction.take() {
									Some(value) => Some(TermDefinitionEntryRef::Direction(value)),
									None => match self.container.take() {
										Some(value) => {
											Some(TermDefinitionEntryRef::Container(value))
										}
										None => match self.nest.take() {
											Some(value) => {
												Some(TermDefinitionEntryRef::Nest(value))
											}
											None => match self.prefix.take() {
												Some(value) => {
													Some(TermDefinitionEntryRef::Prefix(value))
												}
												None => match self.propagate.take() {
													Some(value) => Some(
														TermDefinitionEntryRef::Propagate(value),
													),
													None => self
														.protected
														.take()
														.map(TermDefinitionEntryRef::Protected),
												},
											},
										},
									},
								},
							},
						},
					},
				},
			},
		}
	}
}

impl<'a, C: 'a + AnyContextEntry> ExactSizeIterator for TermDefinitionEntries<'a, C> {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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

	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Nest => "@nest",
			Self::Term(t) => t,
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

	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Iri(i) => i.into_str(),
			Self::CompactIri(i) => i.as_str(),
			Self::Term(t) => t,
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

impl<'a> TermDefinitionTypeRef<'a> {
	pub fn as_str(&self) -> &'a str {
		match self {
			Self::Iri(i) => i.into_str(),
			Self::CompactIri(i) => i.as_str(),
			Self::Term(t) => t,
			Self::Keyword(k) => k.into_str(),
		}
	}
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

impl<'a> VocabRef<'a> {
	pub fn as_str(&self) -> &'a str {
		match self {
			Self::IriRef(i) => i.into_str(),
			Self::CompactIri(c) => c.as_str(),
			Self::Blank(b) => b.as_str(),
			Self::Term(t) => t,
		}
	}
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
