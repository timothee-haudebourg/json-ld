use super::{Loader, RemoteDocument};
use crate::{LoaderError, LoadingResult};
use iref::IriBuf;
use json_syntax::Parse;
use rdf_types::vocabulary::{IriIndex, IriVocabulary};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// Loading error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// No mount point found for the given IRI.
	#[error("no mount point for `{0}`")]
	NoMountPoint(IriBuf),

	/// IO error.
	#[error("IO error while loading `{0}`: {1}")]
	IO(IriBuf, std::io::Error),

	/// Parse error.
	#[error("parse error on `{0}`: {1}")]
	Parse(IriBuf, json_syntax::parse::Error),
}

impl LoaderError for Error {
	fn into_iri_and_message(self) -> (IriBuf, String) {
		match self {
			Self::NoMountPoint(iri) => (iri, "no mount point".to_owned()),
			Self::IO(iri, e) => (iri, e.to_string()),
			Self::Parse(iri, e) => (iri, format!("parse error: {e}")),
		}
	}
}

/// File-system loader.
///
/// This is a special JSON-LD document loader that can load document from the file system by
/// attaching a directory to specific URLs.
///
/// Loaded documents are not cached: a new file system read is made each time
/// an URL is loaded even if it has already been queried before.
pub struct FsLoader<I = IriIndex> {
	mount_points: Vec<(PathBuf, I)>,
}

impl<I> FsLoader<I> {
	/// Bind the given IRI prefix to the given path.
	///
	/// Any document with an IRI matching the given prefix will be loaded from
	/// the referenced local directory.
	#[inline(always)]
	pub fn mount<P: AsRef<Path>>(&mut self, url: I, path: P) {
		self.mount_points.push((path.as_ref().into(), url));
	}

	/// Returns the local file path associated to the given `url` if any.
	pub fn filepath(&self, vocabulary: &impl IriVocabulary<Iri = I>, url: &I) -> Option<PathBuf> {
		let url = vocabulary.iri(url).unwrap();
		for (path, target_url) in &self.mount_points {
			let target_url = vocabulary.iri(target_url).unwrap().as_iri_ref();
			if let Some((suffix, _, _)) = url.as_iri_ref().suffix(target_url) {
				let mut filepath = path.clone();
				for seg in suffix.as_path().segments() {
					filepath.push(seg.as_str())
				}

				return Some(filepath);
			}
		}

		None
	}
}

impl<I> Loader<I> for FsLoader<I> {
	type Error = Error;

	async fn load_with<V>(&mut self, vocabulary: &mut V, url: I) -> LoadingResult<I, Error>
	where
		V: IriVocabulary<Iri = I>,
	{
		match self.filepath(vocabulary, &url) {
			Some(filepath) => {
				let file = File::open(filepath)
					.map_err(|e| Error::IO(vocabulary.iri(&url).unwrap().to_owned(), e))?;
				let mut buf_reader = BufReader::new(file);
				let mut contents = String::new();
				buf_reader
					.read_to_string(&mut contents)
					.map_err(|e| Error::IO(vocabulary.iri(&url).unwrap().to_owned(), e))?;
				let (doc, _) = json_syntax::Value::parse_str(&contents)
					.map_err(|e| Error::Parse(vocabulary.iri(&url).unwrap().to_owned(), e))?;
				Ok(RemoteDocument::new(
					Some(url),
					Some("application/ld+json".parse().unwrap()),
					doc,
				))
			}
			None => Err(Error::NoMountPoint(vocabulary.owned_iri(url).ok().unwrap())),
		}
	}
}

impl<I> Default for FsLoader<I> {
	fn default() -> Self {
		Self {
			mount_points: Vec::new(),
		}
	}
}

impl<I> FsLoader<I> {
	/// Creates a new file system loader with the given content `parser`.
	pub fn new() -> Self {
		Self::default()
	}
}
