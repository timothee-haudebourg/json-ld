use crate::{
	JsonLdLenient,
	syntax::{JsonLdDirection, JsonLdKeyword, JsonLdNullable},
};
use btree_indexmap::BTreeIndexMap;
use langtag::LangTagBuf;
use rdf_syntax::IriRefBuf;

mod reference;
mod term;
mod type_;
mod version;
mod vocab;

pub use reference::*;
pub use term::*;
pub use type_::*;
pub use version::*;
pub use vocab::*;

/// Context definition.
#[derive(Default, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonLdContextDefinition {
	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@base",
			default,
			deserialize_with = "JsonLdNullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub base: Option<JsonLdNullable<IriRefBuf>>,

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
			deserialize_with = "JsonLdNullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub language: Option<JsonLdNullable<JsonLdLenient<LangTagBuf>>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@direction",
			default,
			deserialize_with = "JsonLdNullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub direction: Option<JsonLdNullable<JsonLdDirection>>,

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
	pub type_: Option<JsonLdContextType>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@version", default, skip_serializing_if = "Option::is_none")
	)]
	pub version: Option<JsonLdVersion>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@vocab",
			default,
			deserialize_with = "JsonLdNullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub vocab: Option<JsonLdNullable<JsonLdVocab>>,

	#[cfg_attr(feature = "serde", serde(flatten))]
	pub bindings: Bindings,
}

impl JsonLdContextDefinition {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, key: &JsonLdKeyOrKeyword) -> Option<JsonLdContextEntryValueRef<'_>> {
		match key {
			JsonLdKeyOrKeyword::Keyword(k) => match k {
				JsonLdKeyword::Base => self
					.base
					.as_ref()
					.map(JsonLdNullable::as_deref)
					.map(JsonLdContextEntryValueRef::Base),
				JsonLdKeyword::Import => self
					.import
					.as_deref()
					.map(JsonLdContextEntryValueRef::Import),
				JsonLdKeyword::Language => self
					.language
					.as_ref()
					.map(JsonLdNullable::as_ref)
					.map(JsonLdContextEntryValueRef::Language),
				JsonLdKeyword::Direction => {
					self.direction.map(JsonLdContextEntryValueRef::Direction)
				}
				JsonLdKeyword::Propagate => {
					self.propagate.map(JsonLdContextEntryValueRef::Propagate)
				}
				JsonLdKeyword::Protected => {
					self.protected.map(JsonLdContextEntryValueRef::Protected)
				}
				JsonLdKeyword::Type => self.type_.map(JsonLdContextEntryValueRef::Type),
				JsonLdKeyword::Version => self.version.map(JsonLdContextEntryValueRef::Version),
				JsonLdKeyword::Vocab => self
					.vocab
					.as_ref()
					.map(JsonLdNullable::as_ref)
					.map(JsonLdContextEntryValueRef::Vocab),
				_ => None,
			},
			JsonLdKeyOrKeyword::Key(k) => self
				.bindings
				.get(k)
				.map(JsonLdContextEntryValueRef::Definition),
		}
	}

	pub fn get_binding(&self, key: &JsonLdContextTerm) -> Option<JsonLdNullable<&TermDefinition>> {
		self.bindings.get(key)
	}
}

/// Context bindings.
#[derive(Default, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Bindings(BTreeIndexMap<JsonLdContextTerm, JsonLdNullable<TermDefinition>>);

pub struct JsonLdContextBindingsIter<'a>(
	btree_indexmap::map::Iter<'a, JsonLdContextTerm, JsonLdNullable<TermDefinition>>,
);

impl<'a> Iterator for JsonLdContextBindingsIter<'a> {
	type Item = (&'a JsonLdContextTerm, JsonLdNullable<&'a TermDefinition>);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(k, d)| (k, d.as_ref()))
	}
}

impl<'a> DoubleEndedIterator for JsonLdContextBindingsIter<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		self.0.next_back().map(|(k, d)| (k, d.as_ref()))
	}
}

impl<'a> ExactSizeIterator for JsonLdContextBindingsIter<'a> {}

impl Bindings {
	pub fn insert(
		&mut self,
		key: JsonLdContextTerm,
		def: JsonLdNullable<TermDefinition>,
	) -> Option<JsonLdNullable<TermDefinition>> {
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

	pub fn get(&self, key: &JsonLdContextTerm) -> Option<JsonLdNullable<&TermDefinition>> {
		self.0.get(key).map(JsonLdNullable::as_ref)
	}

	pub fn get_entry(
		&self,
		i: usize,
	) -> Option<(&JsonLdContextTerm, JsonLdNullable<&TermDefinition>)> {
		self.0
			.as_entries()
			.get(i)
			.map(|(key, value)| (key, value.as_ref()))
	}

	pub fn iter(&self) -> JsonLdContextBindingsIter<'_> {
		JsonLdContextBindingsIter(self.0.iter())
	}

	pub fn insert_with(
		&mut self,
		key: JsonLdContextTerm,
		def: JsonLdNullable<TermDefinition>,
	) -> Option<JsonLdNullable<TermDefinition>> {
		self.0.insert(key, def)
	}
}

impl IntoIterator for Bindings {
	type Item = (JsonLdContextTerm, JsonLdNullable<TermDefinition>);
	type IntoIter =
		btree_indexmap::map::IntoIter<JsonLdContextTerm, JsonLdNullable<TermDefinition>>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl FromIterator<(JsonLdContextTerm, JsonLdNullable<TermDefinition>)> for Bindings {
	fn from_iter<T: IntoIterator<Item = (JsonLdContextTerm, JsonLdNullable<TermDefinition>)>>(
		iter: T,
	) -> Self {
		let mut result = Self::new();

		for (key, binding) in iter {
			result.0.insert(key, binding);
		}

		result
	}
}
