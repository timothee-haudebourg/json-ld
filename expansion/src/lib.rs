use std::hash::Hash;

use futures::future::{BoxFuture, FutureExt};
use json_ld_context_processing::{Context, NamespaceMut, Process};
use json_ld_core::{
	BlankIdNamespace, BorrowWithNamespace, ContextLoader, ExpandedDocument, Loader,
};
use json_syntax::Value;
use locspan::Meta;

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
pub type ExpansionResult<T, B, M, L> =
	Result<ExpandedDocument<T, B, M>, Meta<Error<M, <L as ContextLoader<T, M>>::ContextError>, M>>;

fn print_warning<B, N: BlankIdNamespace<B>, M>(namespace: &N, warning: Meta<Warning<B>, M>) {
	eprintln!("{}", warning.value().with_namespace(namespace))
}

fn ignore_warning<B, N: BlankIdNamespace<B>, M>(_namespace: &N, _warning: Meta<Warning<B>, M>) {}

pub trait WarningHandler<B, N: BlankIdNamespace<B>, M>:
	json_ld_core::warning::Handler<N, Meta<Warning<B>, M>>
{
}

impl<B, N: BlankIdNamespace<B>, M, H> WarningHandler<B, N, M> for H where
	H: json_ld_core::warning::Handler<N, Meta<Warning<B>, M>>
{
}

pub trait Expand<T, B, M> {
	fn expand_full<'a, N, C, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		namespace: &'a mut N,
		context: Context<T, B, C>,
		base_url: Option<&'a T>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		N: Send + Sync + NamespaceMut<T, B>,
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: Clone + Send + Sync,
		C: 'a + Process<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send;

	fn expand_in<'a, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		namespace: &'a mut (impl Send + Sync + NamespaceMut<T, B>),
		base_url: Option<&'a T>,
		loader: &'a mut L,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: Process<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L::ContextError: Send,
	{
		self.expand_full(
			namespace,
			Context::<T, B, L::Context>::new(base_url.cloned()),
			base_url,
			loader,
			Options::default(),
			print_warning,
		)
	}

	fn expand<'a, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		base_url: Option<&'a T>,
		loader: &'a mut L,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: Process<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L::ContextError: Send,
		(): NamespaceMut<T, B>,
	{
		static mut NAMESPACE: () = ();
		self.expand_full(
			unsafe { &mut NAMESPACE },
			Context::<T, B, L::Context>::new(base_url.cloned()),
			base_url,
			loader,
			Options::default(),
			print_warning,
		)
	}
}

impl<T, B, M> Expand<T, B, M> for Meta<Value<M>, M> {
	fn expand_full<'a, N, C, L: Loader<T, M> + ContextLoader<T, M>>(
		&'a self,
		namespace: &'a mut N,
		context: Context<T, B, C>,
		base_url: Option<&'a T>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + WarningHandler<B, N, M>,
	) -> BoxFuture<ExpansionResult<T, B, M, L>>
	where
		N: Send + Sync + NamespaceMut<T, B>,
		T: Clone + Eq + Hash + Send + Sync,
		B: 'a + Clone + Eq + Hash + Send + Sync,
		M: 'a + Clone + Send + Sync,
		C: 'a + Process<T, B, M> + From<json_ld_syntax::context::Value<M>>,
		L: Send + Sync,
		L::Output: Into<Value<M>>,
		L::Context: Into<C>,
		L::ContextError: Send,
	{
		async move {
			document::expand(
				namespace, self, context, base_url, loader, options, warnings,
			)
			.await
		}
		.boxed()
	}
}
