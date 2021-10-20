use crate::{
	context::{self, RemoteContext},
	Error, ErrorCode, RemoteDocument,
};
use std::{
	marker::PhantomData,
	str::FromStr
};
use futures::future::{BoxFuture, FutureExt};
use generic_json::Json;
use iref::{Iri, IriBuf};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// JSON document loader.
pub trait Loader {
	/// The type of documents that can be loaded.
	type Document: Json;

	/// Load the document behind the given URL.
	fn load<'a>(
		&'a mut self,
		url: Iri<'_>,
	) -> BoxFuture<'a, Result<RemoteDocument<Self::Document>, Error>>;
}

impl<L: Send + Sync + Loader> context::Loader for L
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

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote ressources.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a ressource.
pub struct NoLoader<J>(PhantomData<J>);

impl<J> NoLoader<J> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<J> Default for NoLoader<J> {
	fn default() -> Self {
		Self::new()
	}
}

impl<J: Json> Loader for NoLoader<J> {
	type Document = J;

	fn load<'a>(
		&'a mut self,
		_url: Iri<'_>,
	) -> BoxFuture<'a, Result<RemoteDocument<Self::Document>, Error>> {
		async move { Err(ErrorCode::LoadingDocumentFailed.into()) }.boxed()
	}
}

/// File-system loader.
///
/// This is a special JSON-LD document loader that can load document from the file system by
/// attaching a directory to specific URLs.
pub struct FsLoader<J> {
	cache: HashMap<IriBuf, RemoteDocument<J>>,
	mount_points: HashMap<PathBuf, IriBuf>,
	parser: Box<dyn 'static + Send + FnMut(&str) -> Result<J, Error>>
}

impl<J> FsLoader<J> {
	pub fn new<E: 'static + std::error::Error>(mut parser: impl 'static + Send + FnMut(&str) -> Result<J, E>) -> Self {
		Self {
			cache: HashMap::new(),
			mount_points: HashMap::new(),
			parser: Box::new(move |s| parser(s).map_err(|e| Error::new(ErrorCode::LoadingDocumentFailed, e)))
		}
	}

	pub fn mount<P: AsRef<Path>>(&mut self, url: Iri, path: P) {
		self.mount_points.insert(path.as_ref().into(), url.into());
	}
}

impl<J: FromStr> Default for FsLoader<J> where J::Err: 'static + std::error::Error {
	fn default() -> Self {
		Self::new(|s| J::from_str(s))
	}
}

impl<J: Json + Clone + Send> Loader for FsLoader<J> {
	type Document = J;

	fn load<'a>(
		&'a mut self,
		url: Iri<'_>,
	) -> BoxFuture<'a, Result<RemoteDocument<Self::Document>, Error>> {
		let url: IriBuf = url.into();
		async move {
			match self.cache.get(&url) {
				Some(doc) => Ok(doc.clone()),
				None => {
					for (path, target_url) in &self.mount_points {
						let url_ref = url.as_iri_ref();
						if let Some((suffix, _, _)) = url_ref.suffix(target_url.as_iri_ref()) {
							let mut filepath = path.clone();
							for seg in suffix.as_path().segments() {
								filepath.push(seg.as_str())
							}

							if let Ok(file) = File::open(filepath) {
								let mut buf_reader = BufReader::new(file);
								let mut contents = String::new();
								if buf_reader.read_to_string(&mut contents).is_ok() {
									let doc = (*self.parser)(contents.as_str())?;
									let remote_doc = RemoteDocument::new(doc, url.as_iri());
									self.cache.insert(url.clone(), remote_doc.clone());
									return Ok(remote_doc);
								} else {
									return Err(ErrorCode::LoadingDocumentFailed.into());
								}
							} else {
								return Err(ErrorCode::LoadingDocumentFailed.into());
							}
						}
					}

					Err(ErrorCode::LoadingDocumentFailed.into())
				}
			}
		}
		.boxed()
	}
}
