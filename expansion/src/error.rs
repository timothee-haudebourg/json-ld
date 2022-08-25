use std::fmt;

#[derive(Debug)]
pub enum Error<E> {
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
	Literal(crate::LiteralExpansionError),
	Value(crate::InvalidValue),
}

impl<E> From<json_ld_context_processing::Error<E>> for Error<E> {
	fn from(e: json_ld_context_processing::Error<E>) -> Self {
		Self::ContextProcessing(e)
	}
}

impl<E> From<crate::LiteralExpansionError> for Error<E> {
	fn from(e: crate::LiteralExpansionError) -> Self {
		Self::Literal(e)
	}
}

impl<E> From<crate::InvalidValue> for Error<E> {
	fn from(e: crate::InvalidValue) -> Self {
		Self::Value(e)
	}
}

impl<E: fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
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
			Self::Literal(e) => e.fmt(f),
			Self::Value(e) => e.fmt(f),
		}
	}
}
