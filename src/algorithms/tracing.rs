use std::sync::Arc;

use into_owned_trait::IntoOwned;
use json_syntax::tracing::{JsonBacktraceBuilder, JsonLocated};
use rdf_syntax::{Iri, IriBuf};

use crate::{ExpandedDocument, FlattenedDocument};

/// Backtrace builder for the JSON-LD processing algorithms.
///
/// Accumulates [`JsonLdSourceFileRef`] frames during processing and produces a
/// [`JsonLdLocated`] value when an error is emitted.
pub type JsonLdBacktraceBuilder<'a> = JsonBacktraceBuilder<'a, JsonLdSourceFileRef<'a>>;

/// A value annotated with a JSON-LD processing backtrace.
///
/// Each frame in the backtrace identifies the source file (which document, at
/// which processing stage) and the JSON fragment path within it where the
/// annotated value was produced.
pub type JsonLdLocated<T> = JsonLocated<T, JsonLdSourceFile>;

/// Borrowed counterpart of [`JsonLdSourceFile`], used while building a backtrace.
///
/// Carries references rather than owned values so that frames can be pushed
/// cheaply onto the [`JsonLdBacktraceBuilder`] stack during processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdSourceFileRef<'a> {
	/// Compact JSON-LD document.
	Compact(Option<&'a Iri>),

	/// Intermediate expanded JSON-LD document, together with its content.
	Expanded(Option<&'a Iri>, &'a Arc<ExpandedDocument>),

	/// Intermediate flattened JSON-LD document, together with its content.
	Flattened(Option<&'a Iri>, &'a Arc<FlattenedDocument>),

	/// JSON-LD context document.
	Context(Option<&'a Iri>),
}

impl IntoOwned for JsonLdSourceFileRef<'_> {
	type Owned = JsonLdSourceFile;

	fn into_owned(self) -> JsonLdSourceFile {
		match self {
			Self::Compact(iri) => JsonLdSourceFile::Compact(iri.map(Iri::to_owned)),
			Self::Expanded(iri, document) => {
				JsonLdSourceFile::Expanded(iri.map(Iri::to_owned), document.clone())
			}
			Self::Flattened(iri, document) => {
				JsonLdSourceFile::Flattened(iri.map(Iri::to_owned), document.clone())
			}
			Self::Context(iri) => JsonLdSourceFile::Context(iri.map(Iri::to_owned)),
		}
	}
}

/// Source file of a JSON-LD processing backtrace frame.
///
/// Identifies which document,and at which stage of the processing pipeline, a
/// particular step took place. The optional [`IriBuf`] is the document's URL;
/// `None` means the document was provided inline with no associated URL.
///
/// The `Expanded` and `Flattened` variants additionally carry a shared
/// reference to the actual intermediate document so that diagnostic renderers
/// can locate and display the relevant JSON fragment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdSourceFile {
	/// Compact JSON-LD document (typically the algorithm's input).
	Compact(Option<IriBuf>),

	/// Intermediate expanded JSON-LD document, together with its content.
	Expanded(Option<IriBuf>, Arc<ExpandedDocument>),

	/// Intermediate flattened JSON-LD document, together with its content.
	Flattened(Option<IriBuf>, Arc<FlattenedDocument>),

	/// JSON-LD context document.
	Context(Option<IriBuf>),
}
