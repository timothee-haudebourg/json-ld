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
//! This crate aims to provide a set of types to build and process expanded
//! JSON-LD documents.
//! It can expand, compact and flatten JSON-LD documents backed by various
//! JSON implementations thanks to the [`generic-json`] crate.
pub use json_ld_compaction as compaction;
pub use json_ld_context_processing as context_processing;
pub use json_ld_core::*;
pub use json_ld_expansion as expansion;
pub use json_ld_syntax as syntax;

pub use compaction::Compact;
pub use context_processing::Process;
pub use expansion::Expand;

mod processor;
pub use processor::*;