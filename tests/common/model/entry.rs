use linked_data::DeserializeLinkedData;
use rdf_syntax::IriBuf;

use crate::common::{TestKind, TestOptions};

/// A single entry in a test manifest.
///
/// Each entry has a type that determines whether it is a positive or
/// negative evaluation test.
#[derive(DeserializeLinkedData)]
#[ld(prefix("mf" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#"))]
#[ld(prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#"))]
#[ld(prefix("test" = "https://w3c.github.io/json-ld-api/tests/vocab#"))]
pub struct ManifestEntry {
	// /// Test identifier (IRI).
	// #[ld(flatten)]
	// pub id: IriBuf,
	/// Human-readable test name.
	#[ld(prop = "mf:name")]
	pub name: String,

	// /// Description of the test purpose.
	// #[ld(prop = "rdfs:comment")]
	// pub purpose: Option<String>,
	/// Input document IRI.
	#[ld(prop = "mf:action")]
	pub input: IriBuf,

	/// Test kind (positive/negative) with associated expected result.
	#[ld(flatten)]
	pub kind: TestKind,

	/// Processing options.
	#[ld(prop = "test:option")]
	pub options: Option<TestOptions>,
}
