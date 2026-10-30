use json_syntax::JsonValue;

use crate::{
	JsonLdBacktraceBuilder, JsonLdExpandedDocument, JsonLdFlattenedDocument, JsonLdLocatedError,
	ProcessedJsonLdContext,
	algorithms::{AsyncProcessingEnvironment, compaction::CompactFragment},
	context::JsonLdEmbedContext,
};

use super::{Compactor, JsonLdCompact, JsonLdCompactionOptions};

impl JsonLdExpandedDocument {
	/// Compacts the input document with the given options, starting the location
	/// backtrace at `location`.
	pub async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedJsonLdContext<'_>,
		options: JsonLdCompactionOptions,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		let compactor = Compactor::new(context, options);

		let compact = self
			.objects()
			.compact_fragment(&env, &compactor, location)
			.await?;

		let mut compact = compactor.wrap_in_graph(compact)?;
		compact.embed_context(context)?;

		Ok(compact)
	}

	/// Compacts the input document with the default options, starting the
	/// location backtrace at `location`.
	pub async fn compact(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedJsonLdContext<'_>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.compact_with(env, context, JsonLdCompactionOptions::default(), location)
			.await
	}
}

impl JsonLdCompact for JsonLdExpandedDocument {
	async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedJsonLdContext<'_>,
		options: JsonLdCompactionOptions,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.compact_with(env, context, options, location).await
	}
}

impl JsonLdCompact for JsonLdFlattenedDocument {
	async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedJsonLdContext<'_>,
		options: JsonLdCompactionOptions,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		let compactor = Compactor::new(context, options);

		let compact = self.compact_fragment(&env, &compactor, location).await?;

		let mut compact = compactor.wrap_in_graph(compact)?;
		compact.embed_context(context)?;

		Ok(compact)
	}
}
