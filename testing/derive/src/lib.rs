//! Derive macro to generate test suite functions from a JSON-LD manifest.
//!
//! # Example
//!
//! ```ignore
//! #[json_ld_testing::test_suite("path/to/manifest/file.jsonld")]
//! fn my_test(test: Test) {
//!   // test body.
//! }
//! ```
//!
//! This generates one `my_test_<test-id>` test function per test in the
//! manifest:
//!
//! ```ignore
//! #[test]
//! fn my_test_t0001() {
//!   my_test(<test-value>)
//! }
//! ```
use proc_macro::TokenStream;

/// Attribute macro that generates test functions from a JSON-LD test manifest.
///
/// The macro loads the manifest file at compile time, expands the JSON-LD,
/// converts to RDF, deserializes the test entries, and generates one `#[test]`
/// function per entry.
///
/// # Attributes
///
/// Additional attributes can be placed on the annotated item:
///
/// - `#[mount("https://example.org/", "local/path")]` — Mount a local
///   directory as a URL prefix for the document loader.
/// - `#[ignore("#fragment", see = "https://issue-link")]` — Skip a
///   specific test by its IRI fragment.
#[proc_macro_attribute]
pub fn test_suite(_args: TokenStream, input: TokenStream) -> TokenStream {
	// TODO: Implement once the json-ld expansion API is stabilized.
	//
	// The implementation should:
	// 1. Parse the manifest IRI from `args`.
	// 2. Parse helper attributes (#[mount], #[ignore])
	//    from the annotated module.
	// 3. Load and expand the JSON-LD manifest document.
	// 4. Convert the expanded document to RDF quads.
	// 5. For each test entry, deserialize into the user-defined types
	//    declared in the module body (using #[iri(...)] annotations).
	// 6. Generate a #[test] function per entry that constructs the test
	//    value and calls `.run()` on it.
	//
	// For now, pass through the input unchanged.
	input
}
