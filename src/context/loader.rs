use std::pin::Pin;
use std::future::Future;
use futures::future::FutureExt;
use iref::{Iri, IriBuf};
use json::JsonValue;
use super::ContextProcessingError;

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

	pub fn context(&self) -> &C {
		&self.context
	}

	pub fn url(&self) -> Iri {
		self.url.as_iri()
	}
}

pub trait ContextLoader<C> {
	fn load<'a>(&'a self, url: Iri) -> Pin<Box<dyn 'a + Future<Output = Result<RemoteContext<C>, ContextProcessingError>>>>;
}

pub async fn load_remote_json_ld_context(url: Iri<'_>) -> Result<JsonValue, ContextProcessingError> {
	use reqwest::header::*;

	let client = reqwest::Client::new();
	let request = client.get(url.as_str()).header(ACCEPT, "application/ld+json, application/json");
	let response = request.send().await?;

	// response.headers().get_all(CONTENT_TYPE).find(|&value| value == "").is_some();
	// let bytes = response.bytes().await?;

	panic!("TODO")
}

pub struct JsonLdContextLoader {
	// ...
}

impl JsonLdContextLoader {
	pub fn new() -> JsonLdContextLoader {
		JsonLdContextLoader {
			// ...
		}
	}
}

impl ContextLoader<JsonValue> for JsonLdContextLoader {
	fn load<'a>(&'a self, url: Iri) -> Pin<Box<dyn 'a + Future<Output = Result<RemoteContext<JsonValue>, ContextProcessingError>>>> {
		let url = IriBuf::from(url);
		async move {
			load_remote_json_ld_context(url.as_iri()).await?;
			panic!("TODO")
		}.boxed()
	}
}
