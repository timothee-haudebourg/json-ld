use std::marker::PhantomData;

use crate::{
	container,
	context::{self, Entry},
	CompactIri, Container, ContainerKind, ContainerRef, Direction, Keyword, LenientLanguageTag,
	LenientLanguageTagBuf, Nullable,
};
use derivative::Derivative;
use iref::Iri;
use locspan::Meta;
use locspan_derive::StrippedPartialEq;
use rdf_types::BlankId;

mod id;
mod index;
mod nest;
mod type_;

pub use id::*;
pub use index::*;
pub use nest::*;
pub use type_::*;

use super::AnyValue;

/// Term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[stripped_ignore(M)]
pub enum TermDefinition<M, C = context::Value<M>> {
	Simple(Simple),
	Expanded(Box<Expanded<M, C>>),
}

#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
pub struct Simple(#[stripped] pub(crate) String);

impl Simple {
	pub fn as_iri(&self) -> Option<Iri> {
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

/// Expanded term definition.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Derivative, Debug)]
#[stripped_ignore(M)]
#[derivative(Default(bound = ""))]
pub struct Expanded<M, C = context::Value<M>> {
	pub id: Option<Entry<Nullable<Id>, M>>,
	pub type_: Option<Entry<Nullable<Type>, M>>,
	pub context: Option<Entry<Box<C>, M>>,
	pub reverse: Option<Entry<context::definition::Key, M>>,
	pub index: Option<Entry<Index, M>>,
	pub language: Option<Entry<Nullable<LenientLanguageTagBuf>, M>>,
	pub direction: Option<Entry<Nullable<Direction>, M>>,
	pub container: Option<Entry<Nullable<Container<M>>, M>>,
	pub nest: Option<Entry<Nest, M>>,
	pub prefix: Option<Entry<bool, M>>,
	pub propagate: Option<Entry<bool, M>>,
	pub protected: Option<Entry<bool, M>>,
}

impl<M, C> Expanded<M, C> {
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

	pub fn simplify(self) -> Nullable<TermDefinition<M, C>> {
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
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SimpleRef<'a>(&'a str);

impl<'a> SimpleRef<'a> {
	pub fn as_iri(self) -> Option<Iri<'a>> {
		Iri::new(self.0).ok()
	}

	pub fn as_compact_iri(self) -> Option<&'a CompactIri> {
		CompactIri::new(self.0).ok()
	}

	pub fn as_blank_id(self) -> Option<&'a BlankId> {
		BlankId::new(self.0).ok()
	}

	pub fn as_str(self) -> &'a str {
		self.0
	}
}

/// Term definition.
#[derive(Derivative)]
#[derivative(Clone(bound = "M: Clone"))]
pub enum TermDefinitionRef<'a, M, C> {
	Simple(SimpleRef<'a>),
	Expanded(ExpandedRef<'a, M, C>),
}

impl<'a, M, C> TermDefinitionRef<'a, M, C> {
	pub fn is_expanded(&self) -> bool {
		matches!(self, Self::Expanded(_))
	}

	pub fn is_object(&self) -> bool {
		self.is_expanded()
	}
}

impl<'a, M: Clone, C> From<&'a TermDefinition<M, C>> for TermDefinitionRef<'a, M, C> {
	fn from(d: &'a TermDefinition<M, C>) -> Self {
		match d {
			TermDefinition::Simple(s) => Self::Simple(SimpleRef(s.as_str())),
			TermDefinition::Expanded(e) => Self::Expanded((&**e).into()),
		}
	}
}

pub type IdEntryRef<'a, M> = Entry<Nullable<IdRef<'a>>, M>;
pub type TypeEntryRef<'a, M> = Entry<Nullable<TypeRef<'a>>, M>;
pub type ReverseEntryRef<'a, M> = Entry<context::definition::KeyRef<'a>, M>;
pub type IndexEntryRef<'a, M> = Entry<IndexRef<'a>, M>;
pub type LanguageEntryRef<'a, M> = Entry<Nullable<LenientLanguageTag<'a>>, M>;
pub type DirectionEntry<M> = Entry<Nullable<Direction>, M>;
pub type ContainerEntryRef<'a, M> = Entry<Nullable<ContainerRef<'a, M>>, M>;
pub type NestEntryRef<'a, M> = Entry<NestRef<'a>, M>;

