use contextual::WithContext;
use json_ld::{JsonLdProcessor, Loader, Print, RemoteDocumentReference, TryFromJson};
use rdf_types::vocabulary::{IndexVocabulary, IriIndex, IriVocabularyMut};
use static_iref::iri;

#[json_ld_testing::test_suite("https://w3c.github.io/json-ld-api/tests/expand-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
#[iri_prefix("rdf" = "http://www.w3.org/1999/02/22-rdf-syntax-ns#")]
#[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
#[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
#[iri_prefix("test" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
mod expand {
	use iref::Iri;

	#[iri("test:ExpandTest")]
	pub struct Test {
		#[iri("rdfs:comment")]
		pub comments: &'static [&'static str],

		#[iri("manifest:action")]
		pub input: &'static Iri,

		#[iri("manifest:name")]
		pub name: &'static str,

		#[iri("test:option")]
		pub options: Options,

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
	}
}

impl expand::Test {
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

		let input = vocabulary.insert(self.input);

		match self.desc {
			expand::Description::Positive { expect } => {
				let json_ld = loader.load_with(&mut vocabulary, input).await.unwrap();
				let expanded = json_ld
					.expand_full(&mut vocabulary, &mut loader, options, ())
					.await
					.unwrap();

				let expect_iri = vocabulary.insert(expect);
				let expected = loader
					.load_with(&mut vocabulary, expect_iri)
					.await
					.unwrap()
					.into_document();
				let expected =
					json_ld::ExpandedDocument::try_from_json_in(&mut vocabulary, expected).unwrap();

				let success = expanded == expected;

				if !success {
					eprintln!("test failed");
					eprintln!("output=\n{}", expanded.with(&vocabulary).pretty_print());
					eprintln!("expected=\n{}", expected.with(&vocabulary).pretty_print());
				}

				assert!(success)
			}
			expand::Description::Negative {
				expected_error_code,
			} => {
				let json_ld = loader.load_with(&mut vocabulary, input).await.unwrap();
				let result: Result<_, _> = json_ld
					.expand_full(&mut vocabulary, &mut loader, options, ())
					.await;

				match result {
					Ok(expanded) => {
						eprintln!("output=\n{}", expanded.with(&vocabulary).pretty_print());
						panic!(
							"expansion succeeded when it should have failed with `{}`",
							expected_error_code
						)
					}
					Err(_e) => {
						// TODO improve error codes.
						// assert_eq!(e.code().as_str(), expected_error_code)
					}
				}
			}
		}
	}
}
