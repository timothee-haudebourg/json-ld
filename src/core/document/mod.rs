use hashbrown::HashSet;
use json_syntax::{JsonCodeMap, JsonPrint, JsonValue, print::JsonPrintOptions};
use mime::Mime;
use rdf_syntax::{Iri, IriBuf};

pub mod expanded;
pub mod flattened;
pub mod profile;

pub use expanded::ExpandedDocument;
pub use flattened::FlattenedDocument;
pub use profile::Profile;

use crate::syntax::ContextDocumentValue;

/// Remote document.
///
/// Stores the content of a loaded remote document along with its original URL.
#[derive(Debug, Clone)]
pub struct Document<T = JsonValue> {
	/// The final URL of the loaded document, after eventual redirection.
	pub url: Option<IriBuf>,

	/// The HTTP `Content-Type` header value of the loaded document, exclusive
	/// of any optional parameters.
	pub content_type: Option<Mime>,

	/// If available, the value of the HTTP `Link Header` [RFC 8288] using the
	/// `http://www.w3.org/ns/json-ld#context` link relation in the response.
	///
	/// If the response's `Content-Type` is `application/ld+json`, the HTTP
	/// `Link Header` is ignored. If multiple HTTP `Link Headers` using the
	/// `http://www.w3.org/ns/json-ld#context` link relation are found, the
	/// loader fails with a `multiple context link headers` error.
	///
	/// [RFC 8288]: https://www.rfc-editor.org/rfc/rfc8288
	pub context_url: Option<IriBuf>,

	pub profile: HashSet<Profile>,

	pub source: Option<JsonLdSourceCode>,

	/// The retrieved document.
	pub document: T,
}

pub type ContextDocument = Document<ContextDocumentValue>;

impl<T> Document<T> {
	/// Creates a new remote document.
	///
	/// `url` is the final URL of the loaded document, after eventual
	/// redirection.
	/// `content_type` is the HTTP `Content-Type` header value of the loaded
	/// document, exclusive of any optional parameters.
	pub fn new(url: Option<IriBuf>, content_type: Option<Mime>, document: T) -> Self {
		Self::new_full(url, content_type, None, HashSet::new(), None, document)
	}

	/// Creates a new remote document.
	///
	/// `url` is the final URL of the loaded document, after eventual
	/// redirection.
	/// `content_type` is the HTTP `Content-Type` header value of the loaded
	/// document, exclusive of any optional parameters.
	/// `context_url` is the value of the HTTP `Link Header` [RFC 8288] using the
	/// `http://www.w3.org/ns/json-ld#context` link relation in the response,
	/// if any.
	/// `profile` is the value of any profile parameter retrieved as part of the
	/// original contentType.
	///
	/// [RFC 8288]: https://www.rfc-editor.org/rfc/rfc8288
	pub fn new_full(
		url: Option<IriBuf>,
		content_type: Option<Mime>,
		context_url: Option<IriBuf>,
		profile: HashSet<Profile>,
		source: Option<JsonLdSourceCode>,
		document: T,
	) -> Self {
		Self {
			url,
			content_type,
			context_url,
			profile,
			source,
			document,
		}
	}

	/// Maps the content of the remote document.
	pub fn map<U>(self, f: impl Fn(T) -> U) -> Document<U> {
		Document {
			url: self.url,
			content_type: self.content_type,
			context_url: self.context_url,
			profile: self.profile,
			source: self.source,
			document: f(self.document),
		}
	}

	/// Tries to map the content of the remote document.
	pub fn try_map<U, E>(self, f: impl Fn(T) -> Result<U, E>) -> Result<Document<U>, E> {
		Ok(Document {
			url: self.url,
			content_type: self.content_type,
			context_url: self.context_url,
			profile: self.profile,
			source: self.source,
			document: f(self.document)?,
		})
	}

	/// Returns a reference to the final URL of the loaded document, after eventual redirection.
	pub fn url(&self) -> Option<&Iri> {
		self.url.as_deref()
	}

	/// Returns the HTTP `Content-Type` header value of the loaded document,
	/// exclusive of any optional parameters.
	pub fn content_type(&self) -> Option<&Mime> {
		self.content_type.as_ref()
	}

	/// Returns the value of the HTTP `Link Header` [RFC 8288] using the
	/// `http://www.w3.org/ns/json-ld#context` link relation in the response,
	/// if any.
	///
	/// If the response's `Content-Type` is `application/ld+json`, the HTTP
	/// `Link Header` is ignored. If multiple HTTP `Link Headers` using the
	/// `http://www.w3.org/ns/json-ld#context` link relation are found, the
	/// loader fails with a `multiple context link headers` error.
	///
	/// [RFC 8288]: https://www.rfc-editor.org/rfc/rfc8288
	pub fn context_url(&self) -> Option<&Iri> {
		self.context_url.as_deref()
	}

	/// Returns a reference to the content of the document.
	pub fn document(&self) -> &T {
		&self.document
	}

	/// Returns a mutable reference to the content of the document.
	pub fn document_mut(&mut self) -> &mut T {
		&mut self.document
	}

	/// Drops the original URL and returns the content of the document.
	pub fn into_document(self) -> T {
		self.document
	}

	/// Drops the content and returns the original URL of the document.
	pub fn into_url(self) -> Option<IriBuf> {
		self.url
	}

	/// Sets the URL of the document.
	pub fn set_url(&mut self, url: Option<IriBuf>) {
		self.url = url
	}
}

impl Document {
	#[cfg(feature = "serde")]
	pub fn try_into_context_document(
		self,
	) -> Result<ContextDocument, crate::syntax::serde::DeserializeError> {
		Ok(ContextDocument {
			url: self.url,
			content_type: self.content_type,
			context_url: self.context_url,
			profile: self.profile,
			source: self.source,
			document: crate::syntax::from_value(self.document)?,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonLdSourceCode {
	pub text: String,
	pub code_map: JsonCodeMap,
}

impl JsonLdSourceCode {
	pub fn new(text: String, code_map: JsonCodeMap) -> Self {
		Self { text, code_map }
	}

	pub fn from_value(value: &JsonValue) -> Self {
		JsonLdSourceCode {
			text: value.pretty_print().to_string(),
			code_map: JsonCodeMap::from_value(value, &JsonPrintOptions::PRETTY),
		}
	}
}
