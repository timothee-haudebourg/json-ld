use crate::syntax::{
	context::{self, ContextTerm},
	CompactIri, CompactIriBuf, Container, Context, Direction, Keyword, LenientLangTag,
	LenientLangTagBuf, Nullable,
};
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
	Simple(SimpleTermDefinition),
	Expanded(Box<ExpandedTermDefinition>),
}

impl TermDefinition {
	pub fn is_expanded(&self) -> bool {
		matches!(self, Self::Expanded(_))
	}

	pub fn is_object(&self) -> bool {
		self.is_expanded()
	}

	pub fn as_expanded(&self) -> ExpandedTermDefinitionRef {
		match self {
			Self::Simple(term) => ExpandedTermDefinitionRef {
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
pub struct SimpleTermDefinition(pub String);

impl SimpleTermDefinition {
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

impl From<IriBuf> for SimpleTermDefinition {
	fn from(value: IriBuf) -> Self {
		Self(value.into_string())
	}
}

impl From<CompactIriBuf> for SimpleTermDefinition {
	fn from(value: CompactIriBuf) -> Self {
		Self(value.into_string())
	}
}

impl From<BlankIdBuf> for SimpleTermDefinition {
	fn from(value: BlankIdBuf) -> Self {
		Self(value.to_string())
	}
}

/// Expanded term definition.
#[derive(Default, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpandedTermDefinition {
	#[cfg_attr(
		feature = "serde",
		serde(rename = "@id", default, skip_serializing_if = "Option::is_none")
	)]
	pub id: Option<Nullable<TermId>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@type", default, skip_serializing_if = "Option::is_none")
	)]
	pub type_: Option<Nullable<TermType>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@context", default, skip_serializing_if = "Option::is_none")
	)]
	pub context: Option<Box<context::Context>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")
	)]
	pub reverse: Option<ContextTerm>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@index", default, skip_serializing_if = "Option::is_none")
	)]
	pub index: Option<Index>,

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
			rename = "@container",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub container: Option<Container>,

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

impl ExpandedTermDefinition {
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
			Nullable::Some(TermDefinition::Simple(SimpleTermDefinition(
				self.id.unwrap().unwrap().into_string(),
			)))
		} else {
			Nullable::Some(TermDefinition::Expanded(Box::new(self)))
		}
	}

	pub fn iter(&self) -> TermDefinitionEntries {
		TermDefinitionEntries {
			id: self.id.as_ref().map(Nullable::as_ref),
			type_: self.type_.as_ref().map(Nullable::as_ref),
			context: self.context.as_deref(),
			reverse: self.reverse.as_ref(),
			index: self.index.as_ref(),
			language: self.language.as_ref().map(Nullable::as_ref),
			direction: self.direction,
			container: self.container,
			nest: self.nest.as_ref(),
			prefix: self.prefix,
			propagate: self.propagate,
			protected: self.protected,
		}
	}

	pub fn as_expanded_ref(&self) -> ExpandedTermDefinitionRef {
		ExpandedTermDefinitionRef {
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
			container: self.container,
			nest: self.nest.as_ref(),
			prefix: self.prefix,
			propagate: self.propagate,
			protected: self.protected,
		}
	}
}

/// Expanded term definition.
#[derive(Default, Debug)]
pub struct ExpandedTermDefinitionRef<'a> {
	pub id: Option<Nullable<IdRef<'a>>>,
	pub type_: Option<Nullable<&'a TermType>>,
	pub context: Option<&'a Context>,
	pub reverse: Option<&'a ContextTerm>,
	pub index: Option<&'a Index>,
	pub language: Option<Nullable<&'a LenientLangTag>>,
	pub direction: Option<Nullable<Direction>>,
	pub container: Option<Container>,
	pub nest: Option<&'a Nest>,
	pub prefix: Option<bool>,
	pub propagate: Option<bool>,
	pub protected: Option<bool>,
}

