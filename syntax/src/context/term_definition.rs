use crate::{
	container,
	context::{self, Entry},
	CompactIri, CompactIriBuf, Container, ContainerKind, Direction, Keyword, LenientLanguageTagBuf,
	Nullable,
};
use derivative::Derivative;
use iref::{Iri, IriBuf};
use locspan::Meta;
use locspan_derive::StrippedPartialEq;
use rdf_types::{BlankId, BlankIdBuf};

mod id;
mod index;
mod nest;
mod type_;

pub use id::*;
pub use index::*;
pub use nest::*;
pub use type_::*;

/// Term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged, bound(deserialize = "M: Default")))]
#[locspan(ignore(M))]
pub enum TermDefinition<M = ()> {
	Simple(Simple),
	Expanded(Box<Expanded<M>>),
}

impl<M> TermDefinition<M> {
	pub fn is_expanded(&self) -> bool {
		matches!(self, Self::Expanded(_))
	}

	pub fn is_object(&self) -> bool {
		self.is_expanded()
	}

	pub fn as_expanded<'a>(&'a self, meta: &'a M) -> ExpandedRef<'a, M> {
		match self {
			Self::Simple(term) => ExpandedRef {
				id: Some(Meta(Nullable::Some(term.as_str().into()), meta)),
				..Default::default()
			},
			Self::Expanded(e) => e.as_expanded_ref(),
		}
	}
}

#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct Simple(#[locspan(stripped)] pub(crate) String);

impl Simple {
	pub fn as_iri(&self) -> Option<&Iri> {
		Iri::new(&self.0).ok()
	}

	pub fn as_compact_iri(&self) -> Option<&CompactIri> {
		CompactIri::new(&self.0).ok()
	}

	pub fn as_blank_id(&self) -> Option<&BlankId> {
		BlankId::new(&self.0).ok()
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

impl From<IriBuf> for Simple {
	fn from(value: IriBuf) -> Self {
		Self(value.into_string())
	}
}

impl From<CompactIriBuf> for Simple {
	fn from(value: CompactIriBuf) -> Self {
		Self(value.into_string())
	}
}

impl From<BlankIdBuf> for Simple {
	fn from(value: BlankIdBuf) -> Self {
		Self(value.to_string())
	}
}

/// Expanded term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Derivative, Debug)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(bound(deserialize = "M: Default"))
)]
#[locspan(ignore(M))]
#[derivative(Default(bound = ""))]
pub struct Expanded<M> {
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@id", default, skip_serializing_if = "Option::is_none")
	)]
	pub id: Option<Entry<Nullable<Id>, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@type", default, skip_serializing_if = "Option::is_none")
	)]
	pub type_: Option<Entry<Nullable<Type>, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@context", default, skip_serializing_if = "Option::is_none")
	)]
	pub context: Option<Entry<Box<context::Context<M>>, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")
	)]
	pub reverse: Option<Entry<context::definition::Key, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@index", default, skip_serializing_if = "Option::is_none")
	)]
	pub index: Option<Entry<Index, M>>,

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
			rename = "@container",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub container: Option<Entry<Nullable<Container<M>>, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@nest", default, skip_serializing_if = "Option::is_none")
	)]
	pub nest: Option<Entry<Nest, M>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@prefix", default, skip_serializing_if = "Option::is_none")
	)]
	pub prefix: Option<Entry<bool, M>>,

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
}

