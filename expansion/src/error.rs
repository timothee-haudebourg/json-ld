pub enum Error {
	ContextProcessing(json_ld_context_processing::Error),
	InvalidIndexValue,
	InvalidSetOrListObject,
	InvalidReversePropertyMap,
	InvalidLanguageTaggedString,
	InvalidBaseDirection,
	InvalidTypedValue,
	InvalidValueObject,
	InvalidValueObjectValue,
	InvalidLanguageTaggedValue,
	InvalidTypeValue,
	KeyExpansionFailed,
	InvalidReversePropertyValue,
	InvalidLanguageMapValue,
	CollidingKeywords,
	InvalidIdValue,
	InvalidIncludedValue,
	InvalidReverseValue,
	InvalidNestValue
}

impl From<json_ld_context_processing::Error> for Error {
	fn from(e: json_ld_context_processing::Error) -> Self {
		Self::ContextProcessing(e)
	}
}