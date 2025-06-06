use std::convert::TryFrom;
use std::fmt;

/// Error code.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum ErrorCode {
	/// Two properties which expand to the same keyword have been detected.
	/// This might occur if a keyword and an alias thereof are used at the same time.
	CollidingKeywords,

	/// Multiple conflicting indexes have been found for the same node.
	ConflictingIndexes,

	/// maximum number of `@context` URLs exceeded.
	ContextOverflow,

	/// A cycle in IRI mappings has been detected.
	CyclicIriMapping,

	/// An `@id` entry was encountered whose value was not a string.
	InvalidIdValue,

	/// An invalid value for `@import` has been found.
	InvalidImportValue,

	/// An included block contains an invalid value.
	InvalidIncludedValue,

	/// An `@index` entry was encountered whose value was not a string.
	InvalidIndexValue,

	/// An invalid value for `@nest` has been found.
	InvalidNestValue,

	/// An invalid value for `@prefix` has been found.
	InvalidPrefixValue,

	/// An invalid value for `@propagate` has been found.
	InvalidPropagateValue,

	/// An invalid value for `@protected` has been found.
	InvalidProtectedValue,

	/// An invalid value for an `@reverse` entry has been detected, i.e., the value was not a map.
	InvalidReverseValue,

	/// The `@version` entry was used in a context with an out of range value.
	InvalidVersionValue,

	/// The value of `@direction` is not "ltr", "rtl", or null and thus invalid.
	InvalidBaseDirection,

	/// An invalid base IRI has been detected, i.e., it is neither an IRI nor null.
	InvalidBaseIri,

	/// An `@container` entry was encountered whose value was not one of the following strings:
	/// `@list`, `@set`, or `@index`.
	InvalidContainerMapping,

	/// An entry in a context is invalid due to processing mode incompatibility.
	InvalidContextEntry,

	/// An attempt was made to nullify a context containing protected term definitions.
	InvalidContextNullification,

	/// The value of the default language is not a string or null and thus invalid.
	InvalidDefaultLanguage,

	/// A local context contains a term that has an invalid or missing IRI mapping.
	InvalidIriMapping,

	/// An invalid JSON literal was detected.
	InvalidJsonLiteral,

	/// An invalid keyword alias definition has been encountered.
	InvalidKeywordAlias,

	/// An invalid value in a language map has been detected. It MUST be a string or an array of
	/// strings.
	InvalidLanguageMapValue,

	/// An `@language` entry in a term definition was encountered whose value was neither a string
	/// nor null and thus invalid.
	InvalidLanguageMapping,

	/// A language-tagged string with an invalid language value was detected.
	InvalidLanguageTaggedString,

	/// A number, true, or false with an associated language tag was detected.
	InvalidLanguageTaggedValue,

	/// An invalid local context was detected.
	InvalidLocalContext,

	/// No valid context document has been found for a referenced remote context.
	InvalidRemoteContext,

	/// An invalid reverse property definition has been detected.
	InvalidReverseProperty,

	/// An invalid reverse property map has been detected. No keywords apart from `@context` are
	/// allowed in reverse property maps.
	InvalidReversePropertyMap,

	/// An invalid value for a reverse property has been detected. The value of an inverse property
	/// must be a node object.
	InvalidReversePropertyValue,

	/// The local context defined within a term definition is invalid.
	InvalidScopedContext,

	/// A script element in HTML input which is the target of a fragment identifier does not have
	/// an appropriate type attribute.
	InvalidScriptElement,

	/// A set object or list object with disallowed entries has been detected.
	InvalidSetOrListObject,

	/// An invalid term definition has been detected.
	InvalidTermDefinition,

	/// An `@type` entry in a term definition was encountered whose value could not be expanded to an
	/// IRI.
	InvalidTypeMapping,

	/// An invalid value for an `@type` entry has been detected, i.e., the value was neither a string
	/// nor an array of strings.
	InvalidTypeValue,

	/// A typed value with an invalid type was detected.
	InvalidTypedValue,

	/// A value object with disallowed entries has been detected.
	InvalidValueObject,

	/// An invalid value for the `@value` entry of a value object has been detected, i.e., it is
	/// neither a scalar nor null.
	InvalidValueObjectValue,

	/// An invalid vocabulary mapping has been detected, i.e., it is neither an IRI nor null.
	InvalidVocabMapping,

	/// When compacting an IRI would result in an IRI which could be confused with a compact IRI
	/// (because its IRI scheme matches a term definition and it has no IRI authority).
	IriConfusedWithPrefix,

	/// Unable to expand a key into a IRI, blank node identifier or keyword
	/// using the current key expansion policy.
	/// Note: this error is not defined in the JSON-LD API specification.
	KeyExpansionFailed,

	/// A keyword redefinition has been detected.
	KeywordRedefinition,

	/// The document could not be loaded or parsed as JSON.
	LoadingDocumentFailed,

	/// There was a problem encountered loading a remote context.
	LoadingRemoteContextFailed,

	/// Multiple HTTP Link Headers [RFC8288](https://tools.ietf.org/html/rfc8288) using the <http://www.w3.org/ns/json-ld#context> link
	/// relation have been detected.
	MultipleContextLinkHeaders,

	/// An attempt was made to change the processing mode which is incompatible with the previous
	/// specified version.
	ProcessingModeConflict,

	/// An attempt was made to redefine a protected term.
	ProtectedTermRedefinition,

	/// Duplicate key in JSON object.
	DuplicateKey,
}

