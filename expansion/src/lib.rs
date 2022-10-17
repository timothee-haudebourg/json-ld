/// This library implements the [JSON-LD expansion algorithm](https://www.w3.org/TR/json-ld-api/#expansion-algorithms)
/// for the [`json-ld` crate](https://crates.io/crates/json-ld).
/// 
/// # Usage
/// 
/// The expansion algorithm is provided by the [`Expand`] trait.
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

pub trait WarningHandler<B, N: BlankIdVocabulary<BlankId=B>, M>:
	json_ld_core::warning::Handler<N, Meta<Warning<B>, M>>
{
}

impl<B, N: BlankIdVocabulary<BlankId=B>, M, H> WarningHandler<B, N, M> for H where
	H: json_ld_core::warning::Handler<N, Meta<Warning<B>, M>>
{
}

/// Document expansion.
/// 
/// This trait provides the functions necessary to expand
/// a JSON-LD document into an [`ExpandedDocument`].
/// It is implemented by [`json_syntax::MetaValue`] representing
/// a JSON object (ith its metadata) and [`RemoteDocument`].
/// 
/// # Example
/// 
/// ```
/// # mod json_ld { pub use json_ld_syntax as syntax; pub use json_ld_core::{RemoteDocument, ExpandedDocument, NoLoader}; pub use json_ld_expansion::Expand; };
/// 
/// use iref::IriBuf;
/// use rdf_types::BlankIdBuf;
/// use static_iref::iri;
/// use locspan::{Meta, Span};
/// use json_ld::{syntax::Parse, RemoteDocument, Expand};
/// 
/// # #[async_std::test]
/// # async fn example() {
/// // Parse the input JSON(-LD) document.
/// // Each fragment of the parsed value will be annotated (the metadata) with its
/// // [`Span`] in the input text.
/// let json = json_ld::syntax::Value::parse_str(
/// 	r##"
/// 	{
/// 		"@graph": [
/// 			{
/// 				"http://example.org/vocab#a": {
/// 					"@graph": [
/// 						{
/// 							"http://example.org/vocab#b": "Chapter One"
/// 						}
/// 					]
/// 				}
/// 			}
/// 		]
/// 	}
/// 	"##,
/// 	|span| span, // the metadata only consists of the `span`.
/// )
/// .unwrap();
/// 
/// // Prepare a dummy document loader using [`json_ld::NoLoader`],
/// // since we won't need to load any remote document while expanding this one.
/// let mut loader: json_ld::NoLoader<IriBuf, Span, json_ld::syntax::Value<Span>> =
/// 	json_ld::NoLoader::new();
/// 
/// // The `expand` method returns an [`json_ld::ExpandedDocument`] (with the metadata).
/// let _: Meta<json_ld::ExpandedDocument<IriBuf, BlankIdBuf, _>, _> =
/// 	json
/// 		.expand(&mut loader)
/// 		.await
/// 		.unwrap();
/// # }
/// ```
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
		N: Send + Sync + VocabularyMut<Iri=T, BlankId=B>,
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
		vocabulary: &'a mut (impl Send + Sync + VocabularyMut<Iri=T, BlankId=B>),
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
		(): VocabularyMut<Iri=T, BlankId=B>,
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
		N: Send + Sync + VocabularyMut<Iri=T, BlankId=B>,
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

impl<T, B, M> Expand<T, B, M> for RemoteDocument<T, M, Value<M>> {
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
		N: Send + Sync + VocabularyMut<Iri=T, BlankId=B>,
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
		vocabulary: &'a mut (impl Send + Sync + VocabularyMut<Iri=T, BlankId=B>),
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
