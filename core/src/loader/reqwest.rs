//! Simple document and context loader based on [`reqwest`](https://crates.io/crates/reqwest)
use super::{Loader, RemoteDocument};
use futures::future::{BoxFuture, FutureExt};
use locspan::Meta;
use rdf_types::{vocabulary::Index, IriVocabulary};
use reqwest::header::ACCEPT;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error<E> {
	Reqwest(reqwest::Error),
	Parse(E),
}

type DynParser<I, M, T, E> =
	dyn 'static + Send + Sync + FnMut(&dyn IriVocabulary<Iri=I>, &I, &str) -> Result<Meta<T, M>, E>;

/// `reqwest`-based loader.
pub struct ReqwestLoader<
	I = Index,
	M = locspan::Location<I>,
	T = json_ld_syntax::Value<M>,
	E = json_ld_syntax::parse::MetaError<M>,
> {
	cache: HashMap<I, Meta<T, M>>,
	parser: Box<DynParser<I, M, T, E>>,
}

impl<I, M, T, E> ReqwestLoader<I, M, T, E> {
	pub fn new(
		parser: impl 'static
			+ Send
			+ Sync
			+ FnMut(&dyn IriVocabulary<Iri=I>, &I, &str) -> Result<Meta<T, M>, E>,
	) -> Self {
		Self {
			cache: HashMap::new(),
			parser: Box::new(parser),
		}
	}
}

impl<I: Clone> Default
	for ReqwestLoader<
		I,
		locspan::Location<I>,
		json_ld_syntax::Value<locspan::Location<I>>,
		json_ld_syntax::parse::MetaError<locspan::Location<I>>,
	>
{
	fn default() -> Self {
		use json_syntax::Parse;
		Self::new(|_, file: &I, s| {
			json_syntax::Value::parse_str(s, |span| locspan::Location::new(file.clone(), span))
		})
	}
}

impl<I: Clone + Eq + Hash + Send + Sync, T: Clone + Send, M: Clone + Send, E> Loader<I, M>
	for ReqwestLoader<I, M, T, E>
{
	type Output = T;
	type Error = Error<E>;

	fn load_with<'a>(
		&'a mut self,
		vocabulary: &'a (impl Sync + IriVocabulary<Iri=I>),
		url: I,
	) -> BoxFuture<'a, Result<RemoteDocument<I, M, T>, Self::Error>>
	where
		I: 'a,
	{
		async move {
			match self.cache.get(&url) {
				Some(t) => Ok(RemoteDocument::new(Some(url), t.clone())),
				None => {
					let client = reqwest::Client::new();
					let request = client
						.get(vocabulary.iri(&url).unwrap().as_str())
						.header(ACCEPT, "application/ld+json, application/json");

					let response = request.send().await.map_err(Error::Reqwest)?;

					todo!()
				},
			}
		}
		.boxed()
	}
}

// pub fn is_json_media_type(ty: &str) -> bool {
// 	ty == "application/json" || ty == "application/ld+json"
// }

// pub async fn load_remote_json_ld_document<J, P>(url: Iri<'_>, parser: &mut P) -> Result<J, Error>
// where
// 	P: Send + Sync + FnMut(&str) -> Result<J, Error>,
// {
// 	log::info!("loading remote document `{}'", url);
// 	use reqwest::header::*;

// 	let client = reqwest::Client::new();
// 	let request = client
// 		.get(url.as_str())
// 		.header(ACCEPT, "application/ld+json, application/json");
// 	let response = request.send().await?;

// 	if response
// 		.headers()
// 		.get_all(CONTENT_TYPE)
// 		.iter()
// 		.any(|value| {
// 			if let Ok(value) = value.to_str() {
// 				is_json_media_type(value)
// 			} else {
// 				false
// 			}
// 		}) {
// 		let body = response.text().await?;
// 		let doc = (*parser)(body.as_str())?;
// 		Ok(doc)
// 	} else {
// 		Err(ErrorCode::LoadingDocumentFailed.into())
// 	}
// }

// pub struct Loader<J> {
// 	vocabulary: HashMap<IriBuf, loader::Id>,
// 	cache: Vec<(J, IriBuf)>,
// 	parser: Box<dyn 'static + Send + Sync + FnMut(&str) -> Result<J, Error>>,
// }

// impl<J: Clone + Send> Loader<J> {
// 	pub fn new<E: 'static + std::error::Error>(
// 		mut parser: impl 'static + Send + Sync + FnMut(&str) -> Result<J, E>,
// 	) -> Self {
// 		Self {
// 			vocabulary: HashMap::new(),
// 			cache: Vec::new(),
// 			parser: Box::new(move |s| {
// 				parser(s).map_err(|e| Error::with_source(ErrorCode::LoadingDocumentFailed, e))
// 			}),
// 		}
// 	}

// 	/// Allocate a identifier to the given IRI.
// 	fn allocate(&mut self, iri: IriBuf, doc: J) -> loader::Id {
// 		let id = loader::Id::new(self.cache.len());
// 		self.vocabulary.insert(iri.clone(), id);
// 		self.cache.push((doc, iri));
// 		id
// 	}

// 	pub async fn load(&mut self, url: Iri<'_>) -> Result<RemoteDocument<J>, Error> {
// 		let url = IriBuf::from(url);
// 		match self.vocabulary.get(&url) {
// 			Some(id) => Ok(RemoteDocument::new(
// 				self.cache[id.unwrap()].0.clone(),
// 				url,
// 				*id,
// 			)),
// 			None => {
// 				let doc = load_remote_json_ld_document(url.as_iri(), &mut self.parser).await?;
// 				let id = self.allocate(url.clone(), doc.clone());
// 				Ok(RemoteDocument::new(doc, url, id))
// 			}
// 		}
// 	}
// }

// impl<J: Json + Clone + Send + Sync> crate::Loader for Loader<J> {
// 	type Document = J;

// 	#[inline(always)]
// 	fn id(&self, iri: Iri<'_>) -> Option<loader::Id> {
// 		self.vocabulary.get(&IriBuf::from(iri)).cloned()
// 	}

// 	#[inline(always)]
// 	fn iri(&self, id: loader::Id) -> Option<Iri<'_>> {
// 		self.cache.get(id.unwrap()).map(|(_, iri)| iri.as_iri())
// 	}

// 	fn load<'a>(&'a mut self, url: Iri<'_>) -> BoxFuture<'a, Result<RemoteDocument<J>, Error>> {
// 		let url: IriBuf = url.into();
// 		async move {
// 			match self.vocabulary.get(&url) {
// 				Some(id) => Ok(RemoteDocument::new(
// 					self.cache[id.unwrap()].0.clone(),
// 					url,
// 					*id,
// 				)),
// 				None => {
// 					let doc = load_remote_json_ld_document(url.as_iri(), &mut self.parser).await?;
// 					let id = self.allocate(url.clone(), doc.clone());
// 					Ok(RemoteDocument::new(doc, url, id))
// 				}
// 			}
// 		}
// 		.boxed()
// 	}
// }

// impl From<reqwest::Error> for Error {
// 	fn from(e: reqwest::Error) -> Error {
// 		Error::with_source(ErrorCode::LoadingDocumentFailed, e)
// 	}
// }
