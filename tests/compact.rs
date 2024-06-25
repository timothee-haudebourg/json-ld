use contextual::WithContext;
use json_ld::{JsonLdProcessor, Loader, Print, RemoteDocument, RemoteDocumentReference};
use rdf_types::vocabulary::{IndexVocabulary, IriIndex, IriVocabularyMut};
use static_iref::iri;

#[json_ld_testing::test_suite("https://w3c.github.io/json-ld-api/tests/compact-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
#[iri_prefix("rdf" = "http://www.w3.org/1999/02/22-rdf-syntax-ns#")]
#[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
#[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
#[iri_prefix("test" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
#[ignore_test("#tp004", see = "https://github.com/w3c/json-ld-api/issues/517")]
mod compact {
	use iref::Iri;

	#[iri("test:CompactTest")]
	pub struct Test {
		#[iri("rdfs:comment")]
		pub comments: &'static [&'static str],

		#[iri("manifest:action")]
		pub input: &'static Iri,

		#[iri("manifest:name")]
		pub name: &'static str,

		#[iri("test:option")]
		pub options: Options,

		#[iri("test:context")]
		pub context: &'static Iri,

		#[iri("rdf:type")]
		pub desc: Description,
	}

	pub enum Description {
		#[iri("test:PositiveEvaluationTest")]
		Positive {
			#[iri("manifest:result")]
			expect: &'static Iri,
		},
		#[iri("test:NegativeEvaluationTest")]
		Negative {
			#[iri("manifest:result")]
			expected_error_code: &'static str,
		},
	}

	#[derive(Default)]
	pub struct Options {
		#[iri("test:base")]
		pub base: Option<&'static Iri>,

		#[iri("test:expandContext")]
		pub expand_context: Option<&'static Iri>,

		#[iri("test:processingMode")]
		pub processing_mode: Option<json_ld::ProcessingMode>,

		#[iri("test:specVersion")]
		pub spec_version: Option<&'static str>,

		#[iri("test:normative")]
		pub normative: Option<bool>,

		#[iri("test:compactToRelative")]
		pub compact_to_relative: Option<bool>,

		#[iri("test:compactArrays")]
		pub compact_arrays: Option<bool>,
	}
}

impl compact::Test {
	fn run(self) {
		let child = std::thread::Builder::new()
			.spawn(|| async_std::task::block_on(self.async_run()))
			.unwrap();

		child.join().unwrap()
	}

	async fn async_run(self) {
		if !self.options.normative.unwrap_or(true) {
			log::warn!("ignoring test `{}` (non normative)", self.name);
			return;
		}

		if self.options.spec_version == Some("json-ld-1.0") {
			log::warn!("ignoring test `{}` (unsupported spec version)", self.name);
			return;
		}

		for comment in self.comments {
			println!("{}", comment)
		}

		let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
		let mut loader = json_ld::FsLoader::default();
		loader.mount(
			iri!("https://w3c.github.io/json-ld-api").to_owned(),
			"tests/json-ld-api",
		);

		let mut options: json_ld::Options<IriIndex> = json_ld::Options::default();
		if let Some(p) = self.options.processing_mode {
			options.processing_mode = p
		}

		options.base = self.options.base.map(|iri| vocabulary.insert(iri));
		options.expand_context = self
			.options
			.expand_context
			.map(|iri| RemoteDocumentReference::Iri(vocabulary.insert(iri)));

		options.compact_arrays = self.options.compact_arrays.unwrap_or(true);
		options.compact_to_relative = self.options.compact_to_relative.unwrap_or(true);

		let input = vocabulary.insert(self.input);
		let context = RemoteDocumentReference::Iri(vocabulary.insert(self.context));

		match self.desc {
			compact::Description::Positive { expect } => {
				let json_ld = loader.load_with(&mut vocabulary, input).await.unwrap();
				let compacted = json_ld
					.compact_full(&mut vocabulary, context, &mut loader, options, ())
					.await
					.unwrap();
				let compacted = RemoteDocument::new(Some(input), None, compacted);

				let expect = vocabulary.insert(expect);
				let mut expect = loader.load_with(&mut vocabulary, expect).await.unwrap();
				expect.set_url(Some(input));

				let expand_options: json_ld::Options<IriIndex> = json_ld::Options::default();
				let success = compacted
					.compare_full(&expect, &mut vocabulary, &mut loader, expand_options, ())
					.await
					.unwrap();

				if !success {
					eprintln!("test failed");
					eprintln!(
						"output=\n{}",
						compacted.with(&vocabulary).document().pretty_print()
					);
					eprintln!(
						"expected=\n{}",
						expect.document().with(&vocabulary).pretty_print()
					);
				}

				assert!(success)
			}
			compact::Description::Negative {
				expected_error_code,
			} => {
				match loader.load_with(&mut vocabulary, input).await {
					Ok(json_ld) => {
						let result: Result<_, _> = json_ld
							.compact_full(&mut vocabulary, context, &mut loader, options, ())
							.await;

						match result {
							Ok(expanded) => {
								eprintln!("output=\n{}", expanded.with(&vocabulary).pretty_print());
								panic!(
									"expansion succeeded when it should have failed with `{}`",
									expected_error_code
								)
							}
							Err(_) => {
								// ...
							}
						}
					}
					Err(_) => {
						// ...
					}
				}
			}
		}
	}
}