/// Expanded term definition.
#[derive(Derivative)]
#[derivative(Default(bound = ""), Clone(bound = "M: Clone"))]
pub struct ExpandedRef<'a, M, C> {
	pub id: Option<IdEntryRef<'a, M>>,
	pub type_: Option<TypeEntryRef<'a, M>>,
	pub context: Option<Entry<&'a C, M>>,
	pub reverse: Option<ReverseEntryRef<'a, M>>,
	pub index: Option<IndexEntryRef<'a, M>>,
	pub language: Option<LanguageEntryRef<'a, M>>,
	pub direction: Option<DirectionEntry<M>>,
	pub container: Option<ContainerEntryRef<'a, M>>,
	pub nest: Option<NestEntryRef<'a, M>>,
	pub prefix: Option<Entry<bool, M>>,
	pub propagate: Option<Entry<bool, M>>,
	pub protected: Option<Entry<bool, M>>,
}

impl<'a, M, C> ExpandedRef<'a, M, C> {
	pub fn iter(&self) -> Entries<'a, M, C>
	where
		M: Clone,
	{
		Entries {
			id: self.id.clone(),
			type_: self.type_.clone(),
			context: self.context.clone(),
			reverse: self.reverse.clone(),
			index: self.index.clone(),
			language: self.language.clone(),
			direction: self.direction.clone(),
			container: self.container.clone(),
			nest: self.nest.clone(),
			prefix: self.prefix.clone(),
			propagate: self.propagate.clone(),
			protected: self.protected.clone(),
		}
	}
}

impl<'a, M, C> IntoIterator for ExpandedRef<'a, M, C> {
	type Item = EntryRef<'a, M, C>;
	type IntoIter = Entries<'a, M, C>;

	fn into_iter(self) -> Entries<'a, M, C> {
		Entries {
			id: self.id,
			type_: self.type_,
			context: self.context,
			reverse: self.reverse,
			index: self.index,
			language: self.language,
			direction: self.direction,
			container: self.container,
			nest: self.nest,
			prefix: self.prefix,
			propagate: self.propagate,
			protected: self.protected,
		}
	}
}

impl<'a, M: Clone, C> From<Meta<Nullable<TermDefinitionRef<'a, M, C>>, M>>
	for ExpandedRef<'a, M, C>
{
	fn from(Meta(d, loc): Meta<Nullable<TermDefinitionRef<'a, M, C>>, M>) -> Self {
		match d {
			Nullable::Null => {
				// If `value` is null, convert it to a map consisting of a single entry
				// whose key is @id and whose value is null.
				Self {
					id: Some(Entry::new(loc.clone(), Meta(Nullable::Null, loc))),
					..Default::default()
				}
			}
			Nullable::Some(TermDefinitionRef::Simple(s)) => Self {
				id: Some(Entry::new(
					loc.clone(),
					Meta(Nullable::Some(s.as_str().into()), loc),
				)),
				..Default::default()
			},
			Nullable::Some(TermDefinitionRef::Expanded(e)) => e,
		}
	}
}

impl<'a, M: Clone, C> From<&'a Expanded<M, C>> for ExpandedRef<'a, M, C> {
	fn from(d: &'a Expanded<M, C>) -> Self {
		Self {
			id: d
				.id
				.as_ref()
				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			type_: d
				.type_
				.as_ref()
				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			context: d.context.as_ref().map(|v| v.borrow_value().into_deref()),
			reverse: d.reverse.as_ref().map(|v| v.borrow_value().cast()),
			index: d.index.as_ref().map(|v| v.borrow_value().cast()),
			language: d
				.language
				.as_ref()
				.map(|v| v.borrow_value().map(|v| v.as_ref().map(|v| v.as_ref()))),
			direction: d.direction.clone(),
			container: d
				.container
				.as_ref()
				.map(|v| v.borrow_value().map(|v| v.as_ref().cast())),
			nest: d.nest.as_ref().map(|v| v.borrow_value().cast()),
			prefix: d.prefix.clone(),
			propagate: d.propagate.clone(),
			protected: d.protected.clone(),
		}
	}
}

