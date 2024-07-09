use super::{
	compact_expanded_full, CompactError, CompactResult, CompareResult, ExpandError, ExpandResult,
	FlattenError, FlattenResult, JsonLdProcessor, Options,
};
use crate::context_processing::{self, Process};
use crate::expansion::{self, Expand};
use crate::IntoDocumentResult;
use crate::{Context, Flatten, Loader, RemoteDocument, RemoteDocumentReference};
use contextual::WithContext;
use json_ld_core::{Document, RemoteContextReference};
use rdf_types::{Generator, VocabularyMut};
use std::hash::Hash;

impl<I> JsonLdProcessor<I> for RemoteDocument<I> {
	async fn compare_full<N>(
		&self,
		other: &Self,
		vocabulary: &mut N,
		loader: &impl Loader,
		options: Options<I>,
		mut warnings: impl context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> CompareResult
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
	{
		if json_ld_syntax::Compare::compare(self.document(), other.document()) {
			let a = JsonLdProcessor::expand_full(
				self,
				vocabulary,
				loader,
				options.clone(),
				&mut warnings,
			)
			.await?;
			let b = JsonLdProcessor::expand_full(other, vocabulary, loader, options, &mut warnings)
				.await?;
			Ok(a == b)
		} else {
			Ok(false)
		}
	}

	async fn expand_full<N>(
		&self,
		vocabulary: &mut N,
		loader: &impl Loader,
		mut options: Options<I>,
		mut warnings: impl context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> ExpandResult<I, N::BlankId>
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
	{
		let mut active_context = Context::new(options.base.clone().or_else(|| self.url().cloned()));

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

	async fn into_document_full<'a, N>(
		self,
		vocabulary: &'a mut N,
		loader: &'a impl Loader,
		options: Options<I>,
		warnings: impl 'a + context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> IntoDocumentResult<I, N::BlankId>
	where
		N: VocabularyMut<Iri = I>,
		I: 'a + Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
	{
		let expanded =
			JsonLdProcessor::expand_full(&self, vocabulary, loader, options, warnings).await?;
		Ok(Document::new(self, expanded))
	}

	async fn compact_full<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteContextReference<I>,
		loader: &'a impl Loader,
		options: Options<I>,
		mut warnings: impl 'a + context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> CompactResult
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
	{
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

	async fn flatten_full<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut impl Generator<N>,
		context: Option<RemoteContextReference<I>>,
		loader: &'a impl Loader,
		options: Options<I>,
		mut warnings: impl 'a + context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> FlattenResult<I, N::BlankId>
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
	{
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
}

impl<I> JsonLdProcessor<I> for RemoteDocumentReference<I, json_syntax::Value> {
	async fn compare_full<N>(
		&self,
		other: &Self,
		vocabulary: &mut N,
		loader: &impl Loader,
		options: Options<I>,
		warnings: impl context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> CompareResult
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
	{
		let a = self.loaded_with(vocabulary, loader).await?;
		let b = other.loaded_with(vocabulary, loader).await?;
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

	async fn expand_full<N>(
		&self,
		vocabulary: &mut N,
		loader: &impl Loader,
		options: Options<I>,
		warnings: impl context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> ExpandResult<I, N::BlankId>
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: Clone + Eq + Hash,
	{
		let doc = self.loaded_with(vocabulary, loader).await?;
		JsonLdProcessor::expand_full(doc.as_ref(), vocabulary, loader, options, warnings).await
	}

	async fn into_document_full<'a, N>(
		self,
		vocabulary: &'a mut N,
		loader: &'a impl Loader,
		options: Options<I>,
		warnings: impl 'a + context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> IntoDocumentResult<I, N::BlankId>
	where
		N: VocabularyMut<Iri = I>,
		I: 'a + Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
	{
		let doc = self.load_with(vocabulary, loader).await?;
		JsonLdProcessor::into_document_full(doc, vocabulary, loader, options, warnings).await
	}

	async fn compact_full<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		context: RemoteContextReference<I>,
		loader: &'a impl Loader,
		options: Options<I>,
		warnings: impl 'a + context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> CompactResult
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
	{
		let doc = self.loaded_with(vocabulary, loader).await?;
		JsonLdProcessor::compact_full(doc.as_ref(), vocabulary, context, loader, options, warnings)
			.await
	}

	async fn flatten_full<'a, N>(
		&'a self,
		vocabulary: &'a mut N,
		generator: &'a mut impl Generator<N>,
		context: Option<RemoteContextReference<I>>,
		loader: &'a impl Loader,
		options: Options<I>,
		warnings: impl 'a + context_processing::WarningHandler<N> + expansion::WarningHandler<N>,
	) -> FlattenResult<I, N::BlankId>
	where
		N: VocabularyMut<Iri = I>,
		I: Clone + Eq + Hash,
		N::BlankId: 'a + Clone + Eq + Hash,
	{
		let doc = self.loaded_with(vocabulary, loader).await?;
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
}
