use crate::LoadError;
use crate::algorithms::flattening::ConflictingIndexes;
use crate::algorithms::{JsonLdLocated, JsonLdLocationStack};

mod code;

pub use code::*;

pub type JsonLdLocatedError = Box<JsonLdLocated<JsonLdError>>;

#[derive(Debug, thiserror::Error)]
pub enum JsonLdError {
	#[error("Invalid context nullification")]
	InvalidContextNullification,

	#[error("Remote document loading failed")]
	LoadingDocumentFailed,

	#[error("Processing mode conflict")]
	ProcessingModeConflict,

	#[error("Invalid context")]
	ContextSyntax(crate::syntax::serde::DeserializeError),

	#[error("Invalid `@context` entry")]
	InvalidContextEntry,

	#[error("Invalid `@import` value")]
	InvalidImportValue,

	#[error("Invalid remote context")]
	InvalidRemoteContext,

	#[error("Invalid base IRI")]
	InvalidBaseIri,

	#[error("Invalid base direction")]
	InvalidBaseDirection,

	#[error("Invalid vocabulary mapping")]
	InvalidVocabMapping,

	#[error("Cyclic IRI mapping")]
	CyclicIriMapping,

	#[error("Invalid term definition")]
	InvalidTermDefinition,

	#[error("Keyword redefinition")]
	KeywordRedefinition,

	#[error("Invalid `@protected` value")]
	InvalidProtectedValue,

	#[error("Invalid type mapping")]
	InvalidTypeMapping,

	#[error("Invalid reverse property")]
	InvalidReverseProperty,

	#[error("Invalid IRI mapping")]
	InvalidIriMapping,

	#[error("Invalid keyword alias")]
	InvalidKeywordAlias,

	#[error("Invalid container mapping")]
	InvalidContainerMapping,

	#[error("Invalid scoped context")]
	InvalidScopedContext,

	#[error("Protected term redefinition")]
	ProtectedTermRedefinition,

