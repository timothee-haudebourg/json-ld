use json_ld::{
	ExpandedDocument, FsLoader, IndexedObject, JsonLdOptions, JsonLdProcessor, Loader,
	ProcessingMode, RemoteDocument,
};
use json_ld_testing::{ManifestEntry, SpecVersion, TestKind};

#[json_ld_testing::test_suite("expand-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
async fn expand(loader: &FsLoader, entry: &ManifestEntry) {
	if should_skip(entry) {
		return;
	}

	let options = build_options(entry);

	match &entry.kind {
		TestKind::Positive { expect, .. } => {
			let input = loader.load(&entry.input).await.unwrap();
			let expanded = JsonLdProcessor::expand_with(&input, loader, options)
				.await
				.expect("expansion failed");

			let expected_doc = loader.load(expect).await.unwrap();
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
			let input = loader.load(&entry.input).await.unwrap();
			let result = JsonLdProcessor::expand_with(&input, loader, options).await;
			assert!(
				result.is_err(),
				"test `{}` should have failed with `{}`",
				entry.name,
				expected_error_code
			);
		}
	}
}

fn should_skip(entry: &ManifestEntry) -> bool {
	if let Some(ref opts) = entry.options {
		if opts.normative == Some(false) {
			return true;
		}
		if opts.spec_version == Some(SpecVersion::JsonLd1_0) {
			return true;
		}
	}
	false
}

fn build_options(entry: &ManifestEntry) -> JsonLdOptions {
	let mut options = JsonLdOptions::default();
	if let Some(ref opts) = entry.options {
		if let Some(mode) = opts.processing_mode {
			options.processing_mode = mode;
		}
		options.base = opts.base.clone();
		if let Some(ref ctx) = opts.expand_context {
			options.expand_context = Some(RemoteDocument::iri(ctx.clone()));
		}
	}
	options
}
