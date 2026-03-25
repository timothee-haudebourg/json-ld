use linked_data::DeserializeLinkedData;

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

#[cfg(feature = "proc_macro2")]
impl quote::ToTokens for TestKind {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		use crate::tokens::{iri_buf_tokens, option_iri_buf_tokens};
		use quote::quote;

		let t = match self {
			Self::Positive { expect, context } => {
				let expect = iri_buf_tokens(expect);
				let context = option_iri_buf_tokens(context);
				quote! {
					json_ld_testing::TestKind::Positive {
						expect: #expect,
						context: #context,
					}
				}
			}
			Self::Negative {
				expected_error_code,
				context,
			} => {
				let context = option_iri_buf_tokens(context);
				quote! {
					json_ld_testing::TestKind::Negative {
						expected_error_code: #expected_error_code.to_owned(),
						context: #context,
					}
				}
			}
		};
		tokens.extend(t);
	}
}
