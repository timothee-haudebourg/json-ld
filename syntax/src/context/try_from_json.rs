use super::{
	definition,
	term_definition::{self, InvalidNest},
	Context, ContextEntry, Definition, Entry, TermDefinition,
};
use crate::{Container, ErrorCode, Keyword, Nullable, TryFromJson, TryFromStrippedJson};
use iref::IriRefBuf;
use locspan::Meta;

#[derive(Debug, Clone, thiserror::Error)]
pub enum InvalidContext {
	#[error("Invalid IRI reference: {0}")]
	InvalidIriRef(String),

	#[error("Unexpected {0}")]
	Unexpected(json_syntax::Kind, &'static [json_syntax::Kind]),

	#[error("Invalid `@direction`")]
	InvalidDirection,

	#[error("Duplicate key")]
	DuplicateKey,

	#[error("Invalid term definition")]
	InvalidTermDefinition,

	#[error("Invalid `@nest` value `{0}`")]
	InvalidNestValue(String),
}

impl InvalidContext {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::InvalidIriRef(_) => ErrorCode::InvalidIriMapping,
			Self::Unexpected(_, _) => ErrorCode::InvalidContextEntry,
			Self::InvalidDirection => ErrorCode::InvalidBaseDirection,
			Self::DuplicateKey => ErrorCode::DuplicateKey,
			Self::InvalidTermDefinition => ErrorCode::InvalidTermDefinition,
			Self::InvalidNestValue(_) => ErrorCode::InvalidNestValue,
		}
	}
}

impl From<crate::Unexpected> for InvalidContext {
	fn from(crate::Unexpected(u, e): crate::Unexpected) -> Self {
		Self::Unexpected(u, e)
	}
}

