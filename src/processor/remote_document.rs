use json_syntax::{JsonValue, tracing::JsonErrorAt};
use rdf_syntax::Iri;

use super::{
	CompactResult, CompareResult, ExpandResult, FlattenResult, JsonLdOptions, JsonLdProcessor,
};
use crate::{
	Document, RemoteContext,
	algorithms::{
		AsyncProcessingEnvironment, Compact, Expand, JsonLdLocatedError, JsonLdLocationStack,
		JsonLdSourceRef,
	},
	context::RawProcessedContext,
	syntax::JsonLdCompare,
};

impl JsonLdProcessor for Document {
	async fn async_compare_with(
		&self,
		other: &Self,
		env: impl AsyncProcessingEnvironment,
		options: JsonLdOptions,
	) -> CompareResult {
		if self.document.compare_json_ld(&other.document) {
			return Ok(true);
		}

		let a = JsonLdProcessor::async_expand_with(self, env.as_ref(), options.clone()).await?;
		let b = JsonLdProcessor::async_expand_with(other, env, options).await?;
		Ok(a == b)
	}

	async fn async_expand_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		mut options: JsonLdOptions,
	) -> ExpandResult {
		// Initialize the active context.
		let mut active_context =
			RawProcessedContext::new(options.base.clone().or_else(|| self.url.clone()));

		// Process expand context if provided.
		if let Some(expand_context) = options.expand_context.take() {
			let context_document = expand_context
				.load(env.loader())
				.await
				.map_err(Into::into)
				.json_err_at(JsonLdLocationStack::Root)?;

			active_context = context_document
				.document
				.context
				.process_with(
					env.as_ref(),
					active_context.original_base_url(),
					&active_context,
					options.context_processing_options(),
					JsonLdLocationStack::new()
						.file(JsonLdSourceRef::Context(context_document.url())),
				)
				.await?
				.into_raw();
		}

		// Process context URL from the loaded document, if any.
		if let Some(context_url) = self.context_url() {
			active_context = RemoteContext::iri(context_url.to_owned())
				.load(env.loader())
				.await
				.map_err(Into::into)
				.json_err_at(JsonLdLocationStack::Root)?
				.document
				.context
				.process_with(
					env.as_ref(),
					Some(context_url),
					&active_context,
					options.context_processing_options(),
					JsonLdLocationStack::new().file(JsonLdSourceRef::Context(Some(context_url))),
				)
				.await?
				.into_raw()
		}

		// Expand the document.
		Expand::expand_with(self, env, &active_context, options.expansion_options()).await
	}

	async fn async_compact_with(
		&self,
		context: RemoteContext,
		env: impl AsyncProcessingEnvironment,
		options: JsonLdOptions,
	) -> CompactResult {
		compact_expanded(
			JsonLdProcessor::async_expand_with(self, env.as_ref(), options.clone().unordered())
				.await?,
			self.url(),
			env,
			context,
			options,
		)
		.await
	}

	async fn async_flatten_with(
		&self,
		context: Option<RemoteContext>,
		env: impl AsyncProcessingEnvironment,
		options: JsonLdOptions,
	) -> FlattenResult {
		let expanded_input =
			JsonLdProcessor::async_expand_with(self, env.as_ref(), options.clone().unordered())
				.await?;

		let generator = rdf_syntax::generator::BlankIdGenerator::new_with_prefix("b".to_string());
		let flattened_output = expanded_input
			.flatten(
				generator,
				options.ordered,
				JsonLdLocationStack::new().file(JsonLdSourceRef::Expanded(self.url())),
			)
			.map_err(|e| (*e).cast())?;

		match context {
			Some(context) => {
				compact_expanded(flattened_output, self.url(), env, context, options).await
			}
			None => Ok(json_syntax::to_value(flattened_output).unwrap()),
		}
	}
}

async fn compact_expanded(
	expanded_input: impl Compact,
	url: Option<&Iri>,
	env: impl AsyncProcessingEnvironment,
	context: RemoteContext,
	options: JsonLdOptions,
) -> Result<JsonValue, JsonLdLocatedError> {
	let context_base = url.or(options.base.as_deref());

	let context = context
		.load(env.loader())
		.await
		.map_err(Into::into)
		.json_err_at(JsonLdLocationStack::Root)?;
	let mut active_context = context
		.document
		.context
		.process_with(
			env.as_ref(),
			context_base,
			&RawProcessedContext::new(None),
			options.context_processing_options(),
			JsonLdLocationStack::new().file(JsonLdSourceRef::Expanded(url)),
		)
		.await?;

	match options.base.as_ref() {
		Some(base) => active_context.set_base_iri(Some(base.clone())),
		None => {
			if options.compact_to_relative && active_context.base_iri().is_none() {
				active_context.set_base_iri(url.map(ToOwned::to_owned));
			}
		}
	}

	expanded_input
		.compact_with(env, &active_context, options.compaction_options())
		.await
}
