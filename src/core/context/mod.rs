//! Context processing algorithm and related types.
mod container;
mod definition;
mod embed;
pub mod inverse;

pub use container::JsonLdContainer;
use langtag::{LangTag, LangTagBuf};

use crate::syntax::context::JsonLdContextTerm;
use crate::syntax::{JsonLdContext, JsonLdKeywordType};
use crate::{JsonLdDirection, JsonLdLenient, JsonLdTerm};
use rdf_syntax::{Iri, IriBuf};
use std::borrow::{Borrow, Cow};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

pub use definition::*;
pub use embed::*;
pub use inverse::InverseJsonLdContext;

/// Processed JSON-LD context.
///
/// Represents the result of the [context processing algorithm][1] implemented
/// by the [`json-ld-context-processing`] crate.
///
/// [1]: <https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm>
/// [`json-ld-context-processing`]: <https://crates.io/crates/json-ld-context-processing>
pub struct JsonLdProcessedContext<'a> {
	unprocessed: Cow<'a, JsonLdContext>,
	raw: RawJsonLdProcessedContext,
}

impl<'a> JsonLdProcessedContext<'a> {
	pub fn new(unprocessed: Cow<'a, JsonLdContext>, raw: RawJsonLdProcessedContext) -> Self {
		Self { unprocessed, raw }
	}

	pub fn unprocessed(&self) -> &JsonLdContext {
		&self.unprocessed
	}

	pub fn as_raw(&self) -> &RawJsonLdProcessedContext {
		&self.raw
	}

	pub fn into_raw(self) -> RawJsonLdProcessedContext {
		self.raw
	}

	pub fn into_owned(self) -> JsonLdProcessedContext<'static> {
		JsonLdProcessedContext {
			unprocessed: Cow::Owned(self.unprocessed.into_owned()),
			raw: self.raw,
		}
	}
}

impl Deref for JsonLdProcessedContext<'_> {
	type Target = RawJsonLdProcessedContext;

	fn deref(&self) -> &Self::Target {
		&self.raw
	}
}

impl DerefMut for JsonLdProcessedContext<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.raw
	}
}

/// Processed JSON-LD context.
///
/// Represents the result of the [context processing algorithm][1] implemented
/// by the [`json-ld-context-processing`] crate.
///
/// [1]: <https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm>
/// [`json-ld-context-processing`]: <https://crates.io/crates/json-ld-context-processing>
#[derive(Default)]
pub struct RawJsonLdProcessedContext {
	original_base_url: Option<IriBuf>,
	base_iri: Option<IriBuf>,
	vocabulary: Option<JsonLdTerm>,
	default_language: Option<JsonLdLenient<LangTagBuf>>,
	default_base_direction: Option<JsonLdDirection>,
	previous_context: Option<Box<Self>>,
	definitions: JsonLdDefinitions,
	inverse: OnceLock<InverseJsonLdContext>,
}

pub type DefinitionEntryRef<'a> = (&'a JsonLdContextTerm, &'a JsonLdProcessedTermDefinition);

impl RawJsonLdProcessedContext {
	/// Create a new context with the given base IRI.
	pub fn new(base_iri: Option<IriBuf>) -> Self {
		Self {
			original_base_url: base_iri.clone(),
			base_iri,
			vocabulary: None,
			default_language: None,
			default_base_direction: None,
			previous_context: None,
			definitions: JsonLdDefinitions::default(),
			inverse: OnceLock::default(),
		}
	}

	/// Returns a reference to the given `term` definition, if any.
	pub fn get<Q>(&self, term: &Q) -> Option<JsonLdProcessedTermDefinitionRef<'_>>
	where
		JsonLdContextTerm: Borrow<Q>,
		JsonLdKeywordType: Borrow<Q>,
		Q: ?Sized + Hash + Eq,
	{
		self.definitions.get(term)
	}

	/// Returns a reference to the given `term` normal definition, if any.
	pub fn get_normal<Q>(&self, term: &Q) -> Option<&JsonLdProcessedNormalTermDefinition>
	where
		JsonLdContextTerm: Borrow<Q>,
		Q: ?Sized + Hash + Eq,
	{
		self.definitions.get_normal(term)
	}

	/// Returns a reference to the `@type` definition, if any.
	pub fn get_type(&self) -> Option<&JsonLdProcessedTypeTermDefinition> {
		self.definitions.get_type()
	}

