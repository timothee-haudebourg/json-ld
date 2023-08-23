use super::{
	BindingsIter, Definition, EntryValueSubItems, Key, KeyOrKeyword, TermBinding, Type, Version,
	Vocab,
};
use crate::{
	context::{self, TermDefinition},
	Direction, Keyword, LenientLanguageTag, LenientLanguageTagBuf, Nullable,
};
use context::Entry;
use derivative::Derivative;
use iref::{Iri, IriBuf, IriRef, IriRefBuf};
use locspan::Meta;

// pub type BaseEntryRef<'a, M> = Entry<Nullable<IriRef<'a>>, M>;

// pub type ImportEntryRef<'a, M> = Entry<IriRef<'a>, M>;

// pub type LanguageEntryRef<'a, M> = Entry<Nullable<LenientLanguageTag<'a>>, M>;

// pub type DirectionEntry<M> = Entry<Nullable<Direction>, M>;

// pub type PropagateEntry<M> = Entry<bool, M>;

// pub type ProtectedEntry<M> = Entry<bool, M>;

// pub type TypeEntry<M> = Entry<Type<M>, M>;

// pub type VersionEntry<M> = Entry<Version, M>;

// pub type VocabEntryRef<'a, M> = Entry<Nullable<VocabRef<'a>>, M>;

// pub enum BindingsIter<'a, M> {
// 	Map(indexmap::map::Iter<'a, Key, TermBinding<M>>),
// 	Slice(core::slice::Iter<'a, (KeyRef<'a>, TermBindingRef<'a, M>)>),
// }

// impl<'a, M: Clone> Iterator for BindingsIter<'a, M> {
// 	type Item = (KeyRef<'a>, TermBindingRef<'a, M>);

// 	fn size_hint(&self) -> (usize, Option<usize>) {
// 		match self {
// 			Self::Map(m) => m.size_hint(),
// 			Self::Slice(s) => s.size_hint(),
// 		}
// 	}

// 	fn next(&mut self) -> Option<Self::Item> {
// 		match self {
// 			Self::Map(m) => m.next().map(|(k, v)| (k.into(), v.into())),
// 			Self::Slice(s) => s.next().cloned(),
// 		}
// 	}
// }

// impl<'a, M: Clone> ExactSizeIterator for BindingsIter<'a, M> {}

// pub trait AnyDefinition<M>: Sized {
// 	fn base(&self) -> Option<BaseEntryRef<M>>;
// 	fn import(&self) -> Option<ImportEntryRef<M>>;
// 	fn language(&self) -> Option<LanguageEntryRef<M>>;
// 	fn direction(&self) -> Option<DirectionEntry<M>>;
// 	fn propagate(&self) -> Option<PropagateEntry<M>>;
// 	fn protected(&self) -> Option<ProtectedEntry<M>>;
// 	fn type_(&self) -> Option<TypeEntry<M>>;
// 	fn version(&self) -> Option<VersionEntry<M>>;
// 	fn vocab(&self) -> Option<VocabEntryRef<M>>;
// 	fn bindings(&self) -> BindingsIter<M>;
// 	fn get_binding(&self, key: &Key) -> Option<TermBindingRef<M>>;

// 	fn get(&self, key: &KeyOrKeyword) -> Option<EntryValueRef<M>> {
// 		match key {
// 			KeyOrKeyword::Keyword(k) => match k {
// 				Keyword::Base => self.base().map(|e| EntryValueRef::Base(e.value)),
// 				Keyword::Import => self.import().map(|e| EntryValueRef::Import(e.value)),
// 				Keyword::Language => self.language().map(|e| EntryValueRef::Language(e.value)),
// 				Keyword::Direction => self.direction().map(|e| EntryValueRef::Direction(e.value)),
// 				Keyword::Propagate => self.propagate().map(|e| EntryValueRef::Propagate(e.value)),
// 				Keyword::Protected => self.protected().map(|e| EntryValueRef::Protected(e.value)),
// 				Keyword::Type => self.type_().map(|e| EntryValueRef::Type(e.value)),
// 				Keyword::Version => self.version().map(|e| EntryValueRef::Version(e.value)),
// 				Keyword::Vocab => self.vocab().map(|e| EntryValueRef::Vocab(e.value)),
// 				_ => None,
// 			},
// 			KeyOrKeyword::Key(k) => self
// 				.get_binding(k)
// 				.map(|b| EntryValueRef::Definition(b.definition)),
// 		}
// 	}

