//! JSON-LD test suite manifest types.
//!
//! These types model the W3C JSON-LD API test suite manifests, using
//! `linked_data` derive macros to deserialize from RDF.
//!
//! Manifest files can be found in the W3C JSON-LD API test suite:
//! <https://w3c.github.io/json-ld-api/tests/>
use linked_data::DeserializeLinkedData;

/// Well-known IRI prefixes used in the test manifests.
pub mod vocab {
	pub const MF: &str = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#";
	pub const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
	pub const TEST: &str = "https://w3c.github.io/json-ld-api/tests/vocab#";
	pub const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
}

/// A test suite manifest.
#[derive(DeserializeLinkedData)]
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

/// A single entry in a test manifest.
///
/// Each entry has a type that determines whether it is a positive or
/// negative evaluation test.
#[derive(DeserializeLinkedData)]
#[ld(prefix("mf" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#"))]
#[ld(prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#"))]
#[ld(prefix("test" = "https://w3c.github.io/json-ld-api/tests/vocab#"))]
pub struct ManifestEntry {
	/// Test identifier (IRI).
	#[ld(flatten)]
	pub id: iref::IriBuf,

	/// Human-readable test name.
	#[ld(prop = "mf:name")]
	pub name: String,

	/// Description of the test purpose.
	#[ld(prop = "rdfs:comment")]
	pub purpose: Option<String>,

	/// Input document IRI.
	#[ld(prop = "mf:action")]
	pub input: iref::IriBuf,

	/// Test kind (positive/negative) with associated expected result.
	#[ld(flatten)]
	pub kind: TestKind,

	/// Processing options.
	#[ld(prop = "test:option")]
	pub options: Option<TestOptions>,
}

/// Whether a test expects success or failure.
#[derive(DeserializeLinkedData)]
#[ld(prefix("mf" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#"))]
#[ld(prefix("test" = "https://w3c.github.io/json-ld-api/tests/vocab#"))]
pub enum TestKind {
	/// Test expects a successful result matching the given document.
	#[ld(type = "test:PositiveEvaluationTest")]
	Positive {
		/// Expected output document IRI.
		#[ld(prop = "mf:result")]
		expect: iref::IriBuf,

		/// Context document IRI (for compaction tests).
		#[ld(prop = "test:context")]
		context: Option<iref::IriBuf>,
	},

	/// Test expects a specific error.
	#[ld(type = "test:NegativeEvaluationTest")]
	Negative {
		/// Expected error code.
		#[ld(prop = "mf:result")]
		expected_error_code: String,

		/// Context document IRI (for compaction tests).
		#[ld(prop = "test:context")]
		context: Option<iref::IriBuf>,
	},
}

/// Processing options for a test.
#[derive(Default, DeserializeLinkedData)]
#[ld(prefix("test" = "https://w3c.github.io/json-ld-api/tests/vocab#"))]
#[ld(prefix("xsd" = "http://www.w3.org/2001/XMLSchema#"))]
pub struct TestOptions {
	/// Base IRI.
	#[ld(prop = "test:base")]
	pub base: Option<iref::IriBuf>,

	/// Expand context IRI.
	#[ld(prop = "test:expandContext")]
	pub expand_context: Option<iref::IriBuf>,

	/// Processing mode.
	#[ld(prop = "test:processingMode")]
	pub processing_mode: Option<String>,

	/// Spec version.
	#[ld(prop = "test:specVersion")]
	pub spec_version: Option<String>,

	/// Whether the test is normative.
	#[ld(prop = "test:normative")]
	pub normative: Option<bool>,

	/// Compact to relative IRIs.
	#[ld(prop = "test:compactToRelative")]
	pub compact_to_relative: Option<bool>,

	/// Compact arrays.
	#[ld(prop = "test:compactArrays")]
	pub compact_arrays: Option<bool>,

	/// Use native types when converting from RDF.
	#[ld(prop = "test:useNativeTypes")]
	pub use_native_types: Option<bool>,

	/// Produce generalized RDF.
	#[ld(prop = "test:produceGeneralizedRdf")]
	pub produce_generalized_rdf: Option<bool>,

	/// RDF direction handling.
	#[ld(prop = "test:rdfDirection")]
	pub rdf_direction: Option<String>,

	/// Content type.
	#[ld(prop = "test:contentType")]
	pub content_type: Option<String>,

	/// HTTP link headers.
	#[ld(prop = "test:httpLink")]
	pub http_link: Option<String>,

	/// HTTP status code.
	#[ld(prop = "test:httpStatus")]
	pub http_status: Option<String>,

	/// Extract all scripts from HTML.
	#[ld(prop = "test:extractAllScripts")]
	pub extract_all_scripts: Option<bool>,
}
