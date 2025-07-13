use crate::{
	algorithms::{compaction::CompactFragment, ProcessingEnvironment},
	Error, ExpandedDocument, FlattenedDocument, ProcessedContext,
};

use super::{Compact, CompactionOptions, Compactor, EmbedContext};

impl ExpandedDocument {
	/// Compacts the input document with the given options.
	pub async fn compact_with(
		&self,
		mut env: impl ProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
	) -> Result<json_syntax::Value, Error> {
		let compactor = Compactor::new(context, options);

		let mut compact = self
			.objects()
			.compact_fragment(&mut env, &compactor)
			.await?;

		compact.embed_context(context, options)?;

		Ok(compact)
	}

	/// Compacts the input document with the default options.
	pub async fn compact(
		&self,
		env: impl ProcessingEnvironment,
		context: &ProcessedContext<'_>,
	) -> Result<json_syntax::Value, Error> {
		self.compact_with(env, context, CompactionOptions::default())
			.await
	}
}

impl Compact for ExpandedDocument {
	async fn compact_with(
		&self,
		env: impl ProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
	) -> Result<json_syntax::Value, Error> {
		self.compact_with(env, context, options).await
	}
}

impl Compact for FlattenedDocument {
	async fn compact_with(
		&self,
		mut env: impl ProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
	) -> Result<json_syntax::Value, Error> {
		let compactor = Compactor::new(context, options);

		let mut compact = self.compact_fragment(&mut env, &compactor).await?;

		compact.embed_context(context, options)?;

		Ok(compact)
	}
}
