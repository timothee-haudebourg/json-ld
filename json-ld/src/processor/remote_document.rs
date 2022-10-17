use crate::{RemoteDocumentReference, Loader, ContextLoader, RemoteDocument, Context, id::Generator, Flatten};
use crate::syntax;
use crate::context_processing::{self, Process, ProcessMeta};
use crate::expansion::{self, Expand};
use super::{JsonLdProcessor, Options, ExpandResult, ExpandError, CompactError, FlattenError, compact_expanded_full};
use futures::future::{BoxFuture, FutureExt};
use locspan::BorrowStripped;
use rdf_types::VocabularyMut;
use contextual::WithContext;
use std::hash::Hash;

impl<I, M> JsonLdProcessor<I, M> for RemoteDocument<I, M, json_syntax::Value<M>> {
	fn compare_full<'a, B, C, N, L>(
		&'a self,
		other: &'a Self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M, C>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<Result<bool, ExpandError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send
	{
		async move {
			if json_ld_syntax::Compare::compare(self.document(), other.document()) {
				let a = JsonLdProcessor::expand_full(self, vocabulary, loader, options.clone(), &mut warnings).await?;
				let b = JsonLdProcessor::expand_full(other, vocabulary, loader, options, &mut warnings).await?;
				Ok(a.stripped() == b.stripped())
			} else {
				Ok(false)
			}
		}.boxed()
	}

	fn expand_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		mut options: Options<I, M, C>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L::Error, L::ContextError>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			let active_context = Context::new(options.base.clone().or_else(|| self.url().cloned()));

			let active_context = match options.expand_context.take() {
				Some(expand_context) => expand_context
					.load_context_with(vocabulary, loader).await.map_err(ExpandError::ContextLoading)?.into_document()
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
					.into_processed(),
				None => active_context,
			};

			// TODO remote contextUrl

			self.document()
				.expand_full(
					vocabulary,
					active_context,
					self.url().or_else(|| options.base.as_ref()),
					loader,
					options.expansion_options(),
					warnings,
				)
				.await.map_err(ExpandError::Expansion)
		}
		.boxed()
	}

	fn compact_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteDocumentReference<I, M, C>,
		loader: &'a mut L,
		options: Options<I, M, C>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
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

			compact_expanded_full(&expanded_input, self.url(), vocabulary, context, loader, options, warnings).await
		}
		.boxed()
	}

	fn flatten_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut (impl Send + Generator<I, B, M, N>),
		context: Option<RemoteDocumentReference<I, M, C>>,
		loader: &'a mut L,
		options: Options<I, M, C>,
		mut warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, FlattenError<I, B, M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send
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

			let flattened_output = Flatten::flatten_with(
				expanded_input,
				vocabulary,
				generator,
				options.ordered
			).map_err(FlattenError::ConflictingIndexes)?;

			match context {
				Some(context) => {
					compact_expanded_full(&flattened_output, self.url(), vocabulary, context, loader, options, warnings).await.map_err(FlattenError::Compact)
				}
				None => {
					Ok(json_ld_syntax::IntoJson::into_json(flattened_output.into_with(vocabulary)))
				}
			}
		}.boxed()
	}
}

impl<I, M> JsonLdProcessor<I, M> for RemoteDocumentReference<I, M, json_syntax::Value<M>> {
	fn compare_full<'a, B, C, N, L>(
		&'a self,
		other: &'a Self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<Result<bool, ExpandError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send
	{
		async move {
			let a = self.loaded_with(vocabulary, loader).await.map_err(ExpandError::Loading)?;
			let b = other.loaded_with(vocabulary, loader).await.map_err(ExpandError::Loading)?;
			JsonLdProcessor::compare_full(a.as_ref(), b.as_ref(), vocabulary, loader, options, warnings).await
		}
		.boxed()
	}

	fn expand_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<ExpandResult<I, B, M, L::Error, L::ContextError>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			let doc = self.loaded_with(vocabulary, loader).await.map_err(ExpandError::Loading)?;
			JsonLdProcessor::expand_full(doc.as_ref(), vocabulary, loader, options, warnings).await
		}
		.boxed()
	}

	fn compact_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteDocumentReference<I, M, C>,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, CompactError<M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			let doc = self.loaded_with(vocabulary, loader).await.map_err(CompactError::Loading)?;
			JsonLdProcessor::compact_full(doc.as_ref(), vocabulary, context, loader, options, warnings).await
		}
		.boxed()
	}

	fn flatten_full<'a, B, C, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut (impl Send + Generator<I, B, M, N>),
		context: Option<RemoteDocumentReference<I, M, C>>,
		loader: &'a mut L,
		options: Options<I, M, C>,
		warnings: impl 'a
			+ Send
			+ context_processing::WarningHandler<N, M>
			+ expansion::WarningHandler<B, N, M>,
	) -> BoxFuture<'a, Result<json_syntax::MetaValue<M>, FlattenError<I, B, M, L::Error, L::ContextError>>>
	where
		I: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		C: 'a + ProcessMeta<I, B, M> + From<json_ld_syntax::context::Value<M>>,
		N: Send + Sync + VocabularyMut<Iri=I, BlankId=B>,
		M: Clone + Send + Sync,
		L: Loader<I, M> + ContextLoader<I, M> + Send + Sync,
		L::Output: Into<syntax::Value<M>>,
		L::Error: Send,
		L::Context: Into<C>,
		L::ContextError: Send
	{
		async move {
			let doc = self.loaded_with(vocabulary, loader).await.map_err(FlattenError::Loading)?;
			JsonLdProcessor::flatten_full(doc.as_ref(), vocabulary, generator, context, loader, options, warnings).await
		}
		.boxed()
	}
}