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

#[derive(Clone, Copy, Debug)]
pub enum JsonLdLoadError {
	LoadingFailed
}

impl From<reqwest::Error> for JsonLdLoadError {
	fn from(_e: reqwest::Error) -> JsonLdLoadError {
		JsonLdLoadError::LoadingFailed
	}
}

impl From<JsonLdLoadError> for ContextProcessingError {
	fn from(_e: JsonLdLoadError) -> ContextProcessingError {
		ContextProcessingError::LoadingRemoteContextFailed
	}
}

pub fn is_json_media_type(ty: &str) -> bool {
	ty == "application/json" || ty == "application/ld+json"
}

pub async fn load_remote_json_ld_document(url: Iri<'_>) -> Result<JsonValue, JsonLdLoadError> {
	info!("loading remote document `{}'", url);
	use reqwest::header::*;

	let client = reqwest::Client::new();
	let request = client.get(url.as_str()).header(ACCEPT, "application/ld+json, application/json");
	let response = request.send().await?;

	if response.headers().get_all(CONTENT_TYPE).iter().find(|&value| {
		if let Ok(value) = value.to_str() {
			is_json_media_type(value)
		} else {
			false
		}
	}).is_some() {
		let body = response.text().await?;

		if let Ok(doc) = json::parse(body.as_str()) {
			Ok(doc)
		} else {
			panic!("invalid json")
		}
	} else {
		panic!("not a json document")
	}
}

pub async fn load_remote_json_ld_context(url: Iri<'_>) -> Result<JsonValue, ContextProcessingError> {
	let doc = load_remote_json_ld_document(url).await?;
	if let JsonValue::Object(obj) = doc {
		if let Some(context) = obj.get("@context") {
			Ok(context.clone())
		} else {
			Err(ContextProcessingError::LoadingRemoteContextFailed)
		}
	} else {
		Err(ContextProcessingError::LoadingRemoteContextFailed)
	}
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
			let doc = load_remote_json_ld_context(url.as_iri()).await?;
			Ok(RemoteContext::new(url.as_iri(), doc))
		}.boxed()
	}
}
