use super::{
	Definition, EntryValueSubItems, Key, KeyOrKeyword, KeyRef, TermBinding, Type, Version, VocabRef,
};
use crate::{context, Direction, Keyword, LenientLanguageTag, Nullable};
use context::{Entry, TermDefinitionRef};
use derivative::Derivative;
use iref::IriRef;
use locspan::Meta;

pub type BaseEntryRef<'a, M> = Entry<Nullable<IriRef<'a>>, M>;

pub type ImportEntryRef<'a, M> = Entry<IriRef<'a>, M>;

pub type LanguageEntryRef<'a, M> = Entry<Nullable<LenientLanguageTag<'a>>, M>;

pub type DirectionEntry<M> = Entry<Nullable<Direction>, M>;

pub type PropagateEntry<M> = Entry<bool, M>;

pub type ProtectedEntry<M> = Entry<bool, M>;

pub type TypeEntry<M> = Entry<Type<M>, M>;

pub type VersionEntry<M> = Entry<Version, M>;

pub type VocabEntryRef<'a, M> = Entry<Nullable<VocabRef<'a>>, M>;

pub enum BindingsIter<'a, M, C = context::Value<M>> {
	Map(indexmap::map::Iter<'a, Key, TermBinding<M, C>>),
	Slice(core::slice::Iter<'a, (KeyRef<'a>, TermBindingRef<'a, M, C>)>),
}

impl<'a, M: Clone, C> Iterator for BindingsIter<'a, M, C> {
	type Item = (KeyRef<'a>, TermBindingRef<'a, M, C>);

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Self::Map(m) => m.size_hint(),
			Self::Slice(s) => s.size_hint(),
		}
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Map(m) => m.next().map(|(k, v)| (k.into(), v.into())),
			Self::Slice(s) => s.next().cloned(),
		}
	}
}

impl<'a, M: Clone, C> ExactSizeIterator for BindingsIter<'a, M, C> {}

// /// Context definition reference.
// pub struct DefinitionRef<'a, M, C> {
// 	base: Option<Entry<Nullable<IriRef<'a>>, M>>,
// 	import: Option<Entry<IriRef<'a>, M>>,
// 	language: Option<Entry<Nullable<LenientLanguageTag<'a>>, M>>,
// 	direction: Option<Entry<Nullable<Direction>, M>>,
// 	propagate: Option<Entry<bool, M>>,
// 	protected: Option<Entry<bool, M>>,
// 	type_: Option<Entry<Type<M>, M>>,
// 	version: Option<Entry<Version, M>>,
// 	vocab: Option<Entry<Nullable<VocabRef<'a>>, M>>,
// 	bindings: BindingsRef<'a, M, C>
// }

pub trait AnyDefinition<M>: Sized {
	type ContextValue: context::AnyValue<M>;

	fn base(&self) -> Option<BaseEntryRef<M>>;
	fn import(&self) -> Option<ImportEntryRef<M>>;
	fn language(&self) -> Option<LanguageEntryRef<M>>;
	fn direction(&self) -> Option<DirectionEntry<M>>;
	fn propagate(&self) -> Option<PropagateEntry<M>>;
	fn protected(&self) -> Option<ProtectedEntry<M>>;
	fn type_(&self) -> Option<TypeEntry<M>>;
	fn version(&self) -> Option<VersionEntry<M>>;
	fn vocab(&self) -> Option<VocabEntryRef<M>>;
	fn bindings(&self) -> BindingsIter<M, Self::ContextValue>;
	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<M, Self::ContextValue>>;

	fn get(&self, key: &KeyOrKeyword) -> Option<EntryValueRef<M, Self::ContextValue>> {
		match key {
			KeyOrKeyword::Keyword(k) => match k {
				Keyword::Base => self.base().map(|e| EntryValueRef::Base(e.value)),
				Keyword::Import => self.import().map(|e| EntryValueRef::Import(e.value)),
				Keyword::Language => self.language().map(|e| EntryValueRef::Language(e.value)),
				Keyword::Direction => self.direction().map(|e| EntryValueRef::Direction(e.value)),
				Keyword::Propagate => self.propagate().map(|e| EntryValueRef::Propagate(e.value)),
				Keyword::Protected => self.protected().map(|e| EntryValueRef::Protected(e.value)),
				Keyword::Type => self.type_().map(|e| EntryValueRef::Type(e.value)),
				Keyword::Version => self.version().map(|e| EntryValueRef::Version(e.value)),
				Keyword::Vocab => self.vocab().map(|e| EntryValueRef::Vocab(e.value)),
				_ => None,
			},
			KeyOrKeyword::Key(k) => self
				.get_binding(k)
				.map(|b| EntryValueRef::Definition(b.definition)),
		}
	}

