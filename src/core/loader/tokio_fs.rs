use json_syntax::{JsonParse, JsonValue};
use rdf_syntax::{Iri, IriBuf};
use std::path::PathBuf;

use crate::{Document, JsonLdSourceCode, LoadError};

use super::{AsyncLoader, FsLoader};

pub use super::fs::Error;

/// Async file-system loader backed by Tokio.
///
/// This is an async counterpart to [`FsLoader`] that performs file I/O using
/// [`tokio::fs`] rather than blocking the thread.
///
/// Loaded documents are not cached: a new file system read is made each time
/// an IRI is loaded even if it has already been queried before.
pub struct TokioFsLoader(FsLoader);

impl Default for TokioFsLoader {
	fn default() -> Self {
		Self(FsLoader::default())
	}
}

impl TokioFsLoader {
	/// Creates a new Tokio file-system loader.
	pub fn new() -> Self {
		Self::default()
	}

	/// Bind the given IRI prefix to the given local directory path.
	///
	/// Any document whose IRI matches the given prefix will be loaded from the
	/// referenced local directory.
	pub fn mount<P: AsRef<std::path::Path>>(&mut self, url: IriBuf, path: P) {
		self.0.mount(url, path)
	}

	/// Returns the local file path associated to the given `url`, if any.
	pub fn filepath(&self, url: &Iri) -> Option<PathBuf> {
		self.0.filepath(url)
	}
}

impl AsyncLoader for TokioFsLoader {
	async fn async_load(&self, url: &Iri) -> Result<Document, LoadError> {
		match self.filepath(url) {
			Some(filepath) => {
				let contents = tokio::fs::read_to_string(&filepath)
					.await
					.map_err(|e| LoadError::new(url.to_owned(), Error::IO(e)))?;
				let (doc, code_map) = JsonValue::parse_str(&contents)
					.map_err(|e| LoadError::new(url.to_owned(), Error::Parse(e)))?;
				Ok(Document::new_full(
					Some(url.to_owned()),
					Some("application/ld+json".parse().unwrap()),
					None,
					Default::default(),
					Some(JsonLdSourceCode::new(contents, code_map)),
					doc,
				))
			}
			None => Err(LoadError::new(url.to_owned(), Error::NoMountPoint)),
		}
	}
}
