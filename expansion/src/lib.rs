//! This library implements the [JSON-LD expansion algorithm](https://www.w3.org/TR/json-ld-api/#expansion-algorithms)
//! for the [`json-ld` crate](https://crates.io/crates/json-ld).
//!
//! # Usage
//!
//! The expansion algorithm is provided by the [`Expand`] trait.
use std::hash::Hash;

use futures::future::{BoxFuture, FutureExt};
use json_ld_context_processing::Context;
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

/// Handler for the possible warnings emmited during the expansion
/// of a JSON-LD document.
pub trait WarningHandler<B, N: BlankIdVocabulary<BlankId = B>, M>:
	json_ld_core::warning::Handler<N, Meta<Warning<B>, M>>
{
}

impl<B, N: BlankIdVocabulary<BlankId = B>, M, H> WarningHandler<B, N, M> for H where
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
///   r##"
///   {
///     "@graph": [
///       {
///         "http://example.org/vocab#a": {
///           "@graph": [
///             {
///               "http://example.org/vocab#b": "Chapter One"
///             }
///           ]
///         }
///       }
///     ]
///   }
///   "##,
///   |span| span, // the metadata only consists of the `span`.
/// )
/// .unwrap();
///
/// // Prepare a dummy document loader using [`json_ld::NoLoader`],
/// // since we won't need to load any remote document while expanding this one.
/// let mut loader: json_ld::NoLoader<IriBuf, Span, json_ld::syntax::Value<Span>> =
///   json_ld::NoLoader::new();
///
/// // The `expand` method returns an [`json_ld::ExpandedDocument`] (with the metadata).
/// let _: Meta<json_ld::ExpandedDocument<IriBuf, BlankIdBuf, _>, _> =
///   json
///     .expand(&mut loader)
///     .await
///     .unwrap();
/// # }
/// ```
pub trait Expand<T, B, M> {
	/// Returns the default base URL passed to the expansion algorithm
	/// and used to initialize the default empty context when calling
	/// [`Expand::expand`] or [`Expand::expand_with`].
	fn default_base_url(&self) -> Option<&T>;

	/// Expand the document with full options.
	///
	/// The `vocabulary` is used to interpret identifiers.
	/// The `context` is used as initial context.
	/// The `base_url` is the initial base URL used to resolve relative IRI references.
	/// The given `loader` is used to load remote documents (such as contexts)
	/// imported by the input and required during expansion.
	/// The `options` are used to tweak the expansion algorithm.
	/// The `warning_handler` is called each time a warning is emitted during expansion.
	fn expand_full<'a, N, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<T, B, M>,
		base_url: Option<&'a T>,
		loader: &'a mut L,
		options: Options,
		warnings_handler: impl 'a + Send + WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::ContextError: Send;

	/// Expand the input JSON-LD document with the given `vocabulary`
	/// to interpret identifiers.
	///
	/// The given `loader` is used to load remote documents (such as contexts)
	/// imported by the input and required during expansion.
	/// The expansion algorithm is called with an empty initial context with
	/// a base URL given by [`Expand::default_base_url`].
	fn expand_with<'a, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut (impl Send + Sync + VocabularyMut<Iri = T, BlankId = B>),
		loader: &'a mut L,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		T: 'a + Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::ContextError: Send,
	{
		self.expand_full(
			vocabulary,
			Context::<T, B, M>::new(self.default_base_url().cloned()),
			self.default_base_url(),
			loader,
			Options::default(),
			(),
		)
	}

	/// Expand the input JSON-LD document.
	///
	/// The given `loader` is used to load remote documents (such as contexts)
	/// imported by the input and required during expansion.
	/// The expansion algorithm is called with an empty initial context with
	/// a base URL given by [`Expand::default_base_url`].
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
		L::ContextError: Send,
		(): VocabularyMut<Iri = T, BlankId = B>,
	{
		self.expand_with(vocabulary::no_vocabulary_mut(), loader)
	}
}

/// Value expansion without base URL.
impl<T, B, M> Expand<T, B, M> for Meta<Value<M>, M> {
	fn default_base_url(&self) -> Option<&T> {
		None
	}

	fn expand_full<'a, N, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<T, B, M>,
		base_url: Option<&'a T>,
		loader: &'a mut L,
		options: Options,
		warnings_handler: impl 'a + Send + WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::ContextError: Send,
	{
		async move {
			document::expand(
				vocabulary,
				self,
				context,
				base_url,
				loader,
				options,
				warnings_handler,
			)
			.await
		}
		.boxed()
	}
}

/// Remote document expansion.
///
/// The default base URL given to the expansion algorithm is the URL of
/// the remote document.
impl<T, B, M> Expand<T, B, M> for RemoteDocument<T, M, Value<M>> {
	fn default_base_url(&self) -> Option<&T> {
		self.url()
	}

	fn expand_full<'a, N, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		vocabulary: &'a mut N,
		context: Context<T, B, M>,
		base_url: Option<&'a T>,
		loader: &'a mut L,
		options: Options,
		warnings_handler: impl 'a + Send + WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::ContextError: Send,
	{
		self.document().expand_full(
			vocabulary,
			context,
			base_url,
			loader,
			options,
			warnings_handler,
		)
	}
}
