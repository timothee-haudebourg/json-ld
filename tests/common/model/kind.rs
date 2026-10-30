use linked_data::DeserializeLinkedData;
use rdf_syntax::IriBuf;

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
		expect: IriBuf,

		/// Context document IRI (for compaction tests).
		#[ld(prop = "test:context")]
		#[allow(unused)] // Unused in non-compaction tests.
		context: Option<IriBuf>,
	},

	/// Test expects a specific error.
	#[ld(type = "test:NegativeEvaluationTest")]
	Negative {
		/// Expected error code.
		#[ld(prop = "mf:result")]
		expected_error_code: String,

		/// Context document IRI (for compaction tests).
		#[ld(prop = "test:context")]
		#[allow(unused)] // Unused in non-compaction tests.
		context: Option<IriBuf>,
	},

	/// Test expects successful processing (syntax validation only).
	#[ld(type = "test:PositiveSyntaxTest")]
	PositiveSyntax,
}
