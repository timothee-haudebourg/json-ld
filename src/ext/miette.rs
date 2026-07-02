use std::{
	collections::{HashMap, hash_map::Entry},
	sync::Arc,
};

use json_syntax::{
	JsonValue,
	locspan::Span,
	tracing::{JsonBacktraceFrame, JsonFragmentPath},
};
use miette::{Diagnostic, NamedSource, SourceSpan};
use rdf_syntax::Iri;

use crate::{
	AsyncLoader, Document, JsonLdError, JsonLdSourceCode,
	algorithms::{JsonLdLocatedError, JsonLdSourceFile},
};

#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct JsonLdDiagnostic {
	pub error: JsonLdError,

	pub src: Option<(NamedSource<Arc<JsonLdDiagnosticSourceCode>>, SourceSpan)>,

	/// Related diagnostic.
	pub related: Vec<RelatedJsonLdDiagnostic>,
}

impl JsonLdDiagnostic {
	pub async fn new_async(
		loader: &impl AsyncLoader,
		input: &Document,
		error: JsonLdLocatedError,
	) -> Self {
		let mut map = HashMap::new();

		map.insert(
			JsonLdSourceFile::Compact(input.url.clone()),
			named_source_from_document(input.clone()),
		);

		let mut src = None;
		let mut related = Vec::new();

		if let Some((location, rest)) = error.location.split_last() {
			src = Some(locate_error(&mut map, loader, location).await);

			for location in rest.iter().rev() {
				let (src, span) = locate_error(&mut map, loader, location).await;
				related.push(RelatedJsonLdDiagnostic { src, span })
			}
		};

		Self {
			error: error.value,
			src,
			related,
		}
	}
}

impl Diagnostic for JsonLdDiagnostic {
	fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
		Some(Box::new(self.error.code()))
	}

	fn source_code(&self) -> Option<&dyn miette::SourceCode> {
		self.src.as_ref().map(|c| &c.0 as &dyn miette::SourceCode)
	}

	fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
		self.src.as_ref().map(|c| {
			Box::new(
				[miette::LabeledSpan::new_with_span(
					Some("here".to_owned()),
					c.1,
				)]
				.into_iter(),
			) as Box<dyn Iterator<Item = miette::LabeledSpan>>
		})
	}

	fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
		Some(Box::new(self.related.iter().map(|d| d as &dyn Diagnostic)))
	}
}

fn url_to_name(url: Option<&Iri>) -> &str {
	match url {
		Some(url) => url.as_str(),
		None => "<input>",
	}
}

fn named_source_from_document(document: Document) -> NamedSource<Arc<JsonLdDiagnosticSourceCode>> {
	let source = document
		.source
		.unwrap_or_else(|| JsonLdSourceCode::from_value(&document.document));

	NamedSource::new(
		url_to_name(document.url.as_deref()),
		Arc::new(JsonLdDiagnosticSourceCode {
			value: document.document,
			source,
		}),
	)
}

async fn get_named_source(
	map: &mut HashMap<JsonLdSourceFile, NamedSource<Arc<JsonLdDiagnosticSourceCode>>>,
	loader: &impl AsyncLoader,
	file: &JsonLdSourceFile,
) -> NamedSource<Arc<JsonLdDiagnosticSourceCode>> {
	match map.entry(file.clone()) {
		Entry::Occupied(e) => e.get().clone(),
		Entry::Vacant(e) => e
			.insert(match file {
				JsonLdSourceFile::Compact(url) | JsonLdSourceFile::Context(url) => {
					let url = url.as_deref().expect("document not found");
					let document = loader.async_load(&url).await.expect("document not found");
					named_source_from_document(document)
				}
				JsonLdSourceFile::Expanded(url, document) => {
					let value = json_syntax::to_value(&**document).unwrap();
					let source = JsonLdSourceCode::from_value(&value);
					NamedSource::new(
						url_to_name(url.as_deref()),
						Arc::new(JsonLdDiagnosticSourceCode { value, source }),
					)
				}
				JsonLdSourceFile::Flattened(url, document) => {
					let value = json_syntax::to_value(&**document).unwrap();
					let source = JsonLdSourceCode::from_value(&value);
					NamedSource::new(
						url_to_name(url.as_deref()),
						Arc::new(JsonLdDiagnosticSourceCode { value, source }),
					)
				}
			})
			.clone(),
	}
}

async fn locate_error(
	map: &mut HashMap<JsonLdSourceFile, NamedSource<Arc<JsonLdDiagnosticSourceCode>>>,
	loader: &impl AsyncLoader,
	location: &JsonBacktraceFrame<JsonLdSourceFile>,
) -> (NamedSource<Arc<JsonLdDiagnosticSourceCode>>, SourceSpan) {
	let named_source = get_named_source(map, loader, &location.file).await;
	let span = named_source
		.inner()
		.locate_fragment(&location.fragment)
		.expect("fragment not found");

	(named_source, span.into())
}

impl miette::SourceCode for JsonLdSourceCode {
	fn read_span<'a>(
		&'a self,
		span: &miette::SourceSpan,
		context_lines_before: usize,
		context_lines_after: usize,
	) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
		self.text
			.read_span(span, context_lines_before, context_lines_after)
	}
}

pub struct JsonLdDiagnosticSourceCode {
	pub source: JsonLdSourceCode,
	pub value: JsonValue,
}

impl JsonLdDiagnosticSourceCode {
	pub fn locate_fragment(&self, path: &JsonFragmentPath) -> Option<Span> {
		let offset = self.value.locate_fragment(&self.source.code_map, path)?;
		Some(self.source.code_map.get(offset)?.span)
	}
}

impl miette::SourceCode for JsonLdDiagnosticSourceCode {
	fn read_span<'a>(
		&'a self,
		span: &SourceSpan,
		context_lines_before: usize,
		context_lines_after: usize,
	) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
		self.source
			.read_span(span, context_lines_before, context_lines_after)
	}
}

#[derive(Debug, thiserror::Error)]
#[error("required by")]
pub struct RelatedJsonLdDiagnostic {
	pub src: NamedSource<Arc<JsonLdDiagnosticSourceCode>>,

	pub span: SourceSpan,
}

impl Diagnostic for RelatedJsonLdDiagnostic {
	fn source_code(&self) -> Option<&dyn miette::SourceCode> {
		Some(&self.src)
	}

	fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
		Some(Box::new(
			[miette::LabeledSpan::new_with_span(
				Some("here".to_owned()),
				self.span,
			)]
			.into_iter(),
		) as Box<dyn Iterator<Item = miette::LabeledSpan>>)
	}
}
