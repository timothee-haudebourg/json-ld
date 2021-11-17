use crate::{Error, ErrorCode};
use futures::future::{BoxFuture, FutureExt};
use iref::{Iri, IriBuf};
use generic_json::Json;

pub struct RemoteContext<C> {
	url: IriBuf,
	context: C,
}

impl<C> RemoteContext<C> {
	pub fn new(url: Iri, context: C) -> RemoteContext<C> {
		RemoteContext {
			url: IriBuf::from(url),
			context,
		}
	}

	pub fn from_parts(url: IriBuf, context: C) -> RemoteContext<C> {
		RemoteContext { url, context }
	}

	pub fn context(&self) -> &C {
		&self.context
	}

	pub fn into_context(self) -> C {
		self.context
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
			context: self.context.into(),
		}
	}
}

pub trait Loader {
	type Output;

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

	fn load_context<'a>(
		&'a mut self,
		url: Iri,
	) -> BoxFuture<'a, Result<RemoteContext<L::Document>, Error>> {
		let url = IriBuf::from(url);
		async move {
			match self.load(url.as_iri()).await {
				Ok(remote_doc) => {
					let (doc, url) = remote_doc.into_parts();
					if let generic_json::Value::Object(obj) = doc.into() {
						for (key, value) in obj {
							if &*key == "@context" {
								return Ok(RemoteContext::from_parts(url, value));
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