use iref::{Iri, IriBuf};
use json_ld::{
	compaction,
	context::{self, Loader as ContextLoader, Local, ProcessedOwned, ProcessingOptions},
	util::json_ld_eq,
	Document, ErrorCode, FsLoader, Loader, ProcessingMode,
};
use serde_json::Value;
use static_iref::iri;

#[derive(Clone, Copy)]
struct Options<'a> {
	processing_mode: ProcessingMode,
	compact_arrays: bool,
	context: Option<Iri<'a>>,
}

impl<'a> From<Options<'a>> for compaction::Options {
	fn from(options: Options<'a>) -> compaction::Options {
		compaction::Options {
			processing_mode: options.processing_mode,
			compact_arrays: options.compact_arrays,
			ordered: false,
			..compaction::Options::default()
		}
	}
}

impl<'a> From<Options<'a>> for ProcessingOptions {
	fn from(options: Options<'a>) -> ProcessingOptions {
		ProcessingOptions {
			processing_mode: options.processing_mode,
			..ProcessingOptions::default()
		}
	}
}

fn base_json_context(base_url: Iri) -> Value {
	let mut object = serde_json::Map::new();
	object.insert("@base".to_string(), Value::from(base_url.as_str()));
	object.into()
}

fn no_metadata<M>(_: Option<&M>) -> () {
	()
}

async fn positive_test(
	options: Options<'_>,
	input_url: Iri<'_>,
	base_url: Iri<'_>,
	output_url: Iri<'_>,
) {
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = loader.load(input_url).await.unwrap();
	let expected_output = loader.load(output_url).await.unwrap();

	let expand_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));
	let compact_context: context::ProcessedOwned<Value, context::Json<Value, IriBuf>> =
		match options.context {
			Some(context_url) => {
				let local_context = loader
					.load_context(context_url)
					.await
					.unwrap()
					.into_context();
				local_context
					.process_with(
						&context::Json::new(Some(base_url)),
						&mut loader,
						Some(base_url),
						options.into(),
					)
					.await
					.unwrap()
					.owned()
			}
			None => {
				let base_json_context = base_json_context(base_url);
				ProcessedOwned::new(base_json_context, context::Json::new(Some(base_url)))
			}
		};

	let output: Value = input
		.compact_with(
			Some(base_url),
			&expand_context,
			&compact_context.inversible(),
			&mut loader,
			options.into(),
			no_metadata,
		)
		.await
		.unwrap();
	let success = json_ld_eq(&output, &*expected_output).await.unwrap();

	if !success {
		println!(
			"output=\n{}",
			serde_json::to_string_pretty(&output).unwrap()
		);
		println!(
			"\nexpected=\n{}",
			serde_json::to_string_pretty(&*expected_output).unwrap()
		);
	}

	assert!(success)
}

async fn negative_test(
	options: Options<'_>,
	input_url: Iri<'_>,
	base_url: Iri<'_>,
	error_code: ErrorCode,
) {
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = loader.load(input_url).await.unwrap();

	let expand_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));
	let compact_context: context::ProcessedOwned<Value, context::Json<Value, IriBuf>> =
		match options.context {
			Some(context_url) => {
				let local_context = loader
					.load_context(context_url)
					.await
					.unwrap()
					.into_context();
				let context = local_context
					.process_with(
						&context::Json::new(Some(base_url)),
						&mut loader,
						Some(base_url),
						options.into(),
					)
					.await;
				match context {
					Ok(context) => context.owned(),
					Err(e) => {
						assert_eq!(e.code(), error_code);
						return;
					}
				}
			}
			None => {
				let base_json_context = base_json_context(base_url);
				ProcessedOwned::new(base_json_context, context::Json::new(Some(base_url)))
			}
		};

	let result: Result<Value, _> = input
		.compact_with(
			Some(base_url),
			&expand_context,
			&compact_context.inversible(),
			&mut loader,
			options.into(),
			no_metadata,
		)
		.await;

	match result {
		Ok(output) => {
			println!(
				"output=\n{}",
				serde_json::to_string_pretty(&output).unwrap()
			);
			panic!(
				"compaction succeeded where it should have failed with code: {}",
				error_code
			)
		}
		Err(e) => {
			assert_eq!(e.code(), error_code)
		}
	}
}

