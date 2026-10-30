use super::{
	JsonLdContextBindingsIter, JsonLdContextDefinition, JsonLdContextTerm, JsonLdContextType,
	JsonLdTermDefinition, JsonLdVersion, JsonLdVocab,
};
use crate::{
	JsonLdLenient,
	syntax::{JsonLdDirection, JsonLdNullable},
};

use langtag::LangTagBuf;
use rdf_syntax::IriRef;

impl JsonLdContextDefinition {
	pub fn iter(&self) -> JsonLdContextEntries<'_> {
		JsonLdContextEntries {
			base: self.base.as_ref().map(JsonLdNullable::as_deref),
			import: self.import.as_deref(),
			language: self.language.as_ref().map(JsonLdNullable::as_ref),
			direction: self.direction,
			propagate: self.propagate,
			protected: self.protected,
			type_: self.type_,
			version: self.version,
			vocab: self.vocab.as_ref().map(JsonLdNullable::as_ref),
			bindings: self.bindings.iter(),
		}
	}
}

pub struct JsonLdContextEntries<'a> {
	base: Option<JsonLdNullable<&'a IriRef>>,
	import: Option<&'a IriRef>,
	language: Option<JsonLdNullable<&'a JsonLdLenient<LangTagBuf>>>,
	direction: Option<JsonLdNullable<JsonLdDirection>>,
	propagate: Option<bool>,
	protected: Option<bool>,
	type_: Option<JsonLdContextType>,
	version: Option<JsonLdVersion>,
	vocab: Option<JsonLdNullable<&'a JsonLdVocab>>,
	bindings: JsonLdContextBindingsIter<'a>,
}

