use clap::{Parser, Subcommand};
use json_ld::{
	AsyncLoader, Document, DocumentSource, ExpandedDocument, FsLoader, Iri, IriBuf, JsonLdOptions,
	JsonLdProcessor, RemoteContext, TokioFsLoader,
	algorithms::{JsonLdLocated, JsonLdSource},
	linked_data::de::from_rdf_quads,
	rdf_syntax::{Quad, generator::BlankIdGenerator},
	syntax::{JsonValue, ParseJson, PrintJson, tracing::JsonFragmentPathSegment},
};
use miette::{Diagnostic, NamedSource, SourceSpan};
use nquads_syntax::grdf_document_from_str;
use std::io::{self, Read};
use std::{borrow::Cow, future::Future};
use thiserror::Error;

// ── CLI definition ────────────────────────────────────────────────────────────

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
		///
		/// For local files, a file:// IRI or a plain file path are both accepted.
		/// HTTP/HTTPS contexts must be downloaded locally first.
		context: String,
	},

	/// Flatten a JSON-LD document, moving all nested nodes to the top level.
	Flatten {
		/// Optional context IRI or file path for compacting the flattened result.
		#[arg(long)]
		context: Option<String>,
	},

	/// Serialize a JSON-LD document to RDF N-Quads.
	ToRdf,

	/// Deserialize RDF N-Quads to an expanded JSON-LD document.
	FromRdf,
}

// ── Error types ───────────────────────────────────────────────────────────────

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
#[error("{error}")]
#[diagnostic(code(json_ld::processing_error))]
struct ProcessingError {
	#[source]
	error: json_ld::Error,

	#[source_code]
	src: Option<NamedSource<String>>,

	#[label("here")]
	span: Option<SourceSpan>,
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

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extension trait providing an async equivalent of [`Result::map_err`].
///
/// Neither `std`, `futures`, nor `tokio` provide a combinator for mapping the
/// error of an already-resolved `Result` through an async function, so we
/// implement the (tiny) shim ourselves rather than pull in an unproven
/// third-party crate for it.
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
		Some(DocumentSource::new(content.to_owned(), code_map)),
		value,
	))
}

// /// Build [`JsonLdOptions`] from an optional base IRI.
// fn build_options(base: Option<IriBuf>) -> JsonLdOptions {
// 	let mut opts = JsonLdOptions::default();
// 	opts.base = base;
// 	opts
// }

/// Resolve a context argument (IRI string or file path) into a
/// `(FsLoader, RemoteContext)` pair ready for use with the processor.
fn resolve_context(arg: &str) -> miette::Result<(FsLoader, RemoteContext)> {
	// HTTP/HTTPS contexts require async loading, which is not supported by this
	// synchronous CLI. Suggest downloading the context first.
	if arg.starts_with("http://") || arg.starts_with("https://") {
		return Err(miette::miette!(
			"HTTP/HTTPS context loading is not supported.\n\
			 Please download the context and pass a local file path instead."
		));
	}

	let mut loader = FsLoader::default();

	let iri = if arg.starts_with("file://") {
		// Already a file:// IRI — use as-is.
		IriBuf::new(arg.to_owned())
			.map_err(|e| miette::miette!("invalid context IRI `{arg}`: {e}"))?
	} else {
		// Treat as a filesystem path.
		let abs = std::path::Path::new(arg)
			.canonicalize()
			.map_err(|e| miette::miette!("cannot access context file `{arg}`: {e}"))?;

		let parent = abs.parent().unwrap_or(std::path::Path::new("/"));

		// Normalise to forward-slash paths for the IRI.
		let dir_str = parent.to_string_lossy().replace('\\', "/");
		let file_name = abs
			.file_name()
			.ok_or_else(|| miette::miette!("context path has no file name: {arg}"))?
			.to_string_lossy();

		let dir_iri_str = format!("file://{dir_str}/");
		let dir_iri = IriBuf::new(dir_iri_str.clone())
			.map_err(|e| miette::miette!("cannot build IRI for context directory: {e}"))?;

		loader.mount(dir_iri, parent);

		IriBuf::new(format!("{dir_iri_str}{file_name}"))
			.map_err(|e| miette::miette!("cannot build IRI for context file: {e}"))?
	};

	Ok((loader, RemoteContext::iri(iri)))
}

