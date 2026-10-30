use crate::{
	JsonLdLenient,
	syntax::{
		JsonLdCompactIri, JsonLdCompactIriBuf, JsonLdContainerValue, JsonLdContext,
		JsonLdDirection, JsonLdKeyword, JsonLdNullable,
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
pub enum JsonLdTermDefinition {
	Simple(JsonLdSimpleTermDefinition),
	Expanded(Box<JsonLdExpandedTermDefinition>),
}

impl JsonLdTermDefinition {
	pub fn is_expanded(&self) -> bool {
		matches!(self, Self::Expanded(_))
	}

	pub fn is_object(&self) -> bool {
		self.is_expanded()
	}

	pub fn as_expanded(&self) -> JsonLdExpandedTermDefinitionRef<'_> {
		match self {
			Self::Simple(term) => JsonLdExpandedTermDefinitionRef {
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
pub struct JsonLdSimpleTermDefinition(pub String);

impl JsonLdSimpleTermDefinition {
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

impl From<IriBuf> for JsonLdSimpleTermDefinition {
	fn from(value: IriBuf) -> Self {
		Self(value.into_string())
	}
}

impl From<JsonLdCompactIriBuf> for JsonLdSimpleTermDefinition {
	fn from(value: JsonLdCompactIriBuf) -> Self {
		Self(value.into_string())
	}
}

impl From<BlankIdBuf> for JsonLdSimpleTermDefinition {
	fn from(value: BlankIdBuf) -> Self {
		Self(value.to_string())
	}
}

/// Expanded term definition.
#[derive(Default, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonLdExpandedTermDefinition {
	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@id",
			default,
			deserialize_with = "JsonLdNullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub id: Option<JsonLdNullable<JsonLdTermId>>,

	#[cfg_attr(
		feature = "serde",
		serde(
			rename = "@type",
			default,
			deserialize_with = "JsonLdNullable::optional",
			skip_serializing_if = "Option::is_none"
		)
	)]
	pub type_: Option<JsonLdNullable<JsonLdTermType>>,

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

impl JsonLdExpandedTermDefinition {
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

	pub fn simplify(self) -> JsonLdNullable<JsonLdTermDefinition> {
		if self.is_null() {
			JsonLdNullable::Null
		} else if self.is_simple_definition() {
			JsonLdNullable::Some(JsonLdTermDefinition::Simple(JsonLdSimpleTermDefinition(
				self.id.unwrap().unwrap().into_string(),
			)))
		} else {
			JsonLdNullable::Some(JsonLdTermDefinition::Expanded(Box::new(self)))
		}
	}

	pub fn iter(&self) -> JsonLdTermDefinitionEntries<'_> {
		JsonLdTermDefinitionEntries {
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

	pub fn as_expanded_ref(&self) -> JsonLdExpandedTermDefinitionRef<'_> {
		JsonLdExpandedTermDefinitionRef {
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
pub struct JsonLdExpandedTermDefinitionRef<'a> {
	pub id: Option<JsonLdNullable<JsonLdIdRef<'a>>>,
	pub type_: Option<JsonLdNullable<&'a JsonLdTermType>>,
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

impl<'a> From<JsonLdNullable<&'a JsonLdTermDefinition>> for JsonLdExpandedTermDefinitionRef<'a> {
	fn from(value: JsonLdNullable<&'a JsonLdTermDefinition>) -> Self {
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
pub struct JsonLdTermDefinitionEntries<'a> {
	id: Option<JsonLdNullable<&'a JsonLdTermId>>,
	type_: Option<JsonLdNullable<&'a JsonLdTermType>>,
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

pub enum JsonLdTermDefinitionEntryRef<'a> {
	Id(JsonLdNullable<&'a JsonLdTermId>),
	Type(JsonLdNullable<&'a JsonLdTermType>),
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

impl<'a> JsonLdTermDefinitionEntryRef<'a> {
	pub fn into_key(self) -> JsonLdTermDefinitionEntryKey {
		match self {
			Self::Id(_) => JsonLdTermDefinitionEntryKey::Id,
			Self::Type(_) => JsonLdTermDefinitionEntryKey::Type,
			Self::Context(_) => JsonLdTermDefinitionEntryKey::Context,
			Self::Reverse(_) => JsonLdTermDefinitionEntryKey::Reverse,
			Self::Index(_) => JsonLdTermDefinitionEntryKey::Index,
			Self::Language(_) => JsonLdTermDefinitionEntryKey::Language,
			Self::Direction(_) => JsonLdTermDefinitionEntryKey::Direction,
			Self::Container(_) => JsonLdTermDefinitionEntryKey::Container,
			Self::Nest(_) => JsonLdTermDefinitionEntryKey::Nest,
			Self::Prefix(_) => JsonLdTermDefinitionEntryKey::Prefix,
			Self::Propagate(_) => JsonLdTermDefinitionEntryKey::Propagate,
			Self::Protected(_) => JsonLdTermDefinitionEntryKey::Protected,
		}
	}

	pub fn key(&self) -> JsonLdTermDefinitionEntryKey {
		match self {
			Self::Id(_) => JsonLdTermDefinitionEntryKey::Id,
			Self::Type(_) => JsonLdTermDefinitionEntryKey::Type,
			Self::Context(_) => JsonLdTermDefinitionEntryKey::Context,
			Self::Reverse(_) => JsonLdTermDefinitionEntryKey::Reverse,
			Self::Index(_) => JsonLdTermDefinitionEntryKey::Index,
			Self::Language(_) => JsonLdTermDefinitionEntryKey::Language,
			Self::Direction(_) => JsonLdTermDefinitionEntryKey::Direction,
			Self::Container(_) => JsonLdTermDefinitionEntryKey::Container,
			Self::Nest(_) => JsonLdTermDefinitionEntryKey::Nest,
			Self::Prefix(_) => JsonLdTermDefinitionEntryKey::Prefix,
			Self::Propagate(_) => JsonLdTermDefinitionEntryKey::Propagate,
			Self::Protected(_) => JsonLdTermDefinitionEntryKey::Protected,
		}
	}

	pub fn into_value(self) -> JsonLdTermDefinitionEntryValueRef<'a> {
		self.value()
	}

	pub fn value(&self) -> JsonLdTermDefinitionEntryValueRef<'a> {
		match self {
			Self::Id(e) => JsonLdTermDefinitionEntryValueRef::Id(*e),
			Self::Type(e) => JsonLdTermDefinitionEntryValueRef::Type(*e),
			Self::Context(e) => JsonLdTermDefinitionEntryValueRef::Context(e),
			Self::Reverse(e) => JsonLdTermDefinitionEntryValueRef::Reverse(e),
			Self::Index(e) => JsonLdTermDefinitionEntryValueRef::Index(e),
			Self::Language(e) => JsonLdTermDefinitionEntryValueRef::Language(*e),
			Self::Direction(e) => JsonLdTermDefinitionEntryValueRef::Direction(*e),
			Self::Container(e) => JsonLdTermDefinitionEntryValueRef::Container(e),
			Self::Nest(e) => JsonLdTermDefinitionEntryValueRef::Nest(e),
			Self::Prefix(e) => JsonLdTermDefinitionEntryValueRef::Prefix(*e),
			Self::Propagate(e) => JsonLdTermDefinitionEntryValueRef::Propagate(*e),
			Self::Protected(e) => JsonLdTermDefinitionEntryValueRef::Protected(*e),
		}
	}

	pub fn into_key_value(
		self,
	) -> (
		JsonLdTermDefinitionEntryKey,
		JsonLdTermDefinitionEntryValueRef<'a>,
	) {
		self.key_value()
	}

	pub fn key_value(
		&self,
	) -> (
		JsonLdTermDefinitionEntryKey,
		JsonLdTermDefinitionEntryValueRef<'a>,
	) {
		match self {
			Self::Id(e) => (
				JsonLdTermDefinitionEntryKey::Id,
				JsonLdTermDefinitionEntryValueRef::Id(*e),
			),
			Self::Type(e) => (
				JsonLdTermDefinitionEntryKey::Type,
				JsonLdTermDefinitionEntryValueRef::Type(*e),
			),
			Self::Context(e) => (
				JsonLdTermDefinitionEntryKey::Context,
				JsonLdTermDefinitionEntryValueRef::Context(e),
			),
			Self::Reverse(e) => (
				JsonLdTermDefinitionEntryKey::Reverse,
				JsonLdTermDefinitionEntryValueRef::Reverse(e),
			),
			Self::Index(e) => (
				JsonLdTermDefinitionEntryKey::Index,
				JsonLdTermDefinitionEntryValueRef::Index(e),
			),
			Self::Language(e) => (
				JsonLdTermDefinitionEntryKey::Language,
				JsonLdTermDefinitionEntryValueRef::Language(*e),
			),
			Self::Direction(e) => (
				JsonLdTermDefinitionEntryKey::Direction,
				JsonLdTermDefinitionEntryValueRef::Direction(*e),
			),
			Self::Container(e) => (
				JsonLdTermDefinitionEntryKey::Container,
				JsonLdTermDefinitionEntryValueRef::Container(e),
			),
			Self::Nest(e) => (
				JsonLdTermDefinitionEntryKey::Nest,
				JsonLdTermDefinitionEntryValueRef::Nest(e),
			),
			Self::Prefix(e) => (
				JsonLdTermDefinitionEntryKey::Prefix,
				JsonLdTermDefinitionEntryValueRef::Prefix(*e),
			),
			Self::Propagate(e) => (
				JsonLdTermDefinitionEntryKey::Propagate,
				JsonLdTermDefinitionEntryValueRef::Propagate(*e),
			),
			Self::Protected(e) => (
				JsonLdTermDefinitionEntryKey::Protected,
				JsonLdTermDefinitionEntryValueRef::Protected(*e),
			),
		}
	}
}

pub enum JsonLdTermDefinitionEntryKey {
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

impl JsonLdTermDefinitionEntryKey {
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

pub enum JsonLdTermDefinitionEntryValueRef<'a> {
	Id(JsonLdNullable<&'a JsonLdTermId>),
	Type(JsonLdNullable<&'a JsonLdTermType>),
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

impl<'a> Iterator for JsonLdTermDefinitionEntries<'a> {
	type Item = JsonLdTermDefinitionEntryRef<'a>;

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
			Some(value) => Some(JsonLdTermDefinitionEntryRef::Id(value)),
			None => match self.type_.take() {
				Some(value) => Some(JsonLdTermDefinitionEntryRef::Type(value)),
				None => match self.context.take() {
					Some(value) => Some(JsonLdTermDefinitionEntryRef::Context(value)),
					None => match self.reverse.take() {
						Some(value) => Some(JsonLdTermDefinitionEntryRef::Reverse(value)),
						None => match self.index.take() {
							Some(value) => Some(JsonLdTermDefinitionEntryRef::Index(value)),
							None => match self.language.take() {
								Some(value) => Some(JsonLdTermDefinitionEntryRef::Language(value)),
								None => match self.direction.take() {
									Some(value) => {
										Some(JsonLdTermDefinitionEntryRef::Direction(value))
									}
									None => match self.container.take() {
										Some(value) => {
											Some(JsonLdTermDefinitionEntryRef::Container(value))
										}
										None => match self.nest.take() {
											Some(value) => {
												Some(JsonLdTermDefinitionEntryRef::Nest(value))
											}
											None => match self.prefix.take() {
												Some(value) => Some(
													JsonLdTermDefinitionEntryRef::Prefix(value),
												),
												None => match self.propagate.take() {
													Some(value) => Some(
														JsonLdTermDefinitionEntryRef::Propagate(
															value,
														),
													),
													None => self.protected.take().map(
														JsonLdTermDefinitionEntryRef::Protected,
													),
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

impl<'a> ExactSizeIterator for JsonLdTermDefinitionEntries<'a> {}
