use super::{BindingsIter, Definition, EntryValueSubItems, Key, Type, Version, Vocab};
use crate::{context::TermDefinition, Direction, LenientLangTagBuf, Nullable};

use iref::IriRef;

impl Definition {
	pub fn iter(&self) -> Entries {
		Entries {
			base: self.base.as_ref().map(Nullable::as_deref),
			import: self.import.as_deref(),
			language: self.language.as_ref().map(Nullable::as_ref),
			direction: self.direction,
			propagate: self.propagate,
			protected: self.protected,
			type_: self.type_,
			version: self.version,
			vocab: self.vocab.as_ref().map(Nullable::as_ref),
			bindings: self.bindings.iter(),
		}
	}
}

pub struct Entries<'a> {
	base: Option<Nullable<&'a IriRef>>,
	import: Option<&'a IriRef>,
	language: Option<Nullable<&'a LenientLangTagBuf>>,
	direction: Option<Nullable<Direction>>,
	propagate: Option<bool>,
	protected: Option<bool>,
	type_: Option<Type>,
	version: Option<Version>,
	vocab: Option<Nullable<&'a Vocab>>,
	bindings: BindingsIter<'a>,
}

impl<'a> Iterator for Entries<'a> {
	type Item = EntryRef<'a>;

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

impl<'a> ExactSizeIterator for Entries<'a> {}

pub enum EntryValueRef<'a> {
	Base(Nullable<&'a IriRef>),
	Import(&'a IriRef),
	Language(Nullable<&'a LenientLangTagBuf>),
	Direction(Nullable<Direction>),
	Propagate(bool),
	Protected(bool),
	Type(Type),
	Version(Version),
	Vocab(Nullable<&'a Vocab>),
	Definition(Nullable<&'a TermDefinition>),
}

impl<'a> EntryValueRef<'a> {
	pub fn is_object(&self) -> bool {
		match self {
			Self::Type(_) => true,
			Self::Definition(Nullable::Some(d)) => d.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> EntryValueSubItems<'a> {
		match self {
			Self::Definition(Nullable::Some(TermDefinition::Expanded(e))) => {
				EntryValueSubItems::TermDefinitionFragment(Box::new(e.iter()))
			}
			_ => EntryValueSubItems::None,
		}
	}
}

pub enum EntryRef<'a> {
	Base(Nullable<&'a IriRef>),
	Import(&'a IriRef),
	Language(Nullable<&'a LenientLangTagBuf>),
	Direction(Nullable<Direction>),
	Propagate(bool),
	Protected(bool),
	Type(Type),
	Version(Version),
	Vocab(Nullable<&'a Vocab>),
	Definition(&'a Key, Nullable<&'a TermDefinition>),
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
	Definition(&'a Key),
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

impl<'a> EntryRef<'a> {
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
			Self::Definition(key, _) => EntryKeyRef::Definition(key),
		}
	}

	pub fn into_value(self) -> EntryValueRef<'a> {
		match self {
			Self::Base(v) => EntryValueRef::Base(v),
			Self::Import(v) => EntryValueRef::Import(v),
			Self::Language(v) => EntryValueRef::Language(v),
			Self::Direction(v) => EntryValueRef::Direction(v),
			Self::Propagate(v) => EntryValueRef::Propagate(v),
			Self::Protected(v) => EntryValueRef::Protected(v),
			Self::Type(v) => EntryValueRef::Type(v),
			Self::Version(v) => EntryValueRef::Version(v),
			Self::Vocab(v) => EntryValueRef::Vocab(v),
			Self::Definition(_, b) => EntryValueRef::Definition(b),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a> {
		match self {
			Self::Base(v) => EntryValueRef::Base(*v),
			Self::Import(v) => EntryValueRef::Import(v),
			Self::Language(v) => EntryValueRef::Language(*v),
			Self::Direction(v) => EntryValueRef::Direction(*v),
			Self::Propagate(v) => EntryValueRef::Propagate(*v),
			Self::Protected(v) => EntryValueRef::Protected(*v),
			Self::Type(v) => EntryValueRef::Type(*v),
			Self::Version(v) => EntryValueRef::Version(*v),
			Self::Vocab(v) => EntryValueRef::Vocab(*v),
			Self::Definition(_, b) => EntryValueRef::Definition(*b),
		}
	}

	pub fn into_key_value(self) -> (EntryKeyRef<'a>, EntryValueRef<'a>) {
		self.key_value()
	}

	pub fn key_value(&self) -> (EntryKeyRef<'a>, EntryValueRef<'a>) {
		match self {
			Self::Base(v) => (EntryKeyRef::Base, EntryValueRef::Base(*v)),
			Self::Import(v) => (EntryKeyRef::Import, EntryValueRef::Import(v)),
			Self::Language(v) => (EntryKeyRef::Language, EntryValueRef::Language(*v)),
			Self::Direction(v) => (EntryKeyRef::Direction, EntryValueRef::Direction(*v)),
			Self::Propagate(v) => (EntryKeyRef::Propagate, EntryValueRef::Propagate(*v)),
			Self::Protected(v) => (EntryKeyRef::Protected, EntryValueRef::Protected(*v)),
			Self::Type(v) => (EntryKeyRef::Type, EntryValueRef::Type(*v)),
			Self::Version(v) => (EntryKeyRef::Version, EntryValueRef::Version(*v)),
			Self::Vocab(v) => (EntryKeyRef::Vocab, EntryValueRef::Vocab(*v)),
			Self::Definition(key, b) => {
				(EntryKeyRef::Definition(key), EntryValueRef::Definition(*b))
			}
		}
	}
}
