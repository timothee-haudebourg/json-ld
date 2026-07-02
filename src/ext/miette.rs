use std::{
	collections::{HashMap, hash_map::Entry},
	sync::Arc,
};

use json_syntax::{
	JsonValue,
	locspan::Span,
	tracing::{JsonBacktraceItem, JsonFragmentPath},
};
use miette::{Diagnostic, NamedSource, SourceSpan};
use rdf_syntax::IriBuf;

use crate::{
	AsyncLoader, Document, DocumentSource, JsonLdError,
	algorithms::{JsonLdLocatedError, JsonLdSource},
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

		if let Some(source) = &input.source {
			map.insert(
				None,
				NamedSource::new(
					"<input>",
					Arc::new(JsonLdDiagnosticSourceCode {
						source: source.clone(),
						value: input.document.clone(),
					}),
				),
			);
		}

		let mut src = None;
		let mut related = Vec::new();

		if let Some((location, rest)) = error.location.split_last() {
			src = locate_error(&mut map, loader, location).await;

			for location in rest.iter().rev() {
				if let Some((src, span)) = locate_error(&mut map, loader, location).await {
					related.push(RelatedJsonLdDiagnostic { src, span })
				}
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

async fn locate_error(
	map: &mut HashMap<Option<IriBuf>, NamedSource<Arc<JsonLdDiagnosticSourceCode>>>,
	loader: &impl AsyncLoader,
	location: &JsonBacktraceItem<JsonLdSource>,
) -> Option<(NamedSource<Arc<JsonLdDiagnosticSourceCode>>, SourceSpan)> {
	match &location.file {
		JsonLdSource::Compact(url) | JsonLdSource::Expanded(url, _) => {
			let named_source = match map.entry(url.clone()) {
				Entry::Occupied(e) => e.get().clone(),
				Entry::Vacant(e) => {
					let url = url.as_deref()?;
					let document = loader.async_load(url).await.ok()?;
					let source = document.source?;
					let named_source = NamedSource::new(
						url.to_string(),
						Arc::new(JsonLdDiagnosticSourceCode {
							source,
							value: document.document,
						}),
					);
					e.insert(named_source.clone());
					named_source
				}
			};

			let span = named_source.inner().locate_fragment(&location.fragment)?;

			Some((named_source, span.into()))
		}
		_ => None,
	}
}

impl miette::SourceCode for DocumentSource {
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
	pub source: DocumentSource,
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
