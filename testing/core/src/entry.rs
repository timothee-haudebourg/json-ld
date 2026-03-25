use linked_data::DeserializeLinkedData;

use crate::{TestKind, TestOptions};

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

#[cfg(feature = "proc_macro2")]
impl quote::ToTokens for ManifestEntry {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		use crate::tokens::ToExprTokens;
		use quote::quote;

		let id = self.id.to_expr_tokens();
		let name = self.name.to_expr_tokens();
		let purpose = self.purpose.to_expr_tokens();
		let input = self.input.to_expr_tokens();
		let kind = self.kind.to_expr_tokens();
		let options = self.options.to_expr_tokens();

		tokens.extend(quote! {
			json_ld_testing::ManifestEntry {
				id: #id,
				name: #name,
				purpose: #purpose,
				input: #input,
				kind: #kind,
				options: #options,
			}
		});
	}
}
