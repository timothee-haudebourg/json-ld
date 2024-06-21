//! Simple document and context loader based on [`reqwest`](https://crates.io/crates/reqwest)
use crate::LoaderError;
use crate::LoadingResult;
use crate::Profile;

use super::{Loader, RemoteDocument};
use hashbrown::HashSet;
use iref::{Iri, IriBuf};
use json_syntax::Parse;
use once_cell::sync::OnceCell;
use rdf_types::vocabulary::{IriVocabulary, IriVocabularyMut};
use reqwest::{
	header::{ACCEPT, CONTENT_TYPE, LINK},
	StatusCode,
};
use reqwest_middleware::ClientWithMiddleware;
use std::{hash::Hash, string::FromUtf8Error};

mod content_type;
mod link;

use content_type::*;
use link::*;

/// Loader options.
pub struct Options<I> {
	/// One or more IRIs to use in the request as a profile parameter.
	///
	/// (See [IANA Considerations](https://www.w3.org/TR/json-ld11/#iana-considerations)).
	pub request_profile: Vec<Profile<I>>,

	/// Maximum number of allowed `Link` header redirections before the loader
	/// fails.
	///
	/// Defaults to 8.
	///
	/// Note: this only controls how many times the loader will use a `Link`
	/// HTTP header to find the target JSON-LD document. The number of allowed
	/// regular HTTP redirections is controlled by the HTTP
	/// [`client`](Self::client).
	pub max_redirections: usize,

	/// HTTP client.
	pub client: ClientWithMiddleware,
}

impl<I> Default for Options<I> {
	fn default() -> Self {
		Self {
			request_profile: Vec::new(),
			max_redirections: 8,
			client: reqwest_middleware::ClientBuilder::new(reqwest::Client::default()).build(),
		}
	}
}

/// Loading error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("internal error: {1}")]
	Reqwest(IriBuf, reqwest_middleware::Error),

	#[error("query failed: status code {1}")]
	QueryFailed(IriBuf, StatusCode),

	#[error("invalid content type")]
	InvalidContentType(IriBuf),

	#[error("multiple context link headers")]
	MultipleContextLinkHeaders(IriBuf),

	#[error("too many redirections")]
	TooManyRedirections(IriBuf),

	#[error("JSON parse error: {0}")]
	Parse(IriBuf, json_syntax::parse::Error<std::io::Error>),
}

impl LoaderError for Error {
	fn into_iri_and_message(self) -> (IriBuf, String) {
		match self {
			Self::Reqwest(iri, e) => (iri, format!("internal error: {e}")),
			Self::QueryFailed(iri, e) => (iri, format!("query failed with status code: {e}")),
			Self::InvalidContentType(iri) => (iri, "invalid content type".to_owned()),
			Self::MultipleContextLinkHeaders(iri) => {
				(iri, "multiple context link headers".to_owned())
			}
			Self::TooManyRedirections(iri) => (iri, "too many redirections".to_owned()),
			Self::Parse(iri, e) => (iri, format!("JSON parse error: {e}")),
		}
	}
}

/// `reqwest`-based loader.
///
/// Only works with the [`tokio`](https://tokio.rs/) runtime.
///
/// The loader will follow indirections and `Link` headers.
///
/// Loaded documents are not cached: a new network query is made each time
/// an URL is loaded even if it has already been queried before.
pub struct ReqwestLoader<I = IriBuf> {
	options: Options<I>,
	data: OnceCell<Data>,
}

impl<I> Default for ReqwestLoader<I> {
	fn default() -> Self {
		Self {
			options: Options::default(),
			data: OnceCell::new(),
		}
	}
}

impl<I> ReqwestLoader<I> {
	/// Creates a new loader with the given parsing function.
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new leader with the given options.
	pub fn new_using(options: Options<I>) -> Self {
		Self {
			options,
			data: OnceCell::new(),
		}
	}
}

/// Precomputed data.
struct Data {
	accept_header: String,
}

