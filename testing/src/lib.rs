use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_derive(TestSuite)]
#[proc_macro_error]
pub fn test_suite(input: TokenStream) -> TokenStream {
    TokenStream::new()
}
