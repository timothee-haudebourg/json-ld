use json_ld::{syntax::PrintJson, Document, FsLoader, JsonLdProcessor, Loader, RemoteDocument};
use json_ld_testing::{ManifestEntry, TestKind};

mod common;

#[json_ld_testing::test_suite("flatten-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
fn flatten(loader: &FsLoader, entry: &ManifestEntry) {
	let options = common::build_options(entry);

	match &entry.kind {
		TestKind::Positive {
			expect, context, ..
		} => {
			let context = context.as_ref().map(|c| RemoteDocument::iri(c.clone()));

			let input = loader.load(&entry.input).unwrap();
			let flattened = input
				.flatten_with(context, loader, options.clone())
				.expect("flattening failed");
			let flattened = Document::new(Some(entry.input.clone()), None, flattened);

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