impl<M> Expanded<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_null(&self) -> bool {
		matches!(
			&self.id,
			None | Some(Entry {
				key_metadata: _,
				value: Meta(Nullable::Null, _)
			})
		) && self.type_.is_none()
			&& self.context.is_none()
			&& self.reverse.is_none()
			&& self.index.is_none()
			&& self.language.is_none()
			&& self.direction.is_none()
			&& self.container.is_none()
			&& self.nest.is_none()
			&& self.prefix.is_none()
			&& self.propagate.is_none()
			&& self.protected.is_none()
	}

	pub fn is_simple_definition(&self) -> bool {
		matches!(
			&self.id,
			Some(Entry {
				key_metadata: _,
				value: Meta(Nullable::Some(_), _)
			})
		) && self.type_.is_none()
			&& self.context.is_none()
			&& self.reverse.is_none()
			&& self.index.is_none()
			&& self.language.is_none()
			&& self.direction.is_none()
			&& self.container.is_none()
			&& self.nest.is_none()
			&& self.prefix.is_none()
			&& self.propagate.is_none()
			&& self.protected.is_none()
	}

	pub fn simplify(self) -> Nullable<TermDefinition<M>> {
		if self.is_null() {
			Nullable::Null
		} else if self.is_simple_definition() {
			let Meta(id_value, _) = self.id.unwrap().value;
			Nullable::Some(TermDefinition::Simple(Simple(
				id_value.unwrap().into_string(),
			)))
		} else {
			Nullable::Some(TermDefinition::Expanded(Box::new(self)))
		}
	}

	pub fn iter(&self) -> Entries<M> {
		Entries {
			id: self.id.as_ref(),
			type_: self.type_.as_ref(),
			context: self.context.as_ref(),
			reverse: self.reverse.as_ref(),
			index: self.index.as_ref(),
			language: self.language.as_ref(),
			direction: self.direction.as_ref(),
			container: self.container.as_ref(),
			nest: self.nest.as_ref(),
			prefix: self.prefix.as_ref(),
			propagate: self.propagate.as_ref(),
			protected: self.protected.as_ref(),
		}
	}

	pub fn as_expanded_ref(&self) -> ExpandedRef<M> {
		ExpandedRef {
			id: self
				.id
				.as_ref()
				.map(|n| Meta(n.0.as_ref().map(|i| i.as_id_ref()), &n.1)),
			type_: self.type_.as_ref(),
			context: self.context.as_ref(),
			reverse: self.reverse.as_ref(),
			index: self.index.as_ref(),
			language: self.language.as_ref(),
			direction: self.direction.as_ref(),
			container: self.container.as_ref(),
			nest: self.nest.as_ref(),
			prefix: self.prefix.as_ref(),
			propagate: self.propagate.as_ref(),
			protected: self.protected.as_ref(),
		}
	}
}

/// Expanded term definition.
#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct ExpandedRef<'a, M> {
	pub id: Option<Meta<Nullable<IdRef<'a>>, &'a M>>,
	pub type_: Option<&'a Entry<Nullable<Type>, M>>,
	pub context: Option<&'a Entry<Box<context::Context<M>>, M>>,
	pub reverse: Option<&'a Entry<context::definition::Key, M>>,
	pub index: Option<&'a Entry<Index, M>>,
	pub language: Option<&'a Entry<Nullable<LenientLanguageTagBuf>, M>>,
	pub direction: Option<&'a Entry<Nullable<Direction>, M>>,
	pub container: Option<&'a Entry<Nullable<Container<M>>, M>>,
	pub nest: Option<&'a Entry<Nest, M>>,
	pub prefix: Option<&'a Entry<bool, M>>,
	pub propagate: Option<&'a Entry<bool, M>>,
	pub protected: Option<&'a Entry<bool, M>>,
}

impl<'a, M> From<&'a Meta<Nullable<TermDefinition<M>>, M>> for ExpandedRef<'a, M> {
	fn from(Meta(value, meta): &'a Meta<Nullable<TermDefinition<M>>, M>) -> Self {
		match value {
			Nullable::Some(d) => d.as_expanded(meta),
			Nullable::Null => Self {
				id: Some(Meta(Nullable::Null, meta)),
				..Default::default()
			},
		}
	}
}

// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
// pub struct SimpleRef<'a>(&'a str);

// impl<'a> SimpleRef<'a> {
// 	pub fn as_iri(self) -> Option<Iri<'a>> {
// 		Iri::new(self.0).ok()
// 	}

// 	pub fn as_compact_iri(self) -> Option<&'a CompactIri> {
// 		CompactIri::new(self.0).ok()
// 	}

// 	pub fn as_blank_id(self) -> Option<&'a BlankId> {
// 		BlankId::new(self.0).ok()
// 	}

// 	pub fn as_str(self) -> &'a str {
// 		self.0
// 	}
// }

// /// Term definition.
// #[derive(Derivative)]
// #[derivative(Clone(bound = "M: Clone"))]
// pub enum TermDefinitionRef<'a, M> {
// 	Simple(SimpleRef<'a>),
// 	Expanded(ExpandedRef<'a, M>),
// }

