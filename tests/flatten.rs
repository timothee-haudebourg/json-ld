use json_ld::{
	syntax::PrintJson, AsyncLoader, Document, FsLoader, JsonLdProcessor, RemoteDocument,
};
use json_ld_testing::{ManifestEntry, TestKind};

mod common;

#[json_ld_testing::test_suite("flatten-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
async fn flatten(loader: &FsLoader, entry: &ManifestEntry) {
	let options = common::build_options(entry);

	match &entry.kind {
		TestKind::Positive {
			expect, context, ..
		} => {
			let context = context.as_ref().map(|c| RemoteDocument::iri(c.clone()));

			let input = loader.async_load(&entry.input).await.unwrap();
			let flattened = input
				.async_flatten_with(context, loader, options.clone())
				.await
				.expect("flattening failed");
			let flattened = Document::new(Some(entry.input.clone()), None, flattened);

			let mut expected = loader.async_load(expect).await.unwrap();
			expected.set_url(Some(entry.input.clone()));

			let success = flattened
				.async_compare(&expected, loader)
				.await
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
			let input = loader.async_load(&entry.input).await.unwrap();
			let result = input
				.async_flatten_with(None, loader, options.clone())
				.await;
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
