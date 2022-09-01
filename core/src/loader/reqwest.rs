//! Simple document and context loader based on [`reqwest`](https://crates.io/crates/reqwest)

use crate::{loader, Error, ErrorCode, RemoteDocument};
use futures::future::{BoxFuture, FutureExt};
use generic_json::Json;
use iref::{Iri, IriBuf};
use std::collections::HashMap;

pub fn is_json_media_type(ty: &str) -> bool {
	ty == "application/json" || ty == "application/ld+json"
}

pub async fn load_remote_json_ld_document<J, P>(url: Iri<'_>, parser: &mut P) -> Result<J, Error>
where
	P: Send + Sync + FnMut(&str) -> Result<J, Error>,
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
		.any(|value| {
			if let Ok(value) = value.to_str() {
				is_json_media_type(value)
			} else {
				false
			}
		}) {
		let body = response.text().await?;
		let doc = (*parser)(body.as_str())?;
		Ok(doc)
	} else {
		Err(ErrorCode::LoadingDocumentFailed.into())
	}
}

pub struct Loader<J> {
	vocabulary: HashMap<IriBuf, loader::Id>,
	cache: Vec<(J, IriBuf)>,
	parser: Box<dyn 'static + Send + Sync + FnMut(&str) -> Result<J, Error>>,
}

impl<J: Clone + Send> Loader<J> {
	pub fn new<E: 'static + std::error::Error>(
		mut parser: impl 'static + Send + Sync + FnMut(&str) -> Result<J, E>,
	) -> Self {
		Self {
			vocabulary: HashMap::new(),
			cache: Vec::new(),
			parser: Box::new(move |s| {
				parser(s).map_err(|e| Error::with_source(ErrorCode::LoadingDocumentFailed, e))
			}),
		}
	}

	/// Allocate a identifier to the given IRI.
	fn allocate(&mut self, iri: IriBuf, doc: J) -> loader::Id {
		let id = loader::Id::new(self.cache.len());
		self.vocabulary.insert(iri.clone(), id);
		self.cache.push((doc, iri));
		id
	}

	pub async fn load(&mut self, url: Iri<'_>) -> Result<RemoteDocument<J>, Error> {
		let url = IriBuf::from(url);
		match self.vocabulary.get(&url) {
			Some(id) => Ok(RemoteDocument::new(
				self.cache[id.unwrap()].0.clone(),
				url,
				*id,
			)),
			None => {
				let doc = load_remote_json_ld_document(url.as_iri(), &mut self.parser).await?;
				let id = self.allocate(url.clone(), doc.clone());
				Ok(RemoteDocument::new(doc, url, id))
			}
		}
	}
}

impl<J: Json + Clone + Send + Sync> crate::Loader for Loader<J> {
	type Document = J;

	#[inline(always)]
	fn id(&self, iri: Iri<'_>) -> Option<loader::Id> {
		self.vocabulary.get(&IriBuf::from(iri)).cloned()
	}

	#[inline(always)]
	fn iri(&self, id: loader::Id) -> Option<Iri<'_>> {
		self.cache.get(id.unwrap()).map(|(_, iri)| iri.as_iri())
	}

	fn load<'a>(&'a mut self, url: Iri<'_>) -> BoxFuture<'a, Result<RemoteDocument<J>, Error>> {
		let url: IriBuf = url.into();
		async move {
			match self.vocabulary.get(&url) {
				Some(id) => Ok(RemoteDocument::new(
					self.cache[id.unwrap()].0.clone(),
					url,
					*id,
				)),
				None => {
					let doc = load_remote_json_ld_document(url.as_iri(), &mut self.parser).await?;
					let id = self.allocate(url.clone(), doc.clone());
					Ok(RemoteDocument::new(doc, url, id))
				}
			}
		}
		.boxed()
	}
}

impl From<reqwest::Error> for Error {
	fn from(e: reqwest::Error) -> Error {
		Error::with_source(ErrorCode::LoadingDocumentFailed, e)
	}
}
