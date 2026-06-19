use json_ld::{AsyncLoader, ExpandedDocument, FsLoader, IndexedObject, JsonLdProcessor};
use json_ld_testing::{ManifestEntry, TestKind};

mod common;

#[json_ld_testing::test_suite("expand-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
async fn expand(loader: &FsLoader, entry: &ManifestEntry) {
	let options = common::build_options(entry);

	match &entry.kind {
		TestKind::Positive { expect, .. } => {
			let input = loader.async_load(&entry.input).await.unwrap();
			let expanded = input
				.async_expand_with(loader, options)
				.await
				.expect("expansion failed");

			let expected_doc = loader.async_load(expect).await.unwrap();
			let expected: Vec<IndexedObject> =
				json_syntax::serde::from_value(expected_doc.into_document())
					.expect("failed to parse expected output");
			let expected: ExpandedDocument = expected.into_iter().collect();

			assert_eq!(expanded, expected, "test `{}` output mismatch", entry.name);
		}
		TestKind::Negative {
			expected_error_code,
			..
		} => {
			let input = loader.async_load(&entry.input).await.unwrap();
			let result = input.async_expand_with(loader, options).await;
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
