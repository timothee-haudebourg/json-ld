use super::{Loader, RemoteDocument};
use crate::{LoadError, LoadingResult};
use iref::{Iri, IriBuf};
use json_syntax::Parse;
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
	#[error("IO: {0}")]
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
#[derive(Default)]
pub struct FsLoader {
	mount_points: Vec<(PathBuf, IriBuf)>,
}

impl FsLoader {
	/// Creates a new file system loader with the given content `parser`.
	pub fn new() -> Self {
		Self::default()
	}

	/// Bind the given IRI prefix to the given path.
	///
	/// Any document with an IRI matching the given prefix will be loaded from
	/// the referenced local directory.
	#[inline(always)]
	pub fn mount<P: AsRef<Path>>(&mut self, url: IriBuf, path: P) {
		self.mount_points.push((path.as_ref().into(), url));
	}

	/// Returns the local file path associated to the given `url` if any.
	pub fn filepath(&self, url: &Iri) -> Option<PathBuf> {
		for (path, target_url) in &self.mount_points {
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

impl Loader for FsLoader {
	async fn load(&self, url: &Iri) -> LoadingResult<IriBuf> {
		match self.filepath(url) {
			Some(filepath) => {
				let file = File::open(filepath)
					.map_err(|e| LoadError::new(url.to_owned(), Error::IO(e)))?;
				let mut buf_reader = BufReader::new(file);
				let mut contents = String::new();
				buf_reader
					.read_to_string(&mut contents)
					.map_err(|e| LoadError::new(url.to_owned(), Error::IO(e)))?;
				let (doc, _) = json_syntax::Value::parse_str(&contents)
					.map_err(|e| LoadError::new(url.to_owned(), Error::Parse(e)))?;
				Ok(RemoteDocument::new(
					Some(url.to_owned()),
					Some("application/ld+json".parse().unwrap()),
					doc,
				))
			}
			None => Err(LoadError::new(url.to_owned(), Error::NoMountPoint)),
		}
	}
}
