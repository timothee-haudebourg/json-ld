use crate::syntax::{Direction, Keyword, LenientLangTagBuf, Nullable};
use indexmap::IndexMap;
use iref::IriRefBuf;

mod import;
mod reference;
mod term;
mod type_;
mod version;
mod vocab;

pub use import::*;
pub use reference::*;
pub use term::*;
pub use type_::*;
pub use version::*;
pub use vocab::*;

/// Context definition.
#[derive(Default, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContextDefinition {
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@base", default, skip_serializing_if = "Option::is_none")
	)]
	pub base: Option<Nullable<IriRefBuf>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@import", default, skip_serializing_if = "Option::is_none")
	)]
	pub import: Option<IriRefBuf>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@language", default, skip_serializing_if = "Option::is_none")
	)]
	pub language: Option<Nullable<LenientLangTagBuf>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@direction",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub direction: Option<Nullable<Direction>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@propagate",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub propagate: Option<bool>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@protected",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub protected: Option<bool>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@type", default, skip_serializing_if = "Option::is_none")
	)]
	pub type_: Option<ContextType>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@version", default, skip_serializing_if = "Option::is_none")
	)]
	pub version: Option<Version>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@vocab", default, skip_serializing_if = "Option::is_none")
	)]
	pub vocab: Option<Nullable<Vocab>>,

	#[cfg_attr(feature = "serde", serde(flatten))]
	pub bindings: Bindings,
}

impl ContextDefinition {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, key: &KeyOrKeyword) -> Option<EntryValueRef> {
		match key {
			KeyOrKeyword::Keyword(k) => match k {
				Keyword::Base => self
					.base
					.as_ref()
					.map(Nullable::as_deref)
					.map(EntryValueRef::Base),
				Keyword::Import => self.import.as_deref().map(EntryValueRef::Import),
				Keyword::Language => self
					.language
					.as_ref()
					.map(Nullable::as_ref)
					.map(EntryValueRef::Language),
				Keyword::Direction => self.direction.map(EntryValueRef::Direction),
				Keyword::Propagate => self.propagate.map(EntryValueRef::Propagate),
				Keyword::Protected => self.protected.map(EntryValueRef::Protected),
				Keyword::Type => self.type_.map(EntryValueRef::Type),
				Keyword::Version => self.version.map(EntryValueRef::Version),
				Keyword::Vocab => self
					.vocab
					.as_ref()
					.map(Nullable::as_ref)
					.map(EntryValueRef::Vocab),
				_ => None,
			},
			KeyOrKeyword::Key(k) => self.bindings.get(k).map(EntryValueRef::Definition),
		}
	}

	pub fn get_binding(&self, key: &ContextTerm) -> Option<Nullable<&TermDefinition>> {
		self.bindings.get(key)
	}
}

/// Context bindings.
#[derive(Default, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Bindings(IndexMap<ContextTerm, Nullable<TermDefinition>>);

pub struct BindingsIter<'a>(indexmap::map::Iter<'a, ContextTerm, Nullable<TermDefinition>>);

impl<'a> Iterator for BindingsIter<'a> {
	type Item = (&'a ContextTerm, Nullable<&'a TermDefinition>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(k, d)| (k, d.as_ref()))
	}
}

impl<'a> DoubleEndedIterator for BindingsIter<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back().map(|(k, d)| (k, d.as_ref()))
	}
}

impl<'a> ExactSizeIterator for BindingsIter<'a> {}

impl Bindings {
	pub fn insert(
		&mut self,
		key: ContextTerm,
		def: Nullable<TermDefinition>,
	) -> Option<Nullable<TermDefinition>> {
		self.0.insert(key, def)
	}
}

impl Bindings {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn get(&self, key: &ContextTerm) -> Option<Nullable<&TermDefinition>> {
		self.0.get(key).map(Nullable::as_ref)
	}

	pub fn get_entry(&self, i: usize) -> Option<(&ContextTerm, Nullable<&TermDefinition>)> {
		self.0
			.get_index(i)
			.map(|(key, value)| (key, value.as_ref()))
	}

	pub fn iter(&self) -> BindingsIter {
		BindingsIter(self.0.iter())
	}

	pub fn insert_with(
		&mut self,
		key: ContextTerm,
		def: Nullable<TermDefinition>,
	) -> Option<Nullable<TermDefinition>> {
		self.0.insert(key, def)
	}
}

impl IntoIterator for Bindings {
	type Item = (ContextTerm, Nullable<TermDefinition>);
	type IntoIter = indexmap::map::IntoIter<ContextTerm, Nullable<TermDefinition>>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl FromIterator<(ContextTerm, Nullable<TermDefinition>)> for Bindings {
	fn from_iter<T: IntoIterator<Item = (ContextTerm, Nullable<TermDefinition>)>>(iter: T) -> Self {
		let mut result = Self::new();

		for (key, binding) in iter {
			result.0.insert(key, binding);
		}

		result
	}
}
