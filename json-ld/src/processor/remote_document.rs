use super::{
	compact_expanded_full, CompactError, CompactResult, CompareResult, ExpandError, ExpandResult,
	FlattenError, FlattenResult, JsonLdProcessor, Options,
};
use crate::context_processing::{self, Process};
use crate::expansion::{self, Expand};
use crate::syntax;
use crate::{
	id::Generator, Context, ContextLoader, Flatten, Loader, RemoteDocument, RemoteDocumentReference,
};
use contextual::WithContext;
use futures::future::{BoxFuture, FutureExt};
use json_ld_core::RemoteContextReference;
use locspan::BorrowStripped;
use rdf_types::VocabularyMut;
use std::hash::Hash;

impl<I, M> JsonLdProcessor<I, M> for RemoteDocument<I, M, json_syntax::Value<M>> {
	fn compare_full<'a, B, N, L>(
		&'a self,
		other: &'a Self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<CompareResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		async move {
			if json_ld_syntax::Compare::compare(self.document(), other.document()) {
				let a = JsonLdProcessor::expand_full(
					self,
					vocabulary,
					loader,
					options.clone(),
					&mut warnings,
				)
				.await?;
				let b =
					JsonLdProcessor::expand_full(other, vocabulary, loader, options, &mut warnings)
						.await?;
				Ok(a.stripped() == b.stripped())
			} else {
				Ok(false)
			}
		}
		.boxed()
	}

	fn expand_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		mut options: Options<I, M>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		async move {
			let mut active_context =
				Context::new(options.base.clone().or_else(|| self.url().cloned()));

			if let Some(expand_context) = options.expand_context.take() {
				active_context = expand_context
					.load_context_with(vocabulary, loader)
					.await
					.map_err(ExpandError::ContextLoading)?
					.into_document()
					.process_full(
						vocabulary,
						&active_context,
						loader,
						active_context.original_base_url().cloned(),
						options.context_processing_options(),
						&mut warnings,
					)
					.await
					.map_err(ExpandError::ContextProcessing)?
					.into_processed()
			};

			if let Some(context_url) = self.context_url() {
				active_context = RemoteDocumentReference::Iri(context_url.clone())
					.load_context_with(vocabulary, loader)
					.await
					.map_err(ExpandError::ContextLoading)?
					.into_document()
					.process_full(
						vocabulary,
						&active_context,
						loader,
						Some(context_url.clone()),
						options.context_processing_options(),
						&mut warnings,
					)
					.await
					.map_err(ExpandError::ContextProcessing)?
					.into_processed()
			}

			self.document()
				.expand_full(
					vocabulary,
					active_context,
					self.url().or(options.base.as_ref()),
					loader,
					options.expansion_options(),
					warnings,
				)
				.await
				.map_err(ExpandError::Expansion)
		}
		.boxed()
	}

	fn compact_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteContextReference<I, M>,
		loader: &'a mut L,
		options: Options<I, M>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		async move {
			let expanded_input = JsonLdProcessor::expand_full(
				self,
				vocabulary,
				loader,
				options.clone().unordered(),
				&mut warnings,
			)
			.await
			.map_err(CompactError::Expand)?;

			compact_expanded_full(
				&expanded_input,
				self.url(),
				vocabulary,
				context,
				loader,
				options,
				warnings,
			)
			.await
		}
		.boxed()
	}

	fn flatten_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut (impl Send + Generator<N, M>),
		context: Option<RemoteContextReference<I, M>>,
		loader: &'a mut L,
		options: Options<I, M>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, FlattenResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		async move {
			let expanded_input = JsonLdProcessor::expand_full(
				self,
				vocabulary,
				loader,
				options.clone().unordered(),
				&mut warnings,
			)
			.await
			.map_err(FlattenError::Expand)?;

			let flattened_output =
				Flatten::flatten_with(expanded_input, vocabulary, generator, options.ordered)
					.map_err(FlattenError::ConflictingIndexes)?;

			match context {
				Some(context) => compact_expanded_full(
					&flattened_output,
					self.url(),
					vocabulary,
					context,
					loader,
					options,
					warnings,
				)
				.await
				.map_err(FlattenError::Compact),
				None => Ok(json_ld_syntax::IntoJson::into_json(
					flattened_output.into_with(vocabulary),
				)),
			}
		}
		.boxed()
	}
}

impl<I, M> JsonLdProcessor<I, M> for RemoteDocumentReference<I, M, json_syntax::Value<M>> {
	fn compare_full<'a, B, N, L>(
		&'a self,
		other: &'a Self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<CompareResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		async move {
			let a = self
				.loaded_with(vocabulary, loader)
				.await
				.map_err(ExpandError::Loading)?;
			let b = other
				.loaded_with(vocabulary, loader)
				.await
				.map_err(ExpandError::Loading)?;
			JsonLdProcessor::compare_full(
				a.as_ref(),
				b.as_ref(),
				vocabulary,
				loader,
				options,
				warnings,
			)
			.await
		}
		.boxed()
	}

	fn expand_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		async move {
			let doc = self
				.loaded_with(vocabulary, loader)
				.await
				.map_err(ExpandError::Loading)?;
			JsonLdProcessor::expand_full(doc.as_ref(), vocabulary, loader, options, warnings).await
		}
		.boxed()
	}

	fn compact_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteContextReference<I, M>,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, CompactResult<I, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		async move {
			let doc = self
				.loaded_with(vocabulary, loader)
				.await
				.map_err(CompactError::Loading)?;
			JsonLdProcessor::compact_full(
				doc.as_ref(),
				vocabulary,
				context,
				loader,
				options,
				warnings,
			)
			.await
		}
		.boxed()
	}

	fn flatten_full<'a, B, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut (impl Send + Generator<N, M>),
		context: Option<RemoteContextReference<I, M>>,
		loader: &'a mut L,
		options: Options<I, M>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, FlattenResult<I, B, M, L>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		N: Send + Sync + VocabularyMut<Iri = I, BlankId = B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::ContextError: Send,
	{
		async move {
			let doc = self
				.loaded_with(vocabulary, loader)
				.await
				.map_err(FlattenError::Loading)?;
			JsonLdProcessor::flatten_full(
				doc.as_ref(),
				vocabulary,
				generator,
				context,
				loader,
				options,
				warnings,
			)
			.await
		}
		.boxed()
	}
}
