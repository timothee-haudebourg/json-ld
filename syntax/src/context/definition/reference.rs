use super::{
	Definition, EntryValueSubItems, Key, KeyOrKeyword, KeyRef, TermBinding, Type, Version, VocabRef,
};
use crate::{context, Direction, Keyword, LenientLanguageTag, Nullable};
use context::{Entry, TermDefinitionRef};
use derivative::Derivative;
use iref::IriRef;
use locspan::Meta;

pub type BaseEntryRef<'a, D> = Entry<
	Nullable<IriRef<'a>>,
	<<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata,
>;

pub type ImportEntryRef<'a, D> =
	Entry<IriRef<'a>, <<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata>;

pub type LanguageEntryRef<'a, D> = Entry<
	Nullable<LenientLanguageTag<'a>>,
	<<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata,
>;

pub type DirectionEntry<D> =
	Entry<Nullable<Direction>, <<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata>;

pub type PropagateEntry<D> =
	Entry<bool, <<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata>;

pub type ProtectedEntry<D> =
	Entry<bool, <<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata>;

pub type TypeEntry<D> = Entry<
	Type<<<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata>,
	<<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata,
>;

pub type VersionEntry<D> =
	Entry<Version, <<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata>;

pub type VocabEntryRef<'a, D> = Entry<
	Nullable<VocabRef<'a>>,
	<<D as AnyDefinition>::ContextValue as context::AnyValue>::Metadata,
>;

pub trait AnyDefinition: Sized {
	type ContextValue: context::AnyValue;

	type Bindings<'a>: Iterator<Item = (KeyRef<'a>, TermBindingRef<'a, Self::ContextValue>)>
		+ Send
		+ Sync
	where
		Self: 'a,
		Self::ContextValue: 'a,
		<Self::ContextValue as context::AnyValue>::Metadata: 'a;

	fn base(&self) -> Option<BaseEntryRef<Self>>;
	fn import(&self) -> Option<ImportEntryRef<Self>>;
	fn language(&self) -> Option<LanguageEntryRef<Self>>;
	fn direction(&self) -> Option<DirectionEntry<Self>>;
	fn propagate(&self) -> Option<PropagateEntry<Self>>;
	fn protected(&self) -> Option<ProtectedEntry<Self>>;
	fn type_(&self) -> Option<TypeEntry<Self>>;
	fn version(&self) -> Option<VersionEntry<Self>>;
	fn vocab(&self) -> Option<VocabEntryRef<Self>>;
	fn bindings(&self) -> Self::Bindings<'_>;
	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<Self::ContextValue>>;

