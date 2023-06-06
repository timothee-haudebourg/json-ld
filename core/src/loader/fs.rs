use super::{Loader, RemoteDocument};
use futures::future::{BoxFuture, FutureExt};
use locspan::Meta;
use rdf_types::{vocabulary::IriIndex, IriVocabulary};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// Loading error.
#[derive(Debug)]
pub enum Error<E> {
	/// No mount point found for the given IRI.
	NoMountPoint,

	/// IO error.
	IO(std::io::Error),

	/// Parse error.
	Parse(E),
}

impl<E> Error<E> {
	pub fn map<U>(self, f: impl FnOnce(E) -> U) -> Error<U> {
		match self {
			Self::NoMountPoint => Error::NoMountPoint,
			Self::IO(e) => Error::IO(e),
			Self::Parse(e) => Error::Parse(f(e)),
		}
	}
}

impl<E: fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::NoMountPoint => write!(f, "no mount point"),
			Self::IO(e) => e.fmt(f),
			Self::Parse(e) => e.fmt(f),
		}
	}
}

/// Dynamic parser type.
type DynParser<I, M, T, E> = dyn 'static
	+ Send
	+ Sync
	+ FnMut(&dyn IriVocabulary<Iri = I>, &I, &str) -> Result<Meta<T, M>, E>;

/// File-system loader.
///
/// This is a special JSON-LD document loader that can load document from the file system by
/// attaching a directory to specific URLs.
///
/// Loaded documents are not cached: a new file system read is made each time
/// an URL is loaded even if it has already been queried before.
pub struct FsLoader<
	I = IriIndex,
	M = locspan::Location<I>,
	T = json_ld_syntax::Value<M>,
	E = json_ld_syntax::parse::MetaError<M>,
> {
	mount_points: HashMap<PathBuf, I>,
	parser: Box<DynParser<I, M, T, E>>,
}

impl<I, M, T, E> FsLoader<I, M, T, E> {
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
			if let Some((suffix, _, _)) =
				url.suffix(vocabulary.iri(target_url).unwrap().as_iri_ref())
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

impl<I: Send, T: Send, M: Send, E> Loader<I, M> for FsLoader<I, M, T, E> {
	type Output = T;
	type Error = Error<E>;

	fn load_with<'a>(
		&'a mut self,
		vocabulary: &'a mut (impl Sync + Send + IriVocabulary<Iri = I>),
		url: I,
	) -> BoxFuture<'a, Result<RemoteDocument<I, M, T>, Self::Error>>
	where
		I: 'a,
	{
		async move {
			match self.filepath(vocabulary, &url) {
				Some(filepath) => {
					let file = File::open(filepath).map_err(Error::IO)?;
					let mut buf_reader = BufReader::new(file);
					let mut contents = String::new();
					buf_reader
						.read_to_string(&mut contents)
						.map_err(Error::IO)?;
					let doc = (*self.parser)(vocabulary, &url, contents.as_str())
						.map_err(Error::Parse)?;
					Ok(RemoteDocument::new(
						Some(url),
						Some("application/ld+json".parse().unwrap()),
						doc,
					))
				}
				None => Err(Error::NoMountPoint),
			}
		}
		.boxed()
	}
}

impl<I, M, T, E> FsLoader<I, M, T, E> {
	/// Creates a new file system loader with the given content `parser`.
	pub fn new(
		parser: impl 'static
			+ Send
			+ Sync
			+ FnMut(&dyn IriVocabulary<Iri = I>, &I, &str) -> Result<Meta<T, M>, E>,
	) -> Self {
		Self {
			mount_points: HashMap::new(),
			parser: Box::new(parser),
		}
	}
}

impl<I: Clone> Default
	for FsLoader<
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
