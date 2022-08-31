use std::fmt;
use locspan::Meta;

#[derive(Debug)]
pub enum Error<M, E> {
	ContextSyntax(json_ld_syntax::context::InvalidContext),
	ContextProcessing(json_ld_context_processing::Error<E>),
	InvalidIndexValue,
	InvalidSetOrListObject,
	InvalidReversePropertyMap,
	InvalidTypeValue,
	KeyExpansionFailed,
	InvalidReversePropertyValue,
	InvalidLanguageMapValue,
	CollidingKeywords,
	InvalidIdValue,
	InvalidIncludedValue,
	InvalidReverseValue,
	InvalidNestValue,
	DuplicateKey(Meta<json_syntax::object::Key, M>),
	Literal(crate::LiteralExpansionError),
	Value(crate::InvalidValue),
}

impl<M: Clone, E> Error<M, E> {
	pub fn duplicate_key_ref(json_syntax::object::Duplicate(a, b): json_syntax::object::Duplicate<&json_syntax::object::Entry<M>>) -> Meta<Self, M> {
		Meta(Self::DuplicateKey(a.key.clone()), b.key.metadata().clone())
	}
}

impl<M, E> From<json_ld_context_processing::Error<E>> for Error<M, E> {
	fn from(e: json_ld_context_processing::Error<E>) -> Self {
		Self::ContextProcessing(e)
	}
}

impl<M, E> From<crate::LiteralExpansionError> for Error<M, E> {
	fn from(e: crate::LiteralExpansionError) -> Self {
		Self::Literal(e)
	}
}

impl<M, E> From<crate::InvalidValue> for Error<M, E> {
	fn from(e: crate::InvalidValue) -> Self {
		Self::Value(e)
	}
}

impl<M, E: fmt::Display> fmt::Display for Error<M, E> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::ContextSyntax(e) => e.fmt(f),
			Self::ContextProcessing(e) => write!(f, "context processing error: {}", e),
			Self::InvalidIndexValue => write!(f, "invalid index value"),
			Self::InvalidSetOrListObject => write!(f, "invalid set or list object"),
			Self::InvalidReversePropertyMap => write!(f, "invalid reverse property map"),
			Self::InvalidTypeValue => write!(f, "invalid type value"),
			Self::KeyExpansionFailed => write!(f, "key expansion failed"),
			Self::InvalidReversePropertyValue => write!(f, "invalid reverse property value"),
			Self::InvalidLanguageMapValue => write!(f, "invalid language map value"),
			Self::CollidingKeywords => write!(f, "colliding keywords"),
			Self::InvalidIdValue => write!(f, "invalid id value"),
			Self::InvalidIncludedValue => write!(f, "invalid included value"),
			Self::InvalidReverseValue => write!(f, "invalid reverse value"),
			Self::InvalidNestValue => write!(f, "invalid nest value"),
			Self::DuplicateKey(Meta(key, _)) => write!(f, "duplicate key `{}`", key),
			Self::Literal(e) => e.fmt(f),
			Self::Value(e) => e.fmt(f),
		}
	}
}
