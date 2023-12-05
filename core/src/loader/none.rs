use super::{Loader, RemoteDocument};
use crate::future::{BoxFuture, FutureExt};
use contextual::{DisplayWithContext, WithContext};
use rdf_types::{vocabulary::IriIndex, IriVocabulary};
use std::fmt;
use std::marker::PhantomData;

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote resource.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a resource.
pub struct NoLoader<I = IriIndex, M = locspan::Location<I>, T = json_ld_syntax::Value<M>>(
	PhantomData<(I, M, T)>,
);

#[derive(Debug)]
pub struct CannotLoad<I>(I);

impl<I: fmt::Display> fmt::Display for CannotLoad<I> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "cannot load `{}`", self.0)
	}
}

impl<I: DisplayWithContext<N>, N> DisplayWithContext<N> for CannotLoad<I> {
	fn fmt_with(&self, vocabulary: &N, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "cannot load `{}`", self.0.with(vocabulary))
	}
}

impl<I, M, T> NoLoader<I, M, T> {
	#[inline(always)]
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<I, M, T> Default for NoLoader<I, M, T> {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl<I: Send, T, M> Loader<I, M> for NoLoader<I, M, T> {
	type Output = T;
	type Error = CannotLoad<I>;

	#[inline(always)]
	fn load_with<'a>(
		&'a mut self,
		_namespace: &mut impl IriVocabulary<Iri = I>,
		url: I,
	) -> BoxFuture<'a, Result<RemoteDocument<I, M, T>, Self::Error>>
	where
		I: 'a,
	{
		async move { Err(CannotLoad(url)) }.boxed()
	}
}
