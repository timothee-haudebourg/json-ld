use super::Loader;
use futures::future::{BoxFuture, FutureExt};
use iref::{Iri, IriBuf};
use locspan::Meta;
use std::marker::PhantomData;

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote resource.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a resource.
pub struct NoLoader<T, M>(PhantomData<(T, M)>);

#[derive(Debug)]
pub struct CannotLoad(IriBuf);

impl<T, M> NoLoader<T, M> {
	#[inline(always)]
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T, M> Default for NoLoader<T, M> {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl<T, M> Loader for NoLoader<T, M> {
	type Output = T;
	type Error = CannotLoad;
	type Metadata = M;

	#[inline(always)]
	fn load<'a>(&'a mut self, url: Iri<'_>) -> BoxFuture<'a, Result<Meta<T, M>, Self::Error>> {
		let url: IriBuf = url.into();
		async move { Err(CannotLoad(url)) }.boxed()
	}
}