#[async_std::test]
async fn compact_0001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0001-out.jsonld");
	println!("drop free-floating nodes");
	println!("Unreferenced nodes not containing properties are dropped");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0001-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0002-out.jsonld");
	println!("basic");
	println!("Basic term and value compaction");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0002-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0003-out.jsonld");
	println!("drop null and unmapped properties");
	println!("Properties mapped to null or which are never mapped are dropped");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0003-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0004-out.jsonld");
	println!("optimize @set, keep empty arrays");
	println!("Containers mapped to @set keep empty arrays");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0004-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0005-out.jsonld");
	println!("@type and prefix compaction");
	println!("Compact uses prefixes in @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0005-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0006-out.jsonld");
	println!("keep expanded object format if @type doesn't match");
	println!("Values not matching a coerced @type remain in expanded form");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0006-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0007-out.jsonld");
	println!("add context");
	println!("External context is added to the compacted document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0007-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0008-out.jsonld");
	println!("alias keywords");
	println!("Aliases for keywords are used in compacted document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0008-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0009-out.jsonld");
	println!("compact @id");
	println!("Value with @id is compacted to string if property cast to @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0009-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0010-out.jsonld");
	println!("array to @graph");
	println!("An array of objects is serialized with @graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0010-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0011-out.jsonld");
	println!("compact date");
	println!("Expanded value with type xsd:dateTime is represented as string with type coercion");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0011-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0012-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0012-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0012-out.jsonld");
	println!("native types");
	println!("Native values are unmodified during compaction");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0012-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0013-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0013-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0013-out.jsonld");
	println!("@value with @language");
	println!("Values with @language remain in expanded form by default");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0013-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0014-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0014-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0014-out.jsonld");
	println!("array to aliased @graph");
	println!("Aliasing @graph uses alias in compacted document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0014-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0015-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0015-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0015-out.jsonld");
	println!("best match compaction");
	println!("Property with values of different types use most appropriate term when compacting");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0015-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0016-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0016-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0016-out.jsonld");
	println!("recursive named graphs");
	println!("Compacting a document with multiple embedded uses of @graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0016-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0017-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0017-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0017-out.jsonld");
	println!("A term mapping to null removes the mapping");
	println!("Mapping a term to null causes the property and its values to be removed from the compacted document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0017-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0018-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0018-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0018-out.jsonld");
	println!("best matching term for lists");
	println!("Lists with values of different types use best term in compacted document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0018-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0019-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0019-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0019-out.jsonld");
	println!("Keep duplicate values in @list and @set");
	println!("Duplicate values in @list or @set are retained in compacted document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0019-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0020-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0020-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0020-out.jsonld");
	println!("Compact @id that is a property IRI when @container is @list");
	println!("A term with @container: @list is also used as the value of an @id, if appropriate");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0020-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0021() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0021-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0021-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0021-out.jsonld");
	println!("Compact properties and types using @vocab");
	println!("@vocab is used to create relative properties and types if no other term matches");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0021-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0022-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0022-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0022-out.jsonld");
	println!("@list compaction of nested properties");
	println!("Compact nested properties using @list containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0022-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0023() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0023-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0023-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0023-out.jsonld");
	println!("prefer @vocab over compacted IRIs");
	println!("@vocab takes precedence over prefixes - even if the result is longer");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0023-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0024() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0024-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0024-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0024-out.jsonld");
	println!("most specific term matching in @list.");
	println!("The most specific term that matches all of the elements in the list, taking into account the default language, must be selected.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0024-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0025() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0025-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0025-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0025-out.jsonld");
	println!("Language maps");
	println!("Multiple values with different languages use language maps if property has @container: @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0025-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0026() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0026-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0026-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0026-out.jsonld");
	println!("Language map term selection with complications");
	println!("Test appropriate property use given language maps with @vocab, a default language, and a competing term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0026-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0027() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0027-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0027-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0027-out.jsonld");
	println!("@container: @set with multiple values");
	println!("Fall back to term with @set container if term with language map is defined");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0027-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0028() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0028-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0028-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0028-out.jsonld");
	println!("Alias keywords and use @vocab");
	println!("Combination of keyword aliases and @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0028-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0029() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0029-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0029-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0029-out.jsonld");
	println!("Simple @index map");
	println!("Output uses index mapping if term is defined with @container: @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0029-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0030() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0030-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0030-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0030-out.jsonld");
	println!("non-matching @container: @index");
	println!("Preserve @index tags if not compacted to an index map");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0030-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0031() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0031-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0031-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0031-out.jsonld");
	println!("Compact @reverse");
	println!("Compact traverses through @reverse");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0031-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0032() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0032-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0032-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0032-out.jsonld");
	println!("Compact keys in reverse-maps");
	println!("Compact traverses through @reverse");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0032-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0033() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0033-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0033-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0033-out.jsonld");
	println!("Compact reverse-map to reverse property");
	println!("A reverse map is replaced with a matching property defined with @reverse");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0033-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0034() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0034-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0034-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0034-out.jsonld");
	println!("Skip property with @reverse if no match");
	println!("Do not use reverse property if no other property matches as normal property");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0034-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0035() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0035-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0035-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0035-out.jsonld");
	println!("Compact @reverse node references using strings");
	println!("Compact node references to strings for reverse properties using @type: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0035-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0036() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0036-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0036-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0036-out.jsonld");
	println!("Compact reverse properties using index containers");
	println!("Compact using both reverse properties and index containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0036-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0037() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0037-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0037-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0037-out.jsonld");
	println!("Compact keys in @reverse using @vocab");
	println!("Compact keys in @reverse using @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0037-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0038() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0038-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0038-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0038a-out.jsonld");
	println!("Index map round-tripping");
	println!("Complex round-tripping use case from Drupal");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0038-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0039() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0039-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0039-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0039-out.jsonld");
	println!("@graph is array");
	println!("Value of @graph is always an array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0039-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0040() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0040-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0040-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0040-out.jsonld");
	println!("@list is array");
	println!("Ensure that value of @list is always an array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0040-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0041() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0041-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0041-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0041-out.jsonld");
	println!("index rejects term having @list");
	println!("If an index is present, a term having an @list container is not selected");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0041-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0042() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0042-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0042-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0042-out.jsonld");
	println!("@list keyword aliasing");
	println!("Make sure keyword aliasing works if a list can't be compacted");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0042-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0043() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0043-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0043-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0043-out.jsonld");
	println!("select term over @vocab");
	println!("Ensure that @vocab compaction isn't used if the result collides with a term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0043-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0044() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0044-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0044-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0044-out.jsonld");
	println!("@type: @vocab in reverse-map");
	println!("Prefer properties with @type: @vocab in reverse-maps if the value can be compacted to a term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0044-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0045() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0045-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0045-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0045-out.jsonld");
	println!("@id value uses relative IRI, not term");
	println!("Values of @id are transformed to relative IRIs, terms are ignored");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0045-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0046() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0046-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0046-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0046-out.jsonld");
	println!("multiple objects without @context use @graph");
	println!("Wrap top-level array into @graph even if no context is passed");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0046-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0047() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0047-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0047-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0047-out.jsonld");
	println!("Round-trip relative URLs");
	println!("Relative URLs remain relative after compaction");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0047-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0048() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0048-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0048-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0048-out.jsonld");
	println!("term with @language: null");
	println!("Prefer terms with a language mapping set to null over terms without language-mapping for non-strings");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0048-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0049() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0049-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0049-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0049-out.jsonld");
	println!("Round tripping of lists that contain just IRIs");
	println!("List compaction without @container: @list still uses strings if @type: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0049-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0050() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0050-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0050-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0050-out.jsonld");
	println!("Reverse properties require @type: @id to use string values");
	println!("Node references in reverse properties are not compacted to strings without explicit type-coercion");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0050-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0051() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0051-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0051-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0051-out.jsonld");
	println!("Round tripping @list with scalar");
	println!("Native values survive round-tripping with @list");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0051-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0052() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0052-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0052-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0052-out.jsonld");
	println!("Round tripping @list with scalar and @graph alias");
	println!("Native values survive round-tripping with @list and @graph alias");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0052-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0053() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0053-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0053-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0053-out.jsonld");
	println!("Use @type: @vocab if no @type: @id");
	println!("Compact to @type: @vocab when no @type: @id term available");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0053-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0054() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0054-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0054-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0054-out.jsonld");
	println!("Compact to @type: @vocab and compact @id to term");
	println!("Compact to @type: @vocab and compact @id to term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0054-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0055() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0055-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0055-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0055-out.jsonld");
	println!("Round tripping @type: @vocab");
	println!("Compacting IRI value of property with @type: @vocab can use term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0055-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0056() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0056-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0056-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0056-out.jsonld");
	println!("Prefer @type: @vocab over @type: @id for terms");
	println!("Compacting IRI value of property with @type: @vocab can use term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0056-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0057() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0057-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0057-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0057-out.jsonld");
	println!("Complex round tripping @type: @vocab and @type: @id");
	println!("Compacting IRI value of property with @type: @vocab can use term; more complex");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0057-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0058() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0058-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0058-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0058-out.jsonld");
	println!("Prefer @type: @id over @type: @vocab for non-terms");
	println!("Choose a term having @type: @id over @type: @value if value is not a term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0058-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0059() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0059-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0059-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0059-out.jsonld");
	println!("Term with @type: @vocab if no @type: @id");
	println!("If there's no term with @type: @id, use terms with @type: @vocab for IRIs not mapped to terms");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0059-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0060() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0060-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0060-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0060-out.jsonld");
	println!("Term with @type: @id if no @type: @vocab and term value");
	println!(
		"If there's no term with @type: @vocab, use terms with @type: @id for IRIs mapped to terms"
	);
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0060-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0061() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0061-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0061-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0061-out.jsonld");
	println!("@type: @vocab/@id with values matching either");
	println!(
		"Separate IRIs for the same property to use term with more specific @type (@id vs. @vocab)"
	);
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0061-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0062() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0062-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0062-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0062-out.jsonld");
	println!("@type: @vocab and relative IRIs");
	println!("Relative IRIs don't round-trip with @type: @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0062-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0063() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0063-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0063-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0063-out.jsonld");
	println!("Compact IRI round-tripping with @type: @vocab");
	println!("Term with @type: @vocab will use compact IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0063-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0064() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0064-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0064-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0064-out.jsonld");
	println!("Compact language-tagged and indexed strings to index-map");
	println!("Given values with both @index and @language and term index-map term, use index map");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0064-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0065() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0065-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0065-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0065-out.jsonld");
	println!("Language-tagged and indexed strings with language-map");
	println!("Language-tagged and indexed strings don't compact to language-map");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0065-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0066() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0066-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0066-out.jsonld");
	println!("Relative IRIs");
	println!("Complex use cases for relative IRI compaction");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0066-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0067() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0067-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0067-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0067-out.jsonld");
	println!("Reverse properties with blank nodes");
	println!("Compact reverse property whose values are unlabeled blank nodes");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0067-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0068() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0068-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0068-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0068-out.jsonld");
	println!("Single value reverse properties");
	println!("Single values of reverse properties are compacted as values of ordinary properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0068-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0069() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0069-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0069-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0069-out.jsonld");
	println!("Single value reverse properties with @set");
	println!(
		"Single values are kept in array form for reverse properties if the container is to @set"
	);
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0069-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0070() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0070-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0070-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0070-out.jsonld");
	println!("compactArrays option");
	println!("Setting compactArrays to false causes single element arrays to be retained");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: false,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0070-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0071() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0071-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0071-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0071-out.jsonld");
	println!("Input has multiple @contexts, output has one");
	println!("Expanding input with multiple @contexts and compacting with just one doesn't output undefined properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0071-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0072() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0072-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0072-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0072-out.jsonld");
	println!("Default language and unmapped properties");
	println!("Ensure that the default language is handled correctly for unmapped properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0072-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0073() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0073-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0073-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0073-out.jsonld");
	println!("Mapped @id and @type");
	println!("Ensure that compaction works with mapped @id and @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0073-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0074() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0074-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0074-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0074-out.jsonld");
	println!("Container as a list with type of @id");
	println!("Ensure that compaction works for empty list when property has container declared as @list and type as @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0074-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0075() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0075-in.jsonld");
	let base_url = iri!("http://example.org/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0075-out.jsonld");
	println!("Compact using relative fragment identifier");
	println!("Compacting a relative round-trips");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0075-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0076() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0076-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0076-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0076-out.jsonld");
	println!("Compacting IRI equivalent to base");
	println!("Compacting IRI equivalent to base, uses last path segment of base ending in '/'");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0076-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0077() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0077-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0077-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0077-out.jsonld");
	println!("Compact a @graph container");
	println!("Compact a @graph container");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0077-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0078() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0078-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0078-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0078-out.jsonld");
	println!("Compact a [@graph, @set] container");
	println!("Compact with [@graph, @set]");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0078-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0079() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0079-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0079-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0079-out.jsonld");
	println!("Compact a @graph container having @index");
	println!("Verify that having both @graph and @index allows @graph container compaction");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0079-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0080() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0080-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0080-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0080-out.jsonld");
	println!("Do not compact a graph having @id with a term having an @graph container");
	println!("Graph compaction works only on simple graphs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0080-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0081() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0081-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0081-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0081-out.jsonld");
	println!("Compact a [@graph, @index] container");
	println!("Compact a @graph container with @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0081-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0082() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0082-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0082-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0082-out.jsonld");
	println!("Compact a [@graph, @index, @set] container");
	println!("Compact a @graph container with @index and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0082-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0083() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0083-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0083-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0083-out.jsonld");
	println!("[@graph, @index] does not compact graph with @id");
	println!("Graph compaction with @graph and @index works only on simple graphs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0083-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0084() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0084-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0084-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0084-out.jsonld");
	println!("Compact a simple graph with a [@graph, @id] container");
	println!("Compact a simple graph using a @graph container with @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0084-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0085() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0085-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0085-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0085-out.jsonld");
	println!("Compact a named graph with a [@graph, @id] container");
	println!("Compact a named graph using a @graph container with @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0085-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0086() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0086-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0086-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0086-out.jsonld");
	println!("Compact a simple graph with a [@graph, @id, @set] container");
	println!("Compact a simple graph using a @graph container with @id and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0086-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0087() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0087-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0087-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0087-out.jsonld");
	println!("Compact a named graph with a [@graph, @id, @set] container");
	println!("Compact a named graph using a @graph container with @id and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0087-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0088() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0088-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0088-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0088-out.jsonld");
	println!("Compact a graph with @index using a [@graph, @id] container");
	println!("Compact a @graph container with @id and @set, discarding an @index value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0088-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0089() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0089-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0089-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0089-out.jsonld");
	println!("Language map term selection with complications");
	println!("Test appropriate property use given language maps with @vocab, a default language, no language, and competing terms");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0089-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0090() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0090-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0090-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0090-out.jsonld");
	println!("Compact input with @graph container to output without @graph container");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0090-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0091() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0091-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0091-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0091-out.jsonld");
	println!("Compact input with @graph container to output without @graph container with compactArrays unset");
	println!("Ensure @graph appears properly in output with compactArrays unset");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: false,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0091-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0092() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0092-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0092-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0092-out.jsonld");
	println!(
		"Compact input with [@graph, @set] container to output without [@graph, @set] container"
	);
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0092-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0093() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0093-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0093-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0093-out.jsonld");
	println!("Compact input with [@graph, @set] container to output without [@graph, @set] container with compactArrays unset");
	println!("Ensure @graph appears properly in output with compactArrays unset");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: false,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0093-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0094() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0094-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0094-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0094-out.jsonld");
	println!(
		"Compact input with [@graph, @set] container to output without [@graph, @set] container"
	);
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0094-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0095() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0095-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0095-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0095-out.jsonld");
	println!("Relative propererty IRIs with @vocab: ''");
	println!("Complex use cases for relative IRI compaction or properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0095-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0096() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0096-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0096-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0096-out.jsonld");
	println!("Compact @graph container (multiple graphs)");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0096-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0097() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0097-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0097-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0097-out.jsonld");
	println!("Compact [@graph, @set] container (multiple graphs)");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0097-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0098() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0098-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0098-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0098-out.jsonld");
	println!("Compact [@graph, @index] container (multiple indexed objects)");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0098-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0099() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0099-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0099-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0099-out.jsonld");
	println!("Compact [@graph, @index, @set] container (multiple indexed objects)");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0099-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0100() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0100-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0100-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0100-out.jsonld");
	println!("Compact [@graph, @id] container (multiple indexed objects)");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0100-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0101() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0101-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0101-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0101-out.jsonld");
	println!("Compact [@graph, @id, @set] container (multiple indexed objects)");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0101-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0102() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0102-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0102-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0102-out.jsonld");
	println!("Compact [@graph, @index] container (multiple indexes and objects)");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0102-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0103() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0103-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0103-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0103-out.jsonld");
	println!("Compact [@graph, @id] container (multiple ids and objects)");
	println!("Ensure @graph appears properly in output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0103-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0104() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0104-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0104-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0104-out.jsonld");
	println!("Compact @type with @container: @set");
	println!("Ensures that a single @type value is represented as an array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0104-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0105() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0105-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0105-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0105-out.jsonld");
	println!("Compact @type with @container: @set using an alias of @type");
	println!("Ensures that a single @type value is represented as an array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0105-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0106() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0106-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0106-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0106-out.jsonld");
	println!("Do not compact @type with @container: @set to an array using an alias of @type");
	println!("Ensures that a single @type value is not represented as an array in 1.0");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0106-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0107() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0107-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0107-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0107-out.jsonld");
	println!("Relative propererty IRIs with @vocab: ''");
	println!("Complex use cases for relative IRI compaction or properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0107-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0108() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0108-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0108-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0108-out.jsonld");
	println!("context with JavaScript Object property names");
	println!("Compact with context including JavaScript Object property names");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0108-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0109() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0109-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0109-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0109-out.jsonld");
	println!("Compact @graph container (multiple objects)");
	println!("Multiple objects in a simple graph with a graph container need to use @included");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0109-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_0110() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0110-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0110-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/0110-out.jsonld");
	println!("Compact [@graph, @set] container (multiple objects)");
	println!("Multiple objects in a simple graph with a graph container need to use @included");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/0110-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c001-out.jsonld");
	println!("adding new term");
	println!("Compaction using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c001-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c002-out.jsonld");
	println!("overriding a term");
	println!("Compaction using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c002-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c003-out.jsonld");
	println!("property and value with different terms mapping to the same expanded property");
	println!("Compaction using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c003-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c004-out.jsonld");
	println!("deep @context affects nested nodes");
	println!("Compaction using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c004-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c005-out.jsonld");
	println!("scoped context layers on intemediate contexts");
	println!("Compaction using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c005-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c006-out.jsonld");
	println!("adding new term");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c006-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c007-out.jsonld");
	println!("overriding a term");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c007-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c008-out.jsonld");
	println!("alias of @type");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c008-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c009-out.jsonld");
	println!("deep @type-scoped @context does NOT affect nested nodes");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c009-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c010-out.jsonld");
	println!("scoped context layers on intemediate contexts");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c010-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c011-out.jsonld");
	println!("applies context for all values");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c011-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c012-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c012-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c012-out.jsonld");
	println!("orders @type terms when applying scoped contexts");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c012-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c013-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c013-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c013-out.jsonld");
	println!("deep property-term scoped @context in @type-scoped @context affects nested nodes");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c013-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c014-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c014-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c014-out.jsonld");
	println!("type-scoped context nullification");
	println!("Nullifying a type-scoped context continues to use the previous context when compacting @type.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c014-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c015-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c015-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c015-out.jsonld");
	println!("type-scoped base");
	println!("type-scoped base");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c015-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c016-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c016-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c016-out.jsonld");
	println!("type-scoped vocab");
	println!("type-scoped vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c016-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c017-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c017-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c017-out.jsonld");
	println!("multiple type-scoped contexts are properly reverted");
	println!("multiple type-scoped contexts are property reverted");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c017-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c018-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c018-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c018-out.jsonld");
	println!("multiple type-scoped types resolved against previous context");
	println!("multiple type-scoped types resolved against previous context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c018-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c019-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c019-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c019-out.jsonld");
	println!("type-scoped context with multiple property scoped terms");
	println!("type-scoped context with multiple property scoped terms");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c019-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c020-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c020-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c020-out.jsonld");
	println!("type-scoped value");
	println!("type-scoped value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c020-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c021() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c021-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c021-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c021-out.jsonld");
	println!("type-scoped value mix");
	println!("type-scoped value mix");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c021-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c022-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c022-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c022-out.jsonld");
	println!("type-scoped property-scoped contexts including @type:@vocab");
	println!("type-scoped property-scoped contexts including @type:@vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c022-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c023() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c023-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c023-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c023-out.jsonld");
	println!("composed type-scoped property-scoped contexts including @type:@vocab");
	println!("composed type-scoped property-scoped contexts including @type:@vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c023-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c024() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c024-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c024-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c024-out.jsonld");
	println!("type-scoped + property-scoped + values evaluates against previous context");
	println!("type-scoped + property-scoped + values evaluates against previous context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c024-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c025() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c025-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c025-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c025-out.jsonld");
	println!("type-scoped + graph container");
	println!("type-scoped + graph container");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c025-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c026() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c026-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c026-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c026-out.jsonld");
	println!("@propagate: true on type-scoped context");
	println!("type-scoped context with @propagate: true survive node-objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c026-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c027() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c027-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c027-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c027-out.jsonld");
	println!("@propagate: false on property-scoped context");
	println!("property-scoped context with @propagate: false do not survive node-objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c027-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_c028() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c028-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c028-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/c028-out.jsonld");
	println!("Empty-property scoped context does not affect term selection.");
	println!("Adding a minimal/empty property-scoped context should not affect the using terms defined in outer context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/c028-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_di01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di01-out.jsonld");
	println!("term direction null");
	println!("Uses term with null direction when two terms conflict on direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/di01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_di02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di02-out.jsonld");
	println!("use alias of @direction");
	println!("Use alias of @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/di02-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_di03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di03-out.jsonld");
	println!("term selection with lists and direction");
	println!("Term selection includes values of @list.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/di03-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_di04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di04-out.jsonld");
	println!("simple language map with term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/di04-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_di05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di05-out.jsonld");
	println!("simple language map with overriding term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/di05-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_di06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di06-out.jsonld");
	println!("simple language map with overriding null direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/di06-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_di07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di07-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/di07-out.jsonld");
	println!("simple language map with mismatching term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/di07-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_e002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/e002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/e002-in.jsonld");
	println!("Absolute IRI confused with Compact IRI");
	println!("Verifies that IRI compaction detects when the result is an absolute IRI with a scheme matching a term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/e002-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::IriConfusedWithPrefix,
	)
	.await
}

