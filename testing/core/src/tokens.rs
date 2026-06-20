//! Shared tokenization helpers for foreign types.
use proc_macro2::TokenStream;
use quote::quote;

/// Trait for types that can be converted to expression tokens.
///
/// Unlike `ToTokens`, this produces tokens for an expression that
/// *constructs* the value, handling owned types like `String` and `IriBuf`
/// that need `.to_owned()` or `iri!(...).to_owned()`.
pub trait ToExprTokens {
	fn to_expr_tokens(&self) -> TokenStream;
}

impl ToExprTokens for bool {
	fn to_expr_tokens(&self) -> TokenStream {
		quote! { #self }
	}
}

impl ToExprTokens for String {
	fn to_expr_tokens(&self) -> TokenStream {
		quote! { #self.to_owned() }
	}
}

impl ToExprTokens for json_ld::Iri {
	fn to_expr_tokens(&self) -> TokenStream {
		let s = self.as_str();
		quote! { json_ld::iref::iri!(#s).to_owned() }
	}
}

impl ToExprTokens for json_ld::IriBuf {
	fn to_expr_tokens(&self) -> TokenStream {
		self.as_iri().to_expr_tokens()
	}
}

impl ToExprTokens for json_ld::ProcessingMode {
	fn to_expr_tokens(&self) -> TokenStream {
		match self {
			Self::JsonLd1_0 => quote! { json_ld::ProcessingMode::JsonLd1_0 },
			Self::JsonLd1_1 => quote! { json_ld::ProcessingMode::JsonLd1_1 },
		}
	}
}

impl<T: ToExprTokens> ToExprTokens for Option<T> {
	fn to_expr_tokens(&self) -> TokenStream {
		match self {
			Some(v) => {
				let t = v.to_expr_tokens();
				quote! { Some(#t) }
			}
			None => quote! { None },
		}
	}
}
