use contextual::WithContext;
use json_ld::{JsonLdProcessor, Loader, Print, RemoteDocumentReference};
use nquads_syntax::{strip_quad, Parse};
use rdf_types::{
	dataset::{isomorphism::are_isomorphic_with, IndexedBTreeDataset},
	interpretation::VocabularyInterpretation,
	vocabulary::{
		BlankIdIndex, EmbedIntoVocabulary, IndexVocabulary, IriIndex, IriVocabularyMut,
		LiteralIndex,
	},
};
use static_iref::iri;

type IndexTerm = rdf_types::Term<rdf_types::Id<IriIndex, BlankIdIndex>, LiteralIndex>;

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
		pub input: &'static Iri,

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
			expect: &'static Iri,
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
		pub base: Option<&'static Iri>,

		#[iri("jld:processingMode")]
		pub processing_mode: Option<json_ld::ProcessingMode>,

		#[iri("jld:specVersion")]
		pub spec_version: Option<&'static str>,

		#[iri("jld:normative")]
		pub normative: Option<bool>,

		#[iri("jld:expandContext")]
		pub expand_context: Option<&'static Iri>,

		#[iri("jld:produceGeneralizedRdf")]
		pub produce_generalized_rdf: bool,

		#[iri("jld:rdfDirection")]
		pub rdf_direction: Option<RdfDirection>,
	}
}

impl to_rdf::Test {
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
		options.rdf_direction = self.options.rdf_direction;
		options.produce_generalized_rdf = self.options.produce_generalized_rdf;

		let input = vocabulary.insert(self.input);

		match self.desc {
			to_rdf::Description::Positive { expect } => {
				let json_ld = loader.load_with(&mut vocabulary, input).await.unwrap();

				let mut generator = rdf_types::generator::Blank::new_with_prefix("b".to_string());
				let mut to_rdf = json_ld
					.to_rdf_full(&mut vocabulary, &mut generator, &mut loader, options, ())
					.await
					.unwrap();

				let dataset: IndexedBTreeDataset<IndexTerm> = to_rdf
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

				let expected_content =
					std::fs::read_to_string(loader.filepath(&expect).unwrap()).unwrap();
				let expected_dataset: IndexedBTreeDataset<IndexTerm> =
					nquads_syntax::GrdfDocument::parse_str(&expected_content)
						.unwrap()
						.into_value()
						.into_iter()
						.map(|q| strip_quad(q.into_value()).embed_into_vocabulary(&mut vocabulary))
						.collect();

				let success = are_isomorphic_with(
					&VocabularyInterpretation::<IndexVocabulary>::new(),
					&dataset,
					&expected_dataset,
				);

				if !success {
					eprintln!("test failed");
					eprintln!("output=");
					for q in dataset {
						eprintln!("{}", q.with(&vocabulary));
					}

					eprintln!("expected=");
					for q in expected_dataset {
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