	fn entries(&self) -> Entries<M, Self::ContextValue> {
		Entries {
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

pub struct Entries<'a, M, C> {
	base: Option<Entry<Nullable<IriRef<'a>>, M>>,
	import: Option<Entry<IriRef<'a>, M>>,
	language: Option<Entry<Nullable<LenientLanguageTag<'a>>, M>>,
	direction: Option<Entry<Nullable<Direction>, M>>,
	propagate: Option<Entry<bool, M>>,
	protected: Option<Entry<bool, M>>,
	type_: Option<Entry<Type<M>, M>>,
	version: Option<Entry<Version, M>>,
	vocab: Option<Entry<Nullable<VocabRef<'a>>, M>>,
	bindings: BindingsIter<'a, M, C>,
}

impl<'a, M: Clone, C> Iterator for Entries<'a, M, C> {
	type Item = EntryRef<'a, M, C>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = self.bindings.len();

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

impl<'a, M: Clone, C> ExactSizeIterator for Entries<'a, M, C> {}

pub enum EntryValueRef<'a, M, C> {
	Base(Meta<Nullable<IriRef<'a>>, M>),
	Import(Meta<IriRef<'a>, M>),
	Language(Meta<Nullable<LenientLanguageTag<'a>>, M>),
	Direction(Meta<Nullable<Direction>, M>),
	Propagate(Meta<bool, M>),
	Protected(Meta<bool, M>),
	Type(Meta<Type<M>, M>),
	Version(Meta<Version, M>),
	Vocab(Meta<Nullable<VocabRef<'a>>, M>),
	Definition(Meta<Nullable<TermDefinitionRef<'a, M, C>>, M>),
}

impl<'a, M: Clone, C> EntryValueRef<'a, M, C> {
	pub fn is_object(&self) -> bool {
		match self {
			Self::Type(_) => true,
			Self::Definition(Meta(Nullable::Some(d), _)) => d.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> EntryValueSubItems<'a, M, C> {
		match self {
			Self::Definition(Meta(Nullable::Some(TermDefinitionRef::Expanded(e)), _)) => {
				EntryValueSubItems::TermDefinitionFragment(e.iter())
			}
			_ => EntryValueSubItems::None,
		}
	}
}

pub enum EntryRef<'a, M, C> {
	Base(Entry<Nullable<IriRef<'a>>, M>),
	Import(Entry<IriRef<'a>, M>),
	Language(Entry<Nullable<LenientLanguageTag<'a>>, M>),
	Direction(Entry<Nullable<Direction>, M>),
	Propagate(Entry<bool, M>),
	Protected(Entry<bool, M>),
	Type(Entry<Type<M>, M>),
	Version(Entry<Version, M>),
	Vocab(Entry<Nullable<VocabRef<'a>>, M>),
	Definition(KeyRef<'a>, TermBindingRef<'a, M, C>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EntryKeyRef<'a> {
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

impl<'a> EntryKeyRef<'a> {
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

impl<'a, M, C> EntryRef<'a, M, C> {
	pub fn into_key(self) -> EntryKeyRef<'a> {
		match self {
			Self::Base(_) => EntryKeyRef::Base,
			Self::Import(_) => EntryKeyRef::Import,
			Self::Language(_) => EntryKeyRef::Language,
			Self::Direction(_) => EntryKeyRef::Direction,
			Self::Propagate(_) => EntryKeyRef::Propagate,
			Self::Protected(_) => EntryKeyRef::Protected,
			Self::Type(_) => EntryKeyRef::Type,
			Self::Version(_) => EntryKeyRef::Version,
			Self::Vocab(_) => EntryKeyRef::Vocab,
			Self::Definition(key, _) => EntryKeyRef::Definition(key),
		}
	}

	pub fn key(&self) -> EntryKeyRef<'a> {
		match self {
			Self::Base(_) => EntryKeyRef::Base,
			Self::Import(_) => EntryKeyRef::Import,
			Self::Language(_) => EntryKeyRef::Language,
			Self::Direction(_) => EntryKeyRef::Direction,
			Self::Propagate(_) => EntryKeyRef::Propagate,
			Self::Protected(_) => EntryKeyRef::Protected,
			Self::Type(_) => EntryKeyRef::Type,
			Self::Version(_) => EntryKeyRef::Version,
			Self::Vocab(_) => EntryKeyRef::Vocab,
			Self::Definition(key, _) => EntryKeyRef::Definition(*key),
		}
	}

	pub fn into_value(self) -> EntryValueRef<'a, M, C> {
		match self {
			Self::Base(v) => EntryValueRef::Base(v.value),
			Self::Import(v) => EntryValueRef::Import(v.value),
			Self::Language(v) => EntryValueRef::Language(v.value),
			Self::Direction(v) => EntryValueRef::Direction(v.value),
			Self::Propagate(v) => EntryValueRef::Propagate(v.value),
			Self::Protected(v) => EntryValueRef::Protected(v.value),
			Self::Type(v) => EntryValueRef::Type(v.value),
			Self::Version(v) => EntryValueRef::Version(v.value),
			Self::Vocab(v) => EntryValueRef::Vocab(v.value),
			Self::Definition(_, b) => EntryValueRef::Definition(b.definition),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a, M, C>
	where
		M: Clone,
	{
		match self {
			Self::Base(v) => EntryValueRef::Base(v.value.clone()),
			Self::Import(v) => EntryValueRef::Import(v.value.clone()),
			Self::Language(v) => EntryValueRef::Language(v.value.clone()),
			Self::Direction(v) => EntryValueRef::Direction(v.value.clone()),
			Self::Propagate(v) => EntryValueRef::Propagate(v.value.clone()),
			Self::Protected(v) => EntryValueRef::Protected(v.value.clone()),
			Self::Type(v) => EntryValueRef::Type(v.value.clone()),
			Self::Version(v) => EntryValueRef::Version(v.value.clone()),
			Self::Vocab(v) => EntryValueRef::Vocab(v.value.clone()),
			Self::Definition(_, b) => EntryValueRef::Definition(b.definition.clone()),
		}
	}

	pub fn key_value(&self) -> (Meta<EntryKeyRef<'a>, M>, EntryValueRef<'a, M, C>)
	where
		M: Clone,
	{
		match self {
			Self::Base(v) => (
				Meta(EntryKeyRef::Base, v.key_metadata.clone()),
				EntryValueRef::Base(v.value.clone()),
			),
			Self::Import(v) => (
				Meta(EntryKeyRef::Import, v.key_metadata.clone()),
				EntryValueRef::Import(v.value.clone()),
			),
			Self::Language(v) => (
				Meta(EntryKeyRef::Language, v.key_metadata.clone()),
				EntryValueRef::Language(v.value.clone()),
			),
			Self::Direction(v) => (
				Meta(EntryKeyRef::Direction, v.key_metadata.clone()),
				EntryValueRef::Direction(v.value.clone()),
			),
			Self::Propagate(v) => (
				Meta(EntryKeyRef::Propagate, v.key_metadata.clone()),
				EntryValueRef::Propagate(v.value.clone()),
			),
			Self::Protected(v) => (
				Meta(EntryKeyRef::Protected, v.key_metadata.clone()),
				EntryValueRef::Protected(v.value.clone()),
			),
			Self::Type(v) => (
				Meta(EntryKeyRef::Type, v.key_metadata.clone()),
				EntryValueRef::Type(v.value.clone()),
			),
			Self::Version(v) => (
				Meta(EntryKeyRef::Version, v.key_metadata.clone()),
				EntryValueRef::Version(v.value.clone()),
			),
			Self::Vocab(v) => (
				Meta(EntryKeyRef::Vocab, v.key_metadata.clone()),
				EntryValueRef::Vocab(v.value.clone()),
			),
			Self::Definition(key, b) => (
				Meta(EntryKeyRef::Definition(*key), b.key_metadata.clone()),
				EntryValueRef::Definition(b.definition.clone()),
			),
		}
	}

	pub fn into_key_value(self) -> (Meta<EntryKeyRef<'a>, M>, EntryValueRef<'a, M, C>) {
		match self {
			Self::Base(v) => (
				Meta(EntryKeyRef::Base, v.key_metadata),
				EntryValueRef::Base(v.value),
			),
			Self::Import(v) => (
				Meta(EntryKeyRef::Import, v.key_metadata),
				EntryValueRef::Import(v.value),
			),
			Self::Language(v) => (
				Meta(EntryKeyRef::Language, v.key_metadata),
				EntryValueRef::Language(v.value),
			),
			Self::Direction(v) => (
				Meta(EntryKeyRef::Direction, v.key_metadata),
				EntryValueRef::Direction(v.value),
			),
			Self::Propagate(v) => (
				Meta(EntryKeyRef::Propagate, v.key_metadata),
				EntryValueRef::Propagate(v.value),
			),
			Self::Protected(v) => (
				Meta(EntryKeyRef::Protected, v.key_metadata),
				EntryValueRef::Protected(v.value),
			),
			Self::Type(v) => (
				Meta(EntryKeyRef::Type, v.key_metadata),
				EntryValueRef::Type(v.value),
			),
			Self::Version(v) => (
				Meta(EntryKeyRef::Version, v.key_metadata),
				EntryValueRef::Version(v.value),
			),
			Self::Vocab(v) => (
				Meta(EntryKeyRef::Vocab, v.key_metadata),
				EntryValueRef::Vocab(v.value),
			),
			Self::Definition(key, b) => (
				Meta(EntryKeyRef::Definition(key), b.key_metadata),
				EntryValueRef::Definition(b.definition),
			),
		}
	}
}

impl<M: Clone + Send + Sync> AnyDefinition<M> for Definition<M> {
	type ContextValue = context::Value<M>;

	fn base(&self) -> Option<Entry<Nullable<IriRef>, M>> {
		self.base
			.as_ref()
			.map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_iri_ref())))
	}

	fn import(&self) -> Option<Entry<IriRef, M>> {
		self.import.as_ref().map(|v| v.borrow_value().cast())
	}

	fn language(&self) -> Option<Entry<Nullable<LenientLanguageTag>, M>> {
		self.language
			.as_ref()
			.map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref())))
	}

	fn direction(&self) -> Option<Entry<Nullable<Direction>, M>> {
		self.direction.clone()
	}

	fn propagate(&self) -> Option<Entry<bool, M>> {
		self.propagate.clone()
	}

	fn protected(&self) -> Option<Entry<bool, M>> {
		self.protected.clone()
	}

	fn type_(&self) -> Option<Entry<Type<M>, M>> {
		self.type_.clone()
	}

	fn version(&self) -> Option<Entry<Version, M>> {
		self.version.clone()
	}

	fn vocab(&self) -> Option<Entry<Nullable<VocabRef>, M>> {
		self.vocab
			.as_ref()
			.map(|v| v.borrow_value().map(|v| v.as_ref().cast()))
	}

	fn bindings(&self) -> BindingsIter<M, context::Value<M>> {
		BindingsIter::Map(self.bindings.iter())
	}

	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<M, context::Value<M>>> {
		self.bindings.get(key).map(Into::into)
	}
}

pub struct Bindings<'a, M>(indexmap::map::Iter<'a, Key, TermBinding<M>>);

impl<'a, M: Clone + Send + Sync> Iterator for Bindings<'a, M> {
	type Item = (KeyRef<'a>, TermBindingRef<'a, M, context::Value<M>>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(key, def)| (key.into(), def.into()))
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = "M: Clone"))]
pub struct TermBindingRef<'a, M, C> {
	pub key_metadata: M,
	pub definition: Meta<Nullable<TermDefinitionRef<'a, M, C>>, M>,
}

impl<'a, M, C> TermBindingRef<'a, M, C> {
	pub fn key_metadata(&self) -> &M {
		&self.key_metadata
	}
}

impl<'a, M: Clone, C> From<&'a TermBinding<M, C>> for TermBindingRef<'a, M, C> {
	fn from(b: &'a TermBinding<M, C>) -> Self {
		Self {
			key_metadata: b.key_metadata.clone(),
			definition: b.definition.borrow_value().map(|b| b.as_ref().cast()),
		}
	}
}
