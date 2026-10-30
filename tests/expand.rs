use json_ld::{
	JsonLdExpandedDocument, JsonLdFsLoader, JsonLdIndexedObject, JsonLdLoader, JsonLdProcessor,
};

mod common;

use common::{ManifestEntry, TestKind};

#[json_ld_testing::test_suite("tests/expand-manifest.jsonld")]
#[base = "https://w3c.github.io/json-ld-api/"]
#[mount("tests/json-ld-api")]
#[prefix("test" = "tests/vocab#")]
#[ignore_if("test:option"."test:normative" == false, note = "non-normative test")]
#[ignore_if("test:option"."test:specVersion" == "json-ld-1.0", note = "unsupported spec version (json-ld-1.0)")]
fn expand(loader: &JsonLdFsLoader, entry: &ManifestEntry) {
	let options = common::build_options(entry);

	match &entry.kind {
		TestKind::Positive { expect, .. } => {
			let input = loader.load(&entry.input).unwrap();
			let expanded = input
				.expand_with(loader, options)
				.expect("expansion failed");

			let expected_doc = loader.load(expect).unwrap();
			let expected: Vec<JsonLdIndexedObject> =
				json_syntax::serde::from_value(expected_doc.into_document())
					.expect("failed to parse expected output");
			let expected: JsonLdExpandedDocument = expected.into_iter().collect();

			assert_eq!(expanded, expected, "test `{}` output mismatch", entry.name);
		}
		TestKind::Negative {
			expected_error_code,
			..
		} => {
			let input = loader.load(&entry.input).unwrap();
			let result = input.expand_with(loader, options);
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