	/// Checks if the given `term` is defined.
	pub fn contains_term<Q>(&self, term: &Q) -> bool
	where
		JsonLdContextTerm: Borrow<Q>,
		JsonLdKeywordType: Borrow<Q>,
		Q: ?Sized + Hash + Eq,
	{
		self.definitions.contains_term(term)
	}

	/// Returns the original base URL of the context.
	pub fn original_base_url(&self) -> Option<&Iri> {
		self.original_base_url.as_deref()
	}

	/// Returns the base IRI of the context.
	pub fn base_iri(&self) -> Option<&Iri> {
		self.base_iri.as_deref()
	}

	/// Returns the `@vocab` value, if any.
	pub fn vocabulary(&self) -> Option<&JsonLdTerm> {
		match &self.vocabulary {
			Some(v) => Some(v),
			None => None,
		}
	}

	/// Returns the default `@language` value.
	pub fn default_language(&self) -> Option<JsonLdLenient<&LangTag>> {
		self.default_language.as_ref().map(|tag| tag.as_deref())
	}

	/// Returns the default `@direction` value.
	pub fn default_base_direction(&self) -> Option<JsonLdDirection> {
		self.default_base_direction
	}

	/// Returns a reference to the previous context.
	pub fn previous_context(&self) -> Option<&Self> {
		match &self.previous_context {
			Some(c) => Some(c),
			None => None,
		}
	}

	/// Returns the number of terms defined.
	pub fn len(&self) -> usize {
		self.definitions.len()
	}

	/// Checks if no terms are defined.
	pub fn is_empty(&self) -> bool {
		self.definitions.is_empty()
	}

	/// Returns a handle to the term definitions.
	pub fn definitions(&self) -> &JsonLdDefinitions {
		&self.definitions
	}

	/// Checks if the context has a protected definition.
	pub fn has_protected_items(&self) -> bool {
		for binding in self.definitions() {
			if binding.definition().protected() {
				return true;
			}
		}

		false
	}

	/// Returns the inverse of this context.
	pub fn inverse(&self) -> &InverseJsonLdContext {
		self.inverse.get_or_init(|| self.into())
	}

	/// Sets the normal definition for the given term `key`.
	pub fn set_normal(
		&mut self,
		key: JsonLdContextTerm,
		definition: Option<JsonLdProcessedNormalTermDefinition>,
	) -> Option<JsonLdProcessedNormalTermDefinition> {
		self.inverse.take();
		self.definitions.set_normal(key, definition)
	}

	/// Sets the `@type` definition.
	pub fn set_type(
		&mut self,
		type_: Option<JsonLdProcessedTypeTermDefinition>,
	) -> Option<JsonLdProcessedTypeTermDefinition> {
		self.definitions.set_type(type_)
	}

	/// Sets the base IRI.
	pub fn set_base_iri(&mut self, iri: Option<IriBuf>) {
		self.inverse.take();
		self.base_iri = iri
	}

	/// Sets the `@vocab` value.
	pub fn set_vocabulary(&mut self, vocab: Option<JsonLdTerm>) {
		self.inverse.take();
		self.vocabulary = vocab;
	}

	/// Sets the default `@language` value.
	pub fn set_default_language(&mut self, lang: Option<JsonLdLenient<LangTagBuf>>) {
		self.inverse.take();
		self.default_language = lang;
	}

	/// Sets the default `@direction` value.
	pub fn set_default_base_direction(&mut self, dir: Option<JsonLdDirection>) {
		self.inverse.take();
		self.default_base_direction = dir;
	}

	/// Sets the previous context.
	pub fn set_previous_context(&mut self, previous: Self) {
		self.inverse.take();
		self.previous_context = Some(Box::new(previous))
	}
}

impl Clone for RawJsonLdProcessedContext {
	fn clone(&self) -> Self {
		Self {
			original_base_url: self.original_base_url.clone(),
			base_iri: self.base_iri.clone(),
			vocabulary: self.vocabulary.clone(),
			default_language: self.default_language.clone(),
			default_base_direction: self.default_base_direction,
			previous_context: self.previous_context.clone(),
			definitions: self.definitions.clone(),
			inverse: OnceLock::default(),
		}
	}
}

impl PartialEq for RawJsonLdProcessedContext {
	fn eq(&self, other: &Self) -> bool {
		self.original_base_url == other.original_base_url
			&& self.base_iri == other.base_iri
			&& self.vocabulary == other.vocabulary
			&& self.default_language == other.default_language
			&& self.default_base_direction == other.default_base_direction
			&& self.previous_context == other.previous_context
	}
}
