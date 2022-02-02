use static_iref::iri;
use async_std::task;
use iref::{Iri, IriBuf};
use json_ld::{
	compaction,
	context::{self, Loader as ContextLoader, Local, ProcessingOptions},
	expansion,
	util::{json_ld_eq, AsJson},
	Document, ErrorCode, FsLoader, Loader, ProcessingMode,
};
use serde_json::Value;

#[derive(Clone, Copy)]
struct Options<'a> {
	processing_mode: ProcessingMode,
	ordered: bool,
	compact_arrays: bool,
	context: Option<Iri<'a>>,
}

impl<'a> From<Options<'a>> for expansion::Options {
	fn from(options: Options<'a>) -> expansion::Options {
		expansion::Options {
			processing_mode: options.processing_mode,
			ordered: options.ordered,
			..expansion::Options::default()
		}
	}
}

impl<'a> From<Options<'a>> for compaction::Options {
	fn from(options: Options<'a>) -> compaction::Options {
		compaction::Options {
			processing_mode: options.processing_mode,
			compact_arrays: options.compact_arrays,
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

fn no_metadata<M>(_: Option<&M>) -> () {
	()
}

fn positive_test(options: Options, input_url: Iri, base_url: Iri, output_url: Iri) {
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let expected_output = task::block_on(loader.load(output_url)).unwrap();

	let expand_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));
	let compact_context: Option<context::ProcessedOwned<Value, context::Json<Value, IriBuf>>> =
		options.context.map(|context_url| {
			let local_context = task::block_on(loader.load_context(context_url))
				.unwrap()
				.into_context();
			task::block_on(local_context.process_with(
				&context::Json::new(Some(base_url)),
				&mut loader,
				Some(base_url),
				options.into(),
			))
			.unwrap()
			.owned()
		});

	let mut id_generator = json_ld::id::generator::Blank::new_with_prefix("b".to_string());
	let output = task::block_on(input.flatten_with(
		&mut id_generator,
		Some(base_url),
		&expand_context,
		&mut loader,
		options.into(),
	))
	.unwrap();

	let json_output: Value = match compact_context {
		Some(compact_context) => task::block_on(output.compact(
			&compact_context.inversible(),
			&mut loader,
			options.into(),
			no_metadata,
		))
		.unwrap(),
		None => output.as_json(),
	};

	let success = json_ld_eq(&json_output, &*expected_output);

	if !success {
		// let expected_ld_output = task::block_on(expected_output.expand_with(
		// 	Some(base_url),
		// 	&expand_context,
		// 	&mut loader,
		// 	options.into(),
		// ))
		// .unwrap();

		println!(
			"output=\n{}",
			serde_json::to_string_pretty(&json_output).unwrap()
		);
		println!(
			"\nexpected=\n{}",
			serde_json::to_string_pretty(&*expected_output).unwrap()
		);
	}

	assert!(success)
}

fn negative_test(options: Options, input_url: Iri, base_url: Iri, error_code: ErrorCode) {
	let mut loader = FsLoader::<Value>::new(|s| serde_json::from_str(s));
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let input = task::block_on(loader.load(input_url)).unwrap();
	let mut input_context: context::Json<Value, IriBuf> = context::Json::new(Some(base_url));

	if let Some(context_url) = options.context {
		let local_context = task::block_on(loader.load_context(context_url))
			.unwrap()
			.into_context();
		input_context = task::block_on(local_context.process_with(
			&input_context,
			&mut loader,
			Some(base_url),
			options.into(),
		))
		.unwrap()
		.into_inner();
	}

	let mut id_generator = json_ld::id::generator::Blank::new();
	let result = task::block_on(input.flatten_with(
		&mut id_generator,
		Some(base_url),
		&input_context,
		&mut loader,
		options.into(),
	));

	match result {
		Ok(output) => {
			let output_json: Value = output.as_json();
			println!(
				"output=\n{}",
				serde_json::to_string_pretty(&output_json).unwrap()
			);
			panic!(
				"expansion succeeded where it should have failed with code: {}",
				error_code
			)
		}
		Err(e) => {
			assert_eq!(e.code(), error_code)
		}
	}
}
#[test]
fn flatten_0001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0001-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0001-out.jsonld");
	println!("drop free-floating nodes");
	println!("Flattening drops unreferenced nodes having only @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0002-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0002-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0002-out.jsonld");
	println!("basic");
	println!("Flattening terms with different types of values");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0003-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0003-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0003-out.jsonld");
	println!("drop null and unmapped properties");
	println!("Verifies that null values and unmapped properties are removed from expanded output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0004-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0004-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0004-out.jsonld");
	println!("optimize @set, keep empty arrays");
	println!("Uses of @set are removed in expansion; values of @set, or just plain values which are empty arrays are retained");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0005-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0005-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0005-out.jsonld");
	println!("do not expand aliased @id/@type");
	println!("If a keyword is aliased, it is not used when flattening");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0006-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0006-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0006-out.jsonld");
	println!("alias keywords");
	println!("Aliased keywords expand in resulting document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0007-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0007-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0007-out.jsonld");
	println!("date type-coercion");
	println!("Expand strings to expanded value with @type: xsd:dateTime");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0008-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0008-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0008-out.jsonld");
	println!("@value with @language");
	println!("Keep expanded values with @language, drop non-conforming value objects containing just @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0009-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0009-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0009-out.jsonld");
	println!("@graph with terms");
	println!("Use of @graph to contain multiple nodes within array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0010-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0010-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0010-out.jsonld");
	println!("native types");
	println!("Flattening native scalar retains native scalar within expanded value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0011-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0011-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0011-out.jsonld");
	println!("coerced @id");
	println!("A value of a property with @type: @id coercion expands to a node reference");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0012-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0012-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0012-out.jsonld");
	println!("@graph with embed");
	println!("Flattening objects containing chained objects flattens all objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0013-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0013-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0013-out.jsonld");
	println!("flatten already expanded");
	println!("Flattening an expanded/flattened document maintains input document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0015-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0015-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0015-out.jsonld");
	println!("collapse set of sets, keep empty lists");
	println!("An array of multiple @set nodes are collapsed into a single array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0016-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0016-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0016-out.jsonld");
	println!("context reset");
	println!(
		"Setting @context to null within an embedded object resets back to initial context state"
	);
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0017-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0017-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0017-out.jsonld");
	println!("@graph and @id aliased");
	println!("Flattening with @graph and @id aliases");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0018-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0018-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0018-out.jsonld");
	println!("override default @language");
	println!("override default @language in terms; only language-tag strings");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0019-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0019-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0019-out.jsonld");
	println!("remove @value = null");
	println!("Flattening a value of null removes the value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0020-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0020-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0020-out.jsonld");
	println!("do not remove @graph if not at top-level");
	println!("@graph used under a node is retained");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0021() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0021-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0021-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0021-out.jsonld");
	println!("do not remove @graph at top-level if not only property");
	println!("@graph used at the top level is retained if there are other properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0022-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0022-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0022-out.jsonld");
	println!("flatten value with default language");
	println!("Flattening with a default language applies that language to string values");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0023() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0023-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0023-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0023-out.jsonld");
	println!("Flattening list/set with coercion");
	println!("Flattening lists and sets with properties having coercion coerces list/set values");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0024() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0024-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0024-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0024-out.jsonld");
	println!("Multiple contexts");
	println!("Tests that contexts in an array are merged");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0025() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0025-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0025-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0025-out.jsonld");
	println!("Problematic IRI flattening tests");
	println!("Flattening different kinds of terms and Compact IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0027() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0027-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0027-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0027-out.jsonld");
	println!("Duplicate values in @list and @set");
	println!("Duplicate values in @list and @set are not merged");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0028() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0028-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0028-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0028-out.jsonld");
	println!("Use @vocab in properties and @type but not in @id");
	println!("@vocab is used to compact properties and @type, but is not used for @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0030() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0030-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0030-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0030-out.jsonld");
	println!("Language maps");
	println!("Language Maps expand values to include @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0031() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0031-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0031-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0031-out.jsonld");
	println!("type-coercion of native types");
	println!("Flattening native types with type coercion adds the coerced type to an expanded value representation and retains the native value representation");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0032() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0032-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0032-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0032-out.jsonld");
	println!("Null term and @vocab");
	println!("Mapping a term to null decouples it from @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0033() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0033-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0033-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0033-out.jsonld");
	println!("Using @vocab with with type-coercion");
	println!("Verifies that terms can be defined using @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0034() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0034-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0034-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0034-out.jsonld");
	println!("Multiple properties expanding to the same IRI");
	println!("Verifies multiple values from separate terms are deterministically made multiple values of the IRI associated with the terms");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0035() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0035-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0035-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0035-out.jsonld");
	println!("Language maps with @vocab, default language, and colliding property");
	println!("Pathological tests of language maps");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0036() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0036-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0036-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0036-out.jsonld");
	println!("Flattening @index");
	println!("Flattening index maps for terms defined with @container: @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0037() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0037-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0037-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0037-out.jsonld");
	println!("Flattening reverse properties");
	println!("Flattening @reverse keeps @reverse");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0039() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0039-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0039-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0039-out.jsonld");
	println!("Using terms in a reverse-maps");
	println!("Terms within @reverse are expanded");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0040() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0040-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0040-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0040-out.jsonld");
	println!("language and index expansion on non-objects");
	println!("Only invoke language and index map expansion if the value is a JSON object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0041() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0041-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0041-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0041-out.jsonld");
	println!("Free-floating sets and lists");
	println!(
		"Free-floating values in sets are removed, free-floating lists are removed completely"
	);
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0042() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0042-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0042-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0042-out.jsonld");
	println!("List objects not equivalent");
	println!("Lists objects are implicit unlabeled blank nodes and thus never equivalent");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0043() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0043-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0043-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0043-out.jsonld");
	println!("Sample test manifest extract");
	println!("Flatten a test manifest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0044() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0044-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0044-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0044-out.jsonld");
	println!("compactArrays option");
	println!("Setting compactArrays to false causes single element arrays to be retained");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: Some(iri!(
				"https://w3c.github.io/json-ld-api/tests/flatten/0044-context.jsonld"
			)),
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0045() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0045-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0045-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0045-out.jsonld");
	println!("Blank nodes with reverse properties");
	println!("Proper (re-)labeling of blank nodes if used with reverse properties.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0046() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0046-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0046-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0046-out.jsonld");
	println!("Empty string as identifier");
	println!(
		"Usage of empty strings in identifiers needs special care when constructing the node map."
	);
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0047() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0047-in.jsonld");
	let base_url = iri!("http://example.org/");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0047-out.jsonld");
	println!("Flatten using relative fragment identifier properly joins to base");
	println!("Compacting a relative round-trips");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0048() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0048-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0048-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0048-out.jsonld");
	println!("@list with embedded object");
	println!("Node definitions contained within lists are flattend to top level.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_0049() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0049-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0049-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/0049-out.jsonld");
	println!("context with JavaScript Object property names");
	println!("Flatten with context including JavaScript Object property names");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_e001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/e001-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/e001-in.jsonld");
	println!("Conflicting indexes");
	println!(
		"Verifies that an exception is raised in Flattening when conflicting indexes are found"
	);
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		ErrorCode::ConflictingIndexes,
	)
}

#[test]
fn flatten_in01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in01-out.jsonld");
	println!("Basic Included array");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_in02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in02-out.jsonld");
	println!("Basic Included object");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_in03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in03-out.jsonld");
	println!("Multiple properties mapping to @included are folded together");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_in04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in04-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in04-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in04-out.jsonld");
	println!("Included containing @included");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_in05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in05-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in05-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in05-out.jsonld");
	println!("Property value with @included");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_in06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in06-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in06-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/in06-out.jsonld");
	println!("json.api example");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_li01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li01-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li01-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li01-out.jsonld");
	println!("@list containing an deep list");
	println!("Lists of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_li02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li02-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li02-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li02-out.jsonld");
	println!("@list containing empty @list");
	println!("Lists of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}

#[test]
fn flatten_li03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li03-in.jsonld");
	let base_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li03-in.jsonld");
	let output_url = iri!("https://w3c.github.io/json-ld-api/tests/flatten/li03-out.jsonld");
	println!("@list containing mixed list values");
	println!("Lists of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			ordered: true,
			compact_arrays: false,
			context: None,
		},
		input_url,
		base_url,
		output_url,
	)
}
