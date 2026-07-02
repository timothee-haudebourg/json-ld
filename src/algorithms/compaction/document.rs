use json_syntax::JsonValue;

use crate::{
	ExpandedDocument, FlattenedDocument, ProcessedContext,
	algorithms::{
		AsyncProcessingEnvironment, JsonLdLocatedError, JsonLdLocationStack,
		compaction::CompactFragment,
	},
};

use super::{Compact, CompactionOptions, Compactor, EmbedContext};

impl ExpandedDocument {
	/// Compacts the input document with the given options, starting the location
	/// backtrace at `location`.
	pub async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
		location: JsonLdLocationStack<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		let compactor = Compactor::new(context, options);

		let mut compact = self
			.objects()
			.compact_fragment(&env, &compactor, location)
			.await?;

		compact.embed_context(context, options)?;

		Ok(compact)
	}

	/// Compacts the input document with the default options, starting the
	/// location backtrace at `location`.
	pub async fn compact(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedContext<'_>,
		location: JsonLdLocationStack<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.compact_with(env, context, CompactionOptions::default(), location)
			.await
	}
}

impl Compact for ExpandedDocument {
	async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
		location: JsonLdLocationStack<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.compact_with(env, context, options, location).await
	}
}

impl Compact for FlattenedDocument {
	async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
		location: JsonLdLocationStack<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		let compactor = Compactor::new(context, options);

		let mut compact = self.compact_fragment(&env, &compactor, location).await?;

		compact.embed_context(context, options)?;

		Ok(compact)
	}
}