impl<M: Clone> TryFromJson<M> for TermDefinition<M> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => Ok(Meta(
				Self::Simple(term_definition::Simple(s.to_string())),
				meta,
			)),
			json_syntax::Value::Object(o) => {
				let mut def = term_definition::Expanded::new();

				for json_syntax::object::Entry {
					key: Meta(key, key_metadata),
					value,
				} in o
				{
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Id) => {
							def.id = Some(Entry::new_with(
								key_metadata,
								Nullable::try_from_json(value)?,
							))
						}
						Ok(Keyword::Type) => {
							def.type_ = Some(Entry::new_with(
								key_metadata,
								Nullable::try_from_json(value)?,
							))
						}
						Ok(Keyword::Context) => {
							def.context = Some(Entry::new_with(
								key_metadata,
								Context::try_from_json(value)?.map(Box::new),
							))
						}
						Ok(Keyword::Reverse) => {
							def.reverse = Some(Entry::new_with(
								key_metadata,
								definition::Key::try_from_json(value)?,
							))
						}
						Ok(Keyword::Index) => {
							def.index = Some(Entry::new_with(
								key_metadata,
								term_definition::Index::try_from_json(value)?,
							))
						}
						Ok(Keyword::Language) => {
							def.language = Some(Entry::new_with(
								key_metadata,
								Nullable::try_from_json(value)?,
							))
						}
						Ok(Keyword::Direction) => {
							def.direction = Some(Entry::new_with(
								key_metadata,
								Nullable::try_from_json(value)?,
							))
						}
						Ok(Keyword::Container) => {
							let container = match value {
								Meta(json_syntax::Value::Null, meta) => Meta(Nullable::Null, meta),
								other => {
									let Meta(container, meta) = Container::try_from_json(other)?;
									Meta(Nullable::Some(container), meta)
								}
							};

							def.container = Some(Entry::new_with(key_metadata, container))
						}
						Ok(Keyword::Nest) => {
							def.nest = Some(Entry::new_with(
								key_metadata,
								term_definition::Nest::try_from_json(value)?,
							))
						}
						Ok(Keyword::Prefix) => {
							def.prefix = Some(Entry::new_with(
								key_metadata,
								bool::try_from_json(value).map_err(Meta::cast)?,
							))
						}
						Ok(Keyword::Propagate) => {
							def.propagate = Some(Entry::new_with(
								key_metadata,
								bool::try_from_json(value).map_err(Meta::cast)?,
							))
						}
						Ok(Keyword::Protected) => {
							def.protected = Some(Entry::new_with(
								key_metadata,
								bool::try_from_json(value).map_err(Meta::cast)?,
							))
						}
						_ => return Err(Meta(InvalidContext::InvalidTermDefinition, key_metadata)),
					}
				}

				Ok(Meta(Self::Expanded(Box::new(def)), meta))
			}
			unexpected => Err(Meta(
				InvalidContext::Unexpected(
					unexpected.kind(),
					&[json_syntax::Kind::String, json_syntax::Kind::Object],
				),
				meta,
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for term_definition::Type {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromJson<M> for definition::TypeContainer {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => match Keyword::try_from(s.as_str()) {
				Ok(Keyword::Set) => Ok(Meta(Self::Set, meta)),
				_ => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for definition::Type<M> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Object(o) => {
				let mut container = None;
				let mut protected = None;

				for json_syntax::object::Entry {
					key: Meta(key, key_metadata),
					value,
				} in o
				{
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Container) => {
							if let Some(prev) = container.replace(Entry::new_with(
								key_metadata,
								definition::TypeContainer::try_from_json(value)?,
							)) {
								return Err(Meta(InvalidContext::DuplicateKey, prev.key_metadata));
							}
						}
						Ok(Keyword::Protected) => {
							if let Some(prev) = protected.replace(Entry::new_with(
								key_metadata,
								bool::try_from_json(value).map_err(Meta::cast)?,
							)) {
								return Err(Meta(InvalidContext::DuplicateKey, prev.key_metadata));
							}
						}
						_ => return Err(Meta(InvalidContext::InvalidTermDefinition, key_metadata)),
					}
				}

				match container {
					Some(container) => Ok(Meta(
						Self {
							container,
							protected,
						},
						meta,
					)),
					None => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
				}
			}
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::Object]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for definition::Version {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Number(n) => match n.as_str() {
				"1.1" => Ok(Meta(Self::V1_1, meta)),
				_ => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::Number]),
				meta,
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for definition::Vocab {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for term_definition::Id {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromJson<M> for definition::Key {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>> {
		match value {
			json_syntax::Value::String(s) => Ok(Meta(Self::from(s.into_string()), meta)),
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for term_definition::Index {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => Ok(Meta(Self::from(s.into_string()), meta)),
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromJson<M> for term_definition::Nest {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => match Self::try_from(s.into_string()) {
				Ok(nest) => Ok(Meta(nest, meta)),
				Err(InvalidNest(s)) => Err(Meta(InvalidContext::InvalidNestValue(s), meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M: Clone> TryFromJson<M> for Context<M> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Array(a) => {
				let mut many = Vec::with_capacity(a.len());

				for item in a {
					many.push(ContextEntry::try_from_json(item)?)
				}

				Ok(Meta(Self::Many(many), meta))
			}
			context => Ok(Meta(
				Self::One(ContextEntry::try_from_json(Meta(context, meta.clone()))?),
				meta,
			)),
		}
	}
}

impl<M: Clone> TryFromJson<M> for ContextEntry<M> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Null => Ok(Meta(Self::Null, meta)),
			json_syntax::Value::String(s) => match IriRefBuf::new(s.into_string()) {
				Ok(iri_ref) => Ok(Meta(Self::IriRef(iri_ref), meta)),
				Err(e) => Err(Meta(InvalidContext::InvalidIriRef(e.0), meta)),
			},
			json_syntax::Value::Object(o) => {
				let mut def = Definition::new();

				for json_syntax::object::Entry {
					key: Meta(key, key_metadata),
					value,
				} in o
				{
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Base) => {
							def.base = Some(Entry::new_with(
								key_metadata,
								Nullable::try_from_json(value)?,
							))
						}
						Ok(Keyword::Import) => {
							def.import = Some(Entry::new_with(
								key_metadata,
								IriRefBuf::try_from_json(value)?,
							))
						}
						Ok(Keyword::Language) => {
							def.language = Some(Entry::new_with(
								key_metadata,
								Nullable::try_from_json(value)?,
							))
						}
						Ok(Keyword::Direction) => {
							def.direction = Some(Entry::new_with(
								key_metadata,
								Nullable::try_from_json(value)?,
							))
						}
						Ok(Keyword::Propagate) => {
							def.propagate = Some(Entry::new_with(
								key_metadata,
								bool::try_from_json(value).map_err(Meta::cast)?,
							))
						}
						Ok(Keyword::Protected) => {
							def.protected = Some(Entry::new_with(
								key_metadata,
								bool::try_from_json(value).map_err(Meta::cast)?,
							))
						}
						Ok(Keyword::Type) => {
							def.type_ = Some(Entry::new_with(
								key_metadata,
								definition::Type::try_from_json(value)?,
							))
						}
						Ok(Keyword::Version) => {
							def.version = Some(Entry::new_with(
								key_metadata,
								definition::Version::try_from_json(value)?,
							))
						}
						Ok(Keyword::Vocab) => {
							def.vocab = Some(Entry::new_with(
								key_metadata,
								Nullable::try_from_json(value)?,
							))
						}
						_ => {
							let term_def = match value {
								Meta(json_syntax::Value::Null, meta) => Meta(Nullable::Null, meta),
								other => TermDefinition::try_from_json(other)?.map(Nullable::Some),
							};

							if let Some(binding) = def
								.bindings
								.insert_with(Meta(key.into(), key_metadata), term_def)
							{
								return Err(Meta(
									InvalidContext::DuplicateKey,
									binding.key_metadata,
								));
							}
						}
					}
				}

				Ok(Meta(Self::Definition(def), meta))
			}
			unexpected => Err(Meta(
				InvalidContext::Unexpected(
					unexpected.kind(),
					&[
						json_syntax::Kind::Null,
						json_syntax::Kind::String,
						json_syntax::Kind::Object,
					],
				),
				meta,
			)),
		}
	}
}