// impl<'a, M> TermDefinitionRef<'a, M> {
// 	pub fn is_expanded(&self) -> bool {
// 		matches!(self, Self::Expanded(_))
// 	}

// 	pub fn is_object(&self) -> bool {
// 		self.is_expanded()
// 	}
// }

// impl<'a, M: Clone> From<&'a TermDefinition<M>> for TermDefinitionRef<'a, M> {
// 	fn from(d: &'a TermDefinition<M>) -> Self {
// 		match d {
// 			TermDefinition::Simple(s) => Self::Simple(SimpleRef(s.as_str())),
// 			TermDefinition::Expanded(e) => Self::Expanded((&**e).into()),
// 		}
// 	}
// }

// pub type IdEntryRef<'a, M> = Entry<Nullable<IdRef<'a>>, M>;
// pub type TypeEntryRef<'a, M> = Entry<Nullable<TypeRef<'a>>, M>;
// pub type ReverseEntryRef<'a, M> = Entry<context::definition::KeyRef<'a>, M>;
// pub type IndexEntryRef<'a, M> = Entry<IndexRef<'a>, M>;
// pub type LanguageEntryRef<'a, M> = Entry<Nullable<LenientLanguageTag<'a>>, M>;
// pub type DirectionEntry<M> = Entry<Nullable<Direction>, M>;
// pub type ContainerEntryRef<'a, M> = Entry<Nullable<ContainerRef<'a, M>>, M>;
// pub type NestEntryRef<'a, M> = Entry<NestRef<'a>, M>;

// /// Expanded term definition.
// #[derive(Derivative)]
// #[derivative(Default(bound = ""), Clone(bound = "M: Clone"))]
// pub struct ExpandedRef<'a, M> {
// 	pub id: Option<&'a Entry<Nullable<Id>, M>>,
// 	pub type_: Option<&'a Entry<Nullable<Type>, M>>,
// 	pub context: Option<&'a Entry<context::Value<M>, M>>,
// 	pub reverse: Option<&'a Entry<context::definition::Key, M>>,
// 	pub index: Option<IndexEntryRef<'a, M>>,
// 	pub language: Option<LanguageEntryRef<'a, M>>,
// 	pub direction: Option<DirectionEntry<M>>,
// 	pub container: Option<ContainerEntryRef<'a, M>>,
// 	pub nest: Option<NestEntryRef<'a, M>>,
// 	pub prefix: Option<Entry<bool, M>>,
// 	pub propagate: Option<Entry<bool, M>>,
// 	pub protected: Option<Entry<bool, M>>,
// }

// impl<'a, M> ExpandedRef<'a, M> {
// 	pub fn iter(&self) -> Entries<'a, M>
// 	where
// 		M: Clone,
// 	{
// 		Entries {
// 			id: self.id.clone(),
// 			type_: self.type_.clone(),
// 			context: self.context.clone(),
// 			reverse: self.reverse.clone(),
// 			index: self.index.clone(),
// 			language: self.language.clone(),
// 			direction: self.direction.clone(),
// 			container: self.container.clone(),
// 			nest: self.nest.clone(),
// 			prefix: self.prefix.clone(),
// 			propagate: self.propagate.clone(),
// 			protected: self.protected.clone(),
// 		}
// 	}
// }

// impl<'a, M> IntoIterator for ExpandedRef<'a, M> {
// 	type Item = EntryRef<'a, M>;
// 	type IntoIter = Entries<'a, M>;

// 	fn into_iter(self) -> Entries<'a, M> {
// 		Entries {
// 			id: self.id,
// 			type_: self.type_,
// 			context: self.context,
// 			reverse: self.reverse,
// 			index: self.index,
// 			language: self.language,
// 			direction: self.direction,
// 			container: self.container,
// 			nest: self.nest,
// 			prefix: self.prefix,
// 			propagate: self.propagate,
// 			protected: self.protected,
// 		}
// 	}
// }