#[async_std::test]
async fn compact_en01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/en01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/en01-in.jsonld");
	println!("Nest term not defined");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/en01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidNestValue,
	)
	.await
}

#[async_std::test]
async fn compact_ep05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep05-in.jsonld");
	println!("processingMode json-ld-1.0 conflicts with @version: 1.1");
	println!("If processingMode is explicitly json-ld-1.0, it will conflict with 1.1 features.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep05-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::ProcessingModeConflict,
	)
	.await
}

#[async_std::test]
async fn compact_ep06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep06-in.jsonld");
	println!("@version must be 1.1");
	println!("If @version is specified, it must be 1.1");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep06-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidVersionValue,
	)
	.await
}

#[async_std::test]
async fn compact_ep07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep07-in.jsonld");
	println!("@prefix is not allowed in 1.0");
	println!("@prefix is not allowed in a term definition 1.0");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep07-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition,
	)
	.await
}

#[async_std::test]
async fn compact_ep08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep08-in.jsonld");
	println!("@prefix must be a boolean");
	println!("@prefix must be a boolean in a term definition in 1.1");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep08-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidPrefixValue,
	)
	.await
}

#[async_std::test]
async fn compact_ep09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep09-in.jsonld");
	println!("@prefix not allowed on compact IRI term");
	println!("If processingMode is json-ld-1.0, or if term contains a colon (:), an invalid term definition has been detected and processing is aborted.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep09-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition,
	)
	.await
}

