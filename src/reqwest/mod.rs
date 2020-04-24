use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use futures::future::FutureExt;
use iref::{Iri, IriBuf};
use json::JsonValue;
use crate::{
	Error,
	ErrorCode,
	context::{
		RemoteContext,
		Loader
	}
};

pub fn is_json_media_type(ty: &str) -> bool {
	ty == "application/json" || ty == "application/ld+json"
}

pub async fn load_remote_json_ld_document(url: Iri<'_>) -> Result<JsonValue, Error> {
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

		match json::parse(body.as_str()) {
			Ok(doc) => Ok(doc),
			Err(e) => panic!("invalid json: {:?}: {}", e, body.as_str())
		}
	} else {
		panic!("not a json document")
	}
}

pub async fn load_remote_json_ld_context(url: Iri<'_>) -> Result<JsonValue, Error> {
	match load_remote_json_ld_document(url).await {
		Ok(doc) => {
			if let JsonValue::Object(obj) = doc {
				if let Some(context) = obj.get("@context") {
					Ok(context.clone())
				} else {
					Err(ErrorCode::InvalidRemoteContext.into())
				}
			} else {
				Err(ErrorCode::InvalidRemoteContext.into())
			}
		},
		Err(_) => {
			Err(ErrorCode::LoadingRemoteContextFailed.into())
		}
	}
}

pub struct ReqwestLoader {
	cache: HashMap<IriBuf, JsonValue>
}

impl ReqwestLoader {
	pub fn new() -> ReqwestLoader {
		ReqwestLoader {
			cache: HashMap::new()
		}
	}
}

impl Loader for ReqwestLoader {
	type Output = JsonValue;

	fn load<'a>(&'a mut self, url: Iri) -> Pin<Box<dyn 'a + Future<Output = Result<RemoteContext<JsonValue>, Error>>>> {
		let url = IriBuf::from(url);
		async move {
			let doc = match self.cache.get(&url) {
				Some(doc) => {
					doc.clone()
				},
				None => {
					let doc = load_remote_json_ld_context(url.as_iri()).await?;
					self.cache.insert(url.clone(), doc.clone());
					doc
				}
			};

			Ok(RemoteContext::new(url.as_iri(), doc))
		}.boxed()
	}
}

impl From<reqwest::Error> for Error {
	fn from(e: reqwest::Error) -> Error {
		Error::new(ErrorCode::LoadingDocumentFailed, e)
	}
}
