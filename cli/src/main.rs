use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use contextual::WithContext;
use iref::IriBuf;
use json_ld::{syntax::Parse, JsonLdProcessor, Print, RemoteDocument, RemoteDocumentReference};
use locspan::{Location, Span};
use rdf_types::{vocabulary::Index, IriVocabulary, IriVocabularyMut};

#[derive(Parser)]
#[clap(name="json-ld", author, version, about, long_about = None)]
struct Args {
	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", parse(from_occurrences))]
	verbosity: usize,

	#[clap(subcommand)]
	command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
	/// Download the document behind the given URL.
	Fetch { url: IriBuf },

	/// Expand the given JSON-LD document.
	Expand {
		/// URL or file path of the document to expand.
		///
		/// Of none, the standard input is used.
		url_or_path: Option<IriOrPath>,

		/// Base URL to use when reading from the standard input or file system.
		#[clap(short, long)]
		base_url: Option<IriBuf>,

		/// Relabel the nodes.
		///
		/// This will give a blank node identifier to unidentified nodes and
		/// replace existing blank node identifiers.
		#[clap(short = 'l', long)]
		relabel: bool,

		/// Put the expanded document in canonical form.
		#[clap(short, long)]
		canonicalize: bool,
	},

	Flatten {
		/// URL or file path of the document to flatten.
		///
		/// Of none, the standard input is used.
		url_or_path: Option<IriOrPath>,

		/// Base URL to use when reading from the standard input or file system.
		#[clap(short, long)]
		base_url: Option<IriBuf>,
	},
}

pub enum IriOrPath {
	Iri(IriBuf),
	Path(PathBuf),
}

impl FromStr for IriOrPath {
	type Err = std::convert::Infallible;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match IriBuf::new(s) {
			Ok(iri) => Ok(Self::Iri(iri)),
			Err(_) => Ok(Self::Path(s.into())),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Source {
	Nowhere,
	StdIn,
	Path,
	Iri(rdf_types::vocabulary::Index),
}

#[tokio::main]
async fn main() {
	// Parse options.
	let args = Args::parse();

	// Init logger.
	stderrlog::new().verbosity(args.verbosity).init().unwrap();

	let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();
	let mut loader: json_ld::loader::ReqwestLoader<Index, Location<Source, Span>> =
		json_ld::loader::ReqwestLoader::new_with_metadata_map(|_, url, span| {
			Location::new(Source::Iri(*url), span)
		});

	match args.command {
		Command::Fetch { url } => {
			let url = vocabulary.insert(url.as_iri());
			match RemoteDocumentReference::iri(url)
				.load_with(&mut vocabulary, &mut loader)
				.await
			{
				Ok(remote_document) => {
					log::info!(
						"document URL: {}",
						vocabulary.iri(remote_document.url().unwrap()).unwrap()
					);

					println!("{}", remote_document.document().pretty_print())
				}
				Err(e) => {
					eprintln!("error: {}", e);
					std::process::exit(1);
				}
			}
		}
		Command::Expand {
			url_or_path,
			base_url,
			relabel,
			canonicalize,
		} => {
			let remote_document = get_remote_document(&mut vocabulary, url_or_path, base_url);

			match remote_document
				.expand_with(&mut vocabulary, &mut loader)
				.await
			{
				Ok(mut expanded) => {
					if relabel {
						let mut generator =
							rdf_types::generator::Blank::new_with_prefix("b".to_string())
								.with_metadata(*expanded.metadata());

						if canonicalize {
							expanded.relabel_and_canonicalize_with(&mut vocabulary, &mut generator)
						} else {
							expanded.relabel_with(&mut vocabulary, &mut generator)
						}
					} else if canonicalize {
						expanded.canonicalize()
					}

					println!("{}", expanded.with(&vocabulary).pretty_print())
				}
				Err(e) => {
					eprintln!("error: {}", e);
					std::process::exit(1);
				}
			}
		}
		Command::Flatten {
			url_or_path,
			base_url,
		} => {
			let remote_document = get_remote_document(&mut vocabulary, url_or_path, base_url);

			let mut generator = rdf_types::generator::Blank::new_with_prefix("b".to_string())
				.with_metadata(Location::new(Source::Nowhere, Span::default()));

			match remote_document
				.flatten_with(&mut vocabulary, &mut generator, &mut loader)
				.await
			{
				Ok(flattened) => {
					println!("{}", flattened.with(&vocabulary).pretty_print())
				}
				Err(e) => {
					eprintln!("error: {}", e);
					std::process::exit(1);
				}
			}
		}
	}
}

fn get_remote_document(
	vocabulary: &mut impl IriVocabularyMut<Iri = Index>,
	url_or_path: Option<IriOrPath>,
	base_url: Option<IriBuf>,
) -> RemoteDocumentReference<Index, Location<Source, Span>> {
	match url_or_path {
		Some(IriOrPath::Iri(url)) => {
			let url = vocabulary.insert(url.as_iri());
			RemoteDocumentReference::iri(url)
		}
		Some(IriOrPath::Path(path)) => {
			let url = base_url.map(|iri| vocabulary.insert(iri.as_iri()));
			let source = url.map(Source::Iri).unwrap_or(Source::Path);

			match std::fs::read_to_string(path) {
				Ok(content) => {
					match json_ld::syntax::Value::parse_str(&content, |span| {
						Location::new(source, span)
					}) {
						Ok(document) => RemoteDocumentReference::Loaded(RemoteDocument::new(
							url,
							Some("application/ld+json".parse().unwrap()),
							document,
						)),
						Err(e) => {
							eprintln!("error: {}", e);
							std::process::exit(1);
						}
					}
				}
				Err(e) => {
					eprintln!("error: {}", e);
					std::process::exit(1);
				}
			}
		}
		None => {
			let url = base_url.map(|iri| vocabulary.insert(iri.as_iri()));
			let source = url.map(Source::Iri).unwrap_or(Source::StdIn);

			match std::io::read_to_string(std::io::stdin()) {
				Ok(content) => {
					match json_ld::syntax::Value::parse_str(&content, |span| {
						Location::new(source, span)
					}) {
						Ok(document) => RemoteDocumentReference::Loaded(RemoteDocument::new(
							url,
							Some("application/ld+json".parse().unwrap()),
							document,
						)),
						Err(e) => {
							eprintln!("error: {}", e);
							std::process::exit(1);
						}
					}
				}
				Err(e) => {
					eprintln!("error: {}", e);
					std::process::exit(1);
				}
			}
		}
	}
}
