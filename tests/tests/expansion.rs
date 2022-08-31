use json_ld::{
	BorrowWithNamespace, ContextLoader, Expand, IriNamespaceMut, Loader, Print, Process,
	TryFromJson,
};
use locspan::Meta;
use static_iref::iri;

const STACK_SIZE: usize = 4 * 1024 * 1024;

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
		pub input: Iri<'static>,

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
			expect: Iri<'static>,
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
		pub base: Option<Iri<'static>>,

		#[iri("test:expandContext")]
		pub context: Option<Iri<'static>>,

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

		let mut options = json_ld::expansion::Options::default();
		if let Some(p) = self.options.processing_mode {
			options.processing_mode = p
		}

		let mut namespace = json_ld::IndexNamespace::new();
		let mut loader: json_ld::FsLoader = json_ld::FsLoader::default();
		loader.mount(
			namespace.insert(iri!("https://w3c.github.io/json-ld-api").into()),
			"json-ld-api",
		);

		let input = namespace.insert(self.input);
		let base = self
			.options
			.base
			.map(|i| namespace.insert(i))
			.unwrap_or(input);

		let context = match self.options.context {
			Some(iri) => {
				let i = namespace.insert(iri);
				let ld_context = loader.load_context_in(&mut namespace, i).await.unwrap();
				ld_context
					.process(&mut namespace, &mut loader, Some(base))
					.await
					.unwrap()
			}
			None => json_ld::Context::new(Some(base)),
		};

		match self.desc {
			expand::Description::Positive { expect } => {
				let json_ld = loader.load_in(&mut namespace, input).await.unwrap();
				let expanded: json_ld::ExpandedDocument = json_ld
					.expand_full(
						&mut namespace,
						context,
						Some(&base),
						&mut loader,
						options,
						(),
					)
					.await
					.unwrap();

				let expect_iri = namespace.insert(expect);
				let expected = loader.load_in(&mut namespace, expect_iri).await.unwrap();
				let Meta(expected, _) =
					json_ld::ExpandedDocument::try_from_json_in(&mut namespace, expected).unwrap();

				let success = expanded == expected;

				if !success {
					eprintln!("test failed");
					eprintln!(
						"output=\n{}",
						expanded.with_namespace(&namespace).pretty_print()
					);
					eprintln!(
						"expected=\n{}",
						expected.with_namespace(&namespace).pretty_print()
					);
				}

				assert!(success)
			}
			expand::Description::Negative {
				expected_error_code,
			} => {
				match loader.load_in(&mut namespace, input).await {
					Ok(json_ld) => {
						let result: Result<json_ld::ExpandedDocument, _> = json_ld
							.expand_full(
								&mut namespace,
								context,
								Some(&base),
								&mut loader,
								options,
								(),
							)
							.await;

						match result {
							Ok(expanded) => {
								eprintln!(
									"output=\n{}",
									expanded.with_namespace(&namespace).pretty_print()
								);
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
