use futures::FutureExt;
use json_ld_core::{ExpandedDocument, FlattenedDocument, Term};
use json_ld_syntax::{IntoJson, Keyword};
use locspan::Meta;
use rdf_types::Vocabulary;
use std::hash::Hash;

use crate::{
	iri::{compact_iri, IriConfusedWithPrefix},
	CompactFragmentMeta,
};

pub trait EmbedContext<I, B, C, M> {
	fn embed_context<N>(
		&mut self,
		vocabulary: &N,
		context: json_ld_context_processing::ProcessedRef<I, B, C, M>,
		options: crate::Options,
	) -> Result<(), Meta<IriConfusedWithPrefix, M>>
	where
		N: Vocabulary<I, B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		M: Clone,
		C: Clone + IntoJson<M>;
}

pub trait CompactMeta<I, B, M> {
	fn compact_full_meta<
		'a,
		N,
		C,
		L: json_ld_core::Loader<I, M> + json_ld_context_processing::ContextLoader<I, M>,
	>(
		&'a self,
		meta: &'a M,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B, C, M>,
		loader: &'a mut L,
		options: crate::Options,
	) -> futures::future::BoxFuture<
		'a,
		Result<json_syntax::MetaValue<M>, crate::MetaError<M, L::ContextError>>,
	>
	where
		N: Send + Sync + rdf_types::VocabularyMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: json_ld_context_processing::ProcessMeta<I, B, M>,
		L: Send + Sync,
		L::Context: Into<C>;
}

pub trait Compact<I, B, M> {
	fn compact_full<
		'a,
		N,
		C,
		L: json_ld_core::Loader<I, M> + json_ld_context_processing::ContextLoader<I, M>,
	>(
		&'a self,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B, C, M>,
		loader: &'a mut L,
		options: crate::Options,
	) -> futures::future::BoxFuture<
		'a,
		Result<json_syntax::MetaValue<M>, crate::MetaError<M, L::ContextError>>,
	>
	where
		N: Send + Sync + rdf_types::VocabularyMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: json_ld_context_processing::ProcessMeta<I, B, M>,
		L: Send + Sync,
		L::Context: Into<C>;
}

impl<T: CompactMeta<I, B, M>, I, B, M> Compact<I, B, M> for Meta<T, M> {
	fn compact_full<
		'a,
		N,
		C,
		L: json_ld_core::Loader<I, M> + json_ld_context_processing::ContextLoader<I, M>,
	>(
		&'a self,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B, C, M>,
		loader: &'a mut L,
		options: crate::Options,
	) -> futures::future::BoxFuture<
		'a,
		Result<json_syntax::MetaValue<M>, crate::MetaError<M, L::ContextError>>,
	>
	where
		N: Send + Sync + rdf_types::VocabularyMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: json_ld_context_processing::ProcessMeta<I, B, M>,
		L: Send + Sync,
		L::Context: Into<C>,
	{
		self.value()
			.compact_full_meta(self.metadata(), vocabulary, context, loader, options)
	}
}

impl<I, B, M> CompactMeta<I, B, M> for ExpandedDocument<I, B, M> {
	fn compact_full_meta<
		'a,
		N,
		C,
		L: json_ld_core::Loader<I, M> + json_ld_context_processing::ContextLoader<I, M>,
	>(
		&'a self,
		meta: &'a M,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B, C, M>,
		loader: &'a mut L,
		options: crate::Options,
	) -> futures::future::BoxFuture<
		'a,
		Result<json_syntax::MetaValue<M>, crate::MetaError<M, L::ContextError>>,
	>
	where
		N: Send + Sync + rdf_types::VocabularyMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: Clone
			+ json_ld_context_processing::ProcessMeta<I, B, M>
			+ json_ld_syntax::context::AnyValue<M>,
		L: Send + Sync,
		L::Context: Into<C>,
	{
		async move {
			let mut compacted_output = self
				.objects()
				.compact_fragment_full_meta(
					meta,
					vocabulary,
					context.processed(),
					context.processed(),
					None,
					loader,
					options,
				)
				.await?;

			compacted_output
				.embed_context(vocabulary, context, options)
				.map_err(Meta::cast)?;

			Ok(compacted_output)
		}
		.boxed()
	}
}

impl<I, B, M> CompactMeta<I, B, M> for FlattenedDocument<I, B, M> {
	fn compact_full_meta<
		'a,
		N,
		C,
		L: json_ld_core::Loader<I, M> + json_ld_context_processing::ContextLoader<I, M>,
	>(
		&'a self,
		meta: &'a M,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B, C, M>,
		loader: &'a mut L,
		options: crate::Options,
	) -> futures::future::BoxFuture<
		'a,
		Result<json_syntax::MetaValue<M>, crate::MetaError<M, L::ContextError>>,
	>
	where
		N: Send + Sync + rdf_types::VocabularyMut<I, B>,
		I: Clone + Hash + Eq + Send + Sync,
		B: Clone + Hash + Eq + Send + Sync,
		M: Clone + Send + Sync,
		C: Clone
			+ json_ld_context_processing::ProcessMeta<I, B, M>
			+ json_ld_syntax::context::AnyValue<M>,
		L: Send + Sync,
		L::Context: Into<C>,
	{
		async move {
			let mut compacted_output = self
				.compact_fragment_full_meta(
					meta,
					vocabulary,
					context.processed(),
					context.processed(),
					None,
					loader,
					options,
				)
				.await?;

			compacted_output
				.embed_context(vocabulary, context, options)
				.map_err(Meta::cast)?;

			Ok(compacted_output)
		}
		.boxed()
	}
}

impl<I, B, C, M> EmbedContext<I, B, C, M> for json_syntax::MetaValue<M> {
	fn embed_context<N>(
		&mut self,
		vocabulary: &N,
		context: json_ld_context_processing::ProcessedRef<I, B, C, M>,
		options: crate::Options,
	) -> Result<(), Meta<IriConfusedWithPrefix, M>>
	where
		N: Vocabulary<I, B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		M: Clone,
		C: Clone + IntoJson<M>,
	{
		let value = self.value_mut().take();

		let obj = match value {
			json_syntax::Value::Array(array) => {
				let mut obj = json_syntax::Object::new();

				if !array.is_empty() {
					let key = compact_iri(
						vocabulary,
						context.processed(),
						Meta(&Term::Keyword(Keyword::Graph), self.metadata()),
						true,
						false,
						options,
					)
					.map_err(Meta::cast)?;

					obj.insert(
						key.unwrap().cast(),
						Meta(array.into(), self.metadata().clone()),
					);
				}

				Some(obj)
			}
			json_syntax::Value::Object(obj) => Some(obj),
			_null => None,
		};

		if let Some(mut obj) = obj {
			let json_context = IntoJson::into_json(context.unprocessed().cloned());

			if !obj.is_empty() && !json_context.is_null() {
				obj.insert(
					Meta("@context".into(), json_context.metadata().clone()),
					json_context,
				);
			}

			*self.value_mut() = obj.into()
		};

		Ok(())
	}
}
