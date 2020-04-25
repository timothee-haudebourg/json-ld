use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, BufReader};
use futures::future::{FutureExt, LocalBoxFuture};
use iref::{Iri, IriBuf};
use json::JsonValue;
use crate::{
	Error,
	ErrorCode,
	RemoteDocument,
	context::{
		self,
		RemoteContext
	}
};

pub trait Loader {
	type Document;

	fn load<'a>(&'a mut self, url: Iri<'_>) -> LocalBoxFuture<'a, Result<RemoteDocument<Self::Document>, Error>>;
}

impl<L: Loader<Document = JsonValue>> context::Loader for L {
	type Output = JsonValue;

	fn load_context<'a>(&'a mut self, url: Iri) -> LocalBoxFuture<'a, Result<RemoteContext<JsonValue>, Error>> {
		let url = IriBuf::from(url);
		async move {
			match self.load(url.as_iri()).await {
				Ok(remote_doc) => {
					let (doc, url) = remote_doc.into_parts();
					if let JsonValue::Object(obj) = doc {
						if let Some(context) = obj.get("@context") {
							Ok(RemoteContext::from_parts(url, context.clone()))
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
		}.boxed_local()
	}
}

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote ressources.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a ressource.
pub struct NoLoader;

impl Loader for NoLoader {
	type Document = JsonValue;

	fn load<'a>(&'a mut self, _url: Iri<'_>) -> LocalBoxFuture<'a, Result<RemoteDocument<Self::Document>, Error>> {
		async move {
			Err(ErrorCode::LoadingDocumentFailed.into())
		}.boxed_local()
	}
}

/// File-system loader.
///
/// This is a special JSON-LD document loader that can load document from the file system by
/// attaching a directory to specific URLs.
pub struct FsLoader {
	cache: HashMap<IriBuf, RemoteDocument>,
	mount_points: HashMap<PathBuf, IriBuf>
}

impl FsLoader {
	pub fn new() -> FsLoader {
		FsLoader {
			cache: HashMap::new(),
			mount_points: HashMap::new()
		}
	}

	pub fn mount<P: AsRef<Path>>(&mut self, url: Iri, path: P) {
		self.mount_points.insert(path.as_ref().into(), url.into());
	}
}

impl Loader for FsLoader {
	type Document = JsonValue;

	fn load<'a>(&'a mut self, url: Iri<'_>) -> LocalBoxFuture<'a, Result<RemoteDocument<Self::Document>, Error>> {
		let url: IriBuf = url.into();
		async move {
			match self.cache.get(&url) {
				Some(doc) => Ok(doc.clone()),
				None => {
					for (path, target_url) in &self.mount_points {
						let url_ref = url.as_iri_ref();
						match url_ref.suffix(target_url.as_iri_ref()) {
							Some((suffix, _, _)) => {
								let mut filepath = path.clone();
								for seg in suffix.as_path().segments() {
									filepath.push(seg.as_str())
								}

								if let Ok(file) = File::open(filepath) {
								    let mut buf_reader = BufReader::new(file);
								    let mut contents = String::new();
								    if buf_reader.read_to_string(&mut contents).is_ok() {
										if let Ok(doc) = json::parse(contents.as_str()) {
											let remote_doc = RemoteDocument::new(doc, url.as_iri());
											self.cache.insert(url.clone(), remote_doc.clone());
											return Ok(remote_doc)
										} else {
											return Err(ErrorCode::LoadingDocumentFailed.into())
										}
									} else {
										return Err(ErrorCode::LoadingDocumentFailed.into())
									}
								} else {
									return Err(ErrorCode::LoadingDocumentFailed.into())
								}
							},
							None => ()
						}
					}

					Err(ErrorCode::LoadingDocumentFailed.into())
				}
			}
		}.boxed_local()
	}
}
