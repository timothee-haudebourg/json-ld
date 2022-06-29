use iref::Iri;
use futures::future::BoxFuture;

/// JSON document loader.
pub trait Loader {
	/// The type of documents that can be loaded.
	type Output;
	type Source;
	type Error;

	/// Loads the document behind the given IRI.
	fn load<'a>(
		&'a mut self,
		url: Iri,
	) -> BoxFuture<'a, Result<Self::Output, Self::Error>>;
}