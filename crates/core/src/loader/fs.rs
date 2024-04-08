use super::{Loader, RemoteDocument};
use crate::LoadingResult;
use json_syntax::Parse;
use rdf_types::vocabulary::{IriIndex, IriVocabulary};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// Loading error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// No mount point found for the given IRI.
	#[error("no mount point")]
	NoMountPoint,

	/// IO error.
	#[error(transparent)]
	IO(std::io::Error),

	/// Parse error.
	#[error("parse error: {0}")]
	Parse(json_syntax::parse::Error),
}

/// File-system loader.
///
/// This is a special JSON-LD document loader that can load document from the file system by
/// attaching a directory to specific URLs.
///
/// Loaded documents are not cached: a new file system read is made each time
/// an URL is loaded even if it has already been queried before.
pub struct FsLoader<I = IriIndex> {
	mount_points: HashMap<PathBuf, I>,
}

impl<I> FsLoader<I> {
	/// Bind the given IRI prefix to the given path.
	///
	/// Any document with an IRI matching the given prefix will be loaded from
	/// the referenced local directory.
	#[inline(always)]
	pub fn mount<P: AsRef<Path>>(&mut self, url: I, path: P) {
		self.mount_points.insert(path.as_ref().into(), url);
	}

	/// Returns the local file path associated to the given `url` if any.
	pub fn filepath(&self, vocabulary: &impl IriVocabulary<Iri = I>, url: &I) -> Option<PathBuf> {
		let url = vocabulary.iri(url).unwrap();
		for (path, target_url) in &self.mount_points {
			if let Some((suffix, _, _)) = url
				.as_iri_ref()
				.suffix(vocabulary.iri(target_url).unwrap().as_iri_ref())
			{
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

impl<I: Send> Loader<I> for FsLoader<I> {
	type Error = Error;

	async fn load_with<V>(&mut self, vocabulary: &mut V, url: I) -> LoadingResult<I, Error>
	where
		V: IriVocabulary<Iri = I>,
	{
		match self.filepath(vocabulary, &url) {
			Some(filepath) => {
				let file = File::open(filepath).map_err(Error::IO)?;
				let mut buf_reader = BufReader::new(file);
				let mut contents = String::new();
				buf_reader
					.read_to_string(&mut contents)
					.map_err(Error::IO)?;
				let (doc, _) = json_syntax::Value::parse_str(&contents).map_err(Error::Parse)?;
				Ok(RemoteDocument::new(
					Some(url),
					Some("application/ld+json".parse().unwrap()),
					doc,
				))
			}
			None => Err(Error::NoMountPoint),
		}
	}
}

impl<I> Default for FsLoader<I> {
	fn default() -> Self {
		Self {
			mount_points: HashMap::new(),
		}
	}
}

impl<I> FsLoader<I> {
	/// Creates a new file system loader with the given content `parser`.
	pub fn new() -> Self {
		Self::default()
	}
}