#[async_std::test]
async fn compact_ep10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep10-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep10-in.jsonld");
	println!("@nest is not allowed in 1.0");
	println!("@nest is not allowed in a term definitionin 1.0");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep10-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition,
	)
	.await
}

#[async_std::test]
async fn compact_ep11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep11-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep11-in.jsonld");
	println!("@context is not allowed in 1.0");
	println!("@context is not allowed in a term definitionin 1.0");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep11-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidTermDefinition,
	)
	.await
}

#[async_std::test]
async fn compact_ep12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep12-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep12-in.jsonld");
	println!("@container may not be an array in 1.0");
	println!("validate appropriate values of @container");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep12-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidContainerMapping,
	)
	.await
}

#[async_std::test]
async fn compact_ep13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep13-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep13-in.jsonld");
	println!("@container may not be @id in 1.0");
	println!("validate appropriate values of @container");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep13-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidContainerMapping,
	)
	.await
}

#[async_std::test]
async fn compact_ep14() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep14-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep14-in.jsonld");
	println!("@container may not be @type in 1.0");
	println!("validate appropriate values of @container");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep14-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidContainerMapping,
	)
	.await
}

#[async_std::test]
async fn compact_ep15() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep15-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/ep15-in.jsonld");
	println!("@container may not be @graph in 1.0");
	println!("validate appropriate values of @container");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/ep15-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidContainerMapping,
	)
	.await
}

