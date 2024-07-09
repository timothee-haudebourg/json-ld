use crate::{
	container, context, CompactIri, CompactIriBuf, Container, ContainerKind, Direction, Keyword,
	LenientLangTag, LenientLangTagBuf, Nullable,
};
use educe::Educe;
use iref::{Iri, IriBuf};
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
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum TermDefinition {
	Simple(Simple),
	Expanded(Box<Expanded>),
}

impl TermDefinition {
	pub fn is_expanded(&self) -> bool {
		matches!(self, Self::Expanded(_))
	}

	pub fn is_object(&self) -> bool {
		self.is_expanded()
	}

	pub fn as_expanded(&self) -> ExpandedRef {
		match self {
			Self::Simple(term) => ExpandedRef {
				id: Some(Nullable::Some(term.as_str().into())),
				..Default::default()
			},
			Self::Expanded(e) => e.as_expanded_ref(),
		}
	}
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
pub struct Simple(pub(crate) String);

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
#[derive(PartialEq, Eq, Clone, Educe, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[educe(Default)]
pub struct Expanded {
	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@id",
			default,
			deserialize_with = "Nullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub id: Option<Nullable<Id>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@type",
			default,
			deserialize_with = "Nullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub type_: Option<Nullable<Type>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@context", default, skip_serializing_if = "Option::is_none")
	)]
	pub context: Option<Box<context::Context>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")
	)]
	pub reverse: Option<context::definition::Key>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@index", default, skip_serializing_if = "Option::is_none")
	)]
	pub index: Option<Index>,

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
			rename = "@container",
			default,
			deserialize_with = "Nullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub container: Option<Nullable<Container>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@nest", default, skip_serializing_if = "Option::is_none")
	)]
	pub nest: Option<Nest>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@prefix", default, skip_serializing_if = "Option::is_none")
	)]
	pub prefix: Option<bool>,

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
}

impl Expanded {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn is_null(&self) -> bool {
		matches!(&self.id, None | Some(Nullable::Null))
			&& self.type_.is_none()
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
		matches!(&self.id, Some(Nullable::Some(_)))
			&& self.type_.is_none()
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

	pub fn simplify(self) -> Nullable<TermDefinition> {
		if self.is_null() {
			Nullable::Null
		} else if self.is_simple_definition() {
			Nullable::Some(TermDefinition::Simple(Simple(
				self.id.unwrap().unwrap().into_string(),
			)))
		} else {
			Nullable::Some(TermDefinition::Expanded(Box::new(self)))
		}
	}

	pub fn iter(&self) -> Entries {
		Entries {
			id: self.id.as_ref().map(Nullable::as_ref),
			type_: self.type_.as_ref().map(Nullable::as_ref),
			context: self.context.as_deref(),
			reverse: self.reverse.as_ref(),
			index: self.index.as_ref(),
			language: self.language.as_ref().map(Nullable::as_ref),
			direction: self.direction,
			container: self.container.as_ref().map(Nullable::as_ref),
			nest: self.nest.as_ref(),
			prefix: self.prefix,
			propagate: self.propagate,
			protected: self.protected,
		}
	}

	pub fn as_expanded_ref(&self) -> ExpandedRef {
		ExpandedRef {
			id: self
				.id
				.as_ref()
				.map(|i| i.as_ref().map(|id| id.as_id_ref())),
			type_: self.type_.as_ref().map(Nullable::as_ref),
			context: self.context.as_deref(),
			reverse: self.reverse.as_ref(),
			index: self.index.as_ref(),
			language: self
				.language
				.as_ref()
				.map(|n| n.as_ref().map(LenientLangTagBuf::as_lenient_lang_tag_ref)),
			direction: self.direction,
			container: self.container.as_ref().map(Nullable::as_ref),
			nest: self.nest.as_ref(),
			prefix: self.prefix,
			propagate: self.propagate,
			protected: self.protected,
		}
	}
}

/// Expanded term definition.
#[derive(Debug, Educe)]
#[educe(Default)]
pub struct ExpandedRef<'a> {
	pub id: Option<Nullable<IdRef<'a>>>,
	pub type_: Option<Nullable<&'a Type>>,
	pub context: Option<&'a context::Context>,
	pub reverse: Option<&'a context::definition::Key>,
	pub index: Option<&'a Index>,
	pub language: Option<Nullable<&'a LenientLangTag>>,
	pub direction: Option<Nullable<Direction>>,
	pub container: Option<Nullable<&'a Container>>,
	pub nest: Option<&'a Nest>,
	pub prefix: Option<bool>,
	pub propagate: Option<bool>,
	pub protected: Option<bool>,
}

impl<'a> From<Nullable<&'a TermDefinition>> for ExpandedRef<'a> {
	fn from(value: Nullable<&'a TermDefinition>) -> Self {
		match value {
			Nullable::Some(d) => d.as_expanded(),
			Nullable::Null => Self {
				id: Some(Nullable::Null),
				..Default::default()
			},
		}
	}
}

