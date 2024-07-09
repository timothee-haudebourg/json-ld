use super::{term_definition, TermDefinition};
use crate::{Direction, Keyword, LenientLangTagBuf, Nullable};
use educe::Educe;
use indexmap::IndexMap;
use iref::IriRefBuf;

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
#[derive(PartialEq, Eq, Clone, Educe, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[educe(Default)]
pub struct Definition {
	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@base",
			default,
			deserialize_with = "Nullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub base: Option<Nullable<IriRefBuf>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@import", default, skip_serializing_if = "Option::is_none")
	)]
	pub import: Option<IriRefBuf>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@language",
			default,
			deserialize_with = "Nullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub language: Option<Nullable<LenientLangTagBuf>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@direction",
			default,
			deserialize_with = "Nullable::optional",
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
	pub type_: Option<Type>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@version", default, skip_serializing_if = "Option::is_none")
	)]
	pub version: Option<Version>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@vocab",
			default,
			deserialize_with = "Nullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub vocab: Option<Nullable<Vocab>>,

	#[cfg_attr(feature = "serde", serde(flatten))]
	pub bindings: Bindings,
}

impl Definition {
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

	pub fn get_binding(&self, key: &Key) -> Option<Nullable<&TermDefinition>> {
		self.bindings.get(key)
	}
}

/// Context bindings.
#[derive(PartialEq, Eq, Clone, Educe, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[educe(Default)]
pub struct Bindings(IndexMap<Key, Nullable<TermDefinition>>);

pub struct BindingsIter<'a>(indexmap::map::Iter<'a, Key, Nullable<TermDefinition>>);

impl<'a> Iterator for BindingsIter<'a> {
	type Item = (&'a Key, Nullable<&'a TermDefinition>);

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
		key: Key,
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

	pub fn get(&self, key: &Key) -> Option<Nullable<&TermDefinition>> {
		self.0.get(key).map(Nullable::as_ref)
	}

	pub fn get_entry(&self, i: usize) -> Option<(&Key, Nullable<&TermDefinition>)> {
		self.0
			.get_index(i)
			.map(|(key, value)| (key, value.as_ref()))
	}

	pub fn iter(&self) -> BindingsIter {
		BindingsIter(self.0.iter())
	}

	pub fn insert_with(
		&mut self,
		key: Key,
		def: Nullable<TermDefinition>,
	) -> Option<Nullable<TermDefinition>> {
		self.0.insert(key, def)
	}
}

impl IntoIterator for Bindings {
	type Item = (Key, Nullable<TermDefinition>);
	type IntoIter = indexmap::map::IntoIter<Key, Nullable<TermDefinition>>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl FromIterator<(Key, Nullable<TermDefinition>)> for Bindings {
	fn from_iter<T: IntoIterator<Item = (Key, Nullable<TermDefinition>)>>(iter: T) -> Self {
		let mut result = Self::new();

		for (key, binding) in iter {
			result.0.insert(key, binding);
		}

		result
	}
}

/// Context definition fragment.
pub enum FragmentRef<'a> {
	/// Context definition entry.
	Entry(EntryRef<'a>),

	/// Context definition entry key.
	Key(EntryKeyRef<'a>),

	/// Context definition entry value.
	Value(EntryValueRef<'a>),

	/// Term definition fragment.
	TermDefinitionFragment(term_definition::FragmentRef<'a>),
}

impl<'a> FragmentRef<'a> {
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

	pub fn sub_items(&self) -> SubItems<'a> {
		match self {
			Self::Entry(e) => SubItems::Entry(Some(e.key()), Some(Box::new(e.value()))),
			Self::Key(_) => SubItems::None,
			Self::Value(v) => SubItems::Value(v.sub_items()),
			Self::TermDefinitionFragment(f) => SubItems::TermDefinitionFragment(f.sub_fragments()),
		}
	}
}

pub enum EntryValueSubItems<'a> {
	None,
	TermDefinitionFragment(Box<term_definition::Entries<'a>>),
}

impl<'a> Iterator for EntryValueSubItems<'a> {
	type Item = FragmentRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::TermDefinitionFragment(d) => d.next().map(|e| {
				FragmentRef::TermDefinitionFragment(term_definition::FragmentRef::Entry(e))
			}),
		}
	}
}

pub enum SubItems<'a> {
	None,
	Entry(Option<EntryKeyRef<'a>>, Option<Box<EntryValueRef<'a>>>),
	Value(EntryValueSubItems<'a>),
	TermDefinitionFragment(term_definition::SubFragments<'a>),
}

impl<'a> Iterator for SubItems<'a> {
	type Item = FragmentRef<'a>;

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

#[cfg(test)]
mod tests {
	use super::Definition;

	#[test]
	fn deserialize_null_vocab() {
		let definition: Definition = json_syntax::from_value(json_syntax::json!({
			"@vocab": null
		}))
		.unwrap();
		assert_eq!(definition.vocab, Some(crate::Nullable::Null))
	}

	#[test]
	fn deserialize_no_vocab() {
		let definition: Definition = json_syntax::from_value(json_syntax::json!({})).unwrap();
		assert_eq!(definition.vocab, None)
	}
}
