use crate::{
	context::InvalidContext, Container, ContainerKind, Direction, LenientLanguageTagBuf, Nullable,
};
use iref::IriRefBuf;
use locspan::Meta;

pub trait TryFromJson<M>: Sized {
	type Error;

	fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>>;
}

pub trait TryFromStrippedJson<M>: Sized {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext>;
}

impl<M> crate::TryFromJson<M> for bool {
	type Error = crate::Unexpected;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<Self::Error, M>> {
		match value {
			json_syntax::Value::Boolean(b) => Ok(Meta(b, meta)),
			unexpected => Err(Meta(
				crate::Unexpected(unexpected.kind(), &[json_syntax::Kind::Boolean]),
				meta,
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for IriRefBuf {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => match IriRefBuf::from_string(s.into_string()) {
				Ok(iri_ref) => Ok(iri_ref),
				Err((e, _)) => Err(InvalidContext::InvalidIriRef(e)),
			},
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromJson<M> for IriRefBuf {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => match IriRefBuf::from_string(s.into_string()) {
				Ok(iri_ref) => Ok(Meta(iri_ref, meta)),
				Err((e, _)) => Err(Meta(InvalidContext::InvalidIriRef(e), meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for LenientLanguageTagBuf {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
		match value {
			json_syntax::Value::String(s) => {
				let (lang, _) = LenientLanguageTagBuf::new(s.into_string());
				Ok(lang)
			}
			unexpected => Err(InvalidContext::Unexpected(
				unexpected.kind(),
				&[json_syntax::Kind::String],
			)),
		}
	}
}

impl<M> TryFromStrippedJson<M> for Direction {
	fn try_from_stripped_json(value: json_syntax::Value<M>) -> Result<Self, InvalidContext> {
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

impl<M, T: TryFromStrippedJson<M>> TryFromJson<M> for Nullable<T> {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::Null => Ok(Meta(Self::Null, meta)),
			some => match T::try_from_stripped_json(some) {
				Ok(some) => Ok(Meta(Self::Some(some), meta)),
				Err(e) => Err(Meta(e, meta)),
			},
		}
	}
}

impl<M> TryFromJson<M> for Container<M> {
	type Error = InvalidContext;

	fn try_from_json(
		value: Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			Meta(json_syntax::Value::Array(a), meta) => {
				let mut container = Vec::new();

				for item in a {
					container.push(ContainerKind::try_from_json(item)?)
				}

				Ok(Meta(Self::Many(container), meta))
			}
			other => ContainerKind::try_from_json(other).map(Meta::cast),
		}
	}
}

impl<M> TryFromJson<M> for ContainerKind {
	type Error = InvalidContext;

	fn try_from_json(
		Meta(value, meta): Meta<json_syntax::Value<M>, M>,
	) -> Result<Meta<Self, M>, Meta<InvalidContext, M>> {
		match value {
			json_syntax::Value::String(s) => match ContainerKind::try_from(s.as_str()) {
				Ok(t) => Ok(Meta(t, meta)),
				Err(_) => Err(Meta(InvalidContext::InvalidTermDefinition, meta)),
			},
			unexpected => Err(Meta(
				InvalidContext::Unexpected(unexpected.kind(), &[json_syntax::Kind::String]),
				meta,
			)),
		}
	}
}
