//! JSON-LD test suite manifest types.
//!
//! These types model the W3C JSON-LD API test suite manifests, using
//! `linked_data` derive macros to deserialize from RDF.
//!
//! Manifest files can be found in the W3C JSON-LD API test suite:
//! <https://w3c.github.io/json-ld-api/tests/>
mod entry;
mod kind;
mod options;

pub use entry::ManifestEntry;
pub use kind::TestKind;
pub use options::TestOptions;
