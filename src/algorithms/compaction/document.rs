use json_syntax::JsonValue;

use crate::{
	ExpandedDocument, FlattenedDocument, ProcessedContext,
	algorithms::{
		AsyncProcessingEnvironment, JsonLdLocatedError, JsonLdSourceRef,
		compaction::CompactFragment,
	},
};

use super::{Compact, CompactionOptions, Compactor, EmbedContext};

impl ExpandedDocument {
	/// Compacts the input document with the given options.
	pub async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
	) -> Result<JsonValue, JsonLdLocatedError> {
		let compactor = Compactor::new(context, options, JsonLdSourceRef::Expanded(None));

		let mut compact = self.objects().compact_fragment(&env, &compactor).await?;

		compact.embed_context(context, options)?;

		Ok(compact)
	}

	/// Compacts the input document with the default options.
	pub async fn compact(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedContext<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.compact_with(env, context, CompactionOptions::default())
			.await
	}
}

impl Compact for ExpandedDocument {
	async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.compact_with(env, context, options).await
	}
}

impl Compact for FlattenedDocument {
	async fn compact_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
	) -> Result<JsonValue, JsonLdLocatedError> {
		let compactor = Compactor::new(context, options, JsonLdSourceRef::Expanded(None));

		let mut compact = self.compact_fragment(&env, &compactor).await?;

		compact.embed_context(context, options)?;

		Ok(compact)
	}
}
