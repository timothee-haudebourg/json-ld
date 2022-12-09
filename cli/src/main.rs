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
		/// URL of the document to expand.
		///
		/// Of none, the standard input is used.
		url: Option<IriBuf>,

		/// Base URL to use when reading from the standard input.
		#[clap(short, long)]
		base_url: Option<IriBuf>,
	},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Source {
	StdIn,
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
		Command::Expand { url, base_url } => {
			let remote_document = match url {
				Some(url) => {
					let url = vocabulary.insert(url.as_iri());
					RemoteDocumentReference::iri(url)
				}
				None => {
					let url = base_url.map(|iri| vocabulary.insert(iri.as_iri()));
					let source = url.map(Source::Iri).unwrap_or(Source::StdIn);

					match std::io::read_to_string(std::io::stdin()) {
						Ok(content) => {
							match json_ld::syntax::Value::parse_str(&content, |span| {
								Location::new(source, span)
							}) {
								Ok(document) => {
									RemoteDocumentReference::Loaded(RemoteDocument::new(
										url,
										Some("application/ld+json".parse().unwrap()),
										document,
									))
								}
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
			};

			match remote_document
				.expand_with(&mut vocabulary, &mut loader)
				.await
			{
				Ok(expanded) => {
					println!("{}", expanded.with(&vocabulary).pretty_print())
				}
				Err(e) => {
					eprintln!("error: {}", e);
					std::process::exit(1);
				}
			}
		}
	}
}
