use crate::{Error, ErrorCode, RemoteDocument};
use futures::future::{BoxFuture, FutureExt};
use generic_json::Json;
use iref::{Iri, IriRef, IriBuf};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::{marker::PhantomData, str::FromStr};

/// Identifier reference.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Id(usize);

impl Id {
	pub fn new(index: usize) -> Self {
		Self(index)
	}

	pub fn unwrap(self) -> usize {
		self.0
	}
}

impl From<Id> for usize {
	fn from(id: Id) -> Self {
		id.0
	}
}

/// JSON document loader.
///
/// Each document is uniquely identified by the loader by a `u32`.
pub trait Loader {
	/// The type of documents that can be loaded.
	type Document: Json;

	/// Returns the unique identifier associated to the given IRI, if any.
	fn id(&self, iri: Iri<'_>) -> Option<Id>;

	/// Returns the unique identifier associated to the given IRI, if any.
	///
	/// Returns `None` if the input `iri` is `None`.
	#[inline(always)]
	fn id_opt(&self, iri: Option<Iri<'_>>) -> Option<Id> {
		iri.and_then(|iri| self.id(iri))
	}

	/// Returns the IRI with the given identifier, if any.
	fn iri(&self, id: Id) -> Option<Iri<'_>>;

	/// Loads the document behind the given IRI.
	fn load<'a>(
		&'a mut self,
		url: Iri<'_>,
	) -> BoxFuture<'a, Result<RemoteDocument<Self::Document>, Error>>;
}

/// Dummy loader.
///
/// A dummy loader that does not load anything.
/// Can be useful when you know that you will never need to load remote ressources.
///
/// Raises an `LoadingDocumentFailed` at every attempt to load a ressource.
pub struct NoLoader<J>(PhantomData<J>);

impl<J> NoLoader<J> {
	#[inline(always)]
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<J> Default for NoLoader<J> {
	#[inline(always)]
	fn default() -> Self {
		Self::new()
	}
}

impl<J: Json> Loader for NoLoader<J> {
	type Document = J;

	#[inline(always)]
	fn id(&self, _iri: Iri<'_>) -> Option<Id> {
		None
	}

	#[inline(always)]
	fn iri(&self, _id: Id) -> Option<Iri<'_>> {
		None
	}

	#[inline(always)]
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
	namespace: HashMap<IriBuf, Id>,
	cache: Vec<(J, IriBuf)>,
	mount_points: HashMap<PathBuf, IriBuf>,
	parser: Box<dyn 'static + Send + Sync + FnMut(&str) -> Result<J, Error>>,
}

impl<J> FsLoader<J> {
	pub fn new<E: 'static + std::error::Error>(
		mut parser: impl 'static + Send + Sync + FnMut(&str) -> Result<J, E>,
	) -> Self {
		Self {
			namespace: HashMap::new(),
			cache: Vec::new(),
			mount_points: HashMap::new(),
			parser: Box::new(move |s| {
				parser(s).map_err(|e| Error::with_source(ErrorCode::LoadingDocumentFailed, e))
			}),
		}
	}

	#[inline(always)]
	pub fn mount<P: AsRef<Path>>(&mut self, url: Iri, path: P) {
		self.mount_points.insert(path.as_ref().into(), url.into());
	}

	pub fn filepath(&self, url: IriRef) -> Option<PathBuf> {
		for (path, target_url) in &self.mount_points {
			if let Some((suffix, _, _)) = url.suffix(target_url.as_iri_ref()) {
				let mut filepath = path.clone();
				for seg in suffix.as_path().segments() {
					filepath.push(seg.as_str())
				}

				return Some(filepath)
			}
		}

		None
	}

	/// Allocate a identifier to the given IRI.
	fn allocate(&mut self, iri: IriBuf, doc: J) -> Id {
		let id = Id::new(self.cache.len());
		self.namespace.insert(iri.clone(), id);
		self.cache.push((doc, iri));
		id
	}
}

impl<J: FromStr> Default for FsLoader<J>
where
	J::Err: 'static + std::error::Error,
{
	#[inline(always)]
	fn default() -> Self {
		Self::new(|s| J::from_str(s))
	}
}

impl<J: Json + Clone + Send> Loader for FsLoader<J> {
	type Document = J;

	#[inline(always)]
	fn id(&self, iri: Iri<'_>) -> Option<Id> {
		self.namespace.get(&IriBuf::from(iri)).cloned()
	}

	#[inline(always)]
	fn iri(&self, id: Id) -> Option<Iri<'_>> {
		self.cache.get(id.unwrap()).map(|(_, iri)| iri.as_iri())
	}

	fn load<'a>(&'a mut self, url: Iri<'_>) -> BoxFuture<'a, Result<RemoteDocument<J>, Error>> {
		let url: IriBuf = url.into();
		async move {
			match self.namespace.get(&url) {
				Some(id) => Ok(RemoteDocument::new(
					self.cache[id.unwrap()].0.clone(),
					url,
					*id,
				)),
				None => {
					match self.filepath(url.as_iri_ref()) {
						Some(filepath) => {
							if let Ok(file) = File::open(filepath) {
								let mut buf_reader = BufReader::new(file);
								let mut contents = String::new();
								if buf_reader.read_to_string(&mut contents).is_ok() {
									let doc = (*self.parser)(contents.as_str())?;
									let id = self.allocate(url.clone(), doc.clone());
									return Ok(RemoteDocument::new(doc, url, id));
								} else {
									return Err(ErrorCode::LoadingDocumentFailed.into());
								}
							} else {
								return Err(ErrorCode::LoadingDocumentFailed.into());
							}
						}
						None => {
							Err(ErrorCode::LoadingDocumentFailed.into())
						}
					}
				}
			}
		}
		.boxed()
	}
}
