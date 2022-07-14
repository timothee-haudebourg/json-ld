use futures::future::{BoxFuture, FutureExt};
use iref::IriBuf;
use json_ld_context_processing::Process;
use json_ld_core::{Context, ContextLoader, ExpandedDocument, Id, Loader};
use json_ld_syntax::Value;
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
pub type ExpansionResult<T, M, L> = Result<ExpandedDocument<T, M>, Meta<Error<L>, M>>;

fn print_warning<M>(warning: Meta<Warning, M>) {
	// panic!("warning: {:?}", warning.value());
	eprintln!("{}", warning.value())
}

fn ignore_warning<M>(_warning: Meta<Warning, M>) {}

pub trait Expand<T: Id, C: Process<T>> {
	fn expand_full<'a, L: Loader + ContextLoader>(
		&'a self,
		active_context: &'a Context<T, C>,
		base_url: Option<IriBuf>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + FnMut(Meta<Warning, C::Metadata>),
	) -> BoxFuture<ExpansionResult<T, C::Metadata, L>>
	where
		T: Send + Sync,
		C: Send + Sync,
		L: Send + Sync,
		<L as Loader>::Output: Into<Value<C, C::Metadata>>,
		<L as ContextLoader>::Output: Into<C>;

	fn expand<'a, L: Loader + ContextLoader>(
		&'a self,
		active_context: &'a Context<T, C>,
		loader: &'a mut L,
	) -> BoxFuture<ExpansionResult<T, C::Metadata, L>>
	where
		T: Send + Sync,
		C: Send + Sync,
		L: Send + Sync,
		<L as Loader>::Output: Into<Value<C, C::Metadata>>,
		<L as ContextLoader>::Output: Into<C>,
	{
		self.expand_full(
			active_context,
			None,
			loader,
			Options::default(),
			print_warning,
		)
	}
}

impl<T: Id, C: Process<T>> Expand<T, C> for Meta<Value<C, C::Metadata>, C::Metadata> {
	fn expand_full<'a, L: Loader + ContextLoader>(
		&'a self,
		active_context: &'a Context<T, C>,
		base_url: Option<IriBuf>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + FnMut(Meta<Warning, C::Metadata>),
	) -> BoxFuture<ExpansionResult<T, C::Metadata, L>>
	where
		T: Send + Sync,
		C: Send + Sync,
		L: Send + Sync,
		<L as Loader>::Output: Into<Value<C, C::Metadata>>,
		<L as ContextLoader>::Output: Into<C>,
	{
		async move {
			document::expand(active_context, self, base_url, loader, options, warnings).await
		}.boxed()
	}
}