impl ErrorCode {
	/// Get the error message corresponding to the error code.
	pub fn as_str(&self) -> &str {
		use ErrorCode::*;

		match self {
			CollidingKeywords => "colliding keywords",
			ConflictingIndexes => "conflicting indexes",
			ContextOverflow => "context overflow",
			CyclicIriMapping => "cyclic IRI mapping",
			InvalidIdValue => "invalid @id value",
			InvalidImportValue => "invalid @import value",
			InvalidIncludedValue => "invalid @included value",
			InvalidIndexValue => "invalid @index value",
			InvalidNestValue => "invalid @nest value",
			InvalidPrefixValue => "invalid @prefix value",
			InvalidPropagateValue => "invalid @propagate value",
			InvalidProtectedValue => "invalid @protected value",
			InvalidReverseValue => "invalid @reverse value",
			InvalidVersionValue => "invalid @version value",
			InvalidBaseDirection => "invalid base direction",
			InvalidBaseIri => "invalid base IRI",
			InvalidContainerMapping => "invalid container mapping",
			InvalidContextEntry => "invalid context entry",
			InvalidContextNullification => "invalid context nullification",
			InvalidDefaultLanguage => "invalid default language",
			InvalidIriMapping => "invalid IRI mapping",
			InvalidJsonLiteral => "invalid JSON literal",
			InvalidKeywordAlias => "invalid keyword alias",
			InvalidLanguageMapValue => "invalid language map value",
			InvalidLanguageMapping => "invalid language mapping",
			InvalidLanguageTaggedString => "invalid language-tagged string",
			InvalidLanguageTaggedValue => "invalid language-tagged value",
			InvalidLocalContext => "invalid local context",
			InvalidRemoteContext => "invalid remote context",
			InvalidReverseProperty => "invalid reverse property",
			InvalidReversePropertyMap => "invalid reverse property map",
			InvalidReversePropertyValue => "invalid reverse property value",
			InvalidScopedContext => "invalid scoped context",
			InvalidScriptElement => "invalid script element",
			InvalidSetOrListObject => "invalid set or list object",
			InvalidTermDefinition => "invalid term definition",
			InvalidTypeMapping => "invalid type mapping",
			InvalidTypeValue => "invalid type value",
			InvalidTypedValue => "invalid typed value",
			InvalidValueObject => "invalid value object",
			InvalidValueObjectValue => "invalid value object value",
			InvalidVocabMapping => "invalid vocab mapping",
			IriConfusedWithPrefix => "IRI confused with prefix",
			KeyExpansionFailed => "key expansion failed",
			KeywordRedefinition => "keyword redefinition",
			LoadingDocumentFailed => "loading document failed",
			LoadingRemoteContextFailed => "loading remote context failed",
			MultipleContextLinkHeaders => "multiple context link headers",
			ProcessingModeConflict => "processing mode conflict",
			ProtectedTermRedefinition => "protected term redefinition",
			DuplicateKey => "duplicate key",
		}
	}
}