// impl<'a, M: Clone> From<Meta<Nullable<TermDefinitionRef<'a, M>>, M>>
// 	for ExpandedRef<'a, M>
// {
// 	fn from(Meta(d, loc): Meta<Nullable<TermDefinitionRef<'a, M>>, M>) -> Self {
// 		match d {
// 			Nullable::Null => {
// 				// If `value` is null, convert it to a map consisting of a single entry
// 				// whose key is @id and whose value is null.
// 				Self {
// 					id: Some(Entry::new_with(loc.clone(), Meta(Nullable::Null, loc))),
// 					..Default::default()
// 				}
// 			}
// 			Nullable::Some(TermDefinitionRef::Simple(s)) => Self {
// 				id: Some(Entry::new_with(
// 					loc.clone(),
// 					Meta(Nullable::Some(s.as_str().into()), loc),
// 				)),
// 				..Default::default()
// 			},
// 			Nullable::Some(TermDefinitionRef::Expanded(e)) => e,
// 		}
// 	}
// }

// impl<'a, M: Clone> From<&'a Expanded<M>> for ExpandedRef<'a, M> {
// 	fn from(d: &'a Expanded<M>) -> Self {
// 		Self {
// 			id: d
// 				.id
// 				.as_ref()
// 				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
// 			type_: d
// 				.type_
// 				.as_ref()
// 				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
// 			context: d.context.as_ref().map(|v| v.borrow_value().into_deref()),
// 			reverse: d.reverse.as_ref().map(|v| v.borrow_value().cast()),
// 			index: d.index.as_ref().map(|v| v.borrow_value().cast()),
// 			language: d
// 				.language
// 				.as_ref()
// 				.map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref()))),
// 			direction: d.direction.clone(),
// 			container: d
// 				.container
// 				.as_ref()
// 				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
// 			nest: d.nest.as_ref().map(|v| v.borrow_value().cast()),
// 			prefix: d.prefix.clone(),
// 			propagate: d.propagate.clone(),
// 			protected: d.protected.clone(),
// 		}
// 	}
// }

/// Term definition entries.
pub struct Entries<'a, M> {
	id: Option<&'a Entry<Nullable<Id>, M>>,
	type_: Option<&'a Entry<Nullable<Type>, M>>,
	context: Option<&'a Entry<Box<context::Context<M>>, M>>,
	reverse: Option<&'a Entry<context::definition::Key, M>>,
	index: Option<&'a Entry<Index, M>>,
	language: Option<&'a Entry<Nullable<LenientLanguageTagBuf>, M>>,
	direction: Option<&'a Entry<Nullable<Direction>, M>>,
	container: Option<&'a Entry<Nullable<Container<M>>, M>>,
	nest: Option<&'a Entry<Nest, M>>,
	prefix: Option<&'a Entry<bool, M>>,
	propagate: Option<&'a Entry<bool, M>>,
	protected: Option<&'a Entry<bool, M>>,
}

