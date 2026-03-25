//! JSON-LD test suite manifest types.
//!
//! These types model the W3C JSON-LD API test suite manifests, using
//! `linked_data` derive macros to deserialize from RDF.
//!
//! Manifest files can be found in the W3C JSON-LD API test suite:
//! <https://w3c.github.io/json-ld-api/tests/>
use linked_data::DeserializeLinkedData;

mod entry;
mod kind;
mod options;
mod spec_version;
#[cfg(feature = "proc_macro2")]
mod tokens;

pub use entry::ManifestEntry;
pub use kind::TestKind;
pub use options::TestOptions;
pub use spec_version::SpecVersion;

/// Well-known IRI prefixes used in the test manifests.
pub mod vocab {
	pub const MF: &str = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#";
	pub const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
	pub const TEST: &str = "https://w3c.github.io/json-ld-api/tests/vocab#";
	pub const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
}

/// A test suite manifest.
#[derive(Default, DeserializeLinkedData)]
#[ld(prefix("mf" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#"))]
#[ld(prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#"))]
pub struct Manifest {
	/// Manifest name.
	#[ld(prop = "rdfs:comment")]
	pub name: Option<String>,

	/// Test entries.
	#[ld(prop = "mf:entries")]
	pub entries: Vec<ManifestEntry>,
}
