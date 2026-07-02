//! This crate is a Rust implementation of the
//! [JSON-LD](https://www.w3.org/TR/json-ld/)
//! data interchange format.
//!
//! [Linked Data (LD)](https://www.w3.org/standards/semanticweb/data)
//! is a [World Wide Web Consortium (W3C)](https://www.w3.org/)
//! initiative built upon standard Web technologies to create an
//! interrelated network of datasets across the Web.
//! The [JavaScript Object Notation (JSON)](https://tools.ietf.org/html/rfc7159) is
//! a widely used, simple, unstructured data serialization format to describe
//! data objects in a human readable way.
//! JSON-LD brings these two technologies together, adding semantics to JSON
//! to create a lightweight data serialization format that can organize data and
//! help Web applications to inter-operate at a large scale.
//!
//! # Usage
//!
//! The entry point for this library is the [`JsonLdProcessor`] trait
//! that provides an access to all the JSON-LD transformation algorithms
//! (context processing, expansion, compaction, etc.).
//! If you want to explore and/or transform [`ExpandedDocument`]s, you may also
//! want to check out the [`Object`] type representing a JSON object.
//!
//! [`JsonLdProcessor`]: crate::JsonLdProcessor
//!
//! ## Expansion
//!
//! If you want to expand a JSON-LD document, first describe the document to
//! be expanded using either [`RemoteDocument`] or [`RemoteDocumentReference`]:
//!   - [`RemoteDocument`] wraps the JSON representation of the document
//!     alongside its remote URL.
//!   - [`RemoteDocumentReference`] may represent only an URL, letting
//!     some loader fetching the remote document by dereferencing the URL.
//!
//! After that, you can simply use the [`JsonLdProcessor::expand`] function on
//! the remote document.
//!
//! [`RemoteDocument`]: crate::RemoteDocument
//! [`RemoteDocumentReference`]: crate::RemoteDocumentReference
//! [`JsonLdProcessor::expand`]: JsonLdProcessor::expand
//!
//! ### Example
//!
//! ```
//! use rdf_syntax::iri;
//! use json_ld::{JsonLdProcessor, Document, syntax::{JsonValue, JsonParse}};
//!
//! // Parse a JSON-LD document.
//! let (value, _) = JsonValue::parse_str(r#"{
//!   "@context": { "name": "http://xmlns.com/foaf/0.1/name" },
//!   "@id": "https://www.rust-lang.org",
//!   "name": "Rust Programming Language"
//! }"#).expect("unable to parse file");
//!
//! let input = Document::new(
//!   Some(iri!("https://example.com/sample.jsonld").to_owned()),
//!   None,
//!   value,
//! );
//!
//! // Use `NoLoader` as we won't need to load any remote document.
//! let expanded = input.expand(json_ld::NoLoader).expect("expansion failed");
//!
//! assert!(!expanded.is_empty());
//! ```
//!
//! Here is another example using a file-system loader.
//!
//! ```no_run
//! use rdf_syntax::iri;
//! use json_ld::{JsonLdProcessor, Loader};
//!
//! let mut loader = json_ld::FsLoader::default();
//! loader.mount(iri!("https://example.com/").to_owned(), "examples");
//!
//! let input = loader.load(iri!("https://example.com/sample.jsonld")).expect("loading failed");
//! let expanded = input.expand(&loader).expect("expansion failed");
//! ```
//!
//! ## Compaction
//!
//! The JSON-LD Compaction is a transformation that consists in applying a
//! context to a given JSON-LD document reducing its size.
//! There are two ways to get a compact JSON-LD document with this library
//! depending on your starting point:
//!   - If you want to get a compact representation for an arbitrary remote
//!     document, simply use the [`JsonLdProcessor::compact`]
//!     (or [`JsonLdProcessor::compact_with`]) method.
//!   - Otherwise to compact an [`ExpandedDocument`] you can use the
//!     [`Compact::compact`] method.
//!
//! [`JsonLdProcessor::compact`]: crate::JsonLdProcessor::compact
//! [`JsonLdProcessor::compact_with`]: crate::JsonLdProcessor::compact_with
//! [`ExpandedDocument`]: crate::ExpandedDocument
//! [`Compact::compact`]: crate::Compact::compact
//!
//! ### Example
//!
//! Here is an example compacting an arbitrary document using [`JsonLdProcessor::compact`].
//!
//! ```no_run
//! use rdf_syntax::iri;
//! use json_ld::{JsonLdProcessor, Loader, RemoteContext, syntax::JsonPrint};
//!
//! let mut loader = json_ld::FsLoader::default();
//! loader.mount(iri!("https://example.com/").to_owned(), "examples");
//!
//! let input = loader.load(iri!("https://example.com/sample.jsonld")).expect("loading failed");
//! let context = RemoteContext::iri(iri!("https://example.com/context.jsonld").to_owned());
//!
//! let compact = input.compact(context, &loader).expect("compaction failed");
//!
//! println!("output: {}", compact.pretty_print());
//! ```
//!
//! ## Flattening
//!
//! The JSON-LD Flattening is a transformation that consists in moving nested
//! nodes out. The result is a list of all the nodes declared in the document.
//! There are two ways to flatten JSON-LD document with this library
//! depending on your starting point:
//!   - If you want to flatten an arbitrary remote document, use
//!     [`JsonLdProcessor::flatten`] (or [`JsonLdProcessor::flatten_with`]).
//!     This returns a compacted JSON-LD document.
//!   - To flatten an already-expanded [`ExpandedDocument`], use the
//!     [`Flatten::flatten`] (or [`Flatten::flatten_with`]) method, which
//!     returns a [`FlattenedDocument`].
//!
//! [`JsonLdProcessor::flatten`]: crate::JsonLdProcessor::flatten
//! [`JsonLdProcessor::flatten_with`]: crate::JsonLdProcessor::flatten_with
//! [`Flatten::flatten`]: crate::Flatten::flatten
//! [`Flatten::flatten_with`]: crate::Flatten::flatten_with
//! [`FlattenedDocument`]: crate::FlattenedDocument
//!
//! ### Example
//!
//! Here is an example flattening an arbitrary document using [`JsonLdProcessor::flatten`].
//!
//! ```no_run
//! use rdf_syntax::iri;
//! use json_ld::{JsonLdProcessor, Loader, syntax::JsonPrint};
//!
//! let mut loader = json_ld::FsLoader::default();
//! loader.mount(iri!("https://example.com/").to_owned(), "examples");
//!
//! let input = loader.load(iri!("https://example.com/sample.jsonld")).expect("loading failed");
//! let flattened = input.flatten(None, &loader).expect("flattening failed");
//!
//! println!("output: {}", flattened.pretty_print());
//! ```
pub use json_syntax;

#[cfg(feature = "algorithms")]
pub mod algorithms;
mod core;
pub mod ext;
#[cfg(feature = "algorithms")]
mod processor;
pub mod syntax;

#[cfg(feature = "algorithms")]
pub use algorithms::{JsonLdError, JsonLdErrorCode, RdfSerializationOptions, Warning};
pub use core::*;
pub use linked_data;
#[cfg(feature = "algorithms")]
pub use processor::*;
