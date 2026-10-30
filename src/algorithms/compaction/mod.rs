//! Compaction algorithm.
//!
//! See: <https://www.w3.org/TR/json-ld-api/#compaction-algorithms>
mod collection;
mod document;
mod iri;
mod object;
mod options;

use json_syntax::{JsonObject, JsonValue};
pub use options::*;

use crate::{
	JsonLdBacktraceBuilder, JsonLdIndexed, JsonLdLocatedError, JsonLdTerm, ProcessedJsonLdContext,
	algorithms::AsyncProcessingEnvironment,
	context::{
		RawJsonLdProcessedContext,
		inverse::{JsonLdLangSelection, JsonLdTypeSelection},
	},
	syntax::JsonLdKeyword,
};

/// Document that can be compacted.
pub trait JsonLdCompact {
	/// Compacts the input document with the given options.
	#[allow(async_fn_in_trait)]
	async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedJsonLdContext<'_>,
		options: JsonLdCompactionOptions,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError>;

	/// Compacts the input document with the default options.
	#[allow(async_fn_in_trait)]
	async fn compact(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedJsonLdContext<'_>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.compact_with(env, context, JsonLdCompactionOptions::default(), location)
			.await
	}
}

impl<T: JsonLdCompact> JsonLdCompact for std::sync::Arc<T> {
	async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedJsonLdContext<'_>,
		options: JsonLdCompactionOptions,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		T::compact_with(self, env, context, options, location).await
	}

	async fn compact(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedJsonLdContext<'_>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		T::compact(self, env, context, location).await
	}
}

/// Compactor.
struct Compactor<'a> {
	pub options: JsonLdCompactionOptions,
	pub active_context: &'a RawJsonLdProcessedContext,
	pub type_scoped_context: &'a RawJsonLdProcessedContext,
	pub active_property: Option<&'a str>,
}

impl<'a> Compactor<'a> {
	pub fn new(
		active_context: &'a RawJsonLdProcessedContext,
		options: JsonLdCompactionOptions,
	) -> Self {
		Self {
			options,
			active_context,
			type_scoped_context: active_context,
			active_property: None,
		}
	}

	pub fn with_active_context<'b>(
		&'b self,
		active_context: &'b RawJsonLdProcessedContext,
	) -> Compactor<'b> {
		Compactor {
			options: self.options,
			active_context,
			type_scoped_context: self.type_scoped_context,
			active_property: self.active_property,
		}
	}

	pub fn with_type_scoped_context<'b>(
		&'b self,
		type_scoped_context: &'b RawJsonLdProcessedContext,
	) -> Compactor<'b> {
		Compactor {
			options: self.options,
			active_context: self.active_context,
			type_scoped_context,
			active_property: self.active_property,
		}
	}

	pub fn with_active_property<'b>(&'b self, active_property: Option<&'b str>) -> Compactor<'b> {
		Compactor {
			options: self.options,
			active_context: self.active_context,
			type_scoped_context: self.type_scoped_context,
			active_property,
		}
	}
}

trait CompactFragment {
	#[allow(async_fn_in_trait)]
	async fn compact_fragment(
		&self,
		env: &impl AsyncProcessingEnvironment,
		compactor: &Compactor,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError>;
}

enum TypeLangValue<'a> {
	Type(JsonLdTypeSelection),
	Lang(JsonLdLangSelection<'a>),
}

/// Type that can be compacted with an index.
trait CompactIndexedFragment {
	#[allow(async_fn_in_trait)]
	#[allow(clippy::too_many_arguments)]
	async fn compact_indexed_fragment(
		&self,
		env: &impl AsyncProcessingEnvironment,
		compactor: &Compactor<'_>,
		index: Option<&str>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError>;
}

impl<T: CompactIndexedFragment> CompactFragment for JsonLdIndexed<T> {
	async fn compact_fragment(
		&self,
		env: &impl AsyncProcessingEnvironment,
		compactor: &Compactor<'_>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.inner()
			.compact_indexed_fragment(env, compactor, self.index(), location)
			.await
	}
}

/// Context embeding method.
///
/// This trait provides the `embed_context` method that can be used
/// to include a JSON-LD context to a JSON-LD document.
/// It is used at the end of compaction algorithm to embed to
/// context used to compact the document into the compacted output.
pub trait EmbedContext {
	/// Embeds the given context into the document.
	fn embed_context(
		&mut self,
		context: &ProcessedJsonLdContext,
		options: JsonLdCompactionOptions,
	) -> Result<(), JsonLdLocatedError>;
}

impl EmbedContext for JsonValue {
	fn embed_context(
		&mut self,
		context: &ProcessedJsonLdContext,
		options: JsonLdCompactionOptions,
	) -> Result<(), JsonLdLocatedError> {
		let value = self.take();

		let obj = match value {
			JsonValue::Array(array) => {
				let mut obj = JsonObject::new();

				if !array.is_empty() {
					let compactor = Compactor {
						options,
						active_context: context,
						type_scoped_context: context,
						active_property: None,
					};

					// No meaningful source location available at this post-processing step.
					let key = compactor.compact_iri(
						&JsonLdTerm::Keyword(JsonLdKeyword::Graph),
						true,
						false,
						JsonLdBacktraceBuilder::Root,
					)?;

					obj.insert(key.unwrap(), array.into());
				}

				Some(obj)
			}
			JsonValue::Object(obj) => Some(obj),
			_null => None,
		};

		if let Some(mut obj) = obj {
			// let json_context = IntoJson::into_json(context.unprocessed().clone());
			let json_context = json_syntax::to_value(context.unprocessed()).unwrap();

			if !obj.is_empty()
				&& !json_context.is_null()
				&& !json_context.is_empty_array_or_object()
			{
				obj.push_front("@context", json_context);
			}

			*self = obj.into()
		};

		Ok(())
	}
}
