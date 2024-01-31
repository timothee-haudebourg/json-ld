use super::{
	definition,
	term_definition::{self, InvalidNest},
	Context, ContextEntry, Definition, TermDefinition,
};
use crate::{Container, ErrorCode, Keyword, Nullable, TryFromJson};
use iref::IriRefBuf;

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

impl TryFromJson for TermDefinition {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => {
				Ok(Self::Simple(term_definition::Simple(s.to_string())))
			}
			json_syntax::Value::Object(o) => {
				let mut def = term_definition::Expanded::new();

				for json_syntax::object::Entry { key, value } in o {
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Id) => def.id = Some(Nullable::try_from_json(value)?),
						Ok(Keyword::Type) => def.type_ = Some(Nullable::try_from_json(value)?),
						Ok(Keyword::Context) => {
							def.context = Some(Box::new(Context::try_from_json(value)?))
						}
						Ok(Keyword::Reverse) => {
							def.reverse = Some(definition::Key::try_from_json(value)?)
						}
						Ok(Keyword::Index) => {
							def.index = Some(term_definition::Index::try_from_json(value)?)
						}
						Ok(Keyword::Language) => {
							def.language = Some(Nullable::try_from_json(value)?)
						}
						Ok(Keyword::Direction) => {
							def.direction = Some(Nullable::try_from_json(value)?)
						}
						Ok(Keyword::Container) => {
							let container = match value {
								json_syntax::Value::Null => Nullable::Null,
								other => {
									let container = Container::try_from_json(other)?;
									Nullable::Some(container)
								}
							};

							def.container = Some(container)
						}
						Ok(Keyword::Nest) => {
							def.nest = Some(term_definition::Nest::try_from_json(value)?)
						}
						Ok(Keyword::Prefix) => def.prefix = Some(bool::try_from_json(value)?),
						Ok(Keyword::Propagate) => def.propagate = Some(bool::try_from_json(value)?),
						Ok(Keyword::Protected) => def.protected = Some(bool::try_from_json(value)?),
						_ => return Err(InvalidContext::InvalidTermDefinition),
					}
				}

				Ok(Self::Expanded(Box::new(def)))
			}
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String, json_syntax::Kind::Object],
			)),
		}
	}
}

impl TryFromJson for term_definition::Type {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for definition::TypeContainer {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => match Keyword::try_from(s.as_str()) {
				Ok(Keyword::Set) => Ok(Self::Set),
				_ => Err(InvalidContext::InvalidTermDefinition),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for definition::Type {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::Object(o) => {
				let mut container = None;
				let mut protected = None;

				for json_syntax::object::Entry { key, value } in o {
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Container) => {
							if container
								.replace(definition::TypeContainer::try_from_json(value)?)
								.is_some()
							{
								return Err(InvalidContext::DuplicateKey);
							}
						}
						Ok(Keyword::Protected) => {
							if protected.replace(bool::try_from_json(value)?).is_some() {
								return Err(InvalidContext::DuplicateKey);
							}
						}
						_ => return Err(InvalidContext::InvalidTermDefinition),
					}
				}

				match container {
					Some(container) => Ok(Self {
						container,
						protected,
					}),
					None => Err(InvalidContext::InvalidTermDefinition),
				}
			}
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::Object],
			)),
		}
	}
}

impl TryFromJson for definition::Version {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::Number(n) => match n.as_str() {
				"1.1" => Ok(Self::V1_1),
				_ => Err(InvalidContext::InvalidTermDefinition),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::Number],
			)),
		}
	}
}

impl TryFromJson for definition::Vocab {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for term_definition::Id {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for definition::Key {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, Self::Error> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for term_definition::Index {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => Ok(Self::from(s.into_string())),
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for term_definition::Nest {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => match Self::try_from(s.into_string()) {
				Ok(nest) => Ok(nest),
				Err(InvalidNest(s)) => Err(InvalidContext::InvalidNestValue(s)),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for Context {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::Array(a) => {
				let mut many = Vec::with_capacity(a.len());

				for item in a {
					many.push(ContextEntry::try_from_json(item)?)
				}

				Ok(Self::Many(many))
			}
			context => Ok(Self::One(ContextEntry::try_from_json(context)?)),
		}
	}
}

impl TryFromJson for ContextEntry {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::Null => Ok(Self::Null),
			json_syntax::Value::String(s) => match IriRefBuf::new(s.into_string()) {
				Ok(iri_ref) => Ok(Self::IriRef(iri_ref)),
				Err(e) => Err(InvalidContext::InvalidIriRef(e.0)),
			},
			json_syntax::Value::Object(o) => {
				let mut def = Definition::new();

				for json_syntax::object::Entry { key, value } in o {
					match Keyword::try_from(key.as_str()) {
						Ok(Keyword::Base) => def.base = Some(Nullable::try_from_json(value)?),
						Ok(Keyword::Import) => def.import = Some(IriRefBuf::try_from_json(value)?),
						Ok(Keyword::Language) => {
							def.language = Some(Nullable::try_from_json(value)?)
						}
						Ok(Keyword::Direction) => {
							def.direction = Some(Nullable::try_from_json(value)?)
						}
						Ok(Keyword::Propagate) => def.propagate = Some(bool::try_from_json(value)?),
						Ok(Keyword::Protected) => def.protected = Some(bool::try_from_json(value)?),
						Ok(Keyword::Type) => {
							def.type_ = Some(definition::Type::try_from_json(value)?)
						}
						Ok(Keyword::Version) => {
							def.version = Some(definition::Version::try_from_json(value)?)
						}
						Ok(Keyword::Vocab) => def.vocab = Some(Nullable::try_from_json(value)?),
						_ => {
							let term_def = match value {
								json_syntax::Value::Null => Nullable::Null,
								other => Nullable::Some(TermDefinition::try_from_json(other)?),
							};

							if def.bindings.insert_with(key.into(), term_def).is_some() {
								return Err(InvalidContext::DuplicateKey);
							}
						}
					}
				}

				Ok(Self::Definition(def))
			}
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[
					json_syntax::Kind::Null,
					json_syntax::Kind::String,
					json_syntax::Kind::Object,
				],
			)),
		}
	}
}
