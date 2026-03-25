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
		use crate::tokens::{option_iri_buf_tokens, option_tokens, processing_mode_tokens};
		use quote::quote;

		let base = option_iri_buf_tokens(&self.base);
		let expand_context = option_iri_buf_tokens(&self.expand_context);
		let processing_mode = match self.processing_mode {
			Some(m) => {
				let m = processing_mode_tokens(m);
				quote! { Some(#m) }
			}
			None => quote! { None },
		};
		let spec_version = option_tokens(&self.spec_version);
		let normative = option_tokens(&self.normative);
		let compact_to_relative = option_tokens(&self.compact_to_relative);
		let compact_arrays = option_tokens(&self.compact_arrays);
		let use_native_types = option_tokens(&self.use_native_types);
		let produce_generalized_rdf = option_tokens(&self.produce_generalized_rdf);
		let rdf_direction = option_tokens(&self.rdf_direction);
		let content_type = option_tokens(&self.content_type);
		let http_link = option_tokens(&self.http_link);
		let http_status = option_tokens(&self.http_status);
		let extract_all_scripts = option_tokens(&self.extract_all_scripts);

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
