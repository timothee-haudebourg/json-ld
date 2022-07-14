use crate::Loader;
use derivative::Derivative;
use json_ld_context_processing::ContextLoader;

#[derive(Derivative)]
#[derivative(Debug(bound = "<L as ContextLoader>::Error: core::fmt::Debug"))]
pub enum Error<L: Loader + ContextLoader> {
	ContextProcessing(json_ld_context_processing::Error<<L as ContextLoader>::Error>),
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
	Value(crate::ValueExpansionError),
}

impl<L: Loader + ContextLoader> From<json_ld_context_processing::Error<<L as ContextLoader>::Error>>
	for Error<L>
{
	fn from(e: json_ld_context_processing::Error<<L as ContextLoader>::Error>) -> Self {
		Self::ContextProcessing(e)
	}
}

impl<L: Loader + ContextLoader> From<crate::LiteralExpansionError> for Error<L> {
	fn from(e: crate::LiteralExpansionError) -> Self {
		Self::Literal(e)
	}
}

impl<L: Loader + ContextLoader> From<crate::ValueExpansionError> for Error<L> {
	fn from(e: crate::ValueExpansionError) -> Self {
		Self::Value(e)
	}
}