pub struct Entries<'a, M, C> {
	id: Option<IdEntryRef<'a, M>>,
	type_: Option<TypeEntryRef<'a, M>>,
	context: Option<Entry<&'a C, M>>,
	reverse: Option<ReverseEntryRef<'a, M>>,
	index: Option<IndexEntryRef<'a, M>>,
	language: Option<LanguageEntryRef<'a, M>>,
	direction: Option<DirectionEntry<M>>,
	container: Option<ContainerEntryRef<'a, M>>,
	nest: Option<NestEntryRef<'a, M>>,
	prefix: Option<Entry<bool, M>>,
	propagate: Option<Entry<bool, M>>,
	protected: Option<Entry<bool, M>>,
}

pub enum EntryRef<'a, M, C> {
	Id(Entry<Nullable<IdRef<'a>>, M>),
	Type(Entry<Nullable<TypeRef<'a>>, M>),
	Context(Entry<&'a C, M>),
	Reverse(Entry<context::definition::KeyRef<'a>, M>),
	Index(Entry<IndexRef<'a>, M>),
	Language(Entry<Nullable<LenientLanguageTag<'a>>, M>),
	Direction(Entry<Nullable<Direction>, M>),
	Container(Entry<Nullable<ContainerRef<'a, M>>, M>),
	Nest(Entry<NestRef<'a>, M>),
	Prefix(Entry<bool, M>),
	Propagate(Entry<bool, M>),
	Protected(Entry<bool, M>),
}

impl<'a, M, C> EntryRef<'a, M, C> {
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

	pub fn into_value(self) -> EntryValueRef<'a, M, C> {
		match self {
			Self::Id(e) => EntryValueRef::Id(e.value),
			Self::Type(e) => EntryValueRef::Type(e.value),
			Self::Context(e) => EntryValueRef::Context(e.value),
			Self::Reverse(e) => EntryValueRef::Reverse(e.value),
			Self::Index(e) => EntryValueRef::Index(e.value),
			Self::Language(e) => EntryValueRef::Language(e.value),
			Self::Direction(e) => EntryValueRef::Direction(e.value),
			Self::Container(e) => EntryValueRef::Container(e.value),
			Self::Nest(e) => EntryValueRef::Nest(e.value),
			Self::Prefix(e) => EntryValueRef::Prefix(e.value),
			Self::Propagate(e) => EntryValueRef::Propagate(e.value),
			Self::Protected(e) => EntryValueRef::Protected(e.value),
		}
	}

	pub fn value(&self) -> EntryValueRef<'a, M, C>
	where
		M: Clone,
	{
		match self {
			Self::Id(e) => EntryValueRef::Id(e.value.clone()),
			Self::Type(e) => EntryValueRef::Type(e.value.clone()),
			Self::Context(e) => EntryValueRef::Context(e.value.clone()),
			Self::Reverse(e) => EntryValueRef::Reverse(e.value.clone()),
			Self::Index(e) => EntryValueRef::Index(e.value.clone()),
			Self::Language(e) => EntryValueRef::Language(e.value.clone()),
			Self::Direction(e) => EntryValueRef::Direction(e.value.clone()),
			Self::Container(e) => EntryValueRef::Container(e.value.clone()),
			Self::Nest(e) => EntryValueRef::Nest(e.value.clone()),
			Self::Prefix(e) => EntryValueRef::Prefix(e.value.clone()),
			Self::Propagate(e) => EntryValueRef::Propagate(e.value.clone()),
			Self::Protected(e) => EntryValueRef::Protected(e.value.clone()),
		}
	}

	pub fn into_key_value(self) -> (EntryKey, EntryValueRef<'a, M, C>) {
		match self {
			Self::Id(e) => (EntryKey::Id, EntryValueRef::Id(e.value)),
			Self::Type(e) => (EntryKey::Type, EntryValueRef::Type(e.value)),
			Self::Context(e) => (EntryKey::Context, EntryValueRef::Context(e.value)),
			Self::Reverse(e) => (EntryKey::Reverse, EntryValueRef::Reverse(e.value)),
			Self::Index(e) => (EntryKey::Index, EntryValueRef::Index(e.value)),
			Self::Language(e) => (EntryKey::Language, EntryValueRef::Language(e.value)),
			Self::Direction(e) => (EntryKey::Direction, EntryValueRef::Direction(e.value)),
			Self::Container(e) => (EntryKey::Container, EntryValueRef::Container(e.value)),
			Self::Nest(e) => (EntryKey::Nest, EntryValueRef::Nest(e.value)),
			Self::Prefix(e) => (EntryKey::Prefix, EntryValueRef::Prefix(e.value)),
			Self::Propagate(e) => (EntryKey::Propagate, EntryValueRef::Propagate(e.value)),
			Self::Protected(e) => (EntryKey::Protected, EntryValueRef::Protected(e.value)),
		}
	}

	pub fn key_value(&self) -> (EntryKey, EntryValueRef<'a, M, C>)
	where
		M: Clone,
	{
		match self {
			Self::Id(e) => (EntryKey::Id, EntryValueRef::Id(e.value.clone())),
			Self::Type(e) => (EntryKey::Type, EntryValueRef::Type(e.value.clone())),
			Self::Context(e) => (EntryKey::Context, EntryValueRef::Context(e.value.clone())),
			Self::Reverse(e) => (EntryKey::Reverse, EntryValueRef::Reverse(e.value.clone())),
			Self::Index(e) => (EntryKey::Index, EntryValueRef::Index(e.value.clone())),
			Self::Language(e) => (EntryKey::Language, EntryValueRef::Language(e.value.clone())),
			Self::Direction(e) => (
				EntryKey::Direction,
				EntryValueRef::Direction(e.value.clone()),
			),
			Self::Container(e) => (
				EntryKey::Container,
				EntryValueRef::Container(e.value.clone()),
			),
			Self::Nest(e) => (EntryKey::Nest, EntryValueRef::Nest(e.value.clone())),
			Self::Prefix(e) => (EntryKey::Prefix, EntryValueRef::Prefix(e.value.clone())),
			Self::Propagate(e) => (
				EntryKey::Propagate,
				EntryValueRef::Propagate(e.value.clone()),
			),
			Self::Protected(e) => (
				EntryKey::Protected,
				EntryValueRef::Protected(e.value.clone()),
			),
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

pub enum EntryValueRef<'a, M, C> {
	Id(Meta<Nullable<IdRef<'a>>, M>),
	Type(Meta<Nullable<TypeRef<'a>>, M>),
	Context(Meta<&'a C, M>),
	Reverse(Meta<context::definition::KeyRef<'a>, M>),
	Index(Meta<IndexRef<'a>, M>),
	Language(Meta<Nullable<LenientLanguageTag<'a>>, M>),
	Direction(Meta<Nullable<Direction>, M>),
	Container(Meta<Nullable<ContainerRef<'a, M>>, M>),
	Nest(Meta<NestRef<'a>, M>),
	Prefix(Meta<bool, M>),
	Propagate(Meta<bool, M>),
	Protected(Meta<bool, M>),
}

impl<'a, M, C> EntryValueRef<'a, M, C> {
	pub fn is_object(&self) -> bool
	where
		C: AnyValue<M>,
	{
		match self {
			Self::Context(c) => c.as_value_ref().is_object(),
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

impl<'a, M, C> Iterator for Entries<'a, M, C> {
	type Item = EntryRef<'a, M, C>;

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

impl<'a, M, C> ExactSizeIterator for Entries<'a, M, C> {}

/// Term definition fragment.
pub enum FragmentRef<'a, M, C> {
	/// Term definition entry.
	Entry(EntryRef<'a, M, C>),

	/// Term definition entry key.
	Key(EntryKey),

	/// Term definition entry value.
	Value(EntryValueRef<'a, M, C>),

	/// Container value fragment.
	ContainerFragment(&'a Meta<ContainerKind, M>),
}

impl<'a, M, C> FragmentRef<'a, M, C> {
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

	pub fn is_object(&self) -> bool
	where
		C: AnyValue<M>,
	{
		match self {
			Self::Value(v) => v.is_object(),
			_ => false,
		}
	}

	pub fn sub_fragments(&self) -> SubFragments<'a, M, C> {
		match self {
			Self::Value(EntryValueRef::Container(Meta(Nullable::Some(c), _))) => {
				SubFragments::Container(c.sub_fragments(), PhantomData)
			}
			_ => SubFragments::None,
		}
	}
}

pub enum SubFragments<'a, M, C> {
	None,
	Container(container::SubValues<'a, M>, PhantomData<C>),
}

impl<'a, M, C: 'a> Iterator for SubFragments<'a, M, C> {
	type Item = FragmentRef<'a, M, C>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Container(c, _) => c.next().map(FragmentRef::ContainerFragment),
		}
	}
}