#[async_std::test]
async fn compact_in01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in01-out.jsonld");
	println!("Basic Included array");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/in01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_in02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in02-out.jsonld");
	println!("Basic Included object");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/in02-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_in03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in03-out.jsonld");
	println!("Multiple properties mapping to @included are folded together");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/in03-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_in04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in04-out.jsonld");
	println!("Included containing @included");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/in04-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_in05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/in05-out.jsonld");
	println!("Property value with @included");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/in05-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js01-out.jsonld");
	println!("Compact JSON literal (boolean true)");
	println!("Tests compacting property with @type @json to a JSON literal (boolean true).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js02-out.jsonld");
	println!("Compact JSON literal (boolean false)");
	println!("Tests compacting property with @type @json to a JSON literal (boolean false).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js02-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js03-out.jsonld");
	println!("Compact JSON literal (double)");
	println!("Tests compacting property with @type @json to a JSON literal (double).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js03-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js04-out.jsonld");
	println!("Compact JSON literal (double-zero)");
	println!("Tests compacting property with @type @json to a JSON literal (double-zero).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js04-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js05-out.jsonld");
	println!("Compact JSON literal (integer)");
	println!("Tests compacting property with @type @json to a JSON literal (integer).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js05-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js06-out.jsonld");
	println!("Compact JSON literal (object)");
	println!("Tests compacting property with @type @json to a JSON literal (object).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js06-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js07-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js07-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js07-out.jsonld");
	println!("Compact JSON literal (array)");
	println!("Tests compacting property with @type @json to a JSON literal (array).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js07-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js08-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js08-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js08-out.jsonld");
	println!("Compact already expanded JSON literal");
	println!("Tests compacting JSON literal does not expand terms inside json.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js08-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js09-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js09-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js09-out.jsonld");
	println!("Compact already expanded JSON literal with aliased keys");
	println!("Tests compacting JSON literal in expanded form.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js09-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js10-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js10-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js10-out.jsonld");
	println!("Compact JSON literal (string)");
	println!("Tests compacting property with @type @json to a JSON literal (string).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js10-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_js11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js11-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js11-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/js11-out.jsonld");
	println!("Compact JSON literal (null)");
	println!("Tests compacting property with @type @json to a JSON literal (null).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/js11-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_la01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/la01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/la01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/la01-out.jsonld");
	println!("most specific term matching in @list.");
	println!("The most specific term that matches all of the elements in the list, taking into account the default language, must be selected, without considering case of language.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/la01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_li01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li01-out.jsonld");
	println!("coerced @list containing an empty list");
	println!("Lists of Lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/li01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_li02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li02-out.jsonld");
	println!("coerced @list containing a list");
	println!("Lists of Lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/li02-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_li03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li03-out.jsonld");
	println!("coerced @list containing an deep list");
	println!("Lists of Lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/li03-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_li04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li04-out.jsonld");
	println!("coerced @list containing multiple lists");
	println!("Lists of Lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/li04-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_li05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/li05-out.jsonld");
	println!("coerced @list containing mixed list values");
	println!("Lists of Lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/li05-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m001-out.jsonld");
	println!("Indexes to object not having an @id");
	println!("Compaction using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m001-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m002-out.jsonld");
	println!("Indexes to object already having an @id");
	println!("Compaction using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m002-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m003-out.jsonld");
	println!("Indexes to object not having an @type");
	println!("Compaction using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m003-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m004-out.jsonld");
	println!("Indexes to object already having an @type");
	println!("Compaction using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m004-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m005-out.jsonld");
	println!("Indexes to object using compact IRI @id");
	println!("Compaction using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m005-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m006-out.jsonld");
	println!("Indexes using compacted @type");
	println!("Compaction using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m006-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m007-out.jsonld");
	println!("When type is in a type map");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m007-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m008-out.jsonld");
	println!("@index map with @none node definition");
	println!("index on @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m008-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m009-out.jsonld");
	println!("@index map with @none value");
	println!("index on @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m009-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m010-out.jsonld");
	println!("@index map with @none value using alias of @none");
	println!("index on @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m010-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m011-out.jsonld");
	println!("@language map with no @language");
	println!("index on @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m011-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m012-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m012-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m012-out.jsonld");
	println!("language map with no @language using alias of @none");
	println!("index on @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m012-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m013-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m013-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m013-out.jsonld");
	println!("id map using @none");
	println!("index on @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m013-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m014-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m014-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m014-out.jsonld");
	println!("id map using @none with alias");
	println!("index on @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m014-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m015-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m015-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m015-out.jsonld");
	println!("type map using @none with alias");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m015-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m016-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m016-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m016-out.jsonld");
	println!("type map using @none with alias");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m016-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m017-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m017-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m017-out.jsonld");
	println!("graph index map using @none");
	println!("index on @graph and @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m017-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m018-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m018-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m018-out.jsonld");
	println!("graph id map using @none");
	println!("index on @graph and @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m018-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m019-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m019-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m019-out.jsonld");
	println!("graph id map using alias of @none");
	println!("index on @graph and @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m019-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m020-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m020-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m020-out.jsonld");
	println!("node reference compacts to string value of type map");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m020-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m021() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m021-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m021-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m021-out.jsonld");
	println!("node reference compacts to string value of type map with @type: @id");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m021-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_m022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m022-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m022-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/m022-out.jsonld");
	println!("node reference compacts to string value of type map with @type: @vocab");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/m022-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n001-out.jsonld");
	println!("Indexes to @nest for property with @nest");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n001-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n002-out.jsonld");
	println!("Indexes to @nest for all properties with @nest");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n002-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n003-out.jsonld");
	println!("Nests using alias of @nest");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n003-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n004-out.jsonld");
	println!("Arrays of nested values");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n004-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n005-out.jsonld");
	println!("Nested @container: @list");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n005-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n006-out.jsonld");
	println!("Nested @container: @index");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n006-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n007-out.jsonld");
	println!("Nested @container: @language");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n007-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n008-out.jsonld");
	println!("Nested @container: @type");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n008-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n009-out.jsonld");
	println!("Nested @container: @id");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n009-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n010-out.jsonld");
	println!("Multiple nest aliases");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n010-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_n011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/n011-out.jsonld");
	println!("Nests using alias of @nest (defined with @id)");
	println!("Compaction using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/n011-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_p001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p001-out.jsonld");
	println!("Compact IRI will not use an expanded term definition in 1.0");
	println!("Terms with an expanded term definition are not used for creating compact IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_0,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/p001-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_p002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p002-out.jsonld");
	println!("Compact IRI does not use expanded term definition in 1.1");
	println!("Terms with an expanded term definition are not used for creating compact IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/p002-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_p003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p003-out.jsonld");
	println!("Compact IRI does not use simple term that does not end with a gen-delim");
	println!("Terms not ending with a gen-delim are not used for creating compact IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/p003-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_p005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p005-out.jsonld");
	println!("Compact IRI uses term with definition including @prefix: true");
	println!("Expanded term definition may set prefix explicitly in 1.1");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/p005-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_p006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p006-out.jsonld");
	println!("Compact IRI uses term with definition including @prefix: true");
	println!("Expanded term definition may set prefix explicitly in 1.1");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/p006-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_p007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p007-out.jsonld");
	println!("Compact IRI not used as prefix");
	println!("Terms including a colon are excluded from being used as a prefix");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/p007-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_p008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/p008-out.jsonld");
	println!("Compact IRI does not use term with definition including @prefix: false");
	println!("Expanded term definition may set prefix explicitly in 1.1");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/p008-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_pi01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi01-out.jsonld");
	println!("property-valued index indexes property value, instead of property (value)");
	println!("Compacting property-valued indexes.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pi01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_pi02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi02-out.jsonld");
	println!("property-valued index indexes property value, instead of property (multiple values)");
	println!("Compacting property-valued indexes.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pi02-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_pi03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi03-out.jsonld");
	println!("property-valued index indexes property value, instead of property (node)");
	println!("Compacting property-valued indexes.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pi03-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_pi04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi04-out.jsonld");
	println!("property-valued index indexes property value, instead of property (multiple nodes)");
	println!("Compacting property-valued indexes.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pi04-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_pi05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi05-out.jsonld");
	println!("property-valued index indexes using @none if no property value exists");
	println!("Compacting property-valued indexes.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pi05-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_pi06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pi06-out.jsonld");
	println!(
		"property-valued index indexes using @none if no property value does not compact to string"
	);
	println!("Compacting property-valued indexes.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pi06-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_pr01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr01-in.jsonld");
	println!("Check illegal clearing of context with protected terms");
	println!("Check error when clearing a context with protected terms.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pr01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::InvalidContextNullification,
	)
	.await
}

