use json_ld::{
	rdf_syntax::{self, dataset::IndexedBTreeDataset, Term},
	FsLoader, JsonLdProcessor, Loader,
};
use json_ld_testing::{ManifestEntry, TestKind};
use nquads_syntax::grdf_document_from_str;

mod common;

#[json_ld_testing::test_suite("toRdf-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
#[ignore_test("#te122", see = "https://github.com/w3c/json-ld-api/issues/480")]
#[ignore_test("#tli12", see = "https://github.com/w3c/json-ld-api/issues/533")]
fn to_rdf(loader: &FsLoader, entry: &ManifestEntry) {
	let options = common::build_options(entry);

	match &entry.kind {
		TestKind::Positive { expect, .. } => {
			let generator =
				rdf_syntax::generator::BlankIdGenerator::new_with_prefix("b".to_string());
			let input = loader.load(&entry.input).unwrap();
			let quads = input
				.to_rdf_with(loader, generator, options)
				.expect("to_rdf failed");

			let dataset: IndexedBTreeDataset<Term> = quads.into_iter().collect();

			let expected_path = loader.filepath(expect.as_iri()).unwrap();
			let expected_content = std::fs::read_to_string(expected_path).unwrap();
			let (expected_quads, _) = grdf_document_from_str(&expected_content).unwrap();
			let expected_dataset: IndexedBTreeDataset<Term> = expected_quads.into_iter().collect();

			let success = rdf_syntax::are_isomorphic(&dataset, &expected_dataset);

			if !success {
				eprintln!("output=");
				for q in &dataset {
					eprintln!("  {q}");
				}
				eprintln!("expected=");
				for q in &expected_dataset {
					eprintln!("  {q}");
				}
			}

			assert!(success, "test `{}` output mismatch", entry.name);
		}
		TestKind::Negative {
			expected_error_code,
			..
		} => {
			let generator =
				rdf_syntax::generator::BlankIdGenerator::new_with_prefix("b".to_string());
			let input = loader.load(&entry.input).unwrap();
			let result = input.to_rdf_with(loader, generator, options);
			assert!(
				result.is_err(),
				"test `{}` should have failed with `{}`",
				entry.name,
				expected_error_code
			);
		}
		TestKind::PositiveSyntax => {
			let generator =
				rdf_syntax::generator::BlankIdGenerator::new_with_prefix("b".to_string());
			let input = loader.load(&entry.input).unwrap();
			input
				.to_rdf_with(loader, generator, options)
				.expect("positive syntax test failed");
		}
	}
}
