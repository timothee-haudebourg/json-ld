use super::{Loader, RemoteDocument};
use futures::future::{BoxFuture, FutureExt};
use locspan::Meta;
use rdf_types::{vocabulary::Index, IriVocabulary};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error<E> {
	NoMountPoint,
	IO(std::io::Error),
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

type DynParser<I, T, M, E> =
	dyn 'static + Send + Sync + FnMut(&dyn IriVocabulary<I>, &I, &str) -> Result<Meta<T, M>, E>;

/// File-system loader.
///
/// This is a special JSON-LD document loader that can load document from the file system by
/// attaching a directory to specific URLs.
pub struct FsLoader<
	I = Index,
	T = json_syntax::Value<locspan::Location<I>>,
	M = locspan::Location<I>,
	E = json_syntax::parse::MetaError<locspan::Location<I>>,
> {
	mount_points: HashMap<PathBuf, I>,
	cache: HashMap<I, Meta<T, M>>,
	parser: Box<DynParser<I, T, M, E>>,
}

impl<I, T, M, E> FsLoader<I, T, M, E> {
	#[inline(always)]
	pub fn mount<P: AsRef<Path>>(&mut self, url: I, path: P) {
		self.mount_points.insert(path.as_ref().into(), url);
	}

	pub fn filepath(&self, vocabulary: &impl IriVocabulary<I>, url: &I) -> Option<PathBuf> {
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

impl<I: Clone + Eq + Hash + Send, T: Clone + Send, M: Clone + Send, E> Loader<I, M>
	for FsLoader<I, T, M, E>
{
	type Output = T;
	type Error = Error<E>;

	fn load_in<'a>(
		&'a mut self,
		vocabulary: &'a (impl Sync + IriVocabulary<I>),
		url: I,
	) -> BoxFuture<'a, Result<RemoteDocument<I, T, M>, Self::Error>>
	where
		I: 'a,
	{
		async move {
			match self.cache.get(&url) {
				Some(t) => Ok(RemoteDocument::new(Some(url), t.clone())),
				None => match self.filepath(vocabulary, &url) {
					Some(filepath) => {
						let file = File::open(filepath).map_err(Error::IO)?;
						let mut buf_reader = BufReader::new(file);
						let mut contents = String::new();
						buf_reader
							.read_to_string(&mut contents)
							.map_err(Error::IO)?;
						let doc = (*self.parser)(vocabulary, &url, contents.as_str())
							.map_err(Error::Parse)?;
						self.cache.insert(url.clone(), doc.clone());
						Ok(RemoteDocument::new(Some(url), doc))
					}
					None => Err(Error::NoMountPoint),
				},
			}
		}
		.boxed()
	}
}

impl<I, T, M, E> FsLoader<I, T, M, E> {
	pub fn new(
		parser: impl 'static
			+ Send
			+ Sync
			+ FnMut(&dyn IriVocabulary<I>, &I, &str) -> Result<Meta<T, M>, E>,
	) -> Self {
		Self {
			mount_points: HashMap::new(),
			cache: HashMap::new(),
			parser: Box::new(parser),
		}
	}
}

impl<I: Clone> Default
	for FsLoader<
		I,
		json_syntax::Value<locspan::Location<I>>,
		locspan::Location<I>,
		json_syntax::parse::MetaError<locspan::Location<I>>,
	>
{
	fn default() -> Self {
		use json_syntax::Parse;
		Self::new(|_, file: &I, s| {
			json_syntax::Value::parse_str(s, |span| locspan::Location::new(file.clone(), span))
		})
	}
}