// 	fn entries(&self) -> Entries<M> {
// 		Entries {
// 			base: self.base(),
// 			import: self.import(),
// 			language: self.language(),
// 			direction: self.direction(),
// 			propagate: self.propagate(),
// 			protected: self.protected(),
// 			type_: self.type_(),
// 			version: self.version(),
// 			vocab: self.vocab(),
// 			bindings: self.bindings(),
// 		}
// 	}
// }

impl<M> Definition<M> {
	pub fn iter(&self) -> Entries<M> {
		Entries {
			base: self.base.as_ref(),
			import: self.import.as_ref(),
			language: self.language.as_ref(),
			direction: self.direction.as_ref(),
			propagate: self.propagate.as_ref(),
			protected: self.protected.as_ref(),
			type_: self.type_.as_ref(),
			version: self.version.as_ref(),
			vocab: self.vocab.as_ref(),
			bindings: self.bindings.iter(),
		}
	}
}

pub struct Entries<'a, M> {
	base: Option<&'a Entry<Nullable<IriRefBuf>, M>>,
	import: Option<&'a Entry<IriRefBuf, M>>,
	language: Option<&'a Entry<Nullable<LenientLanguageTagBuf>, M>>,
	direction: Option<&'a Entry<Nullable<Direction>, M>>,
	propagate: Option<&'a Entry<bool, M>>,
	protected: Option<&'a Entry<bool, M>>,
	type_: Option<&'a Entry<Type<M>, M>>,
	version: Option<&'a Entry<Version, M>>,
	vocab: Option<&'a Entry<Nullable<Vocab>, M>>,
	bindings: BindingsIter<'a, M>,
}