impl Data {
	fn new<I>(options: &Options<I>, vocabulary: &impl IriVocabulary<Iri = I>) -> Self {
		let mut json_ld_params = String::new();

		if !options.request_profile.is_empty() {
			json_ld_params.push_str("; profile=");

			if options.request_profile.len() > 1 {
				json_ld_params.push('"');
			}

			for (i, p) in options.request_profile.iter().enumerate() {
				if i > 0 {
					json_ld_params.push(' ');
				}

				json_ld_params.push_str(p.iri(vocabulary).as_str());
			}

			if options.request_profile.len() > 1 {
				json_ld_params.push('"');
			}
		}

		Self {
			accept_header: format!("application/ld+json{json_ld_params}, application/json"),
		}
	}
}

/// HTTP body parse error.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	/// Invalid encoding.
	#[error("invalid encoding")]
	InvalidEncoding(FromUtf8Error),

	/// JSON parse error.
	#[error("JSON parse error: {0}")]
	Json(json_ld_syntax::parse::Error),
}

impl<I: Clone + Eq + Hash> Loader<I> for ReqwestLoader<I> {
	type Error = Error;

	async fn load_with<V>(&mut self, vocabulary: &mut V, mut url_id: I) -> LoadingResult<I, Error>
	where
		V: IriVocabularyMut<Iri = I>,
	{
		let data = self
			.data
			.get_or_init(|| Data::new(&self.options, vocabulary));
		let mut redirection_number = 0;

		'next_url: loop {
			let url = vocabulary.iri(&url_id).unwrap().to_owned();

			if redirection_number > self.options.max_redirections {
				return Err(Error::TooManyRedirections(url.to_owned()));
			}

			log::debug!("downloading: {}", url);
			let request = self
				.options
				.client
				.get(url.as_str())
				.header(ACCEPT, &data.accept_header);

			let response = request
				.send()
				.await
				.map_err(|e| Error::Reqwest(url.clone(), e))?;

			match response.status() {
				StatusCode::OK => {
					let mut content_types = response
						.headers()
						.get_all(CONTENT_TYPE)
						.into_iter()
						.filter_map(ContentType::new);

					match content_types.find(ContentType::is_json_ld) {
						Some(content_type) => {
							let mut context_url = None;
							if *content_type.media_type() != "application/ld+json" {
								for link in response.headers().get_all(LINK).into_iter() {
									if let Some(link) = Link::new(link) {
										if link.rel()
											== Some(b"http://www.w3.org/ns/json-ld#context")
										{
											if context_url.is_some() {
												return Err(Error::MultipleContextLinkHeaders(url));
											}

											let u = link.href().resolved(&url);
											context_url = Some(vocabulary.insert(u.as_iri()));
										}
									}
								}
							}

							let mut profile = HashSet::new();
							for p in content_type
								.profile()
								.into_iter()
								.flat_map(|p| p.split(|b| *b == b' '))
							{
								if let Ok(p) = std::str::from_utf8(p) {
									if let Ok(iri) = Iri::new(p) {
										profile.insert(Profile::new(iri, vocabulary));
									}
								}
							}

							let bytes = response
								.bytes()
								.await
								.map_err(|e| Error::Reqwest(url.clone(), e.into()))?;

							let decoder = utf8_decode::Decoder::new(bytes.iter().copied());
							let (document, _) = json_syntax::Value::parse_utf8(decoder)
								.map_err(|e| Error::Parse(url.clone(), e))?;

							break Ok(RemoteDocument::new_full(
								Some(url_id),
								Some(content_type.into_media_type()),
								context_url,
								profile,
								document,
							));
						}
						None => {
							log::debug!("no valid media type found");
							for link in response.headers().get_all(LINK).into_iter() {
								if let Some(link) = Link::new(link) {
									if link.rel() == Some(b"alternate")
										&& link.type_() == Some(b"application/ld+json")
									{
										log::debug!("link found");
										let u = link.href().resolved(&url);
										url_id = vocabulary.insert(u.as_iri());
										redirection_number += 1;
										continue 'next_url;
									}
								}
							}

							break Err(Error::InvalidContentType(url));
						}
					}
				}
				code => break Err(Error::QueryFailed(url, code)),
			}
		}
	}
}
