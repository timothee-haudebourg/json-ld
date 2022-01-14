use crate::{loader, Error, ErrorCode};
use futures::future::{BoxFuture, FutureExt};
use generic_json::Json;
use iref::{Iri, IriBuf};

pub struct RemoteContext<C> {
	url: IriBuf,
	source: loader::Id,
	context: C,
}

impl<C> RemoteContext<C> {
	pub fn new(url: Iri, source: loader::Id, context: C) -> RemoteContext<C> {
		RemoteContext {
			url: IriBuf::from(url),
			source,
			context,
		}
	}

	pub fn from_parts(url: IriBuf, source: loader::Id, context: C) -> RemoteContext<C> {
		RemoteContext {
			url,
			source,
			context,
		}
	}

	pub fn context(&self) -> &C {
		&self.context
	}

	pub fn into_context(self) -> C {
		self.context
	}

	pub fn source(&self) -> loader::Id {
		self.source
	}

	pub fn url(&self) -> Iri {
		self.url.as_iri()
	}

	pub fn cast<D>(self) -> RemoteContext<D>
	where
		C: Into<D>,
	{
		RemoteContext {
			url: self.url,
			source: self.source,
			context: self.context.into(),
		}
	}
}

pub trait Loader {
	type Output;

	/// Returns the unique identifier associated to the given IRI, if any.
	fn id(&self, iri: Iri<'_>) -> Option<crate::loader::Id>;

	/// Returns the unique identifier associated to the given IRI, if any.
	///
	/// Returns `None` if the input `iri` is `None`.
	#[inline(always)]
	fn id_opt(&self, iri: Option<Iri<'_>>) -> Option<crate::loader::Id> {
		iri.and_then(|iri| self.id(iri))
	}

	/// Returns the IRI with the given identifier, if any.
	fn iri(&self, id: crate::loader::Id) -> Option<Iri<'_>>;

	fn load_context<'a>(
		&'a mut self,
		url: Iri,
	) -> BoxFuture<'a, Result<RemoteContext<Self::Output>, Error>>;
}

impl<L: Send + Sync + crate::Loader> Loader for L
where
	<L::Document as Json>::Object: IntoIterator,
{
	type Output = L::Document;

	fn id(&self, iri: Iri<'_>) -> Option<crate::loader::Id> {
		self.id(iri)
	}

	fn iri(&self, id: crate::loader::Id) -> Option<Iri<'_>> {
		self.iri(id)
	}

	fn load_context<'a>(
		&'a mut self,
		url: Iri,
	) -> BoxFuture<'a, Result<RemoteContext<L::Document>, Error>> {
		let url = IriBuf::from(url);
		async move {
			match self.load(url.as_iri()).await {
				Ok(remote_doc) => {
					let (doc, source, url) = remote_doc.into_parts();
					if let generic_json::Value::Object(obj) = doc.into() {
						for (key, value) in obj {
							if &*key == "@context" {
								return Ok(RemoteContext::from_parts(url, source, value));
							}
						}
					}

					Err(ErrorCode::InvalidRemoteContext.into())
				}
				Err(_) => Err(ErrorCode::LoadingRemoteContextFailed.into()),
			}
		}
		.boxed()
	}
}
