//! Simple document and context loader based on [`reqwest`](https://crates.io/crates/reqwest)

use crate::{
	context::{self, RemoteContext},
	Error, ErrorCode, RemoteDocument,
};
use generic_json::Json;
use futures::future::{BoxFuture, FutureExt};
use iref::{Iri, IriBuf};
use std::collections::HashMap;

pub fn is_json_media_type(ty: &str) -> bool {
	ty == "application/json" || ty == "application/ld+json"
}

pub async fn load_remote_json_ld_document<J, P>(
	url: Iri<'_>,
	parser: &mut P
) -> Result<RemoteDocument<J>, Error>
where
	P: Send + Sync + FnMut(&str) -> Result<J, Error>
{
	log::info!("loading remote document `{}'", url);
	use reqwest::header::*;

	let client = reqwest::Client::new();
	let request = client
		.get(url.as_str())
		.header(ACCEPT, "application/ld+json, application/json");
	let response = request.send().await?;

	if response
		.headers()
		.get_all(CONTENT_TYPE)
		.iter()
		.find(|&value| {
			if let Ok(value) = value.to_str() {
				is_json_media_type(value)
			} else {
				false
			}
		})
		.is_some()
	{
		let body = response.text().await?;

		match (*parser)(body.as_str()) {
			Ok(doc) => Ok(RemoteDocument::new(doc, url.into())),
			Err(e) => panic!("invalid json: {:?}: {}", e, body.as_str()),
		}
	} else {
		panic!("not a json document")
	}
}

pub struct Loader<J> {
	cache: HashMap<IriBuf, RemoteDocument<J>>,
	parser: Box<dyn 'static + Send + Sync + FnMut(&str) -> Result<J, Error>>
}

impl<J: Clone + Send> Loader<J> {
	pub fn new<E: 'static + std::error::Error>(mut parser: impl 'static + Send + Sync + FnMut(&str) -> Result<J, E>) -> Self {
		Self {
			cache: HashMap::new(),
			parser: Box::new(move |s| parser(s).map_err(|e| Error::new(ErrorCode::LoadingDocumentFailed, e)))
		}
	}

	pub async fn load(&mut self, url: Iri<'_>) -> Result<RemoteDocument<J>, Error> {
		let url = IriBuf::from(url);
		match self.cache.get(&url) {
			Some(doc) => Ok(doc.clone()),
			None => {
				let doc = load_remote_json_ld_document(url.as_iri(), &mut self.parser).await?;
				self.cache.insert(url, doc.clone());
				Ok(doc)
			}
		}
	}
}

impl<J: Json + Clone + Send + Sync> crate::Loader for Loader<J> {
	type Document = J;

	fn load<'a>(
		&'a mut self,
		url: Iri<'_>,
	) -> BoxFuture<'a, Result<RemoteDocument<J>, Error>> {
		let url: IriBuf = url.into();
		async move {
			match self.cache.get(&url) {
				Some(doc) => Ok(doc.clone()),
				None => {
					let doc = load_remote_json_ld_document(url.as_iri(), &mut self.parser).await?;
					self.cache.insert(url, doc.clone());
					Ok(doc)
				}
			}
		}.boxed()
	}
}

impl From<reqwest::Error> for Error {
	fn from(e: reqwest::Error) -> Error {
		Error::new(ErrorCode::LoadingDocumentFailed, e)
	}
}
