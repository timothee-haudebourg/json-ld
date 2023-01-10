use json_ld_syntax::ErrorCode;
use locspan::Meta;

#[derive(Debug, thiserror::Error)]
pub enum Error<M, E> {
	#[error("Invalid context: {0}")]
	ContextSyntax(json_ld_syntax::context::InvalidContext),

	#[error("Context processing failed: {0}")]
	ContextProcessing(json_ld_context_processing::Error<E>),

	#[error("Invalid `@index` value")]
	InvalidIndexValue,

	#[error("Invalid set or list object")]
	InvalidSetOrListObject,

	#[error("Invalid `@reverse` property map")]
	InvalidReversePropertyMap,

	#[error("Invalid `@type` value")]
	InvalidTypeValue,

	#[error("Key expansion failed")]
	KeyExpansionFailed,

	#[error("Invalid `@reverse` property value")]
	InvalidReversePropertyValue,

	#[error("Invalid `@language` map value")]
	InvalidLanguageMapValue,

	#[error("Colliding keywords")]
	CollidingKeywords,

	#[error("Invalid `@id` value")]
	InvalidIdValue,

	#[error("Invalid `@included` value")]
	InvalidIncludedValue,

	#[error("Invalid `@reverse` value")]
	InvalidReverseValue,

	#[error("Invalid `@nest` value")]
	InvalidNestValue,

	#[error("Duplicate key `{0}`")]
	DuplicateKey(Meta<json_syntax::object::Key, M>),

	#[error(transparent)]
	Literal(crate::LiteralExpansionError),

	#[error(transparent)]
	Value(crate::InvalidValue),
}

impl<M, E> Error<M, E> {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::ContextSyntax(e) => e.code(),
			Self::ContextProcessing(e) => e.code(),
			Self::InvalidIndexValue => ErrorCode::InvalidIndexValue,
			Self::InvalidSetOrListObject => ErrorCode::InvalidSetOrListObject,
			Self::InvalidReversePropertyMap => ErrorCode::InvalidReversePropertyMap,
			Self::InvalidTypeValue => ErrorCode::InvalidTypeValue,
			Self::KeyExpansionFailed => ErrorCode::KeyExpansionFailed,
			Self::InvalidReversePropertyValue => ErrorCode::InvalidReversePropertyValue,
			Self::InvalidLanguageMapValue => ErrorCode::InvalidLanguageMapValue,
			Self::CollidingKeywords => ErrorCode::CollidingKeywords,
			Self::InvalidIdValue => ErrorCode::InvalidIdValue,
			Self::InvalidIncludedValue => ErrorCode::InvalidIncludedValue,
			Self::InvalidReverseValue => ErrorCode::InvalidReverseValue,
			Self::InvalidNestValue => ErrorCode::InvalidNestValue,
			Self::DuplicateKey(_) => ErrorCode::DuplicateKey,
			Self::Literal(e) => e.code(),
			Self::Value(e) => e.code(),
		}
	}
}

impl<M: Clone, E> Error<M, E> {
	pub fn duplicate_key_ref(
		json_syntax::object::Duplicate(a, b): json_syntax::object::Duplicate<
			&json_syntax::object::Entry<M>,
		>,
	) -> Meta<Self, M> {
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