	fn get(&self, key: &KeyOrKeyword) -> Option<EntryValueRef<Self::ContextValue>> {
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

	fn entries(&self) -> Entries<Self::ContextValue, Self::Bindings<'_>> {
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

pub struct Entries<'a, C: context::AnyValue, B> {
	base: Option<Entry<Nullable<IriRef<'a>>, C::Metadata>>,
	import: Option<Entry<IriRef<'a>, C::Metadata>>,
	language: Option<Entry<Nullable<LenientLanguageTag<'a>>, C::Metadata>>,
	direction: Option<Entry<Nullable<Direction>, C::Metadata>>,
	propagate: Option<Entry<bool, C::Metadata>>,
	protected: Option<Entry<bool, C::Metadata>>,
	type_: Option<Entry<Type<C::Metadata>, C::Metadata>>,
	version: Option<Entry<Version, C::Metadata>>,
	vocab: Option<Entry<Nullable<VocabRef<'a>>, C::Metadata>>,
	bindings: B,
}

impl<'a, C: 'a + context::AnyValue, B> Iterator for Entries<'a, C, B>
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

impl<'a, C: 'a + context::AnyValue, B> ExactSizeIterator for Entries<'a, C, B> where
	B: Iterator<Item = (KeyRef<'a>, TermBindingRef<'a, C>)>
{
}

pub enum EntryValueRef<'a, C: context::AnyValue> {
	Base(Meta<Nullable<IriRef<'a>>, C::Metadata>),
	Import(Meta<IriRef<'a>, C::Metadata>),
	Language(Meta<Nullable<LenientLanguageTag<'a>>, C::Metadata>),
	Direction(Meta<Nullable<Direction>, C::Metadata>),
	Propagate(Meta<bool, C::Metadata>),
	Protected(Meta<bool, C::Metadata>),
	Type(Meta<Type<C::Metadata>, C::Metadata>),
	Version(Meta<Version, C::Metadata>),
	Vocab(Meta<Nullable<VocabRef<'a>>, C::Metadata>),
	Definition(Meta<Nullable<TermDefinitionRef<'a, C>>, C::Metadata>),
}

impl<'a, C: context::AnyValue> EntryValueRef<'a, C> {
	pub fn is_object(&self) -> bool {
		match self {
			Self::Type(_) => true,
			Self::Definition(Meta(Nullable::Some(d), _)) => d.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> EntryValueSubItems<'a, C> {
		match self {
			Self::Definition(Meta(Nullable::Some(TermDefinitionRef::Expanded(e)), _)) => {
				EntryValueSubItems::TermDefinitionFragment(e.iter())
			}
			_ => EntryValueSubItems::None,
		}
	}
}

pub enum EntryRef<'a, C: context::AnyValue> {
	Base(Entry<Nullable<IriRef<'a>>, C::Metadata>),
	Import(Entry<IriRef<'a>, C::Metadata>),
	Language(Entry<Nullable<LenientLanguageTag<'a>>, C::Metadata>),
	Direction(Entry<Nullable<Direction>, C::Metadata>),
	Propagate(Entry<bool, C::Metadata>),
	Protected(Entry<bool, C::Metadata>),
	Type(Entry<Type<C::Metadata>, C::Metadata>),
	Version(Entry<Version, C::Metadata>),
	Vocab(Entry<Nullable<VocabRef<'a>>, C::Metadata>),
	Definition(KeyRef<'a>, TermBindingRef<'a, C>),
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

impl<'a, C: context::AnyValue> EntryRef<'a, C> {
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

	pub fn into_value(self) -> EntryValueRef<'a, C> {
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

	pub fn value(&self) -> EntryValueRef<'a, C> {
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

	pub fn key_value(&self) -> (Meta<EntryKeyRef<'a>, C::Metadata>, EntryValueRef<'a, C>) {
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

	pub fn into_key_value(self) -> (Meta<EntryKeyRef<'a>, C::Metadata>, EntryValueRef<'a, C>) {
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

impl<M: Clone + Send + Sync> AnyDefinition for Definition<M> {
	type ContextValue = context::Value<M>;
	type Bindings<'a> = Bindings<'a, M> where M: 'a;

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

	fn bindings(&self) -> Self::Bindings<'_> {
		Bindings(self.bindings.iter())
	}

	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<context::Value<M>>> {
		self.bindings.get(key).map(Into::into)
	}
}

pub struct Bindings<'a, M>(indexmap::map::Iter<'a, Key, TermBinding<M>>);

impl<'a, M: Clone + Send + Sync> Iterator for Bindings<'a, M> {
	type Item = (KeyRef<'a>, TermBindingRef<'a, context::Value<M>>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(key, def)| (key.into(), def.into()))
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct TermBindingRef<'a, C: context::AnyValue> {
	pub key_metadata: C::Metadata,
	pub definition: Meta<Nullable<TermDefinitionRef<'a, C>>, C::Metadata>,
}

impl<'a, C: context::AnyValue> TermBindingRef<'a, C> {
	pub fn key_metadata(&self) -> &C::Metadata {
		&self.key_metadata
	}
}

impl<'a, M: Clone + Send + Sync> From<&'a TermBinding<M>>
	for TermBindingRef<'a, context::Value<M>>
{
	fn from(b: &'a TermBinding<M>) -> Self {
		Self {
			key_metadata: b.key_metadata.clone(),
			definition: b.definition.borrow_value().map(|b| b.as_ref().cast()),
		}
	}
}
