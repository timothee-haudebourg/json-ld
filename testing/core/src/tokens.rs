//! Shared tokenization helpers for foreign types.
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn iri_buf_tokens(iri: &iref::IriBuf) -> TokenStream {
	let s = iri.as_str();
	quote! { iref::IriBuf::new(#s).unwrap() }
}

pub fn option_iri_buf_tokens(opt: &Option<iref::IriBuf>) -> TokenStream {
	match opt {
		Some(iri) => {
			let t = iri_buf_tokens(iri);
			quote! { Some(#t) }
		}
		None => quote! { None },
	}
}

pub fn processing_mode_tokens(mode: json_ld::ProcessingMode) -> TokenStream {
	match mode {
		json_ld::ProcessingMode::JsonLd1_0 => quote! { json_ld::ProcessingMode::JsonLd1_0 },
		json_ld::ProcessingMode::JsonLd1_1 => quote! { json_ld::ProcessingMode::JsonLd1_1 },
	}
}

pub fn option_tokens<T: ToTokens>(opt: &Option<T>) -> TokenStream {
	match opt {
		Some(v) => quote! { Some(#v) },
		None => quote! { None },
	}
}
