/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote ressources.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a ressource.
pub struct NoLoader<J>(PhantomData<J>);

impl<J> NoLoader<J> {
	#[inline(always)]
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<J> Default for NoLoader<J> {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl<J: Json> Loader for NoLoader<J> {
	type Document = J;

	#[inline(always)]
	fn id(&self, _iri: Iri<'_>) -> Option<Id> {
		None
	}

	#[inline(always)]
	fn iri(&self, _id: Id) -> Option<Iri<'_>> {
		None
	}

	#[inline(always)]
	fn load<'a>(
		&'a mut self,
		_url: Iri<'_>,
	) -> BoxFuture<'a, Result<RemoteDocument<Self::Document>, Error>> {
		async move { Err(ErrorCode::LoadingDocumentFailed.into()) }.boxed()
	}
}