impl<'a, M> Iterator for Entries<'a, M> {
	type Item = EntryRef<'a, M>;

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

impl<'a, M> ExactSizeIterator for Entries<'a, M> {}

pub enum EntryValueRef<'a, M> {
	Base(&'a Meta<Nullable<IriRefBuf>, M>),
	Import(&'a Meta<IriRefBuf, M>),
	Language(&'a Meta<Nullable<LenientLanguageTagBuf>, M>),
	Direction(&'a Meta<Nullable<Direction>, M>),
	Propagate(&'a Meta<bool, M>),
	Protected(&'a Meta<bool, M>),
	Type(&'a Meta<Type<M>, M>),
	Version(&'a Meta<Version, M>),
	Vocab(&'a Meta<Nullable<Vocab>, M>),
	Definition(&'a Meta<Nullable<TermDefinition<M>>, M>),
}

impl<'a, M> EntryValueRef<'a, M> {
	pub fn is_object(&self) -> bool {
		match self {
			Self::Type(_) => true,
			Self::Definition(Meta(Nullable::Some(d), _)) => d.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> EntryValueSubItems<'a, M> {
		match self {
			Self::Definition(Meta(Nullable::Some(TermDefinition::Expanded(e)), _)) => {
				EntryValueSubItems::TermDefinitionFragment(Box::new(e.iter()))
			}
			_ => EntryValueSubItems::None,
		}
	}
}

pub enum EntryRef<'a, M> {
	Base(&'a Entry<Nullable<IriRefBuf>, M>),
	Import(&'a Entry<IriRefBuf, M>),
	Language(&'a Entry<Nullable<LenientLanguageTagBuf>, M>),
	Direction(&'a Entry<Nullable<Direction>, M>),
	Propagate(&'a Entry<bool, M>),
	Protected(&'a Entry<bool, M>),
	Type(&'a Entry<Type<M>, M>),
	Version(&'a Entry<Version, M>),
	Vocab(&'a Entry<Nullable<Vocab>, M>),
	Definition(&'a Key, &'a TermBinding<M>),
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

impl<'a, M> EntryRef<'a, M> {
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

	pub fn into_value(self) -> EntryValueRef<'a, M> {
		match self {
			Self::Base(v) => EntryValueRef::Base(&v.value),
			Self::Import(v) => EntryValueRef::Import(&v.value),
			Self::Language(v) => EntryValueRef::Language(&v.value),
			Self::Direction(v) => EntryValueRef::Direction(&v.value),
			Self::Propagate(v) => EntryValueRef::Propagate(&v.value),
			Self::Protected(v) => EntryValueRef::Protected(&v.value),
			Self::Type(v) => EntryValueRef::Type(&v.value),
			Self::Version(v) => EntryValueRef::Version(&v.value),
			Self::Vocab(v) => EntryValueRef::Vocab(&v.value),
			Self::Definition(_, b) => EntryValueRef::Definition(&b.definition),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a, M> {
		match self {
			Self::Base(v) => EntryValueRef::Base(&v.value),
			Self::Import(v) => EntryValueRef::Import(&v.value),
			Self::Language(v) => EntryValueRef::Language(&v.value),
			Self::Direction(v) => EntryValueRef::Direction(&v.value),
			Self::Propagate(v) => EntryValueRef::Propagate(&v.value),
			Self::Protected(v) => EntryValueRef::Protected(&v.value),
			Self::Type(v) => EntryValueRef::Type(&v.value),
			Self::Version(v) => EntryValueRef::Version(&v.value),
			Self::Vocab(v) => EntryValueRef::Vocab(&v.value),
			Self::Definition(_, b) => EntryValueRef::Definition(&b.definition),
		}
	}

	pub fn into_key_value(self) -> (Meta<EntryKeyRef<'a>, &'a M>, EntryValueRef<'a, M>) {
		self.key_value()
	}

	pub fn key_value(&self) -> (Meta<EntryKeyRef<'a>, &'a M>, EntryValueRef<'a, M>) {
		match self {
			Self::Base(v) => (
				Meta(EntryKeyRef::Base, &v.key_metadata),
				EntryValueRef::Base(&v.value),
			),
			Self::Import(v) => (
				Meta(EntryKeyRef::Import, &v.key_metadata),
				EntryValueRef::Import(&v.value),
			),
			Self::Language(v) => (
				Meta(EntryKeyRef::Language, &v.key_metadata),
				EntryValueRef::Language(&v.value),
			),
			Self::Direction(v) => (
				Meta(EntryKeyRef::Direction, &v.key_metadata),
				EntryValueRef::Direction(&v.value),
			),
			Self::Propagate(v) => (
				Meta(EntryKeyRef::Propagate, &v.key_metadata),
				EntryValueRef::Propagate(&v.value),
			),
			Self::Protected(v) => (
				Meta(EntryKeyRef::Protected, &v.key_metadata),
				EntryValueRef::Protected(&v.value),
			),
			Self::Type(v) => (
				Meta(EntryKeyRef::Type, &v.key_metadata),
				EntryValueRef::Type(&v.value),
			),
			Self::Version(v) => (
				Meta(EntryKeyRef::Version, &v.key_metadata),
				EntryValueRef::Version(&v.value),
			),
			Self::Vocab(v) => (
				Meta(EntryKeyRef::Vocab, &v.key_metadata),
				EntryValueRef::Vocab(&v.value),
			),
			Self::Definition(key, b) => (
				Meta(EntryKeyRef::Definition(*key), &b.key_metadata),
				EntryValueRef::Definition(&b.definition),
			),
		}
	}
}