pub enum EntryRef<'a, M> {
	Id(&'a Entry<Nullable<Id>, M>),
	Type(&'a Entry<Nullable<Type>, M>),
	Context(&'a Entry<Box<context::Context<M>>, M>),
	Reverse(&'a Entry<context::definition::Key, M>),
	Index(&'a Entry<Index, M>),
	Language(&'a Entry<Nullable<LenientLanguageTagBuf>, M>),
	Direction(&'a Entry<Nullable<Direction>, M>),
	Container(&'a Entry<Nullable<Container<M>>, M>),
	Nest(&'a Entry<Nest, M>),
	Prefix(&'a Entry<bool, M>),
	Propagate(&'a Entry<bool, M>),
	Protected(&'a Entry<bool, M>),
}

impl<'a, M> EntryRef<'a, M> {
	pub fn into_key(self) -> EntryKey {
		match self {
			Self::Id(_) => EntryKey::Id,
			Self::Type(_) => EntryKey::Type,
			Self::Context(_) => EntryKey::Context,
			Self::Reverse(_) => EntryKey::Reverse,
			Self::Index(_) => EntryKey::Index,
			Self::Language(_) => EntryKey::Language,
			Self::Direction(_) => EntryKey::Direction,
			Self::Container(_) => EntryKey::Container,
			Self::Nest(_) => EntryKey::Nest,
			Self::Prefix(_) => EntryKey::Prefix,
			Self::Propagate(_) => EntryKey::Propagate,
			Self::Protected(_) => EntryKey::Protected,
		}
	}

	pub fn key(&self) -> EntryKey {
		match self {
			Self::Id(_) => EntryKey::Id,
			Self::Type(_) => EntryKey::Type,
			Self::Context(_) => EntryKey::Context,
			Self::Reverse(_) => EntryKey::Reverse,
			Self::Index(_) => EntryKey::Index,
			Self::Language(_) => EntryKey::Language,
			Self::Direction(_) => EntryKey::Direction,
			Self::Container(_) => EntryKey::Container,
			Self::Nest(_) => EntryKey::Nest,
			Self::Prefix(_) => EntryKey::Prefix,
			Self::Propagate(_) => EntryKey::Propagate,
			Self::Protected(_) => EntryKey::Protected,
		}
	}

	pub fn into_value(self) -> EntryValueRef<'a, M> {
		self.value()
	}

	pub fn value(&self) -> EntryValueRef<'a, M> {
		match self {
			Self::Id(e) => EntryValueRef::Id(&e.value),
			Self::Type(e) => EntryValueRef::Type(&e.value),
			Self::Context(e) => EntryValueRef::Context(&e.value),
			Self::Reverse(e) => EntryValueRef::Reverse(&e.value),
			Self::Index(e) => EntryValueRef::Index(&e.value),
			Self::Language(e) => EntryValueRef::Language(&e.value),
			Self::Direction(e) => EntryValueRef::Direction(&e.value),
			Self::Container(e) => EntryValueRef::Container(&e.value),
			Self::Nest(e) => EntryValueRef::Nest(&e.value),
			Self::Prefix(e) => EntryValueRef::Prefix(&e.value),
			Self::Propagate(e) => EntryValueRef::Propagate(&e.value),
			Self::Protected(e) => EntryValueRef::Protected(&e.value),
		}
	}

	pub fn into_key_value(self) -> (EntryKey, EntryValueRef<'a, M>) {
		self.key_value()
	}

	pub fn key_value(&self) -> (EntryKey, EntryValueRef<'a, M>) {
		match self {
			Self::Id(e) => (EntryKey::Id, EntryValueRef::Id(&e.value)),
			Self::Type(e) => (EntryKey::Type, EntryValueRef::Type(&e.value)),
			Self::Context(e) => (EntryKey::Context, EntryValueRef::Context(&e.value)),
			Self::Reverse(e) => (EntryKey::Reverse, EntryValueRef::Reverse(&e.value)),
			Self::Index(e) => (EntryKey::Index, EntryValueRef::Index(&e.value)),
			Self::Language(e) => (EntryKey::Language, EntryValueRef::Language(&e.value)),
			Self::Direction(e) => (EntryKey::Direction, EntryValueRef::Direction(&e.value)),
			Self::Container(e) => (EntryKey::Container, EntryValueRef::Container(&e.value)),
			Self::Nest(e) => (EntryKey::Nest, EntryValueRef::Nest(&e.value)),
			Self::Prefix(e) => (EntryKey::Prefix, EntryValueRef::Prefix(&e.value)),
			Self::Propagate(e) => (EntryKey::Propagate, EntryValueRef::Propagate(&e.value)),
			Self::Protected(e) => (EntryKey::Protected, EntryValueRef::Protected(&e.value)),
		}
	}
}

pub enum EntryKey {
	Id,
	Type,
	Context,
	Reverse,
	Index,
	Language,
	Direction,
	Container,
	Nest,
	Prefix,
	Propagate,
	Protected,
}

impl EntryKey {
	pub fn keyword(&self) -> Keyword {
		match self {
			Self::Id => Keyword::Id,
			Self::Type => Keyword::Type,
			Self::Context => Keyword::Context,
			Self::Reverse => Keyword::Reverse,
			Self::Index => Keyword::Index,
			Self::Language => Keyword::Language,
			Self::Direction => Keyword::Direction,
			Self::Container => Keyword::Container,
			Self::Nest => Keyword::Nest,
			Self::Prefix => Keyword::Prefix,
			Self::Propagate => Keyword::Propagate,
			Self::Protected => Keyword::Protected,
		}
	}

	pub fn as_str(&self) -> &'static str {
		self.keyword().into_str()
	}
}

pub enum EntryValueRef<'a, M> {
	Id(&'a Meta<Nullable<Id>, M>),
	Type(&'a Meta<Nullable<Type>, M>),
	Context(&'a Meta<Box<context::Context<M>>, M>),
	Reverse(&'a Meta<context::definition::Key, M>),
	Index(&'a Meta<Index, M>),
	Language(&'a Meta<Nullable<LenientLanguageTagBuf>, M>),
	Direction(&'a Meta<Nullable<Direction>, M>),
	Container(&'a Meta<Nullable<Container<M>>, M>),
	Nest(&'a Meta<Nest, M>),
	Prefix(&'a Meta<bool, M>),
	Propagate(&'a Meta<bool, M>),
	Protected(&'a Meta<bool, M>),
}

impl<'a, M> EntryValueRef<'a, M> {
	pub fn is_object(&self) -> bool {
		match self {
			Self::Context(c) => c.is_object(),
			_ => false,
		}
	}

	pub fn is_array(&self) -> bool {
		match self {
			Self::Container(Meta(Nullable::Some(c), _)) => c.is_array(),
			_ => false,
		}
	}
}

impl<'a, M> Iterator for Entries<'a, M> {
	type Item = EntryRef<'a, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = 0;

		if self.id.is_some() {
			len += 1
		}

		if self.type_.is_some() {
			len += 1
		}

		if self.context.is_some() {
			len += 1
		}

		if self.reverse.is_some() {
			len += 1
		}

		if self.index.is_some() {
			len += 1
		}

		if self.language.is_some() {
			len += 1
		}

		if self.direction.is_some() {
			len += 1
		}

		if self.container.is_some() {
			len += 1
		}

		if self.nest.is_some() {
			len += 1
		}

		if self.prefix.is_some() {
			len += 1
		}

		if self.propagate.is_some() {
			len += 1
		}

		if self.protected.is_some() {
			len += 1
		}

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self.id.take() {
			Some(value) => Some(EntryRef::Id(value)),
			None => match self.type_.take() {
				Some(value) => Some(EntryRef::Type(value)),
				None => match self.context.take() {
					Some(value) => Some(EntryRef::Context(value)),
					None => match self.reverse.take() {
						Some(value) => Some(EntryRef::Reverse(value)),
						None => match self.index.take() {
							Some(value) => Some(EntryRef::Index(value)),
							None => match self.language.take() {
								Some(value) => Some(EntryRef::Language(value)),
								None => match self.direction.take() {
									Some(value) => Some(EntryRef::Direction(value)),
									None => match self.container.take() {
										Some(value) => Some(EntryRef::Container(value)),
										None => match self.nest.take() {
											Some(value) => Some(EntryRef::Nest(value)),
											None => match self.prefix.take() {
												Some(value) => Some(EntryRef::Prefix(value)),
												None => match self.propagate.take() {
													Some(value) => Some(EntryRef::Propagate(value)),
													None => self
														.protected
														.take()
														.map(EntryRef::Protected),
												},
											},
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

/// Term definition fragment.
pub enum FragmentRef<'a, M> {
	/// Term definition entry.
	Entry(EntryRef<'a, M>),

	/// Term definition entry key.
	Key(EntryKey),

	/// Term definition entry value.
	Value(EntryValueRef<'a, M>),

	/// Container value fragment.
	ContainerFragment(&'a Meta<ContainerKind, M>),
}

impl<'a, M> FragmentRef<'a, M> {
	pub fn is_key(&self) -> bool {
		matches!(self, Self::Key(_))
	}

	pub fn is_entry(&self) -> bool {
		matches!(self, Self::Entry(_))
	}

	pub fn is_array(&self) -> bool {
		match self {
			Self::Value(v) => v.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Value(v) => v.is_object(),
			_ => false,
		}
	}

	pub fn sub_fragments(&self) -> SubFragments<'a, M> {
		match self {
			Self::Value(EntryValueRef::Container(Meta(Nullable::Some(c), _))) => {
				SubFragments::Container(c.sub_fragments())
			}
			_ => SubFragments::None,
		}
	}
}

pub enum SubFragments<'a, M> {
	None,
	Container(container::SubValues<'a, M>),
}

impl<'a, M> Iterator for SubFragments<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Container(c) => c.next().map(FragmentRef::ContainerFragment),
		}
	}
}
