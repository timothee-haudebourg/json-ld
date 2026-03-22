use iref::Iri;
use json_syntax::JsonValue;

use super::{CompactResult, CompareResult, ExpandResult, FlattenResult};
use super::{JsonLdOptions, JsonLdProcessor};
use crate::algorithms::Compact;
use crate::context::RawProcessedContext;
use crate::syntax::JsonLdCompare;
use crate::{algorithms::ProcessingEnvironment, RemoteContext};
use crate::{Document, Error};

impl JsonLdProcessor for Document {
	async fn compare_with(
		&self,
		other: &Self,
		env: impl ProcessingEnvironment,
		options: JsonLdOptions,
	) -> CompareResult {
		if self.document.compare_json_ld(&other.document) {
			let a = JsonLdProcessor::expand_with(self, env.as_ref(), options.clone()).await?;
			let b = JsonLdProcessor::expand_with(other, env, options).await?;
			Ok(a == b)
		} else {
			Ok(false)
		}
	}

	async fn expand_with(
		&self,
		env: impl ProcessingEnvironment,
		mut options: JsonLdOptions,
	) -> ExpandResult {
		// Initialize the active context.
		let mut active_context =
			RawProcessedContext::new(options.base.clone().or_else(|| self.url.clone()));

		// Process expand context if provided.
		if let Some(expand_context) = options.expand_context.take() {
			active_context = expand_context
				.load(env.loader())
				.await?
				.document
				.context
				.process_with(
					env.as_ref(),
					active_context.original_base_url(),
					&active_context,
					options.context_processing_options(),
				)
				.await?
				.into_raw();
		}

		// Process context URL from the loaded document, if any.
		if let Some(context_url) = self.context_url() {
			active_context = RemoteContext::iri(context_url.to_owned())
				.load(env.loader())
				.await?
				.document
				.context
				.process_with(
					env.as_ref(),
					Some(context_url),
					&active_context,
					options.context_processing_options(),
				)
				.await?
				.into_raw()
		}

		// Expand the document.
		self.expand_with(env, &active_context, options.expansion_options())
			.await
	}

	async fn compact_with(
		&self,
		context: RemoteContext,
		env: impl ProcessingEnvironment,
		options: JsonLdOptions,
	) -> CompactResult {
		compact_expanded(
			JsonLdProcessor::expand_with(self, env.as_ref(), options.clone().unordered()).await?,
			self.url(),
			env,
			context,
			options,
		)
		.await
	}

	async fn flatten_with(
		&self,
		_context: Option<RemoteContext>,
		_env: impl ProcessingEnvironment,
		_options: JsonLdOptions,
	) -> FlattenResult {
		// let expanded_input =
		// 	JsonLdProcessor::expand_with(self, env, options.clone().unordered()).await?;

		// let mut generator = rdf_types::generator::BlankIdGenerator::new();
		// let flattened_output = expanded_input.flatten(generator, options.ordered)?;

		// match context {
		// 	Some(context) => {
		// 		compact_expanded(flattened_output, self.url(), env, context, options).await
		// 	}
		// 	None => Ok(json_syntax::to_value(flattened_output).unwrap()),
		// }
		todo!()
	}

	async fn to_rdf_with<G>(
		&self,
		_env: impl ProcessingEnvironment,
		_generator: G,
		_options: JsonLdOptions,
	) {
		todo!()
	}
}

async fn compact_expanded(
	expanded_input: impl Compact,
	url: Option<&Iri>,
	env: impl ProcessingEnvironment,
	context: RemoteContext,
	options: JsonLdOptions,
) -> Result<JsonValue, Error> {
	let context_base = url.or(options.base.as_deref());

	let context = context.load(env.loader()).await?;
	let mut active_context = context
		.document
		.context
		.process_with(
			env.as_ref(),
			context_base,
			&RawProcessedContext::new(None),
			options.context_processing_options(),
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