impl<'a> From<Nullable<&'a TermDefinition>> for ExpandedTermDefinitionRef<'a> {
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
pub struct TermDefinitionEntries<'a> {
	id: Option<Nullable<&'a TermId>>,
	type_: Option<Nullable<&'a TermType>>,
	context: Option<&'a context::Context>,
	reverse: Option<&'a ContextTerm>,
	index: Option<&'a Index>,
	language: Option<Nullable<&'a LenientLangTagBuf>>,
	direction: Option<Nullable<Direction>>,
	container: Option<Container>,
	nest: Option<&'a Nest>,
	prefix: Option<bool>,
	propagate: Option<bool>,
	protected: Option<bool>,
}

pub enum TermDefinitionEntryRef<'a> {
	Id(Nullable<&'a TermId>),
	Type(Nullable<&'a TermType>),
	Context(&'a context::Context),
	Reverse(&'a ContextTerm),
	Index(&'a Index),
	Language(Nullable<&'a LenientLangTagBuf>),
	Direction(Nullable<Direction>),
	Container(Container),
	Nest(&'a Nest),
	Prefix(bool),
	Propagate(bool),
	Protected(bool),
}

impl<'a> TermDefinitionEntryRef<'a> {
	pub fn into_key(self) -> TermDefinitionEntryKey {
		match self {
			Self::Id(_) => TermDefinitionEntryKey::Id,
			Self::Type(_) => TermDefinitionEntryKey::Type,
			Self::Context(_) => TermDefinitionEntryKey::Context,
			Self::Reverse(_) => TermDefinitionEntryKey::Reverse,
			Self::Index(_) => TermDefinitionEntryKey::Index,
			Self::Language(_) => TermDefinitionEntryKey::Language,
			Self::Direction(_) => TermDefinitionEntryKey::Direction,
			Self::Container(_) => TermDefinitionEntryKey::Container,
			Self::Nest(_) => TermDefinitionEntryKey::Nest,
			Self::Prefix(_) => TermDefinitionEntryKey::Prefix,
			Self::Propagate(_) => TermDefinitionEntryKey::Propagate,
			Self::Protected(_) => TermDefinitionEntryKey::Protected,
		}
	}

	pub fn key(&self) -> TermDefinitionEntryKey {
		match self {
			Self::Id(_) => TermDefinitionEntryKey::Id,
			Self::Type(_) => TermDefinitionEntryKey::Type,
			Self::Context(_) => TermDefinitionEntryKey::Context,
			Self::Reverse(_) => TermDefinitionEntryKey::Reverse,
			Self::Index(_) => TermDefinitionEntryKey::Index,
			Self::Language(_) => TermDefinitionEntryKey::Language,
			Self::Direction(_) => TermDefinitionEntryKey::Direction,
			Self::Container(_) => TermDefinitionEntryKey::Container,
			Self::Nest(_) => TermDefinitionEntryKey::Nest,
			Self::Prefix(_) => TermDefinitionEntryKey::Prefix,
			Self::Propagate(_) => TermDefinitionEntryKey::Propagate,
			Self::Protected(_) => TermDefinitionEntryKey::Protected,
		}
	}

	pub fn into_value(self) -> TermDefinitionEntryValueRef<'a> {
		self.value()
	}

	pub fn value(&self) -> TermDefinitionEntryValueRef<'a> {
		match self {
			Self::Id(e) => TermDefinitionEntryValueRef::Id(*e),
			Self::Type(e) => TermDefinitionEntryValueRef::Type(*e),
			Self::Context(e) => TermDefinitionEntryValueRef::Context(e),
			Self::Reverse(e) => TermDefinitionEntryValueRef::Reverse(e),
			Self::Index(e) => TermDefinitionEntryValueRef::Index(e),
			Self::Language(e) => TermDefinitionEntryValueRef::Language(*e),
			Self::Direction(e) => TermDefinitionEntryValueRef::Direction(*e),
			Self::Container(e) => TermDefinitionEntryValueRef::Container(*e),
			Self::Nest(e) => TermDefinitionEntryValueRef::Nest(e),
			Self::Prefix(e) => TermDefinitionEntryValueRef::Prefix(*e),
			Self::Propagate(e) => TermDefinitionEntryValueRef::Propagate(*e),
			Self::Protected(e) => TermDefinitionEntryValueRef::Protected(*e),
		}
	}

	pub fn into_key_value(self) -> (TermDefinitionEntryKey, TermDefinitionEntryValueRef<'a>) {
		self.key_value()
	}

	pub fn key_value(&self) -> (TermDefinitionEntryKey, TermDefinitionEntryValueRef<'a>) {
		match self {
			Self::Id(e) => (
				TermDefinitionEntryKey::Id,
				TermDefinitionEntryValueRef::Id(*e),
			),
			Self::Type(e) => (
				TermDefinitionEntryKey::Type,
				TermDefinitionEntryValueRef::Type(*e),
			),
			Self::Context(e) => (
				TermDefinitionEntryKey::Context,
				TermDefinitionEntryValueRef::Context(e),
			),
			Self::Reverse(e) => (
				TermDefinitionEntryKey::Reverse,
				TermDefinitionEntryValueRef::Reverse(e),
			),
			Self::Index(e) => (
				TermDefinitionEntryKey::Index,
				TermDefinitionEntryValueRef::Index(e),
			),
			Self::Language(e) => (
				TermDefinitionEntryKey::Language,
				TermDefinitionEntryValueRef::Language(*e),
			),
			Self::Direction(e) => (
				TermDefinitionEntryKey::Direction,
				TermDefinitionEntryValueRef::Direction(*e),
			),
			Self::Container(e) => (
				TermDefinitionEntryKey::Container,
				TermDefinitionEntryValueRef::Container(*e),
			),
			Self::Nest(e) => (
				TermDefinitionEntryKey::Nest,
				TermDefinitionEntryValueRef::Nest(e),
			),
			Self::Prefix(e) => (
				TermDefinitionEntryKey::Prefix,
				TermDefinitionEntryValueRef::Prefix(*e),
			),
			Self::Propagate(e) => (
				TermDefinitionEntryKey::Propagate,
				TermDefinitionEntryValueRef::Propagate(*e),
			),
			Self::Protected(e) => (
				TermDefinitionEntryKey::Protected,
				TermDefinitionEntryValueRef::Protected(*e),
			),
		}
	}
}

pub enum TermDefinitionEntryKey {
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

impl TermDefinitionEntryKey {
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

pub enum TermDefinitionEntryValueRef<'a> {
	Id(Nullable<&'a TermId>),
	Type(Nullable<&'a TermType>),
	Context(&'a context::Context),
	Reverse(&'a ContextTerm),
	Index(&'a Index),
	Language(Nullable<&'a LenientLangTagBuf>),
	Direction(Nullable<Direction>),
	Container(Container),
	Nest(&'a Nest),
	Prefix(bool),
	Propagate(bool),
	Protected(bool),
}

impl<'a> Iterator for TermDefinitionEntries<'a> {
	type Item = TermDefinitionEntryRef<'a>;

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
			Some(value) => Some(TermDefinitionEntryRef::Id(value)),
			None => match self.type_.take() {
				Some(value) => Some(TermDefinitionEntryRef::Type(value)),
				None => match self.context.take() {
					Some(value) => Some(TermDefinitionEntryRef::Context(value)),
					None => match self.reverse.take() {
						Some(value) => Some(TermDefinitionEntryRef::Reverse(value)),
						None => match self.index.take() {
							Some(value) => Some(TermDefinitionEntryRef::Index(value)),
							None => match self.language.take() {
								Some(value) => Some(TermDefinitionEntryRef::Language(value)),
								None => match self.direction.take() {
									Some(value) => Some(TermDefinitionEntryRef::Direction(value)),
									None => match self.container.take() {
										Some(value) => {
											Some(TermDefinitionEntryRef::Container(value))
										}
										None => match self.nest.take() {
											Some(value) => {
												Some(TermDefinitionEntryRef::Nest(value))
											}
											None => match self.prefix.take() {
												Some(value) => {
													Some(TermDefinitionEntryRef::Prefix(value))
												}
												None => match self.propagate.take() {
													Some(value) => Some(
														TermDefinitionEntryRef::Propagate(value),
													),
													None => self
														.protected
														.take()
														.map(TermDefinitionEntryRef::Protected),
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

impl<'a> ExactSizeIterator for TermDefinitionEntries<'a> {}