	#[error(transparent)]
	ContextLoadingFailed(#[from] LoadError),

	#[error("Unable to parse JSON-LD context: {0}")]
	RemoteContextSyntax(crate::syntax::serde::DeserializeError),

	#[error("Invalid `@index` value")]
	InvalidIndexValue,

	#[error("Invalid typed value")]
	InvalidTypedValue,

	#[error("Invalid value object")]
	InvalidValueObject,

	#[error("Invalid value object value")]
	InvalidValueObjectValue,

	#[error("Invalid set or list object")]
	InvalidSetOrListObject,

	#[error("Invalid `@reverse` property map")]
	InvalidReversePropertyMap,

	#[error("Invalid `@type` value")]
	InvalidTypeValue,

	#[error("Key `{0}` expansion failed")]
	KeyExpansionFailed(String),

	#[error("Invalid `@reverse` property value")]
	InvalidReversePropertyValue,

	#[error("Invalid language-tagged string")]
	InvalidLanguageTaggedString,

	#[error("Invalid language-tagged value")]
	InvalidLanguageTaggedValue,

	#[error("Invalid `@language` map value")]
	InvalidLanguageMapValue,

	#[error("Colliding keywords")]
	CollidingKeywords,

	#[error(transparent)]
	ConflictingIndexes(#[from] ConflictingIndexes),

	#[error("Invalid `@id` value")]
	InvalidIdValue,

	#[error("Invalid `@included` value")]
	InvalidIncludedValue,

	#[error("Invalid `@reverse` value")]
	InvalidReverseValue,

	#[error("Invalid `@nest` value")]
	InvalidNestValue,

	#[error("Duplicate key `{0}`")]
	DuplicateKey(json_syntax::object::Key),

	#[error("IRI confused with prefix")]
	IriConfusedWithPrefix,
}

// impl From<Box<ConflictingIndexes> for JsonLdError {
//     //
// }

impl JsonLdError {
	pub fn at(self, location: JsonLdLocationStack<'_>) -> JsonLdLocatedError {
		Box::new(JsonLdLocated::new(self, location.build()))
	}

	pub fn duplicate_key_ref(d: json_syntax::object::DuplicateEntryRef) -> Self {
		Self::DuplicateKey(d.0.0.clone())
	}

	pub fn code(&self) -> JsonLdErrorCode {
		match self {
			Self::InvalidContextNullification => JsonLdErrorCode::InvalidContextNullification,
			Self::LoadingDocumentFailed => JsonLdErrorCode::LoadingDocumentFailed,
			Self::ProcessingModeConflict => JsonLdErrorCode::ProcessingModeConflict,
			Self::ContextSyntax(_) => JsonLdErrorCode::InvalidContextEntry,
			Self::InvalidContextEntry => JsonLdErrorCode::InvalidContextEntry,
			Self::InvalidImportValue => JsonLdErrorCode::InvalidImportValue,
			Self::InvalidRemoteContext => JsonLdErrorCode::InvalidRemoteContext,
			Self::InvalidBaseIri => JsonLdErrorCode::InvalidBaseIri,
			Self::InvalidBaseDirection => JsonLdErrorCode::InvalidBaseDirection,
			Self::InvalidVocabMapping => JsonLdErrorCode::InvalidVocabMapping,
			Self::CyclicIriMapping => JsonLdErrorCode::CyclicIriMapping,
			Self::InvalidTermDefinition => JsonLdErrorCode::InvalidTermDefinition,
			Self::KeywordRedefinition => JsonLdErrorCode::KeywordRedefinition,
			Self::InvalidProtectedValue => JsonLdErrorCode::InvalidPropagateValue,
			Self::InvalidTypeMapping => JsonLdErrorCode::InvalidTypeMapping,
			Self::InvalidReverseProperty => JsonLdErrorCode::InvalidReverseProperty,
			Self::InvalidIriMapping => JsonLdErrorCode::InvalidIriMapping,
			Self::InvalidKeywordAlias => JsonLdErrorCode::InvalidKeywordAlias,
			Self::InvalidContainerMapping => JsonLdErrorCode::InvalidContainerMapping,
			Self::InvalidScopedContext => JsonLdErrorCode::InvalidScopedContext,
			Self::ProtectedTermRedefinition => JsonLdErrorCode::ProtectedTermRedefinition,
			Self::ContextLoadingFailed(_) => JsonLdErrorCode::LoadingRemoteContextFailed,
			Self::RemoteContextSyntax(_) => JsonLdErrorCode::LoadingRemoteContextFailed,
			Self::InvalidIndexValue => JsonLdErrorCode::InvalidIndexValue,
			Self::InvalidTypedValue => JsonLdErrorCode::InvalidTypedValue,
			Self::InvalidValueObject => JsonLdErrorCode::InvalidValueObject,
			Self::InvalidValueObjectValue => JsonLdErrorCode::InvalidValueObjectValue,
			Self::InvalidSetOrListObject => JsonLdErrorCode::InvalidSetOrListObject,
			Self::InvalidReversePropertyMap => JsonLdErrorCode::InvalidReversePropertyMap,
			Self::InvalidTypeValue => JsonLdErrorCode::InvalidTypeValue,
			Self::KeyExpansionFailed(_) => JsonLdErrorCode::KeyExpansionFailed,
			Self::InvalidReversePropertyValue => JsonLdErrorCode::InvalidReversePropertyValue,
			Self::InvalidLanguageTaggedString => JsonLdErrorCode::InvalidLanguageTaggedString,
			Self::InvalidLanguageTaggedValue => JsonLdErrorCode::InvalidLanguageTaggedValue,
			Self::InvalidLanguageMapValue => JsonLdErrorCode::InvalidLanguageMapValue,
			Self::CollidingKeywords => JsonLdErrorCode::CollidingKeywords,
			Self::ConflictingIndexes(_) => JsonLdErrorCode::ConflictingIndexes,
			Self::InvalidIdValue => JsonLdErrorCode::InvalidIdValue,
			Self::InvalidIncludedValue => JsonLdErrorCode::InvalidIncludedValue,
			Self::InvalidReverseValue => JsonLdErrorCode::InvalidReverseValue,
			Self::InvalidNestValue => JsonLdErrorCode::InvalidNestValue,
			Self::DuplicateKey(_) => JsonLdErrorCode::DuplicateKey,
			Self::IriConfusedWithPrefix => JsonLdErrorCode::IriConfusedWithPrefix,
		}
	}
}