#[async_std::test]
async fn compact_pr02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr02-in.jsonld");
	println!("Check illegal overriding of protected term");
	println!("Check error when overriding a protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pr02-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition,
	)
	.await
}

#[async_std::test]
async fn compact_pr03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr03-in.jsonld");
	println!("Check illegal overriding of protected term from type-scoped context");
	println!("Check error when overriding a protected term from type-scoped context.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pr03-context.jsonld"
			)),
		},
		input_url,
		base_url,
		ErrorCode::ProtectedTermRedefinition,
	)
	.await
}

#[async_std::test]
async fn compact_pr04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr04-out.jsonld");
	println!("Check legal overriding of protected term from property-scoped context");
	println!("Check overriding a protected term from property-scoped context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pr04-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_pr05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/pr05-out.jsonld");
	println!("Check legal overriding of type-scoped protected term from nested node");
	println!("Check legal overriding of type-scoped protected term from nested node.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/pr05-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_r001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/r001-in.jsonld");
	let base_url = iri!("http://example.org/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/r001-out.jsonld");
	println!("Expands and compacts to document base by default");
	println!("Compact IRI attempts to compact document-relative IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/r001-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_r002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/r002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/r002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/r002-out.jsonld");
	println!("Expands and does not compact to document base with compactToRelative false");
	println!("With compactToRelative option set to false, IRIs which could be made relative to the document base are not made relative.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/r002-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_s001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/s001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/s001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/s001-out.jsonld");
	println!("@context with single array values");
	println!("@context values may be in an array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/s001-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_s002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/s002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/s002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/s002-out.jsonld");
	println!("@context with array including @set uses array values");
	println!("@context values may include @set along with another compatible value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/s002-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_tn01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn01-out.jsonld");
	println!("@type: @none does not compact values");
	println!("@type: @none does not compact values.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/tn01-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_tn02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn02-out.jsonld");
	println!("@type: @none does not use arrays by default");
	println!("@type: @none honors @container.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/tn02-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}

#[async_std::test]
async fn compact_tn03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/compact/tn03-out.jsonld");
	println!("@type: @none uses arrays with @container: @set");
	println!("@type: @none honors @container.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			compact_arrays: true,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/compact/tn03-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
	.await
}
