use super::{term_definition, Entry, TermDefinition};
use crate::{Direction, Keyword, LenientLanguageTagBuf, Nullable};
use educe::Educe;
use indexmap::IndexMap;
use iref::IriRefBuf;
use locspan::Meta;
use locspan_derive::StrippedPartialEq;

mod import;
mod key;
mod reference;
mod type_;
mod version;
mod vocab;

pub use import::*;
pub use key::*;
pub use reference::*;
pub use type_::*;
pub use version::*;
pub use vocab::*;

/// Context definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Educe, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "M: Default")))]
#[locspan(ignore(M))]
#[educe(Default)]
pub struct Definition<M = ()> {
	#[locspan(unwrap_deref2_stripped)]
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@base", default, skip_serializing_if = "Option::is_none")
	)]
	pub base: Option<Entry<Nullable<IriRefBuf>, M>>,

	#[locspan(unwrap_deref2_stripped)]
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@import", default, skip_serializing_if = "Option::is_none")
	)]
	pub import: Option<Entry<IriRefBuf, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@language", default, skip_serializing_if = "Option::is_none")
	)]
	pub language: Option<Entry<Nullable<LenientLanguageTagBuf>, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@direction",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub direction: Option<Entry<Nullable<Direction>, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@propagate",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub propagate: Option<Entry<bool, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@protected",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub protected: Option<Entry<bool, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@type", default, skip_serializing_if = "Option::is_none")
	)]
	pub type_: Option<Entry<Type<M>, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@version", default, skip_serializing_if = "Option::is_none")
	)]
	pub version: Option<Entry<Version, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@vocab", default, skip_serializing_if = "Option::is_none")
	)]
	pub vocab: Option<Entry<Nullable<Vocab>, M>>,

	#[cfg_attr(feature = "serde", serde(flatten))]
	pub bindings: Bindings<M>,
}

impl<M> Definition<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, key: &KeyOrKeyword) -> Option<EntryValueRef<M>> {
		match key {
			KeyOrKeyword::Keyword(k) => match k {
				Keyword::Base => self
					.base
					.as_ref()
					.map(|e| EntryValueRef::Base(e.as_value())),
				Keyword::Import => self
					.import
					.as_ref()
					.map(|e| EntryValueRef::Import(e.as_value())),
				Keyword::Language => self
					.language
					.as_ref()
					.map(|e| EntryValueRef::Language(e.as_value())),
				Keyword::Direction => self
					.direction
					.as_ref()
					.map(|e| EntryValueRef::Direction(e.as_value())),
				Keyword::Propagate => self
					.propagate
					.as_ref()
					.map(|e| EntryValueRef::Propagate(e.as_value())),
				Keyword::Protected => self
					.protected
					.as_ref()
					.map(|e| EntryValueRef::Protected(e.as_value())),
				Keyword::Type => self
					.type_
					.as_ref()
					.map(|e| EntryValueRef::Type(e.as_value())),
				Keyword::Version => self
					.version
					.as_ref()
					.map(|e| EntryValueRef::Version(e.as_value())),
				Keyword::Vocab => self
					.vocab
					.as_ref()
					.map(|e| EntryValueRef::Vocab(e.as_value())),
				_ => None,
			},
			KeyOrKeyword::Key(k) => self
				.bindings
				.get(k)
				.map(|b| EntryValueRef::Definition(&b.definition)),
		}
	}

	pub fn get_binding(&self, key: &Key) -> Option<&Meta<Nullable<TermDefinition<M>>, M>> {
		self.bindings.get(key).map(|b| &b.definition)
	}
}

/// Context bindings.
#[derive(PartialEq, Eq, Clone, Educe, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "serde",
	serde(transparent, bound(deserialize = "M: Default"))
)]
#[educe(Default)]
pub struct Bindings<M = ()>(IndexMap<Key, TermBinding<M>>);

pub type BindingsIter<'a, M> = indexmap::map::Iter<'a, Key, TermBinding<M>>;

impl Bindings {
	pub fn insert(&mut self, key: Key, def: Nullable<TermDefinition>) -> Option<TermBinding> {
		self.0.insert(key, TermBinding::new((), Meta::none(def)))
	}
}

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

	pub fn iter(&self) -> BindingsIter<M> {
		self.0.iter()
	}

	pub fn insert_with(
		&mut self,
		Meta(key, key_metadata): Meta<Key, M>,
		def: Meta<Nullable<TermDefinition<M>>, M>,
	) -> Option<TermBinding<M>> {
		self.0.insert(key, TermBinding::new(key_metadata, def))
	}
}

