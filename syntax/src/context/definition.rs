use super::{term_definition, AnyValue, Entry, TermDefinition};
use crate::{Direction, LenientLanguageTagBuf, Nullable};
use derivative::Derivative;
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
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Derivative, Debug)]
#[stripped_ignore(M)]
#[derivative(Default(bound = ""))]
pub struct Definition<M, C = super::Value<M>> {
	#[stripped_option_deref]
	pub base: Option<Entry<Nullable<IriRefBuf>, M>>,
	#[stripped_option_deref]
	pub import: Option<Entry<IriRefBuf, M>>,
	pub language: Option<Entry<Nullable<LenientLanguageTagBuf>, M>>,
	pub direction: Option<Entry<Nullable<Direction>, M>>,
	pub propagate: Option<Entry<bool, M>>,
	pub protected: Option<Entry<bool, M>>,
	pub type_: Option<Entry<Type<M>, M>>,
	pub version: Option<Entry<Version, M>>,
	pub vocab: Option<Entry<Nullable<Vocab>, M>>,
	pub bindings: Bindings<M, C>,
}

impl<M, C> Definition<M, C> {
	pub fn new() -> Self {
		Self::default()
	}
}

/// Context bindings.
#[derive(PartialEq, Eq, Clone, Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct Bindings<M, C = super::Value<M>>(IndexMap<Key, TermBinding<M, C>>);

impl<M, C> Bindings<M, C> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn get(&self, key: &Key) -> Option<&TermBinding<M, C>> {
		self.0.get(key)
	}

	pub fn iter(&self) -> indexmap::map::Iter<Key, TermBinding<M, C>> {
		self.0.iter()
	}

	pub fn insert(
		&mut self,
		Meta(key, key_metadata): Meta<Key, M>,
		def: Meta<Nullable<TermDefinition<M, C>>, M>,
	) -> Option<TermBinding<M, C>> {
		self.0.insert(key, TermBinding::new(key_metadata, def))
	}
}

impl<M, C> IntoIterator for Bindings<M, C> {
	type Item = (Key, TermBinding<M, C>);
	type IntoIter = indexmap::map::IntoIter<Key, TermBinding<M, C>>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<M, C> FromIterator<(Key, TermBinding<M, C>)> for Bindings<M, C> {
	fn from_iter<T: IntoIterator<Item = (Key, TermBinding<M, C>)>>(iter: T) -> Self {
		let mut result = Self::new();

		for (key, binding) in iter {
			result.0.insert(key, binding);
		}

		result
	}
}

impl<M, C> FromIterator<(Meta<Key, M>, Meta<Nullable<TermDefinition<M, C>>, M>)>
	for Bindings<M, C>
{
	fn from_iter<
		T: IntoIterator<Item = (Meta<Key, M>, Meta<Nullable<TermDefinition<M, C>>, M>)>,
	>(
		iter: T,
	) -> Self {
		let mut result = Self::new();

		for (key, definition) in iter {
			result.insert(key, definition);
		}

		result
	}
}

impl<M, C: locspan::StrippedPartialEq> locspan::StrippedPartialEq for Bindings<M, C> {
	fn stripped_eq(&self, other: &Self) -> bool {
		self.len() == other.len()
			&& self
				.iter()
				.all(|(key, a)| other.get(key).map(|b| a.stripped_eq(b)).unwrap_or(false))
	}
}

/// Term binding.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[stripped_ignore(M)]
pub struct TermBinding<M, C = super::Value<M>> {
	#[stripped_ignore]
	pub key_metadata: M,
	pub definition: Meta<Nullable<TermDefinition<M, C>>, M>,
}

impl<M, C> TermBinding<M, C> {
	pub fn new(key_metadata: M, definition: Meta<Nullable<TermDefinition<M, C>>, M>) -> Self {
		Self {
			key_metadata,
			definition,
		}
	}
}

/// Context definition fragment.
pub enum FragmentRef<'a, M, C> {
	/// Context definition entry.
	Entry(EntryRef<'a, M, C>),

	/// Context definition entry key.
	Key(EntryKeyRef<'a>),

	/// Context definition entry value.
	Value(EntryValueRef<'a, M, C>),

	/// Term definition fragment.
	TermDefinitionFragment(term_definition::FragmentRef<'a, M, C>),
}

impl<'a, M, C> FragmentRef<'a, M, C> {
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

	pub fn is_object(&self) -> bool
	where
		M: Clone,
		C: AnyValue<M>,
	{
		match self {
			Self::Value(v) => v.is_object(),
			Self::TermDefinitionFragment(v) => v.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> SubItems<'a, M, C>
	where
		M: Clone,
	{
		match self {
			Self::Entry(e) => SubItems::Entry(Some(e.key()), Some(e.value())),
			Self::Key(_) => SubItems::None,
			Self::Value(v) => SubItems::Value(v.sub_items()),
			Self::TermDefinitionFragment(f) => SubItems::TermDefinitionFragment(f.sub_fragments()),
		}
	}
}

pub enum EntryValueSubItems<'a, M, C> {
	None,
	TermDefinitionFragment(term_definition::Entries<'a, M, C>),
}

impl<'a, M, C> Iterator for EntryValueSubItems<'a, M, C> {
	type Item = FragmentRef<'a, M, C>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::TermDefinitionFragment(d) => d.next().map(|e| {
				FragmentRef::TermDefinitionFragment(term_definition::FragmentRef::Entry(e))
			}),
		}
	}
}

pub enum SubItems<'a, M, C> {
	None,
	Entry(Option<EntryKeyRef<'a>>, Option<EntryValueRef<'a, M, C>>),
	Value(EntryValueSubItems<'a, M, C>),
	TermDefinitionFragment(term_definition::SubFragments<'a, M, C>),
}

impl<'a, M, C> Iterator for SubItems<'a, M, C> {
	type Item = FragmentRef<'a, M, C>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Entry(k, v) => k
				.take()
				.map(FragmentRef::Key)
				.or_else(|| v.take().map(FragmentRef::Value)),
			Self::Value(d) => d.next(),
			Self::TermDefinitionFragment(d) => d.next().map(FragmentRef::TermDefinitionFragment),
		}
	}
}
