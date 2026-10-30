use crate::{
	JsonLdLenient,
	syntax::{
		CompactIriBuf, JsonLdCompactIri, JsonLdContainerValue, JsonLdContext, JsonLdDirection,
		JsonLdKeyword, JsonLdNullable,
		context::{self, JsonLdContextTerm},
	},
};
use langtag::{LangTag, LangTagBuf};
use rdf_syntax::{BlankId, BlankIdBuf};
use rdf_syntax::{Iri, IriBuf};

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

	pub fn as_expanded(&self) -> ExpandedTermDefinitionRef<'_> {
		match self {
			Self::Simple(term) => ExpandedTermDefinitionRef {
				id: Some(JsonLdNullable::Some(term.as_str().into())),
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

	pub fn as_compact_iri(&self) -> Option<&JsonLdCompactIri> {
		JsonLdCompactIri::new(&self.0).ok()
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
		serde(
			rename = "@id",
			default,
			deserialize_with = "JsonLdNullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub id: Option<JsonLdNullable<TermId>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@type",
			default,
			deserialize_with = "JsonLdNullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub type_: Option<JsonLdNullable<TermType>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@context",
			default,
			deserialize_with = "context::JsonLdContext::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub context: Option<Box<context::JsonLdContext>>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@reverse", default, skip_serializing_if = "Option::is_none")
	)]
	pub reverse: Option<JsonLdContextTerm>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@index", default, skip_serializing_if = "Option::is_none")
	)]
	pub index: Option<JsonLdIndex>,

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
			rename = "@container",
			default,
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub container: Option<JsonLdContainerValue>,

	#[cfg_attr(
		feature = "serde",
		serde(rename = "@nest", default, skip_serializing_if = "Option::is_none")
	)]
	pub nest: Option<JsonLdNest>,

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
		matches!(&self.id, None | Some(JsonLdNullable::Null))
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
		matches!(&self.id, Some(JsonLdNullable::Some(_)))
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

	pub fn simplify(self) -> JsonLdNullable<TermDefinition> {
		if self.is_null() {
			JsonLdNullable::Null
		} else if self.is_simple_definition() {
			JsonLdNullable::Some(TermDefinition::Simple(SimpleTermDefinition(
				self.id.unwrap().unwrap().into_string(),
			)))
		} else {
			JsonLdNullable::Some(TermDefinition::Expanded(Box::new(self)))
		}
	}

	pub fn iter(&self) -> TermDefinitionEntries<'_> {
		TermDefinitionEntries {
			id: self.id.as_ref().map(JsonLdNullable::as_ref),
			type_: self.type_.as_ref().map(JsonLdNullable::as_ref),
			context: self.context.as_deref(),
			reverse: self.reverse.as_ref(),
			index: self.index.as_ref(),
			language: self.language.as_ref().map(JsonLdNullable::as_ref),
			direction: self.direction,
			container: self.container.as_ref(),
			nest: self.nest.as_ref(),
			prefix: self.prefix,
			propagate: self.propagate,
			protected: self.protected,
		}
	}

	pub fn as_expanded_ref(&self) -> ExpandedTermDefinitionRef<'_> {
		ExpandedTermDefinitionRef {
			id: self
				.id
				.as_ref()
				.map(|i| i.as_ref().map(|id| id.as_id_ref())),
			type_: self.type_.as_ref().map(JsonLdNullable::as_ref),
			context: self.context.as_deref(),
			reverse: self.reverse.as_ref(),
			index: self.index.as_ref(),
			language: self
				.language
				.as_ref()
				.map(|n| n.as_ref().map(JsonLdLenient::as_deref)),
			direction: self.direction,
			container: self.container.as_ref(),
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
	pub id: Option<JsonLdNullable<IdRef<'a>>>,
	pub type_: Option<JsonLdNullable<&'a TermType>>,
	pub context: Option<&'a JsonLdContext>,
	pub reverse: Option<&'a JsonLdContextTerm>,
	pub index: Option<&'a JsonLdIndex>,
	pub language: Option<JsonLdNullable<JsonLdLenient<&'a LangTag>>>,
	pub direction: Option<JsonLdNullable<JsonLdDirection>>,
	pub container: Option<&'a JsonLdContainerValue>,
	pub nest: Option<&'a JsonLdNest>,
	pub prefix: Option<bool>,
	pub propagate: Option<bool>,
	pub protected: Option<bool>,
}

impl<'a> From<JsonLdNullable<&'a TermDefinition>> for ExpandedTermDefinitionRef<'a> {
	fn from(value: JsonLdNullable<&'a TermDefinition>) -> Self {
		match value {
			JsonLdNullable::Some(d) => d.as_expanded(),
			JsonLdNullable::Null => Self {
				id: Some(JsonLdNullable::Null),
				..Default::default()
			},
		}
	}
}

/// Term definition entries.
pub struct TermDefinitionEntries<'a> {
	id: Option<JsonLdNullable<&'a TermId>>,
	type_: Option<JsonLdNullable<&'a TermType>>,
	context: Option<&'a context::JsonLdContext>,
	reverse: Option<&'a JsonLdContextTerm>,
	index: Option<&'a JsonLdIndex>,
	language: Option<JsonLdNullable<&'a JsonLdLenient<LangTagBuf>>>,
	direction: Option<JsonLdNullable<JsonLdDirection>>,
	container: Option<&'a JsonLdContainerValue>,
	nest: Option<&'a JsonLdNest>,
	prefix: Option<bool>,
	propagate: Option<bool>,
	protected: Option<bool>,
}

pub enum TermDefinitionEntryRef<'a> {
	Id(JsonLdNullable<&'a TermId>),
	Type(JsonLdNullable<&'a TermType>),
	Context(&'a context::JsonLdContext),
	Reverse(&'a JsonLdContextTerm),
	Index(&'a JsonLdIndex),
	Language(JsonLdNullable<&'a JsonLdLenient<LangTagBuf>>),
	Direction(JsonLdNullable<JsonLdDirection>),
	Container(&'a JsonLdContainerValue),
	Nest(&'a JsonLdNest),
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
			Self::Container(e) => TermDefinitionEntryValueRef::Container(e),
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
				TermDefinitionEntryValueRef::Container(e),
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
	pub fn keyword(&self) -> JsonLdKeyword {
		match self {
			Self::Id => JsonLdKeyword::Id,
			Self::Type => JsonLdKeyword::Type,
			Self::Context => JsonLdKeyword::Context,
			Self::Reverse => JsonLdKeyword::Reverse,
			Self::Index => JsonLdKeyword::Index,
			Self::Language => JsonLdKeyword::Language,
			Self::Direction => JsonLdKeyword::Direction,
			Self::Container => JsonLdKeyword::Container,
			Self::Nest => JsonLdKeyword::Nest,
			Self::Prefix => JsonLdKeyword::Prefix,
			Self::Propagate => JsonLdKeyword::Propagate,
			Self::Protected => JsonLdKeyword::Protected,
		}
	}

	pub fn as_str(&self) -> &'static str {
		self.keyword().into_str()
	}
}

pub enum TermDefinitionEntryValueRef<'a> {
	Id(JsonLdNullable<&'a TermId>),
	Type(JsonLdNullable<&'a TermType>),
	Context(&'a context::JsonLdContext),
	Reverse(&'a JsonLdContextTerm),
	Index(&'a JsonLdIndex),
	Language(JsonLdNullable<&'a JsonLdLenient<LangTagBuf>>),
	Direction(JsonLdNullable<JsonLdDirection>),
	Container(&'a JsonLdContainerValue),
	Nest(&'a JsonLdNest),
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
