use super::{Loader, RemoteDocument};
use futures::future::{BoxFuture, FutureExt};
use locspan::Meta;
use rdf_types::{vocabulary::IriIndex, IriVocabulary};
use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use url::Url;

/// File-URL loader.
///
/// This is a special JSON-LD document loader that can load document from any file: URL.
///
/// Loaded documents are not cached: a new file system read is made each time
/// an URL is loaded even if it has already been queried before.
pub struct FileUrlLoader<
	I = IriIndex,
	M = locspan::Location<I>,
	T = json_ld_syntax::Value<M>,
	E = json_ld_syntax::parse::MetaError<M>,
> {
	parser: Box<DynParser<I, M, T, E>>,
}

impl<I: Send, T: Send, M: Send, E> Loader<I, M> for FileUrlLoader<I, M, T, E> {
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
			let url_str = vocabulary.iri(&url).unwrap().as_str();
			if !url_str.starts_with("file:") {
				return Err(Error::NotFileUrl(url_str.into()));
			}
			let url_parsed = Url::parse(url_str).map_err(Error::InvalidUrl)?;
			let path = url_parsed
				.to_file_path()
				.map_err(|_| Error::BadFileUrl(url_str.into()))?;
			let file = File::open(path).map_err(Error::IO)?;
			let mut buf_reader = BufReader::new(file);
			let mut contents = String::new();
			buf_reader
				.read_to_string(&mut contents)
				.map_err(Error::IO)?;
			let doc = (*self.parser)(vocabulary, &url, contents.as_str()).map_err(Error::Parse)?;
			Ok(RemoteDocument::new(
				Some(url),
				Some("application/ld+json".parse().unwrap()),
				doc,
			))
		}
		.boxed()
	}
}

impl<I, M, T, E> FileUrlLoader<I, M, T, E> {
	/// Creates a new file system loader with the given content `parser`.
	pub fn new(
		parser: impl 'static
			+ Send
			+ Sync
			+ FnMut(&dyn IriVocabulary<Iri = I>, &I, &str) -> Result<Meta<T, M>, E>,
	) -> Self {
		Self {
			parser: Box::new(parser),
		}
	}
}

impl<I: Clone> Default
	for FileUrlLoader<
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

/// Loading error.
#[derive(Debug)]
pub enum Error<E> {
	/// URL parse error
	InvalidUrl(url::ParseError),

	/// URL is not a file
	NotFileUrl(String),

	/// file: URL does not encode a correct path
	BadFileUrl(String),

	/// IO error.
	IO(std::io::Error),

	/// Parse error.
	Parse(E),
}

impl<E: fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidUrl(e) => e.fmt(f),
			Self::NotFileUrl(url) => write!(f, "Not a file: URL: {url}"),
			Self::BadFileUrl(url) => write!(f, "Invalid path in {url}"),
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
