use std::hash::Hash;

use futures::future::{BoxFuture, FutureExt};
use json_ld_context_processing::{Context, ProcessMeta};
use json_ld_core::{ContextLoader, ExpandedDocument, Loader, RemoteDocument};
use json_syntax::Value;
use locspan::Meta;
use rdf_types::{vocabulary, BlankIdVocabulary, VocabularyMut};

mod array;
mod document;
mod element;
mod error;
mod expanded;
mod literal;
mod node;
mod options;
mod value;
mod warning;

pub use error::*;
pub use expanded::*;
pub use options::*;
pub use warning::*;

pub(crate) use array::*;
pub(crate) use document::filter_top_level_item;
pub(crate) use element::*;
pub(crate) use json_ld_context_processing::syntax::expand_iri_simple as expand_iri;
pub(crate) use literal::*;
pub(crate) use node::*;
pub(crate) use value::*;

/// Result of the document expansion.
pub type ExpansionResult<T, B, M, L> = Result<
	Meta<ExpandedDocument<T, B, M>, M>,
	Meta<Error<M, <L as ContextLoader<T, M>>::ContextError>, M>,
>;

pub trait WarningHandler<B, N: BlankIdVocabulary<B>, M>:
	json_ld_core::warning::Handler<N, Meta<Warning<B>, M>>
{
}

impl<B, N: BlankIdVocabulary<B>, M, H> WarningHandler<B, N, M> for H where
	H: json_ld_core::warning::Handler<N, Meta<Warning<B>, M>>
{
}

pub trait Expand<T, B, M> {
	fn expand_full<'a, N, C, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<T, B, C, M>,
		base_url: Option<&'a T>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		N: Send + Sync + VocabularyMut<T, B>,
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: Clone + Send + Sync,
		C: 'a + ProcessMeta<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send;

	fn expand_in<'a, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut (impl Send + Sync + VocabularyMut<T, B>),
		loader: &'a mut L,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		T: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: ProcessMeta<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L::ContextError: Send,
	{
		self.expand_full(
			vocabulary,
			Context::<T, B, L::Context, M>::new(None),
			None,
			loader,
			Options::default(),
			(),
		)
	}

	fn expand<'a, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		loader: &'a mut L,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		T: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: ProcessMeta<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L::ContextError: Send,
		(): VocabularyMut<T, B>,
	{
		self.expand_in(vocabulary::no_vocabulary_mut(), loader)
	}
}

impl<T, B, M> Expand<T, B, M> for Meta<Value<M>, M> {
	fn expand_full<'a, N, C, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<T, B, C, M>,
		base_url: Option<&'a T>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		N: Send + Sync + VocabularyMut<T, B>,
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		C: 'a + ProcessMeta<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			document::expand(
				vocabulary, self, context, base_url, loader, options, warnings,
			)
			.await
		}
		.boxed()
	}
}

impl<T, B, M> Expand<T, B, M> for RemoteDocument<T, Value<M>, M> {
	fn expand_full<'a, N, C, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<T, B, C, M>,
		base_url: Option<&'a T>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		N: Send + Sync + VocabularyMut<T, B>,
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		C: 'a + ProcessMeta<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		self.document()
			.expand_full(vocabulary, context, base_url, loader, options, warnings)
	}

	fn expand_in<'a, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut (impl Send + Sync + VocabularyMut<T, B>),
		loader: &'a mut L,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		T: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: ProcessMeta<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L::ContextError: Send,
	{
		self.document().expand_full(
			vocabulary,
			Context::<T, B, L::Context, M>::new(self.url().cloned()),
			self.url(),
			loader,
			Options::default(),
			(),
		)
	}
}
