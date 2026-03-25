use linked_data::{DeserializeLinkedData, SerializeLinkedData};

/// JSON-LD specification version.
#[derive(
	Default, Clone, Copy, PartialEq, Eq, Hash, Debug, SerializeLinkedData, DeserializeLinkedData,
)]
pub enum SpecVersion {
	/// JSON-LD 1.0.
	#[ld(literal = "json-ld-1.0")]
	JsonLd1_0,

	/// JSON-LD 1.1.
	#[default]
	#[ld(literal = "json-ld-1.1")]
	JsonLd1_1,
}

#[cfg(feature = "proc_macro2")]
impl quote::ToTokens for SpecVersion {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let t = match self {
			Self::JsonLd1_0 => quote::quote! { json_ld_testing::SpecVersion::JsonLd1_0 },
			Self::JsonLd1_1 => quote::quote! { json_ld_testing::SpecVersion::JsonLd1_1 },
		};
		tokens.extend(t);
	}
}

#[cfg(feature = "proc_macro2")]
impl crate::tokens::ToExprTokens for SpecVersion {
	fn to_expr_tokens(&self) -> proc_macro2::TokenStream {
		quote::ToTokens::to_token_stream(self)
	}
}