impl<M> IntoIterator for Bindings<M> {
	type Item = (Key, TermBinding<M>);
	type IntoIter = indexmap::map::IntoIter<Key, TermBinding<M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<M> FromIterator<(Key, TermBinding<M>)> for Bindings<M> {
	fn from_iter<T: IntoIterator<Item = (Key, TermBinding<M>)>>(iter: T) -> Self {
		let mut result = Self::new();

		for (key, binding) in iter {
			result.0.insert(key, binding);
		}

		result
	}
}

impl FromIterator<(Key, Nullable<TermDefinition>)> for Bindings {
	fn from_iter<T: IntoIterator<Item = (Key, Nullable<TermDefinition>)>>(iter: T) -> Self {
		let mut result = Self::new();

		for (key, definition) in iter {
			result.insert(key, definition);
		}

		result
	}
}

impl<M> FromIterator<(Meta<Key, M>, Meta<Nullable<TermDefinition<M>>, M>)> for Bindings<M> {
	fn from_iter<T: IntoIterator<Item = (Meta<Key, M>, Meta<Nullable<TermDefinition<M>>, M>)>>(
		iter: T,
	) -> Self {
		let mut result = Self::new();

		for (key, definition) in iter {
			result.insert_with(key, definition);
		}

		result
	}
}

impl<M, N> locspan::StrippedPartialEq<Bindings<N>> for Bindings<M> {
	fn stripped_eq(&self, other: &Bindings<N>) -> bool {
		self.len() == other.len()
			&& self
				.iter()
				.all(|(key, a)| other.get(key).map(|b| a.stripped_eq(b)).unwrap_or(false))
	}
}

/// Term binding.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "M: Default")))]
#[locspan(ignore(M))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct TermBinding<M = ()> {
	#[locspan(ignore)]
	#[cfg_attr(feature = "serde", serde(skip))]
	pub key_metadata: M,

	pub definition: Meta<Nullable<TermDefinition<M>>, M>,
}

impl<M> TermBinding<M> {
	pub fn new(key_metadata: M, definition: Meta<Nullable<TermDefinition<M>>, M>) -> Self {
		Self {
			key_metadata,
			definition,
		}
	}
}

/// Context definition fragment.
pub enum FragmentRef<'a, M> {
	/// Context definition entry.
	Entry(EntryRef<'a, M>),

	/// Context definition entry key.
	Key(EntryKeyRef<'a>),

	/// Context definition entry value.
	Value(EntryValueRef<'a, M>),

	/// Term definition fragment.
	TermDefinitionFragment(term_definition::FragmentRef<'a, M>),
}

impl<'a, M> FragmentRef<'a, M> {
	pub fn is_key(&self) -> bool {
		match self {
			Self::Key(_) => true,
			Self::TermDefinitionFragment(f) => f.is_key(),
			_ => false,
		}
	}

	pub fn is_entry(&self) -> bool {
		match self {
			Self::Entry(_) => true,
			Self::TermDefinitionFragment(f) => f.is_entry(),
			_ => false,
		}
	}

	pub fn is_array(&self) -> bool {
		match self {
			Self::TermDefinitionFragment(i) => i.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Value(v) => v.is_object(),
			Self::TermDefinitionFragment(v) => v.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> SubItems<'a, M> {
		match self {
			Self::Entry(e) => SubItems::Entry(Some(e.key()), Some(Box::new(e.value()))),
			Self::Key(_) => SubItems::None,
			Self::Value(v) => SubItems::Value(v.sub_items()),
			Self::TermDefinitionFragment(f) => SubItems::TermDefinitionFragment(f.sub_fragments()),
		}
	}
}

pub enum EntryValueSubItems<'a, M> {
	None,
	TermDefinitionFragment(Box<term_definition::Entries<'a, M>>),
}

impl<'a, M> Iterator for EntryValueSubItems<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::TermDefinitionFragment(d) => d.next().map(|e| {
				FragmentRef::TermDefinitionFragment(term_definition::FragmentRef::Entry(e))
			}),
		}
	}
}

pub enum SubItems<'a, M> {
	None,
	Entry(Option<EntryKeyRef<'a>>, Option<Box<EntryValueRef<'a, M>>>),
	Value(EntryValueSubItems<'a, M>),
	TermDefinitionFragment(term_definition::SubFragments<'a, M>),
}

impl<'a, M> Iterator for SubItems<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Entry(k, v) => k
				.take()
				.map(FragmentRef::Key)
				.or_else(|| v.take().map(|v| FragmentRef::Value(*v))),
			Self::Value(d) => d.next(),
			Self::TermDefinitionFragment(d) => d.next().map(FragmentRef::TermDefinitionFragment),
		}
	}
}
