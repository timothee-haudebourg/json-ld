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

	/// Test expects successful processing (syntax validation only).
	#[ld(type = "test:PositiveSyntaxTest")]
	PositiveSyntax,
}

#[cfg(feature = "proc_macro2")]
impl quote::ToTokens for TestKind {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		use crate::tokens::ToExprTokens;
		use quote::quote;

		let t = match self {
			Self::Positive { expect, context } => {
				let expect = expect.to_expr_tokens();
				let context = context.to_expr_tokens();
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
				let expected_error_code = expected_error_code.to_expr_tokens();
				let context = context.to_expr_tokens();
				quote! {
					json_ld_testing::TestKind::Negative {
						expected_error_code: #expected_error_code,
						context: #context,
					}
				}
			}
			Self::PositiveSyntax => {
				quote! { json_ld_testing::TestKind::PositiveSyntax }
			}
		};
		tokens.extend(t);
	}
}

#[cfg(feature = "proc_macro2")]
impl crate::tokens::ToExprTokens for TestKind {
	fn to_expr_tokens(&self) -> proc_macro2::TokenStream {
		quote::ToTokens::to_token_stream(self)
	}
}
