use json_ld::JsonLdProcessingMode;
use linked_data::DeserializeLinkedData;
use rdf_syntax::IriBuf;

/// Processing options for a test.
#[derive(Default, DeserializeLinkedData)]
#[ld(prefix("test" = "https://w3c.github.io/json-ld-api/tests/vocab#"))]
#[ld(prefix("xsd" = "http://www.w3.org/2001/XMLSchema#"))]
pub struct TestOptions {
	/// Base IRI.
	#[ld(prop = "test:base")]
	pub base: Option<IriBuf>,

	/// Expand context IRI.
	#[ld(prop = "test:expandContext")]
	pub expand_context: Option<IriBuf>,

	/// Processing mode.
	#[ld(prop = "test:processingMode")]
	pub processing_mode: Option<JsonLdProcessingMode>,

	// 	/// Spec version.
	// 	#[ld(prop = "test:specVersion")]
	// 	pub spec_version: Option<SpecVersion>,
	//
	// 	/// Whether the test is normative.
	// 	#[ld(prop = "test:normative")]
	// 	pub normative: Option<bool>,
	/// Compact to relative IRIs.
	#[ld(prop = "test:compactToRelative")]
	pub compact_to_relative: Option<bool>,

	/// Compact arrays.
	#[ld(prop = "test:compactArrays")]
	pub compact_arrays: Option<bool>,

	// /// Use native types when converting from RDF.
	// #[ld(prop = "test:useNativeTypes")]
	// pub use_native_types: Option<bool>,
	/// Produce generalized RDF.
	#[ld(prop = "test:produceGeneralizedRdf")]
	pub produce_generalized_rdf: Option<bool>,
	// 	/// RDF direction handling.
	// 	#[ld(prop = "test:rdfDirection")]
	// 	pub rdf_direction: Option<String>,
	//
	// 	/// Content type.
	// 	#[ld(prop = "test:contentType")]
	// 	pub content_type: Option<String>,
	//
	// 	/// HTTP link headers.
	// 	#[ld(prop = "test:httpLink")]
	// 	pub http_link: Option<String>,
	//
	// 	/// HTTP status code.
	// 	#[ld(prop = "test:httpStatus")]
	// 	pub http_status: Option<String>,
	//
	// 	/// Extract all scripts from HTML.
	// 	#[ld(prop = "test:extractAllScripts")]
	// 	pub extract_all_scripts: Option<bool>,
}
