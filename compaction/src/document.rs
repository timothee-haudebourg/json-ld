use json_ld_core::{
	future::{BoxFuture, FutureExt},
	ExpandedDocument, FlattenedDocument, Loader, Term,
};
use json_ld_syntax::{IntoJson, Keyword};
use rdf_types::{vocabulary, Vocabulary};
use std::hash::Hash;

use crate::{
	iri::{compact_iri, IriConfusedWithPrefix},
	CompactFragment,
};

pub type CompactDocumentResult<I, L> =
	Result<json_syntax::Value, crate::Error<<L as Loader<I>>::Error>>;

/// Context embeding method.
///
/// This trait provides the `embed_context` method that can be used
/// to include a JSON-LD context to a JSON-LD document.
/// It is used at the end of compaction algorithm to embed to
/// context used to compact the document into the compacted output.
pub trait EmbedContext {
	/// Embeds the given context into the document.
	fn embed_context<N>(
		&mut self,
		vocabulary: &N,
		context: json_ld_context_processing::ProcessedRef<N::Iri, N::BlankId>,
		options: crate::Options,
	) -> Result<(), IriConfusedWithPrefix>
	where
		N: Vocabulary,
		N::Iri: Clone + Hash + Eq,
		N::BlankId: Clone + Hash + Eq;
}

/// Compaction function.
pub trait Compact<I, B> {
	/// Compacts the input document with full options.
	fn compact_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B>,
		loader: &'a mut L,
		options: crate::Options,
	) -> BoxFuture<'a, CompactDocumentResult<I, L>>
	where
		N: rdf_types::VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader<I>,
		//
		N: Send + Sync,
		I: Send + Sync,
		B: Send + Sync,
		L: Send + Sync,
		L::Error: Send;

	/// Compacts the input document with the given `vocabulary` to
	/// interpret identifiers.
	fn compact_with<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B>,
		loader: &'a mut L,
	) -> BoxFuture<'a, CompactDocumentResult<I, L>>
	where
		N: rdf_types::VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader<I>,
		//
		N: Send + Sync,
		I: Send + Sync,
		B: Send + Sync,
		L: Send + Sync,
		L::Error: Send,
	{
		self.compact_full(vocabulary, context, loader, crate::Options::default())
	}

	/// Compacts the input document.
	fn compact<'a, L>(
		&'a self,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B>,
		loader: &'a mut L,
	) -> BoxFuture<'a, CompactDocumentResult<I, L>>
	where
		(): rdf_types::VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader<I>,
		//
		I: Send + Sync,
		B: Send + Sync,
		L: Send + Sync,
		L::Error: Send,
	{
		self.compact_with(vocabulary::no_vocabulary_mut(), context, loader)
	}
}

impl<I, B> Compact<I, B> for ExpandedDocument<I, B> {
	fn compact_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B>,
		loader: &'a mut L,
		options: crate::Options,
	) -> BoxFuture<'a, CompactDocumentResult<I, L>>
	where
		N: rdf_types::VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader<I>,
		//
		N: Send + Sync,
		I: Send + Sync,
		B: Send + Sync,
		L: Send + Sync,
		L::Error: Send,
	{
		async move {
			let mut compacted_output = self
				.objects()
				.compact_fragment_full(
					vocabulary,
					context.processed(),
					context.processed(),
					None,
					loader,
					options,
				)
				.await?;

			compacted_output.embed_context(vocabulary, context, options)?;

			Ok(compacted_output)
		}
		.boxed()
	}
}

impl<I, B> Compact<I, B> for FlattenedDocument<I, B> {
	fn compact_full<'a, N, L>(
		&'a self,
		vocabulary: &'a mut N,
		context: json_ld_context_processing::ProcessedRef<'a, 'a, I, B>,
		loader: &'a mut L,
		options: crate::Options,
	) -> BoxFuture<'a, CompactDocumentResult<I, L>>
	where
		N: rdf_types::VocabularyMut<Iri = I, BlankId = B>,
		I: Clone + Hash + Eq,
		B: Clone + Hash + Eq,
		L: Loader<I>,
		//
		N: Send + Sync,
		I: Send + Sync,
		B: Send + Sync,
		L: Send + Sync,
		L::Error: Send,
	{
		async move {
			let mut compacted_output = self
				.compact_fragment_full(
					vocabulary,
					context.processed(),
					context.processed(),
					None,
					loader,
					options,
				)
				.await?;

			compacted_output.embed_context(vocabulary, context, options)?;

			Ok(compacted_output)
		}
		.boxed()
	}
}

impl EmbedContext for json_syntax::Value {
	fn embed_context<N>(
		&mut self,
		vocabulary: &N,
		context: json_ld_context_processing::ProcessedRef<N::Iri, N::BlankId>,
		options: crate::Options,
	) -> Result<(), IriConfusedWithPrefix>
	where
		N: Vocabulary,
		N::Iri: Clone + Hash + Eq,
		N::BlankId: Clone + Hash + Eq,
	{
		let value = self.take();

		let obj = match value {
			json_syntax::Value::Array(array) => {
				let mut obj = json_syntax::Object::new();

				if !array.is_empty() {
					let key = compact_iri(
						vocabulary,
						context.processed(),
						&Term::Keyword(Keyword::Graph),
						true,
						false,
						options,
					)?;

					obj.insert(key.unwrap().into(), array.into());
				}

				Some(obj)
			}
			json_syntax::Value::Object(obj) => Some(obj),
			_null => None,
		};

		if let Some(mut obj) = obj {
			let json_context = IntoJson::into_json(context.unprocessed().clone());

			if !obj.is_empty()
				&& !json_context.is_null()
				&& !json_context.is_empty_array_or_object()
			{
				obj.insert_front("@context".into(), json_context);
			}

			*self = obj.into()
		};

		Ok(())
	}
}
