//! Simple document and context loader based on [`reqwest`](https://crates.io/crates/reqwest)
use crate::LoadError;
use crate::LoadingResult;
use crate::Profile;

use super::{Loader, RemoteDocument};
use hashbrown::HashSet;
use iref::{Iri, IriBuf};
use json_syntax::Parse;
use reqwest::{
	header::{ACCEPT, CONTENT_TYPE, LINK},
	StatusCode,
};
use reqwest_middleware::ClientWithMiddleware;
use std::string::FromUtf8Error;

mod content_type;
mod link;

use content_type::*;
use link::*;

/// Loader options.
pub struct Options {
	/// One or more IRIs to use in the request as a profile parameter.
	///
	/// (See [IANA Considerations](https://www.w3.org/TR/json-ld11/#iana-considerations)).
	pub request_profile: Vec<Profile>,

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

impl Default for Options {
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
	#[error("internal error: {0}")]
	Reqwest(reqwest_middleware::Error),

	#[error("query failed: status code {0}")]
	QueryFailed(StatusCode),

	#[error("invalid content type")]
	InvalidContentType,

	#[error("multiple context link headers")]
	MultipleContextLinkHeaders,

	#[error("too many redirections")]
	TooManyRedirections,

	#[error("JSON parse error: {0}")]
	Parse(json_syntax::parse::Error<std::io::Error>),
}

/// `reqwest`-based loader.
///
/// Only works with the [`tokio`](https://tokio.rs/) runtime.
///
/// The loader will follow indirections and `Link` headers.
///
/// Loaded documents are not cached: a new network query is made each time
/// an URL is loaded even if it has already been queried before.
pub struct ReqwestLoader {
	options: Options,
	accept_header: String,
}

impl Default for ReqwestLoader {
	fn default() -> Self {
		Self::new_using(Options::default())
	}
}

impl ReqwestLoader {
	/// Creates a new loader with the given parsing function.
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new leader with the given options.
	pub fn new_using(options: Options) -> Self {
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

				json_ld_params.push_str(p.iri().as_str());
			}

			if options.request_profile.len() > 1 {
				json_ld_params.push('"');
			}
		}

		Self {
			options,
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

impl Loader for ReqwestLoader {
	async fn load(&self, url: &Iri) -> LoadingResult<IriBuf> {
		let mut redirection_number = 0;
		let mut url = url.to_owned();
		'next_url: loop {
			if redirection_number > self.options.max_redirections {
				return Err(LoadError::new(url.clone(), Error::TooManyRedirections));
			}

			log::debug!("downloading: {}", url);
			let request = self
				.options
				.client
				.get(url.as_str())
				.header(ACCEPT, &self.accept_header);

			let response = request
				.send()
				.await
				.map_err(|e| LoadError::new(url.clone(), e))?;

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
												return Err(LoadError::new(
													url,
													Error::MultipleContextLinkHeaders,
												));
											}

											context_url = Some(link.href().resolved(&url));
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
										profile.insert(Profile::new(iri));
									}
								}
							}

							let bytes = response.bytes().await.map_err(|e| {
								LoadError::new(url.clone(), Error::Reqwest(e.into()))
							})?;

							let decoder = utf8_decode::Decoder::new(bytes.iter().copied());
							let (document, _) = json_syntax::Value::parse_utf8(decoder)
								.map_err(|e| LoadError::new(url.clone(), Error::Parse(e)))?;

							break Ok(RemoteDocument::new_full(
								Some(url),
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
										url = link.href().resolved(&url);
										redirection_number += 1;
										continue 'next_url;
									}
								}
							}

							break Err(LoadError::new(url, Error::InvalidContentType));
						}
					}
				}
				code => break Err(LoadError::new(url, Error::QueryFailed(code))),
			}
		}
	}
}