impl<'a> Iterator for JsonLdContextEntries<'a> {
	type Item = JsonLdContextEntryRef<'a>;

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
			Some(value) => Some(JsonLdContextEntryRef::Base(value)),
			None => match self.import.take() {
				Some(value) => Some(JsonLdContextEntryRef::Import(value)),
				None => match self.language.take() {
					Some(value) => Some(JsonLdContextEntryRef::Language(value)),
					None => match self.direction.take() {
						Some(value) => Some(JsonLdContextEntryRef::Direction(value)),
						None => match self.propagate.take() {
							Some(value) => Some(JsonLdContextEntryRef::Propagate(value)),
							None => match self.protected.take() {
								Some(value) => Some(JsonLdContextEntryRef::Protected(value)),
								None => match self.type_.take() {
									Some(value) => Some(JsonLdContextEntryRef::Type(value)),
									None => match self.version.take() {
										Some(value) => Some(JsonLdContextEntryRef::Version(value)),
										None => match self.vocab.take() {
											Some(value) => {
												Some(JsonLdContextEntryRef::Vocab(value))
											}
											None => self.bindings.next().map(|(k, v)| {
												JsonLdContextEntryRef::Definition(k, v)
											}),
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

impl<'a> ExactSizeIterator for JsonLdContextEntries<'a> {}

pub enum JsonLdContextEntryValueRef<'a> {
	Base(JsonLdNullable<&'a IriRef>),
	Import(&'a IriRef),
	Language(JsonLdNullable<&'a JsonLdLenient<LangTagBuf>>),
	Direction(JsonLdNullable<JsonLdDirection>),
	Propagate(bool),
	Protected(bool),
	Type(JsonLdContextType),
	Version(JsonLdVersion),
	Vocab(JsonLdNullable<&'a JsonLdVocab>),
	Definition(JsonLdNullable<&'a JsonLdTermDefinition>),
}

impl<'a> JsonLdContextEntryValueRef<'a> {
	pub fn is_object(&self) -> bool {
		match self {
			Self::Type(_) => true,
			Self::Definition(JsonLdNullable::Some(d)) => d.is_object(),
			_ => false,
		}
	}
}

pub enum JsonLdContextEntryRef<'a> {
	Base(JsonLdNullable<&'a IriRef>),
	Import(&'a IriRef),
	Language(JsonLdNullable<&'a JsonLdLenient<LangTagBuf>>),
	Direction(JsonLdNullable<JsonLdDirection>),
	Propagate(bool),
	Protected(bool),
	Type(JsonLdContextType),
	Version(JsonLdVersion),
	Vocab(JsonLdNullable<&'a JsonLdVocab>),
	Definition(
		&'a JsonLdContextTerm,
		JsonLdNullable<&'a JsonLdTermDefinition>,
	),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JsonLdContextEntryKeyRef<'a> {
	Base,
	Import,
	Language,
	Direction,
	Propagate,
	Protected,
	Type,
	Version,
	Vocab,
	Definition(&'a JsonLdContextTerm),
}

impl<'a> JsonLdContextEntryKeyRef<'a> {
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

impl<'a> JsonLdContextEntryRef<'a> {
	pub fn into_key(self) -> JsonLdContextEntryKeyRef<'a> {
		match self {
			Self::Base(_) => JsonLdContextEntryKeyRef::Base,
			Self::Import(_) => JsonLdContextEntryKeyRef::Import,
			Self::Language(_) => JsonLdContextEntryKeyRef::Language,
			Self::Direction(_) => JsonLdContextEntryKeyRef::Direction,
			Self::Propagate(_) => JsonLdContextEntryKeyRef::Propagate,
			Self::Protected(_) => JsonLdContextEntryKeyRef::Protected,
			Self::Type(_) => JsonLdContextEntryKeyRef::Type,
			Self::Version(_) => JsonLdContextEntryKeyRef::Version,
			Self::Vocab(_) => JsonLdContextEntryKeyRef::Vocab,
			Self::Definition(key, _) => JsonLdContextEntryKeyRef::Definition(key),
		}
	}

	pub fn key(&self) -> JsonLdContextEntryKeyRef<'a> {
		match self {
			Self::Base(_) => JsonLdContextEntryKeyRef::Base,
			Self::Import(_) => JsonLdContextEntryKeyRef::Import,
			Self::Language(_) => JsonLdContextEntryKeyRef::Language,
			Self::Direction(_) => JsonLdContextEntryKeyRef::Direction,
			Self::Propagate(_) => JsonLdContextEntryKeyRef::Propagate,
			Self::Protected(_) => JsonLdContextEntryKeyRef::Protected,
			Self::Type(_) => JsonLdContextEntryKeyRef::Type,
			Self::Version(_) => JsonLdContextEntryKeyRef::Version,
			Self::Vocab(_) => JsonLdContextEntryKeyRef::Vocab,
			Self::Definition(key, _) => JsonLdContextEntryKeyRef::Definition(key),
		}
	}

	pub fn into_value(self) -> JsonLdContextEntryValueRef<'a> {
		match self {
			Self::Base(v) => JsonLdContextEntryValueRef::Base(v),
			Self::Import(v) => JsonLdContextEntryValueRef::Import(v),
			Self::Language(v) => JsonLdContextEntryValueRef::Language(v),
			Self::Direction(v) => JsonLdContextEntryValueRef::Direction(v),
			Self::Propagate(v) => JsonLdContextEntryValueRef::Propagate(v),
			Self::Protected(v) => JsonLdContextEntryValueRef::Protected(v),
			Self::Type(v) => JsonLdContextEntryValueRef::Type(v),
			Self::Version(v) => JsonLdContextEntryValueRef::Version(v),
			Self::Vocab(v) => JsonLdContextEntryValueRef::Vocab(v),
			Self::Definition(_, b) => JsonLdContextEntryValueRef::Definition(b),
		}
	}

	pub fn value(&self) -> JsonLdContextEntryValueRef<'a> {
		match self {
			Self::Base(v) => JsonLdContextEntryValueRef::Base(*v),
			Self::Import(v) => JsonLdContextEntryValueRef::Import(v),
			Self::Language(v) => JsonLdContextEntryValueRef::Language(*v),
			Self::Direction(v) => JsonLdContextEntryValueRef::Direction(*v),
			Self::Propagate(v) => JsonLdContextEntryValueRef::Propagate(*v),
			Self::Protected(v) => JsonLdContextEntryValueRef::Protected(*v),
			Self::Type(v) => JsonLdContextEntryValueRef::Type(*v),
			Self::Version(v) => JsonLdContextEntryValueRef::Version(*v),
			Self::Vocab(v) => JsonLdContextEntryValueRef::Vocab(*v),
			Self::Definition(_, b) => JsonLdContextEntryValueRef::Definition(*b),
		}
	}

	pub fn into_key_value(self) -> (JsonLdContextEntryKeyRef<'a>, JsonLdContextEntryValueRef<'a>) {
		self.key_value()
	}

	pub fn key_value(&self) -> (JsonLdContextEntryKeyRef<'a>, JsonLdContextEntryValueRef<'a>) {
		match self {
			Self::Base(v) => (
				JsonLdContextEntryKeyRef::Base,
				JsonLdContextEntryValueRef::Base(*v),
			),
			Self::Import(v) => (
				JsonLdContextEntryKeyRef::Import,
				JsonLdContextEntryValueRef::Import(v),
			),
			Self::Language(v) => (
				JsonLdContextEntryKeyRef::Language,
				JsonLdContextEntryValueRef::Language(*v),
			),
			Self::Direction(v) => (
				JsonLdContextEntryKeyRef::Direction,
				JsonLdContextEntryValueRef::Direction(*v),
			),
			Self::Propagate(v) => (
				JsonLdContextEntryKeyRef::Propagate,
				JsonLdContextEntryValueRef::Propagate(*v),
			),
			Self::Protected(v) => (
				JsonLdContextEntryKeyRef::Protected,
				JsonLdContextEntryValueRef::Protected(*v),
			),
			Self::Type(v) => (
				JsonLdContextEntryKeyRef::Type,
				JsonLdContextEntryValueRef::Type(*v),
			),
			Self::Version(v) => (
				JsonLdContextEntryKeyRef::Version,
				JsonLdContextEntryValueRef::Version(*v),
			),
			Self::Vocab(v) => (
				JsonLdContextEntryKeyRef::Vocab,
				JsonLdContextEntryValueRef::Vocab(*v),
			),
			Self::Definition(key, b) => (
				JsonLdContextEntryKeyRef::Definition(key),
				JsonLdContextEntryValueRef::Definition(*b),
			),
		}
	}
}