impl<'a> TryFrom<&'a str> for ErrorCode {
	type Error = ();

	fn try_from(name: &'a str) -> Result<ErrorCode, ()> {
		use ErrorCode::*;
		match name {
			"colliding keywords" => Ok(CollidingKeywords),
			"conflicting indexes" => Ok(ConflictingIndexes),
			"context overflow" => Ok(ContextOverflow),
			"cyclic IRI mapping" => Ok(CyclicIriMapping),
			"invalid @id value" => Ok(InvalidIdValue),
			"invalid @import value" => Ok(InvalidImportValue),
			"invalid @included value" => Ok(InvalidIncludedValue),
			"invalid @index value" => Ok(InvalidIndexValue),
			"invalid @nest value" => Ok(InvalidNestValue),
			"invalid @prefix value" => Ok(InvalidPrefixValue),
			"invalid @propagate value" => Ok(InvalidPropagateValue),
			"invalid @protected value" => Ok(InvalidProtectedValue),
			"invalid @reverse value" => Ok(InvalidReverseValue),
			"invalid @version value" => Ok(InvalidVersionValue),
			"invalid base direction" => Ok(InvalidBaseDirection),
			"invalid base IRI" => Ok(InvalidBaseIri),
			"invalid container mapping" => Ok(InvalidContainerMapping),
			"invalid context entry" => Ok(InvalidContextEntry),
			"invalid context nullification" => Ok(InvalidContextNullification),
			"invalid default language" => Ok(InvalidDefaultLanguage),
			"invalid IRI mapping" => Ok(InvalidIriMapping),
			"invalid JSON literal" => Ok(InvalidJsonLiteral),
			"invalid keyword alias" => Ok(InvalidKeywordAlias),
			"invalid language map value" => Ok(InvalidLanguageMapValue),
			"invalid language mapping" => Ok(InvalidLanguageMapping),
			"invalid language-tagged string" => Ok(InvalidLanguageTaggedString),
			"invalid language-tagged value" => Ok(InvalidLanguageTaggedValue),
			"invalid local context" => Ok(InvalidLocalContext),
			"invalid remote context" => Ok(InvalidRemoteContext),
			"invalid reverse property" => Ok(InvalidReverseProperty),
			"invalid reverse property map" => Ok(InvalidReversePropertyMap),
			"invalid reverse property value" => Ok(InvalidReversePropertyValue),
			"invalid scoped context" => Ok(InvalidScopedContext),
			"invalid script element" => Ok(InvalidScriptElement),
			"invalid set or list object" => Ok(InvalidSetOrListObject),
			"invalid term definition" => Ok(InvalidTermDefinition),
			"invalid type mapping" => Ok(InvalidTypeMapping),
			"invalid type value" => Ok(InvalidTypeValue),
			"invalid typed value" => Ok(InvalidTypedValue),
			"invalid value object" => Ok(InvalidValueObject),
			"invalid value object value" => Ok(InvalidValueObjectValue),
			"invalid vocab mapping" => Ok(InvalidVocabMapping),
			"IRI confused with prefix" => Ok(IriConfusedWithPrefix),
			"key expansion failed" => Ok(KeyExpansionFailed),
			"keyword redefinition" => Ok(KeywordRedefinition),
			"loading document failed" => Ok(LoadingDocumentFailed),
			"loading remote context failed" => Ok(LoadingRemoteContextFailed),
			"multiple context link headers" => Ok(MultipleContextLinkHeaders),
			"processing mode conflict" => Ok(ProcessingModeConflict),
			"protected term redefinition" => Ok(ProtectedTermRedefinition),
			_ => Err(()),
		}
	}
}

impl fmt::Display for ErrorCode {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
