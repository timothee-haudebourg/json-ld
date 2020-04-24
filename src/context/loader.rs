use futures::future::LocalBoxFuture;
use iref::{Iri, IriBuf};
use crate::Error;

pub struct RemoteContext<C> {
	url: IriBuf,
	context: C
}

impl<C> RemoteContext<C> {
	pub fn new(url: Iri, context: C) -> RemoteContext<C> {
		RemoteContext {
			url: IriBuf::from(url),
			context: context
		}
	}

	pub fn from_parts(url: IriBuf, context: C) -> RemoteContext<C> {
		RemoteContext {
			url: url,
			context: context
		}
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

	pub fn cast<D>(self) -> RemoteContext<D> where C: Into<D> {
		RemoteContext {
			url: self.url,
			context: self.context.into()
		}
	}
}

pub trait Loader {
	type Output;

	fn load_context<'a>(&'a mut self, url: Iri) -> LocalBoxFuture<'a, Result<RemoteContext<Self::Output>, Error>>;
}
