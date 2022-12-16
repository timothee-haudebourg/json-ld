use contextual::WithContext;
use json_ld::{JsonLdProcessor, Loader, Print, RemoteDocumentReference};
use locspan::{Meta, Strip};
use nquads_syntax::Parse;
use rdf_types::{IndexVocabulary, IriVocabularyMut};
use static_iref::iri;

const STACK_SIZE: usize = 4 * 1024 * 1024;

#[json_ld_testing::test_suite("https://w3c.github.io/json-ld-api/tests/toRdf-manifest.jsonld")]
#[mount("https://w3c.github.io/json-ld-api", "tests/json-ld-api")]
#[iri_prefix("rdf" = "http://www.w3.org/1999/02/22-rdf-syntax-ns#")]
#[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
#[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
#[iri_prefix("jld" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
#[ignore_test("#te122", see = "https://github.com/w3c/json-ld-api/issues/480")]
#[ignore_test("#tli12", see = "https://github.com/w3c/json-ld-api/issues/533")]
mod to_rdf {
	use iref::Iri;
	use json_ld::rdf::RdfDirection;

	#[iri("jld:ToRDFTest")]
	pub struct Test {
		#[iri("rdfs:comment")]
		pub comments: &'static [&'static str],

		#[iri("manifest:action")]
		pub input: Iri<'static>,

		#[iri("manifest:name")]
		pub name: &'static str,

		#[iri("jld:option")]
		pub options: Options,

		#[iri("rdf:type")]
		pub desc: Description,
	}

	pub enum Description {
		#[iri("jld:PositiveEvaluationTest")]
		Positive {
			#[iri("manifest:result")]
			expect: Iri<'static>,
		},
		#[iri("jld:NegativeEvaluationTest")]
		Negative {
			#[iri("manifest:result")]
			expected_error_code: &'static str,
		},
		#[iri("jld:PositiveSyntaxTest")]
		PositiveSyntax {
			// ...
		},
	}

	#[derive(Default)]
	pub struct Options {
		#[iri("jld:base")]
		pub base: Option<Iri<'static>>,

		#[iri("jld:processingMode")]
		pub processing_mode: Option<json_ld::ProcessingMode>,

		#[iri("jld:specVersion")]
		pub spec_version: Option<&'static str>,

		#[iri("jld:normative")]
		pub normative: Option<bool>,

		#[iri("jld:expandContext")]
		pub expand_context: Option<Iri<'static>>,

		#[iri("jld:produceGeneralizedRdf")]
		pub produce_generalized_rdf: bool,

		#[iri("jld:rdfDirection")]
		pub rdf_direction: Option<RdfDirection>,
	}
}

impl to_rdf::Test {
	fn run(self) {
		let child = std::thread::Builder::new()
			.stack_size(STACK_SIZE)
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
		let mut loader: json_ld::FsLoader = json_ld::FsLoader::default();
		loader.mount(
			vocabulary.insert(iri!("https://w3c.github.io/json-ld-api").into()),
			"json-ld-api",
		);

		let mut options: json_ld::Options = json_ld::Options::default();
		if let Some(p) = self.options.processing_mode {
			options.processing_mode = p
		}

		options.base = self.options.base.map(|iri| vocabulary.insert(iri));
		options.expand_context = self
			.options
			.expand_context
			.map(|iri| RemoteDocumentReference::Iri(vocabulary.insert(iri)));
		options.rdf_direction = self.options.rdf_direction;
		options.produce_generalized_rdf = self.options.produce_generalized_rdf;

		let input = vocabulary.insert(self.input);

		match self.desc {
			to_rdf::Description::Positive { expect } => {
				let json_ld = loader.load_with(&mut vocabulary, input).await.unwrap();

				let mut generator = rdf_types::generator::Blank::new_with_prefix("b".to_string())
					.with_metadata(locspan::Location::new(input, locspan::Span::default()));
				let mut to_rdf = json_ld
					.to_rdf_full(&mut vocabulary, &mut generator, &mut loader, options, ())
					.await
					.unwrap();

				let dataset: grdf::HashDataset<_, _, _, _> = to_rdf
					.quads()
					.cloned()
					.map(|rdf_types::Quad(s, p, o, g)| {
						rdf_types::Quad(
							s.into_term(),
							p.into_term(),
							o,
							g.map(rdf_types::Subject::into_term),
						)
					})
					.collect();

				let expect_url = vocabulary.insert(expect);
				let expected_content =
					std::fs::read_to_string(loader.filepath(&vocabulary, &expect_url).unwrap())
						.unwrap();
				let expected_dataset: grdf::HashDataset<_, _, _, _> =
					nquads_syntax::GrdfDocument::parse_str(&expected_content, |span| span)
						.unwrap()
						.into_value()
						.into_iter()
						.map(|Meta(q, _)| q.strip().insert_into(&mut vocabulary))
						.collect();

				let success = dataset.is_isomorphic_to(&expected_dataset);

				if !success {
					eprintln!("test failed");
					eprintln!("output=");
					for q in dataset.into_quads() {
						eprintln!("{}", q.with(&vocabulary));
					}

					eprintln!("expected=");
					for q in expected_dataset.into_quads() {
						eprintln!("{}", q.with(&vocabulary));
					}
				}

				assert!(success)
			}
			to_rdf::Description::Negative {
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
			to_rdf::Description::PositiveSyntax {} => {
				// ...
			}
		}
	}
}
