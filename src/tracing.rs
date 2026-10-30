use std::sync::Arc;

use into_owned_trait::IntoOwned;
use json_syntax::tracing::{JsonBacktrace, JsonBacktraceBuf, JsonBacktraceBuilder, JsonLocated};
use rdf_syntax::{Iri, IriBuf};

use crate::{JsonLdExpandedDocument, JsonLdFlattenedDocument};

/// Backtrace builder for the JSON-LD processing algorithms.
///
/// Accumulates [`JsonLdSourceFileRef`] frames during processing and produces a
/// [`JsonLdLocated`] value when an error is emitted.
pub type JsonLdBacktraceBuilder<'a> = JsonBacktraceBuilder<'a, JsonLdSourceFileRef<'a>>;

pub type JsonLdBacktrace = JsonBacktrace<JsonLdSourceFile>;

pub type JsonLdBacktraceBuf = JsonBacktraceBuf<JsonLdSourceFile>;

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
pub struct JsonLdSourceFileRef<'a> {
	/// URL of the document, if any.
	pub iri: Option<&'a Iri>,

	/// Form of the document at this stage of the processing.
	pub form: JsonLdDocumentFormRef<'a>,
}

impl<'a> JsonLdSourceFileRef<'a> {
	/// Original document, as provided to the algorithm (a JSON-LD document or
	/// a context document).
	pub fn original(iri: Option<&'a Iri>) -> Self {
		Self {
			iri,
			form: JsonLdDocumentFormRef::Original,
		}
	}

	/// Intermediate expanded version of the document.
	pub fn expanded(iri: Option<&'a Iri>, document: &'a Arc<JsonLdExpandedDocument>) -> Self {
		Self {
			iri,
			form: JsonLdDocumentFormRef::Expanded(document),
		}
	}

	/// Intermediate flattened version of the document.
	pub fn flattened(iri: Option<&'a Iri>, document: &'a Arc<JsonLdFlattenedDocument>) -> Self {
		Self {
			iri,
			form: JsonLdDocumentFormRef::Flattened(document),
		}
	}
}

impl IntoOwned for JsonLdSourceFileRef<'_> {
	type Owned = JsonLdSourceFile;

	fn into_owned(self) -> JsonLdSourceFile {
		JsonLdSourceFile {
			iri: self.iri.map(Iri::to_owned),
			form: self.form.into_owned(),
		}
	}
}

/// Borrowed counterpart of [`JsonLdDocumentForm`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdDocumentFormRef<'a> {
	/// Original document.
	Original,

	/// Expanded version of the document.
	Expanded(&'a Arc<JsonLdExpandedDocument>),

	/// Flattened version of the document.
	Flattened(&'a Arc<JsonLdFlattenedDocument>),
}

impl IntoOwned for JsonLdDocumentFormRef<'_> {
	type Owned = JsonLdDocumentForm;

	fn into_owned(self) -> JsonLdDocumentForm {
		match self {
			Self::Original => JsonLdDocumentForm::Original,
			Self::Expanded(document) => JsonLdDocumentForm::Expanded(document.clone()),
			Self::Flattened(document) => JsonLdDocumentForm::Flattened(document.clone()),
		}
	}
}

/// Source file of a JSON-LD processing backtrace frame.
///
/// Identifies which document, and at which stage of the processing pipeline, a
/// particular step took place. The optional [`IriBuf`] is the document's URL;
/// `None` means the document was provided inline with no associated URL.
///
/// The [`JsonLdDocumentForm`] additionally carries a shared reference to the
/// actual intermediate document, if any, so that diagnostic renderers can
/// locate and display the relevant JSON fragment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonLdSourceFile {
	/// URL of the document, if any.
	pub iri: Option<IriBuf>,

	/// Form of the document at this stage of the processing.
	pub form: JsonLdDocumentForm,
}

impl JsonLdSourceFile {
	/// Original document, as provided to the algorithm (a JSON-LD document or
	/// a context document).
	pub fn original(iri: Option<IriBuf>) -> Self {
		Self {
			iri,
			form: JsonLdDocumentForm::Original,
		}
	}

	/// Intermediate expanded version of the document.
	pub fn expanded(iri: Option<IriBuf>, document: Arc<JsonLdExpandedDocument>) -> Self {
		Self {
			iri,
			form: JsonLdDocumentForm::Expanded(document),
		}
	}

	/// Intermediate flattened version of the document.
	pub fn flattened(iri: Option<IriBuf>, document: Arc<JsonLdFlattenedDocument>) -> Self {
		Self {
			iri,
			form: JsonLdDocumentForm::Flattened(document),
		}
	}
}

/// Form of a document at a given stage of the JSON-LD processing pipeline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonLdDocumentForm {
	/// Original document, as provided to the algorithm (a JSON-LD document or
	/// a context document).
	Original,

	/// Intermediate expanded version of the document, together with its
	/// content.
	Expanded(Arc<JsonLdExpandedDocument>),

	/// Intermediate flattened version of the document, together with its
	/// content.
	Flattened(Arc<JsonLdFlattenedDocument>),
}
