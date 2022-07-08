use futures::future::{BoxFuture, FutureExt};
use iref::IriBuf;
use locspan::Meta;
use json_ld_core::{ExpandedDocument, Id, Context};
use json_ld_context_processing::{Process, ContextLoader};
use json_ld_syntax::Value;

mod warning;
mod error;
mod options;
mod loader;
mod expanded;
mod element;
mod array;
mod literal;
mod value;
mod node;
mod document;

pub use warning::*;
pub use error::*;
pub use options::*;
pub use loader::*;
pub use expanded::*;

pub(crate) use json_ld_context_processing::syntax::expand_iri_simple as expand_iri;
pub(crate) use element::*;
pub(crate) use array::*;
pub(crate) use literal::*;
pub(crate) use value::*;
pub(crate) use node::*;
pub(crate) use document::filter_top_level_item;

/// Result of the document expansion.
pub type ExpansionResult<T, M> = Result<ExpandedDocument<T, M>, Meta<Error, M>>;

pub trait Expand<T: Id, C: Process<T>> {
	fn expand_full<'a, L: Loader + ContextLoader>(
		&'a self,
		active_context: &'a Context<T, C>,
		base_url: Option<IriBuf>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + FnMut(Meta<Warning, C::Metadata>)
	) -> BoxFuture<ExpansionResult<T, C::Metadata>>
	where
		T: Send + Sync,
		C: Send + Sync,
		L: Send + Sync,
		<L as Loader>::Output: Into<Value<C, C::Metadata>>,
		<L as ContextLoader>::Output: Into<C>;
}

impl<T: Id, C: Process<T>> Expand<T, C> for Meta<Value<C, C::Metadata>, C::Metadata> {
	fn expand_full<'a, L: Loader + ContextLoader>(
		&'a self,
		active_context: &'a Context<T, C>,
		base_url: Option<IriBuf>,
		loader: &'a mut L,
		options: Options,
		warnings: impl 'a + Send + FnMut(Meta<Warning, C::Metadata>)
	) -> BoxFuture<ExpansionResult<T, C::Metadata>>
	where
		T: Send + Sync,
		C: Send + Sync,
		L: Send + Sync,
		<L as Loader>::Output: Into<Value<C, C::Metadata>>,
		<L as ContextLoader>::Output: Into<C>
	{
		async move {
			document::expand(active_context, self, base_url, loader, options, warnings).await
		}.boxed()
	}
}