async fn locate_error(
	input: &Document,
	loader: &impl AsyncLoader,
	e: &JsonLdLocated<json_ld::Error>,
) -> Option<(NamedSource<String>, SourceSpan)> {
	let last = e.location.last()?;

	match &last.file {
		JsonLdSource::Compact(url) | JsonLdSource::Expanded(url) => {
			let document = match url {
				Some(url) => Cow::Owned(loader.async_load(url).await.unwrap()),
				None => Cow::Borrowed(input),
			};

			let name = match url {
				Some(url) => url.to_string(),
				None => "<input>".to_owned(),
			};

			let source = document.source.as_ref()?;
			let named_source = NamedSource::new(name, source.text.clone());
			let fragment_offset = document
				.document
				.locate_fragment(&source.code_map, &last.fragment)?;
			let span = source.code_map.get(fragment_offset)?.span;
			Some((named_source, span.into()))
		}
		_ => None,
	}
}

async fn into_processing_error(
	input: &Document,
	loader: &impl AsyncLoader,
	e: JsonLdLocated<json_ld::Error>,
) -> ProcessingError {
	let (src, span) = match locate_error(input, loader, &e).await {
		Some((src, span)) => (Some(src), Some(span)),
		None => (None, None),
	};

	ProcessingError {
		error: e.value,
		src,
		span,
	}
}

/// Serialize an [`ExpandedDocument`] to a pretty-printed JSON value.
fn expanded_to_json(doc: ExpandedDocument) -> JsonValue {
	json_ld::syntax::to_value(doc.into_objects()).unwrap()
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> miette::Result<()> {
	let cli = Cli::parse();

	let input = read_stdin()?;

	let loader = TokioFsLoader::new();
	cli.command
		.run(&loader, cli.base.as_deref(), &input)
		.await?;

	Ok(())
}

impl Command {
	async fn run(
		&self,
		loader: &impl AsyncLoader,
		base: Option<&Iri>,
		input: &str,
	) -> miette::Result<()> {
		match self {
			Self::Expand => {
				let doc = parse_document(input, base.clone())?;

				let mut options = JsonLdOptions::default();
				options.base = base.map(ToOwned::to_owned);

				let expanded = doc
					.async_expand_with(&loader, options)
					.await
					.map_err_async(|e| into_processing_error(&doc, loader, e))
					.await?;

				println!("{}", expanded_to_json(expanded).pretty_print());
				Ok(())
			}
			_ => todo!(), // Self::Compact { context } => {
			              // 	let src = read_stdin()?;
			              // 	let doc = parse_document(&src, base.clone())?;
			              // 	let (loader, ctx) = resolve_context(context)?;
			              // 	let options = build_options(base);

			              // 	let compact = doc
			              // 		.compact_with(ctx, &loader, options)
			              // 		.map_err(into_processing_error)?;

			              // 	println!("{}", compact.pretty_print());
			              // }

			              // Self::Flatten { context } => {
			              // 	let src = read_stdin()?;
			              // 	let doc = parse_document(&src, base.clone())?;
			              // 	let options = build_options(base);

			              // 	// Resolve the optional context; fall back to a bare loader if none.
			              // 	let (loader, ctx_iri): (FsLoader, Option<RemoteContext>) =
			              // 		if let Some(c) = context.as_deref() {
			              // 			let (l, ctx) = resolve_context(c)?;
			              // 			(l, Some(ctx))
			              // 		} else {
			              // 			(FsLoader::default(), None)
			              // 		};

			              // 	let flat = doc
			              // 		.flatten_with(ctx_iri, &loader, options)
			              // 		.map_err(into_processing_error)?;

			              // 	println!("{}", flat.pretty_print());
			              // }

			              // Self::ToRdf => {
			              // 	let src = read_stdin()?;
			              // 	let doc = parse_document(&src, base.clone())?;
			              // 	let loader = FsLoader::default();
			              // 	let generator = BlankIdGenerator::new_with_prefix("b".to_string());
			              // 	let options = build_options(base);

			              // 	let quads = doc
			              // 		.to_rdf_with(&loader, generator, options)
			              // 		.map_err(into_processing_error)?;

			              // 	for Quad(subject, predicate, object, graph) in quads {
			              // 		match graph {
			              // 			Some(g) => println!("{subject} {predicate} {object} {g} ."),
			              // 			None => println!("{subject} {predicate} {object} ."),
			              // 		}
			              // 	}
			              // }

			              // Self::FromRdf => {
			              // 	let src = read_stdin()?;

			              // 	let (quads, _code_map) =
			              // 		grdf_document_from_str(&src).map_err(|e| NQuadsParseError {
			              // 			message: e.to_string(),
			              // 		})?;

			              // 	let expanded: ExpandedDocument =
			              // 		from_rdf_quads(quads, []).map_err(|e| FromRdfError {
			              // 			message: e.to_string(),
			              // 		})?;

			              // 	println!("{}", expanded_to_json(expanded).pretty_print());
			              // }
		}
	}
}
