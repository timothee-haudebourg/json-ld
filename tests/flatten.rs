use json_ld::{
	JsonLdDocument, JsonLdFsLoader, JsonLdLoader, JsonLdProcessor, RemoteJsonLdDocument,
};
use json_syntax::JsonPrint;

mod common;

use common::{ManifestEntry, TestKind};

#[json_ld_testing::test_suite("tests/flatten-manifest.jsonld")]
#[base = "https://w3c.github.io/json-ld-api/"]
#[mount("tests/json-ld-api")]
#[prefix("test" = "tests/vocab#")]
#[ignore_if("test:option"."test:normative" == false, note = "non-normative test")]
#[ignore_if("test:option"."test:specVersion" == "json-ld-1.0", note = "unsupported spec version (json-ld-1.0)")]
fn flatten(loader: &JsonLdFsLoader, entry: &ManifestEntry) {
	let options = common::build_options(entry);

	match &entry.kind {
		TestKind::Positive {
			expect, context, ..
		} => {
			let context = context
				.as_ref()
				.map(|c| RemoteJsonLdDocument::iri(c.clone()));

			let input = loader.load(&entry.input).unwrap();
			let flattened = input
				.flatten_with(context, loader, options.clone())
				.expect("flattening failed");
			let flattened = JsonLdDocument::new(Some(entry.input.clone()), None, flattened);

			let mut expected = loader.load(expect).unwrap();
			expected.set_url(Some(entry.input.clone()));

			let success = flattened
				.compare(&expected, loader)
				.expect("comparison failed");

			if !success {
				eprintln!("output=\n{}", flattened.document.pretty_print());
				eprintln!("expected=\n{}", expected.document.pretty_print());
			}

			assert!(success, "test `{}` output mismatch", entry.name);
		}
		TestKind::Negative {
			expected_error_code,
			..
		} => {
			let input = loader.load(&entry.input).unwrap();
			let result = input.flatten_with(None, loader, options.clone());
			assert!(
				result.is_err(),
				"test `{}` should have failed with `{}`",
				entry.name,
				expected_error_code
			);
		}
		TestKind::PositiveSyntax => {}
	}
}
