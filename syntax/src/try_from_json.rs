use crate::{
	context::InvalidContext, Container, ContainerKind, Direction, LenientLangTagBuf, Nullable,
};
use iref::IriRefBuf;

pub trait TryFromJson: Sized {
	type Error;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, Self::Error>;
}

impl crate::TryFromJson for bool {
	type Error = crate::Unexpected;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, Self::Error> {
		match value {
			json_syntax::Value::Boolean(b) => Ok(b),
			unexpected => Err(crate::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::Boolean],
			)),
		}
	}
}

impl TryFromJson for IriRefBuf {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => match IriRefBuf::new(s.into_string()) {
				Ok(iri_ref) => Ok(iri_ref),
				Err(e) => Err(InvalidContext::InvalidIriRef(e.0)),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for LenientLangTagBuf {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => {
				let (lang, _) = LenientLangTagBuf::new(s.into_string());
				Ok(lang)
			}
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl TryFromJson for Direction {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => match Direction::try_from(s.as_str()) {
				Ok(d) => Ok(d),
				Err(_) => Err(InvalidContext::InvalidDirection),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<T: TryFromJson> TryFromJson for Nullable<T> {
	type Error = T::Error;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, Self::Error> {
		match value {
			json_syntax::Value::Null => Ok(Self::Null),
			some => T::try_from_json(some).map(Self::Some),
		}
	}
}

impl TryFromJson for Container {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::Array(a) => {
				let mut container = Vec::new();

				for item in a {
					container.push(ContainerKind::try_from_json(item)?)
				}

				Ok(Self::Many(container))
			}
			other => ContainerKind::try_from_json(other).map(Into::into),
		}
	}
}

impl TryFromJson for ContainerKind {
	type Error = InvalidContext;

	fn try_from_json(value: json_syntax::Value) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => match ContainerKind::try_from(s.as_str()) {
				Ok(t) => Ok(t),
				Err(_) => Err(InvalidContext::InvalidTermDefinition),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}