/// Term definition entries.
pub struct Entries<'a> {
	id: Option<Nullable<&'a Id>>,
	type_: Option<Nullable<&'a Type>>,
	context: Option<&'a context::Context>,
	reverse: Option<&'a context::definition::Key>,
	index: Option<&'a Index>,
	language: Option<Nullable<&'a LenientLangTagBuf>>,
	direction: Option<Nullable<Direction>>,
	container: Option<Nullable<&'a Container>>,
	nest: Option<&'a Nest>,
	prefix: Option<bool>,
	propagate: Option<bool>,
	protected: Option<bool>,
}

pub enum EntryRef<'a> {
	Id(Nullable<&'a Id>),
	Type(Nullable<&'a Type>),
	Context(&'a context::Context),
	Reverse(&'a context::definition::Key),
	Index(&'a Index),
	Language(Nullable<&'a LenientLangTagBuf>),
	Direction(Nullable<Direction>),
	Container(Nullable<&'a Container>),
	Nest(&'a Nest),
	Prefix(bool),
	Propagate(bool),
	Protected(bool),
}

impl<'a> EntryRef<'a> {
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

	pub fn into_value(self) -> EntryValueRef<'a> {
		self.value()
	}

	pub fn value(&self) -> EntryValueRef<'a> {
		match self {
			Self::Id(e) => EntryValueRef::Id(*e),
			Self::Type(e) => EntryValueRef::Type(*e),
			Self::Context(e) => EntryValueRef::Context(e),
			Self::Reverse(e) => EntryValueRef::Reverse(e),
			Self::Index(e) => EntryValueRef::Index(e),
			Self::Language(e) => EntryValueRef::Language(*e),
			Self::Direction(e) => EntryValueRef::Direction(*e),
			Self::Container(e) => EntryValueRef::Container(*e),
			Self::Nest(e) => EntryValueRef::Nest(e),
			Self::Prefix(e) => EntryValueRef::Prefix(*e),
			Self::Propagate(e) => EntryValueRef::Propagate(*e),
			Self::Protected(e) => EntryValueRef::Protected(*e),
		}
	}

	pub fn into_key_value(self) -> (EntryKey, EntryValueRef<'a>) {
		self.key_value()
	}

	pub fn key_value(&self) -> (EntryKey, EntryValueRef<'a>) {
		match self {
			Self::Id(e) => (EntryKey::Id, EntryValueRef::Id(*e)),
			Self::Type(e) => (EntryKey::Type, EntryValueRef::Type(*e)),
			Self::Context(e) => (EntryKey::Context, EntryValueRef::Context(e)),
			Self::Reverse(e) => (EntryKey::Reverse, EntryValueRef::Reverse(e)),
			Self::Index(e) => (EntryKey::Index, EntryValueRef::Index(e)),
			Self::Language(e) => (EntryKey::Language, EntryValueRef::Language(*e)),
			Self::Direction(e) => (EntryKey::Direction, EntryValueRef::Direction(*e)),
			Self::Container(e) => (EntryKey::Container, EntryValueRef::Container(*e)),
			Self::Nest(e) => (EntryKey::Nest, EntryValueRef::Nest(e)),
			Self::Prefix(e) => (EntryKey::Prefix, EntryValueRef::Prefix(*e)),
			Self::Propagate(e) => (EntryKey::Propagate, EntryValueRef::Propagate(*e)),
			Self::Protected(e) => (EntryKey::Protected, EntryValueRef::Protected(*e)),
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

pub enum EntryValueRef<'a> {
	Id(Nullable<&'a Id>),
	Type(Nullable<&'a Type>),
	Context(&'a context::Context),
	Reverse(&'a context::definition::Key),
	Index(&'a Index),
	Language(Nullable<&'a LenientLangTagBuf>),
	Direction(Nullable<Direction>),
	Container(Nullable<&'a Container>),
	Nest(&'a Nest),
	Prefix(bool),
	Propagate(bool),
	Protected(bool),
}

impl<'a> EntryValueRef<'a> {
	pub fn is_object(&self) -> bool {
		match self {
			Self::Context(c) => c.is_object(),
			_ => false,
		}
	}

	pub fn is_array(&self) -> bool {
		match self {
			Self::Container(Nullable::Some(c)) => c.is_array(),
			_ => false,
		}
	}
}

impl<'a> Iterator for Entries<'a> {
	type Item = EntryRef<'a>;

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

impl<'a> ExactSizeIterator for Entries<'a> {}

/// Term definition fragment.
pub enum FragmentRef<'a> {
	/// Term definition entry.
	Entry(EntryRef<'a>),

	/// Term definition entry key.
	Key(EntryKey),

	/// Term definition entry value.
	Value(EntryValueRef<'a>),

	/// Container value fragment.
	ContainerFragment(&'a ContainerKind),
}

impl<'a> FragmentRef<'a> {
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

	pub fn sub_fragments(&self) -> SubFragments<'a> {
		match self {
			Self::Value(EntryValueRef::Container(Nullable::Some(c))) => {
				SubFragments::Container(c.sub_fragments())
			}
			_ => SubFragments::None,
		}
	}
}

pub enum SubFragments<'a> {
	None,
	Container(container::SubValues<'a>),
}

impl<'a> Iterator for SubFragments<'a> {
	type Item = FragmentRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Container(c) => c.next().map(FragmentRef::ContainerFragment),
		}
	}
}
