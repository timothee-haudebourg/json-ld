use linked_data::DeserializeLinkedData;

use crate::SpecVersion;

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
	pub processing_mode: Option<json_ld::ProcessingMode>,

	/// Spec version.
	#[ld(prop = "test:specVersion")]
	pub spec_version: Option<SpecVersion>,

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

#[cfg(feature = "proc_macro2")]
impl quote::ToTokens for TestOptions {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		use crate::tokens::ToExprTokens;
		use quote::quote;

		let base = self.base.to_expr_tokens();
		let expand_context = self.expand_context.to_expr_tokens();
		let processing_mode = self.processing_mode.to_expr_tokens();
		let spec_version = self.spec_version.to_expr_tokens();
		let normative = self.normative.to_expr_tokens();
		let compact_to_relative = self.compact_to_relative.to_expr_tokens();
		let compact_arrays = self.compact_arrays.to_expr_tokens();
		let use_native_types = self.use_native_types.to_expr_tokens();
		let produce_generalized_rdf = self.produce_generalized_rdf.to_expr_tokens();
		let rdf_direction = self.rdf_direction.to_expr_tokens();
		let content_type = self.content_type.to_expr_tokens();
		let http_link = self.http_link.to_expr_tokens();
		let http_status = self.http_status.to_expr_tokens();
		let extract_all_scripts = self.extract_all_scripts.to_expr_tokens();

		tokens.extend(quote! {
			json_ld_testing::TestOptions {
				base: #base,
				expand_context: #expand_context,
				processing_mode: #processing_mode,
				spec_version: #spec_version,
				normative: #normative,
				compact_to_relative: #compact_to_relative,
				compact_arrays: #compact_arrays,
				use_native_types: #use_native_types,
				produce_generalized_rdf: #produce_generalized_rdf,
				rdf_direction: #rdf_direction,
				content_type: #content_type,
				http_link: #http_link,
				http_status: #http_status,
				extract_all_scripts: #extract_all_scripts,
			}
		});
	}
}

#[cfg(feature = "proc_macro2")]
impl crate::tokens::ToExprTokens for TestOptions {
	fn to_expr_tokens(&self) -> proc_macro2::TokenStream {
		quote::ToTokens::to_token_stream(self)
	}
}
