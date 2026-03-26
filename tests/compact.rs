use json_ld::{syntax::PrintJson, Document, FsLoader, JsonLdProcessor, Loader, RemoteDocument};
use json_ld_testing::{ManifestEntry, TestKind};

mod common;

#[json_ld_testing::test_suite("compact-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
#[ignore_test("#tp004", see = "https://github.com/w3c/json-ld-api/issues/517")]
async fn compact(loader: &FsLoader, entry: &ManifestEntry) {
	let options = common::build_options(entry);

	match &entry.kind {
		TestKind::Positive {
			expect, context, ..
		} => {
			let context_url = context
				.as_ref()
				.expect("compact positive test must have a context");
			let context = RemoteDocument::iri(context_url.clone());

			let input = loader.load(&entry.input).await.unwrap();
			let compacted = input
				.compact_with(context, loader, options.clone())
				.await
				.expect("compaction failed");
			let compacted = Document::new(Some(entry.input.clone()), None, compacted);

			let mut expected = loader.load(expect).await.unwrap();
			expected.set_url(Some(entry.input.clone()));

			let success = compacted
				.compare(&expected, loader)
				.await
				.expect("comparison failed");

			if !success {
				eprintln!("test `{}` failed", entry.name);
				eprintln!("output=\n{}", compacted.document.pretty_print());
				eprintln!("expected=\n{}", expected.document.pretty_print());
			}

			assert!(success, "test `{}` output mismatch", entry.name);
		}
		TestKind::Negative {
			expected_error_code,
			context,
			..
		} => {
			let context_url = context
				.as_ref()
				.expect("compact negative test must have a context");
			let context = RemoteDocument::iri(context_url.clone());

			let input = loader.load(&entry.input).await.unwrap();
			let result = input.compact_with(context, loader, options.clone()).await;
			assert!(
				result.is_err(),
				"test `{}` should have failed with `{}`",
				entry.name,
				expected_error_code
			);
		}
	}
}
