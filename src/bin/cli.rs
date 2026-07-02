use clap::{Parser, Subcommand};
use json_ld::{
	AsyncLoader, Document, ExpandedDocument, Iri, IriBuf, JsonLdOptions, JsonLdProcessor,
	JsonLdSourceCode, RemoteDocument, TokioFsLoader, ext::miette::JsonLdDiagnostic,
};
use json_syntax::{JsonParse, JsonPrint, JsonValue};
use miette::{Diagnostic, NamedSource, SourceSpan};
use nquads_syntax::grdf_document_from_str;
use rdf_syntax::{RdfDisplay, generator::BlankIdGenerator};
use std::future::Future;
use std::io::{self, Read};
use thiserror::Error;

#[derive(Parser)]
#[command(
	name = "json-ld",
	about = "A JSON-LD processor: expand, compact, flatten, or convert to/from RDF.\n\
	         Input is read from standard input.",
	version
)]
struct Cli {
	#[command(subcommand)]
	command: Command,

	/// Override the base IRI for the document.
	#[arg(long, global = true)]
	base: Option<IriBuf>,
}

#[derive(Subcommand)]
enum Command {
	/// Expand a JSON-LD document to its canonical expanded form.
	Expand,

	/// Compact a JSON-LD document relative to a context.
	Compact {
		/// Context IRI or local file path used for compaction.
		context: IriBuf,
	},

	/// Flatten a JSON-LD document, moving all nested nodes to the top level.
	Flatten {
		/// Optional context IRI or file path for compacting the flattened result.
		#[arg(long)]
		context: Option<IriBuf>,
	},

	/// Serialize a JSON-LD document to RDF N-Quads.
	ToRdf,

	/// Deserialize RDF N-Quads to an expanded JSON-LD document.
	FromRdf,
}

impl Command {
	async fn run(
		self,
		loader: &impl AsyncLoader,
		input: &str,
		options: JsonLdOptions,
	) -> miette::Result<()> {
		match self {
			Self::Expand => {
				let doc = parse_document(input, options.base.as_deref())?;

				let expanded = doc
					.async_expand_with(&loader, options)
					.await
					.map_err_async(|e| JsonLdDiagnostic::new_async(loader, &doc, e))
					.await?;

				println!("{}", expanded_to_json(expanded).pretty_print());
				Ok(())
			}
			Self::Compact { context } => {
				let doc = parse_document(input, options.base.as_deref())?;

				let compacted = doc
					.async_compact_with(RemoteDocument::Iri(context), loader, options)
					.await
					.map_err_async(|e| JsonLdDiagnostic::new_async(loader, &doc, e))
					.await?;

				println!("{}", compacted.pretty_print());
				Ok(())
			}
			Self::Flatten { context } => {
				let doc = parse_document(input, options.base.as_deref())?;

				let flattened = doc
					.async_flatten_with(context.map(RemoteDocument::Iri), loader, options)
					.await
					.map_err_async(|e| JsonLdDiagnostic::new_async(loader, &doc, e))
					.await?;

				println!("{}", flattened.pretty_print());
				Ok(())
			}
			Self::ToRdf => {
				let doc = parse_document(input, options.base.as_deref())?;
				let generator = BlankIdGenerator::new_with_prefix("b".to_string());

				let quads = doc
					.async_to_rdf_with(loader, generator, options)
					.await
					.map_err_async(|e| JsonLdDiagnostic::new_async(loader, &doc, e))
					.await?;

				for q in &quads {
					println!("{} .", q.rdf_display());
				}
				Ok(())
			}
			Self::FromRdf => {
				let (quads, _) = grdf_document_from_str(input).map_err(|e| NQuadsParseError {
					message: e.to_string(),
				})?;

				let expanded =
					json_ld::linked_data::de::from_rdf_quads::<ExpandedDocument>(quads, [])
						.map_err(|e| FromRdfError {
							message: e.to_string(),
						})?;

				println!("{}", expanded_to_json(expanded).pretty_print());
				Ok(())
			}
		}
	}
}

#[tokio::main]
async fn main() -> miette::Result<()> {
	let cli = Cli::parse();

	let loader = TokioFsLoader::new();
	let input = read_stdin()?;

	let mut options = JsonLdOptions::default();
	options.base = cli.base;

	cli.command.run(&loader, &input, options).await?;

	Ok(())
}

/// Read all of standard input into a `String`.
fn read_stdin() -> miette::Result<String> {
	let mut buf = String::new();
	io::stdin()
		.read_to_string(&mut buf)
		.map_err(|e| StdinError { source: e })?;
	Ok(buf)
}

/// Parse a JSON-LD document from a `&str` (e.g. stdin contents).
///
/// Wraps the parse error in a miette diagnostic with source highlighting.
fn parse_document(content: &str, base: Option<&Iri>) -> miette::Result<Document> {
	let (value, code_map) = JsonValue::parse_str(content).map_err(|e| JsonParseError {
		message: e.to_string(),
		src: NamedSource::new("stdin", content.to_owned()),
		span: SourceSpan::new(e.position().into(), 1usize),
	})?;

	Ok(Document::new_full(
		base.map(ToOwned::to_owned),
		Some("application/ld+json".parse().unwrap()),
		None,
		Default::default(),
		Some(JsonLdSourceCode::new(content.to_owned(), code_map)),
		value,
	))
}

/// Serialize an [`ExpandedDocument`] to a pretty-printed JSON value.
fn expanded_to_json(doc: ExpandedDocument) -> JsonValue {
	json_ld::syntax::to_value(doc.into_objects()).unwrap()
}

#[derive(Debug, Error, Diagnostic)]
#[error("could not read from stdin: {source}")]
#[diagnostic(code(json_ld::io_error))]
struct StdinError {
	#[source]
	source: io::Error,
}

#[derive(Debug, Error, Diagnostic)]
#[error("JSON parse error: {message}")]
#[diagnostic(
	code(json_ld::json_parse_error),
	help("check the JSON syntax at the highlighted position")
)]
struct JsonParseError {
	message: String,
	#[source_code]
	src: NamedSource<String>,
	#[label("unexpected input here")]
	span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("N-Quads parse error: {message}")]
#[diagnostic(
	code(json_ld::nquads_parse_error),
	help("ensure the input is valid N-Quads or N-Triples syntax")
)]
struct NQuadsParseError {
	message: String,
}

#[derive(Debug, Error, Diagnostic)]
#[error("failed to deserialize RDF to JSON-LD: {message}")]
#[diagnostic(code(json_ld::from_rdf_error))]
struct FromRdfError {
	message: String,
}

trait ResultExt<T, E> {
	/// Maps a `Result<T, E>` to `Result<T, U>` by applying the async function
	/// `f` to a contained `Err` value, leaving an `Ok` value untouched.
	async fn map_err_async<U, F, Fut>(self, f: F) -> Result<T, U>
	where
		F: FnOnce(E) -> Fut,
		Fut: Future<Output = U>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
	async fn map_err_async<U, F, Fut>(self, f: F) -> Result<T, U>
	where
		F: FnOnce(E) -> Fut,
		Fut: Future<Output = U>,
	{
		match self {
			Ok(t) => Ok(t),
			Err(e) => Err(f(e).await),
		}
	}
}
