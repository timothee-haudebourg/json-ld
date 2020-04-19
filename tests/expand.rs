#![feature(proc_macro_hygiene)]

extern crate tokio;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use std::fs::File;
use std::io::{Read, BufReader};
use tokio::runtime::Runtime;
use iref::{Iri, IriBuf};
use json_ld::{
	ErrorCode,
	ActiveContext,
	JsonLdContextLoader,
	Context,
	LocalContext,
	ProcessingMode,
	ExpansionOptions,
	AsJson,
	json_ld_eq
};

struct Options<'a> {
	processing_mode: ProcessingMode,
	expand_context: Option<&'a str>,
	ordered: bool
}

impl<'a> From<Options<'a>> for ExpansionOptions {
	fn from(options: Options<'a>) -> ExpansionOptions {
		ExpansionOptions {
			processing_mode: options.processing_mode,
			ordered: options.ordered
		}
	}
}

fn positive_test(options: Options, input_url: Iri, input_filename: &str, output_filename: &str) {
	let mut runtime = Runtime::new().unwrap();
	let mut loader = JsonLdContextLoader::new();

	let input_file = File::open(input_filename).unwrap();
	let mut input_buffer = BufReader::new(input_file);
	let mut input_text = String::new();
	input_buffer.read_to_string(&mut input_text).unwrap();
	let input = json::parse(input_text.as_str()).unwrap();

	let output_file = File::open(output_filename).unwrap();
	let mut output_buffer = BufReader::new(output_file);
	let mut output_text = String::new();
	output_buffer.read_to_string(&mut output_text).unwrap();
	let output = json::parse(output_text.as_str()).unwrap();

	let mut input_context: Context<IriBuf> = Context::new(input_url, input_url);

	if let Some(context_filename) = options.expand_context {
		let context_file = File::open(context_filename).unwrap();
		let mut context_buffer = BufReader::new(context_file);
		let mut context_text = String::new();
		context_buffer.read_to_string(&mut context_text).unwrap();
		let mut doc = json::parse(context_text.as_str()).unwrap();
		input_context = runtime.block_on(doc.remove("@context").process(&input_context, &mut loader, Some(input_url))).unwrap();
	}

	let result = runtime.block_on(json_ld::expand(&input_context, &input, Some(input_url), &mut loader, options.into())).unwrap();

	let result_json = result.as_json();
	let success = json_ld_eq(&result_json, &output);

	if !success {
		println!("output=\n{}", result_json.pretty(2));
		println!("\nexpected=\n{}", output.pretty(2));
	}

	assert!(success)
}

fn negative_test(options: Options, input_url: Iri, input_filename: &str, error_code: ErrorCode) {
	let mut runtime = Runtime::new().unwrap();
	let mut loader = JsonLdContextLoader::new();

	let input_file = File::open(input_filename).unwrap();
	let mut input_buffer = BufReader::new(input_file);
	let mut input_text = String::new();
	input_buffer.read_to_string(&mut input_text).unwrap();
	let input = json::parse(input_text.as_str()).unwrap();

	let mut input_context: Context<IriBuf> = Context::new(input_url, input_url);

	if let Some(context_filename) = options.expand_context {
		let context_file = File::open(context_filename).unwrap();
		let mut context_buffer = BufReader::new(context_file);
		let mut context_text = String::new();
		context_buffer.read_to_string(&mut context_text).unwrap();
		let mut doc = json::parse(context_text.as_str()).unwrap();
		input_context = runtime.block_on(doc.remove("@context").process(&input_context, &mut loader, Some(input_url))).unwrap();
	}

	match runtime.block_on(json_ld::expand(&input_context, &input, Some(input_url), &mut loader, options.into())) {
		Ok(result) => {
			println!("output=\n{}", result.as_json().pretty(2));
			panic!("expansion succeeded where it should have failed with code: {}", error_code)
		},
		Err(e) => {
			assert_eq!(e.code(), error_code)
		}
	}
}

#[test]
fn expand_0001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0001-in.jsonld");
	println!("drop free-floating nodes");
	println!("Expand drops unreferenced nodes having only @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0001-in.jsonld",
		"tests/expand/0001-out.jsonld"
	)
}

#[test]
fn expand_0002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0002-in.jsonld");
	println!("basic");
	println!("Expanding terms with different types of values");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0002-in.jsonld",
		"tests/expand/0002-out.jsonld"
	)
}

#[test]
fn expand_0003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0003-in.jsonld");
	println!("drop null and unmapped properties");
	println!("Verifies that null values and unmapped properties are removed from expanded output");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0003-in.jsonld",
		"tests/expand/0003-out.jsonld"
	)
}

#[test]
fn expand_0004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0004-in.jsonld");
	println!("optimize @set, keep empty arrays");
	println!("Uses of @set are removed in expansion; values of @set, or just plain values which are empty arrays are retained");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0004-in.jsonld",
		"tests/expand/0004-out.jsonld"
	)
}

#[test]
fn expand_0005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0005-in.jsonld");
	println!("do not expand aliased @id/@type");
	println!("If a keyword is aliased, it is not used when expanding");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0005-in.jsonld",
		"tests/expand/0005-out.jsonld"
	)
}

#[test]
fn expand_0006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0006-in.jsonld");
	println!("alias keywords");
	println!("Aliased keywords expand in resulting document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0006-in.jsonld",
		"tests/expand/0006-out.jsonld"
	)
}

#[test]
fn expand_0007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0007-in.jsonld");
	println!("date type-coercion");
	println!("Expand strings to expanded value with @type: xsd:dateTime");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0007-in.jsonld",
		"tests/expand/0007-out.jsonld"
	)
}

#[test]
fn expand_0008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0008-in.jsonld");
	println!("@value with @language");
	println!("Keep expanded values with @language, drop non-conforming value objects containing just @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0008-in.jsonld",
		"tests/expand/0008-out.jsonld"
	)
}

#[test]
fn expand_0009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0009-in.jsonld");
	println!("@graph with terms");
	println!("Use of @graph to contain multiple nodes within array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0009-in.jsonld",
		"tests/expand/0009-out.jsonld"
	)
}

#[test]
fn expand_0010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0010-in.jsonld");
	println!("native types");
	println!("Expanding native scalar retains native scalar within expanded value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0010-in.jsonld",
		"tests/expand/0010-out.jsonld"
	)
}

#[test]
fn expand_0011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0011-in.jsonld");
	println!("coerced @id");
	println!("A value of a property with @type: @id coercion expands to a node reference");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0011-in.jsonld",
		"tests/expand/0011-out.jsonld"
	)
}

#[test]
fn expand_0012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0012-in.jsonld");
	println!("@graph with embed");
	println!("Use of @graph to contain multiple nodes within array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0012-in.jsonld",
		"tests/expand/0012-out.jsonld"
	)
}

#[test]
fn expand_0013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0013-in.jsonld");
	println!("expand already expanded");
	println!("Expand does not mess up already expanded document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0013-in.jsonld",
		"tests/expand/0013-out.jsonld"
	)
}

#[test]
fn expand_0014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0014-in.jsonld");
	println!("@set of @value objects with keyword aliases");
	println!("Expanding aliased @set and @value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0014-in.jsonld",
		"tests/expand/0014-out.jsonld"
	)
}

#[test]
fn expand_0015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0015-in.jsonld");
	println!("collapse set of sets, keep empty lists");
	println!("An array of multiple @set nodes are collapsed into a single array");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0015-in.jsonld",
		"tests/expand/0015-out.jsonld"
	)
}

#[test]
fn expand_0016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0016-in.jsonld");
	println!("context reset");
	println!("Setting @context to null within an embedded object resets back to initial context state");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0016-in.jsonld",
		"tests/expand/0016-out.jsonld"
	)
}

#[test]
fn expand_0017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0017-in.jsonld");
	println!("@graph and @id aliased");
	println!("Expanding with @graph and @id aliases");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0017-in.jsonld",
		"tests/expand/0017-out.jsonld"
	)
}

#[test]
fn expand_0018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0018-in.jsonld");
	println!("override default @language");
	println!("override default @language in terms; only language-tag strings");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0018-in.jsonld",
		"tests/expand/0018-out.jsonld"
	)
}

#[test]
fn expand_0019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0019-in.jsonld");
	println!("remove @value = null");
	println!("Expanding a value of null removes the value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0019-in.jsonld",
		"tests/expand/0019-out.jsonld"
	)
}

#[test]
fn expand_0020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0020-in.jsonld");
	println!("do not remove @graph if not at top-level");
	println!("@graph used under a node is retained");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0020-in.jsonld",
		"tests/expand/0020-out.jsonld"
	)
}

#[test]
fn expand_0021() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0021-in.jsonld");
	println!("do not remove @graph at top-level if not only property");
	println!("@graph used at the top level is retained if there are other properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0021-in.jsonld",
		"tests/expand/0021-out.jsonld"
	)
}

#[test]
fn expand_0022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0022-in.jsonld");
	println!("expand value with default language");
	println!("Expanding with a default language applies that language to string values");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0022-in.jsonld",
		"tests/expand/0022-out.jsonld"
	)
}

#[test]
fn expand_0023() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0023-in.jsonld");
	println!("Expanding list/set with coercion");
	println!("Expanding lists and sets with properties having coercion coerces list/set values");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0023-in.jsonld",
		"tests/expand/0023-out.jsonld"
	)
}

#[test]
fn expand_0024() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0024-in.jsonld");
	println!("Multiple contexts");
	println!("Tests that contexts in an array are merged");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0024-in.jsonld",
		"tests/expand/0024-out.jsonld"
	)
}

#[test]
fn expand_0025() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0025-in.jsonld");
	println!("Problematic IRI expansion tests");
	println!("Expanding different kinds of terms and Compact IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0025-in.jsonld",
		"tests/expand/0025-out.jsonld"
	)
}

#[test]
fn expand_0027() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0027-in.jsonld");
	println!("Duplicate values in @list and @set");
	println!("Duplicate values in @list and @set are not merged");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0027-in.jsonld",
		"tests/expand/0027-out.jsonld"
	)
}

#[test]
fn expand_0028() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0028-in.jsonld");
	println!("Use @vocab in properties and @type but not in @id");
	println!("@vocab is used to compact properties and @type, but is not used for @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0028-in.jsonld",
		"tests/expand/0028-out.jsonld"
	)
}

#[test]
fn expand_0029() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0029-in.jsonld");
	println!("Relative IRIs");
	println!("@base is used to compact @id; test with different relative IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0029-in.jsonld",
		"tests/expand/0029-out.jsonld"
	)
}

#[test]
fn expand_0030() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0030-in.jsonld");
	println!("Language maps");
	println!("Language Maps expand values to include @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0030-in.jsonld",
		"tests/expand/0030-out.jsonld"
	)
}

#[test]
fn expand_0031() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0031-in.jsonld");
	println!("type-coercion of native types");
	println!("Expanding native types with type coercion adds the coerced type to an expanded value representation and retains the native value representation");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0031-in.jsonld",
		"tests/expand/0031-out.jsonld"
	)
}

#[test]
fn expand_0032() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0032-in.jsonld");
	println!("Null term and @vocab");
	println!("Mapping a term to null decouples it from @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0032-in.jsonld",
		"tests/expand/0032-out.jsonld"
	)
}

#[test]
fn expand_0033() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0033-in.jsonld");
	println!("Using @vocab with with type-coercion");
	println!("Verifies that terms can be defined using @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0033-in.jsonld",
		"tests/expand/0033-out.jsonld"
	)
}

#[test]
fn expand_0034() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0034-in.jsonld");
	println!("Multiple properties expanding to the same IRI");
	println!("Verifies multiple values from separate terms are deterministically made multiple values of the IRI associated with the terms");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0034-in.jsonld",
		"tests/expand/0034-out.jsonld"
	)
}

#[test]
fn expand_0035() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0035-in.jsonld");
	println!("Language maps with @vocab, default language, and colliding property");
	println!("Pathological tests of language maps");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0035-in.jsonld",
		"tests/expand/0035-out.jsonld"
	)
}

#[test]
fn expand_0036() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0036-in.jsonld");
	println!("Expanding @index");
	println!("Expanding index maps for terms defined with @container: @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0036-in.jsonld",
		"tests/expand/0036-out.jsonld"
	)
}

#[test]
fn expand_0037() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0037-in.jsonld");
	println!("Expanding @reverse");
	println!("Expanding @reverse keeps @reverse");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0037-in.jsonld",
		"tests/expand/0037-out.jsonld"
	)
}

#[test]
fn expand_0039() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0039-in.jsonld");
	println!("Using terms in a reverse-maps");
	println!("Terms within @reverse are expanded");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0039-in.jsonld",
		"tests/expand/0039-out.jsonld"
	)
}

#[test]
fn expand_0040() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0040-in.jsonld");
	println!("language and index expansion on non-objects");
	println!("Only invoke language and index map expansion if the value is a JSON object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0040-in.jsonld",
		"tests/expand/0040-out.jsonld"
	)
}

#[test]
fn expand_0041() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0041-in.jsonld");
	println!("@language: null resets the default language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0041-in.jsonld",
		"tests/expand/0041-out.jsonld"
	)
}

#[test]
fn expand_0042() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0042-in.jsonld");
	println!("Reverse properties");
	println!("Expanding terms defined as reverse properties uses @reverse in expanded document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0042-in.jsonld",
		"tests/expand/0042-out.jsonld"
	)
}

#[test]
fn expand_0043() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0043-in.jsonld");
	println!("Using reverse properties inside a @reverse-container");
	println!("Expanding a reverse property within a @reverse undoes both reversals");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0043-in.jsonld",
		"tests/expand/0043-out.jsonld"
	)
}

#[test]
fn expand_0044() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0044-in.jsonld");
	println!("Index maps with language mappings");
	println!("Ensure index maps use language mapping");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0044-in.jsonld",
		"tests/expand/0044-out.jsonld"
	)
}

#[test]
fn expand_0045() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0045-in.jsonld");
	println!("Top-level value objects");
	println!("Expanding top-level value objects causes them to be removed");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0045-in.jsonld",
		"tests/expand/0045-out.jsonld"
	)
}

#[test]
fn expand_0046() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0046-in.jsonld");
	println!("Free-floating nodes");
	println!("Expanding free-floating nodes causes them to be removed");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0046-in.jsonld",
		"tests/expand/0046-out.jsonld"
	)
}

#[test]
fn expand_0047() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0047-in.jsonld");
	println!("Free-floating values in sets and free-floating lists");
	println!("Free-floating values in sets are removed, free-floating lists are removed completely");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0047-in.jsonld",
		"tests/expand/0047-out.jsonld"
	)
}

#[test]
fn expand_0048() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0048-in.jsonld");
	println!("Terms are ignored in @id");
	println!("Values of @id are not expanded as terms");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0048-in.jsonld",
		"tests/expand/0048-out.jsonld"
	)
}

#[test]
fn expand_0049() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0049-in.jsonld");
	println!("String values of reverse properties");
	println!("String values of a reverse property with @type: @id are treated as IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0049-in.jsonld",
		"tests/expand/0049-out.jsonld"
	)
}

#[test]
fn expand_0050() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0050-in.jsonld");
	println!("Term definitions with prefix separate from prefix definitions");
	println!("Term definitions using compact IRIs don't inherit the definitions of the prefix");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0050-in.jsonld",
		"tests/expand/0050-out.jsonld"
	)
}

#[test]
fn expand_0051() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0051-in.jsonld");
	println!("Expansion of keyword aliases in term definitions");
	println!("Expanding terms which are keyword aliases");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0051-in.jsonld",
		"tests/expand/0051-out.jsonld"
	)
}

#[test]
fn expand_0052() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0052-in.jsonld");
	println!("@vocab-relative IRIs in term definitions");
	println!("If @vocab is defined, term definitions are expanded relative to @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0052-in.jsonld",
		"tests/expand/0052-out.jsonld"
	)
}

#[test]
fn expand_0053() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0053-in.jsonld");
	println!("Expand absolute IRI with @type: @vocab");
	println!("Expanding values of properties of @type: @vocab does not further expand absolute IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0053-in.jsonld",
		"tests/expand/0053-out.jsonld"
	)
}

#[test]
fn expand_0054() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0054-in.jsonld");
	println!("Expand term with @type: @vocab");
	println!("Expanding values of properties of @type: @vocab does not expand term values");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0054-in.jsonld",
		"tests/expand/0054-out.jsonld"
	)
}

#[test]
fn expand_0055() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0055-in.jsonld");
	println!("Expand @vocab-relative term with @type: @vocab");
	println!("Expanding values of properties of @type: @vocab expands relative IRIs using @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0055-in.jsonld",
		"tests/expand/0055-out.jsonld"
	)
}

#[test]
fn expand_0056() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0056-in.jsonld");
	println!("Use terms with @type: @vocab but not with @type: @id");
	println!("Checks that expansion uses appropriate base depending on term definition having @type @id or @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0056-in.jsonld",
		"tests/expand/0056-out.jsonld"
	)
}

#[test]
fn expand_0057() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0057-in.jsonld");
	println!("Expand relative IRI with @type: @vocab");
	println!("Relative values of terms with @type: @vocab expand relative to @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0057-in.jsonld",
		"tests/expand/0057-out.jsonld"
	)
}

#[test]
fn expand_0058() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0058-in.jsonld");
	println!("Expand compact IRI with @type: @vocab");
	println!("Compact IRIs are expanded normally even if term has @type: @vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0058-in.jsonld",
		"tests/expand/0058-out.jsonld"
	)
}

#[test]
fn expand_0059() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0059-in.jsonld");
	println!("Reset @vocab by setting it to null");
	println!("Setting @vocab to null removes a previous definition");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0059-in.jsonld",
		"tests/expand/0059-out.jsonld"
	)
}

#[test]
fn expand_0060() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0060-in.jsonld");
	println!("Overwrite document base with @base and reset it again");
	println!("Setting @base to an IRI and then resetting it to nil");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0060-in.jsonld",
		"tests/expand/0060-out.jsonld"
	)
}

#[test]
fn expand_0061() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0061-in.jsonld");
	println!("Coercing native types to arbitrary datatypes");
	println!("Expanding native types when coercing to arbitrary datatypes");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0061-in.jsonld",
		"tests/expand/0061-out.jsonld"
	)
}

#[test]
fn expand_0062() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0062-in.jsonld");
	println!("Various relative IRIs with with @base");
	println!("Pathological relative IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0062-in.jsonld",
		"tests/expand/0062-out.jsonld"
	)
}

#[test]
fn expand_0063() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0063-in.jsonld");
	println!("Reverse property and index container");
	println!("Expaning reverse properties with an index-container");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0063-in.jsonld",
		"tests/expand/0063-out.jsonld"
	)
}

#[test]
fn expand_0064() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0064-in.jsonld");
	println!("bnode values of reverse properties");
	println!("Expand reverse property whose values are unlabeled blank nodes");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0064-in.jsonld",
		"tests/expand/0064-out.jsonld"
	)
}

#[test]
fn expand_0065() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0065-in.jsonld");
	println!("Drop unmapped keys in reverse map");
	println!("Keys that are not mapped to an IRI in a reverse-map are dropped");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0065-in.jsonld",
		"tests/expand/0065-out.jsonld"
	)
}

#[test]
fn expand_0066() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0066-in.jsonld");
	println!("Reverse-map keys with @vocab");
	println!("Expand uses @vocab to expand keys in reverse-maps");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0066-in.jsonld",
		"tests/expand/0066-out.jsonld"
	)
}

#[test]
fn expand_0067() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0067-in.jsonld");
	println!("prefix://suffix not a compact IRI");
	println!("prefix:suffix values are not interpreted as compact IRIs if suffix begins with two slashes");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0067-in.jsonld",
		"tests/expand/0067-out.jsonld"
	)
}

#[test]
fn expand_0068() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0068-in.jsonld");
	println!("_:suffix values are not a compact IRI");
	println!("prefix:suffix values are not interpreted as compact IRIs if prefix is an underscore");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0068-in.jsonld",
		"tests/expand/0068-out.jsonld"
	)
}

#[test]
fn expand_0069() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0069-in.jsonld");
	println!("Compact IRI as term with type mapping");
	println!("Redefine compact IRI to define type mapping using the compact IRI itself as value of @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0069-in.jsonld",
		"tests/expand/0069-out.jsonld"
	)
}

#[test]
fn expand_0070() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0070-in.jsonld");
	println!("Compact IRI as term defined using equivalent compact IRI");
	println!("Redefine compact IRI to define type mapping using the compact IRI itself as string value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0070-in.jsonld",
		"tests/expand/0070-out.jsonld"
	)
}

#[test]
fn expand_0072() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0072-in.jsonld");
	println!("Redefine term using @vocab, not itself");
	println!("Redefining a term as itself when @vocab is defined uses @vocab, not previous term definition");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0072-in.jsonld",
		"tests/expand/0072-out.jsonld"
	)
}

#[test]
fn expand_0073() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0073-in.jsonld");
	println!("@context not first property");
	println!("Objects are unordered, so serialized node definition containing @context may have @context at the end of the node definition");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0073-in.jsonld",
		"tests/expand/0073-out.jsonld"
	)
}

#[test]
fn expand_0074() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0074-in.jsonld");
	println!("@id not first property");
	println!("Objects are unordered, so serialized node definition containing @id may have @id at the end of the node definition");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0074-in.jsonld",
		"tests/expand/0074-out.jsonld"
	)
}

#[test]
fn expand_0075() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0075-in.jsonld");
	println!("@vocab as blank node identifier");
	println!("Use @vocab to map all properties to blank node identifiers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0075-in.jsonld",
		"tests/expand/0075-out.jsonld"
	)
}

#[test]
fn expand_0076() {
	let input_url = iri!("http://example/base/");
	println!("base option overrides document location");
	println!("Use of the base option overrides the document location");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0076-in.jsonld",
		"tests/expand/0076-out.jsonld"
	)
}

#[test]
fn expand_0077() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0077-in.jsonld");
	println!("expandContext option");
	println!("Use of the expandContext option to expand the input document");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: Some("tests/expand/0077-context.jsonld"),
			ordered: false
		},
		input_url,
		"tests/expand/0077-in.jsonld",
		"tests/expand/0077-out.jsonld"
	)
}

#[test]
fn expand_0078() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0078-in.jsonld");
	println!("multiple reverse properties");
	println!("Use of multiple reverse properties");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0078-in.jsonld",
		"tests/expand/0078-out.jsonld"
	)
}

#[test]
fn expand_0079() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0079-in.jsonld");
	println!("expand @graph container");
	println!("Use of @graph containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0079-in.jsonld",
		"tests/expand/0079-out.jsonld"
	)
}

#[test]
fn expand_0080() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0080-in.jsonld");
	println!("expand [@graph, @set] container");
	println!("Use of [@graph, @set] containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0080-in.jsonld",
		"tests/expand/0080-out.jsonld"
	)
}

#[test]
fn expand_0081() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0081-in.jsonld");
	println!("Creates an @graph container if value is a graph");
	println!("Don't double-expand an already expanded graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0081-in.jsonld",
		"tests/expand/0081-out.jsonld"
	)
}

#[test]
fn expand_0082() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0082-in.jsonld");
	println!("expand [@graph, @index] container");
	println!("Use of @graph containers with @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0082-in.jsonld",
		"tests/expand/0082-out.jsonld"
	)
}

#[test]
fn expand_0083() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0083-in.jsonld");
	println!("expand [@graph, @index, @set] container");
	println!("Use of @graph containers with @index and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0083-in.jsonld",
		"tests/expand/0083-out.jsonld"
	)
}

#[test]
fn expand_0084() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0084-in.jsonld");
	println!("Do not expand [@graph, @index] container if value is a graph");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0084-in.jsonld",
		"tests/expand/0084-out.jsonld"
	)
}

#[test]
fn expand_0085() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0085-in.jsonld");
	println!("expand [@graph, @id] container");
	println!("Use of @graph containers with @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0085-in.jsonld",
		"tests/expand/0085-out.jsonld"
	)
}

#[test]
fn expand_0086() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0086-in.jsonld");
	println!("expand [@graph, @id, @set] container");
	println!("Use of @graph containers with @id and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0086-in.jsonld",
		"tests/expand/0086-out.jsonld"
	)
}

#[test]
fn expand_0087() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0087-in.jsonld");
	println!("Do not expand [@graph, @id] container if value is a graph");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0087-in.jsonld",
		"tests/expand/0087-out.jsonld"
	)
}

#[test]
fn expand_0088() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0088-in.jsonld");
	println!("Do not expand native values to IRIs");
	println!("Value Expansion does not expand native values, such as booleans, to a node object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0088-in.jsonld",
		"tests/expand/0088-out.jsonld"
	)
}

#[test]
fn expand_0089() {
	let input_url = iri!("http://example/base/");
	println!("empty @base applied to the base option");
	println!("Use of an empty @base is applied to the base option");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0089-in.jsonld",
		"tests/expand/0089-out.jsonld"
	)
}

#[test]
fn expand_0090() {
	let input_url = iri!("http://example/base/");
	println!("relative @base overrides base option and document location");
	println!("Use of a relative @base overrides base option and document location");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0090-in.jsonld",
		"tests/expand/0090-out.jsonld"
	)
}

#[test]
fn expand_0091() {
	let input_url = iri!("http://example/base/");
	println!("relative and absolute @base overrides base option and document location");
	println!("Use of a relative and absolute @base overrides base option and document location");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0091-in.jsonld",
		"tests/expand/0091-out.jsonld"
	)
}

#[test]
fn expand_0092() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0092-in.jsonld");
	println!("Various relative IRIs as properties with with @vocab: ''");
	println!("Pathological relative property IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0092-in.jsonld",
		"tests/expand/0092-out.jsonld"
	)
}

#[test]
fn expand_0093() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0093-in.jsonld");
	println!("expand @graph container (multiple objects)");
	println!("Use of @graph containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0093-in.jsonld",
		"tests/expand/0093-out.jsonld"
	)
}

#[test]
fn expand_0094() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0094-in.jsonld");
	println!("expand [@graph, @set] container (multiple objects)");
	println!("Use of [@graph, @set] containers");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0094-in.jsonld",
		"tests/expand/0094-out.jsonld"
	)
}

#[test]
fn expand_0095() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0095-in.jsonld");
	println!("Creates an @graph container if value is a graph (multiple objects)");
	println!("Double-expand an already expanded graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0095-in.jsonld",
		"tests/expand/0095-out.jsonld"
	)
}

#[test]
fn expand_0096() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0096-in.jsonld");
	println!("expand [@graph, @index] container (multiple indexed objects)");
	println!("Use of @graph containers with @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0096-in.jsonld",
		"tests/expand/0096-out.jsonld"
	)
}

#[test]
fn expand_0097() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0097-in.jsonld");
	println!("expand [@graph, @index, @set] container (multiple objects)");
	println!("Use of @graph containers with @index and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0097-in.jsonld",
		"tests/expand/0097-out.jsonld"
	)
}

#[test]
fn expand_0098() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0098-in.jsonld");
	println!("Do not expand [@graph, @index] container if value is a graph (multiple objects)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0098-in.jsonld",
		"tests/expand/0098-out.jsonld"
	)
}

#[test]
fn expand_0099() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0099-in.jsonld");
	println!("expand [@graph, @id] container (multiple objects)");
	println!("Use of @graph containers with @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0099-in.jsonld",
		"tests/expand/0099-out.jsonld"
	)
}

#[test]
fn expand_0100() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0100-in.jsonld");
	println!("expand [@graph, @id, @set] container (multiple objects)");
	println!("Use of @graph containers with @id and @set");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0100-in.jsonld",
		"tests/expand/0100-out.jsonld"
	)
}

#[test]
fn expand_0101() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0101-in.jsonld");
	println!("Do not expand [@graph, @id] container if value is a graph (multiple objects)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0101-in.jsonld",
		"tests/expand/0101-out.jsonld"
	)
}

#[test]
fn expand_0102() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0102-in.jsonld");
	println!("Expand @graph container if value is a graph (multiple objects)");
	println!("Creates a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0102-in.jsonld",
		"tests/expand/0102-out.jsonld"
	)
}

#[test]
fn expand_0103() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0103-in.jsonld");
	println!("Expand @graph container if value is a graph (multiple graphs)");
	println!("Creates a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0103-in.jsonld",
		"tests/expand/0103-out.jsonld"
	)
}

#[test]
fn expand_0104() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0104-in.jsonld");
	println!("Creates an @graph container if value is a graph (mixed graph and object)");
	println!("Double-expand an already expanded graph");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0104-in.jsonld",
		"tests/expand/0104-out.jsonld"
	)
}

#[test]
fn expand_0105() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0105-in.jsonld");
	println!("Do not expand [@graph, @index] container if value is a graph (mixed graph and object)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0105-in.jsonld",
		"tests/expand/0105-out.jsonld"
	)
}

#[test]
fn expand_0106() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0106-in.jsonld");
	println!("Do not expand [@graph, @id] container if value is a graph (mixed graph and object)");
	println!("Does not create a new graph object if indexed value is already a graph object");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0106-in.jsonld",
		"tests/expand/0106-out.jsonld"
	)
}

#[test]
fn expand_0107() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0107-in.jsonld");
	println!("expand [@graph, @index] container (indexes with multiple objects)");
	println!("Use of @graph containers with @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0107-in.jsonld",
		"tests/expand/0107-out.jsonld"
	)
}

#[test]
fn expand_0108() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0108-in.jsonld");
	println!("expand [@graph, @id] container (multiple ids and objects)");
	println!("Use of @graph containers with @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0108-in.jsonld",
		"tests/expand/0108-out.jsonld"
	)
}

#[test]
fn expand_0109() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0109-in.jsonld");
	println!("IRI expansion of fragments including ':'");
	println!("Do not treat as absolute IRIs values that look like compact IRIs if they're not absolute");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0109-in.jsonld",
		"tests/expand/0109-out.jsonld"
	)
}

#[test]
fn expand_0110() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0110-in.jsonld");
	println!("Various relative IRIs as properties with with relative @vocab");
	println!("Pathological relative property IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0110-in.jsonld",
		"tests/expand/0110-out.jsonld"
	)
}

#[test]
fn expand_0111() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0111-in.jsonld");
	println!("Various relative IRIs as properties with with relative @vocab itself relative to an existing vocabulary base");
	println!("Pathological relative property IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0111-in.jsonld",
		"tests/expand/0111-out.jsonld"
	)
}

#[test]
fn expand_0112() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0112-in.jsonld");
	println!("Various relative IRIs as properties with with relative @vocab relative to another relative vocabulary base");
	println!("Pathological relative property IRIs");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0112-in.jsonld",
		"tests/expand/0112-out.jsonld"
	)
}

#[test]
fn expand_0113() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0113-in.jsonld");
	println!("context with JavaScript Object property names");
	println!("Expand with context including JavaScript Object property names");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0113-in.jsonld",
		"tests/expand/0113-out.jsonld"
	)
}

#[test]
fn expand_0114() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0114-in.jsonld");
	println!("Expansion allows multiple properties expanding to @type");
	println!("An exception for the colliding keywords error is made for @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0114-in.jsonld",
		"tests/expand/0114-out.jsonld"
	)
}

#[test]
fn expand_0117() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0117-in.jsonld");
	println!("A term starting with a colon can expand to a different IRI");
	println!("Terms may begin with a colon and not be treated as IRIs.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0117-in.jsonld",
		"tests/expand/0117-out.jsonld"
	)
}

#[test]
fn expand_0118() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0118-in.jsonld");
	println!("Expanding a value staring with a colon does not treat that value as an IRI");
	println!("Terms may begin with a colon and not be treated as IRIs.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0118-in.jsonld",
		"tests/expand/0118-out.jsonld"
	)
}

#[test]
fn expand_0119() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0119-in.jsonld");
	println!("Ignore some terms with @, allow others.");
	println!("Processors SHOULD generate a warning and MUST ignore terms having the form of a keyword.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0119-in.jsonld",
		"tests/expand/0119-out.jsonld"
	)
}

#[test]
fn expand_0120() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0120-in.jsonld");
	println!("Ignore some values of @id with @, allow others.");
	println!("Processors SHOULD generate a warning and MUST ignore values of @id having the form of a keyword.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0120-in.jsonld",
		"tests/expand/0120-out.jsonld"
	)
}

#[test]
fn expand_0121() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0121-in.jsonld");
	println!("Ignore some values of @reverse with @, allow others.");
	println!("Processors SHOULD generate a warning and MUST ignore values of @reverse having the form of a keyword.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0121-in.jsonld",
		"tests/expand/0121-out.jsonld"
	)
}

#[test]
fn expand_0122() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0122-in.jsonld");
	println!("Ignore some IRIs when that start with @ when expanding.");
	println!("Processors SHOULD generate a warning and MUST ignore IRIs having the form of a keyword.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0122-in.jsonld",
		"tests/expand/0122-out.jsonld"
	)
}

#[test]
fn expand_0123() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0123-in.jsonld");
	println!("Value objects including invalid literal datatype IRIs are rejected");
	println!("Processors MUST validate datatype IRIs.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0123-in.jsonld",
		ErrorCode::InvalidTypedValue
	)
}

#[test]
fn expand_0124() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0124-in.jsonld");
	println!("compact IRI as @vocab");
	println!("Verifies that @vocab defined as a compact IRI expands properly");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0124-in.jsonld",
		"tests/expand/0124-out.jsonld"
	)
}

#[test]
fn expand_0125() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0125-in.jsonld");
	println!("term as @vocab");
	println!("Verifies that @vocab defined as a term expands properly");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0125-in.jsonld",
		"tests/expand/0125-out.jsonld"
	)
}

#[test]
fn expand_0126() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0126-in.jsonld");
	println!("A scoped context may include itself recursively (direct)");
	println!("Verifies that no exception is raised on expansion when processing a scoped context referencing itself directly");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0126-in.jsonld",
		"tests/expand/0126-out.jsonld"
	)
}

#[test]
fn expand_0127() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0127-in.jsonld");
	println!("A scoped context may include itself recursively (indirect)");
	println!("Verifies that no exception is raised on expansion when processing a scoped context referencing itself indirectly");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0127-in.jsonld",
		"tests/expand/0127-out.jsonld"
	)
}

#[test]
fn expand_0128() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0128-in.jsonld");
	println!("Two scoped context may include a shared context");
	println!("Verifies that no exception is raised on expansion when processing two scoped contexts referencing a shared context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0128-in.jsonld",
		"tests/expand/0128-out.jsonld"
	)
}

#[test]
fn expand_0129() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0129-in.jsonld");
	println!("Base without trailing slash, without path");
	println!("Verify URI resolution relative to base (without trailing slash, without path) according to RFC 3986");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0129-in.jsonld",
		"tests/expand/0129-out.jsonld"
	)
}

#[test]
fn expand_0130() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/0130-in.jsonld");
	println!("Base without trailing slash, with path");
	println!("Verify URI resolution relative to base (without trailing slash, with path) according to RFC 3986");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/0130-in.jsonld",
		"tests/expand/0130-out.jsonld"
	)
}

#[test]
fn expand_c001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c001-in.jsonld");
	println!("adding new term");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c001-in.jsonld",
		"tests/expand/c001-out.jsonld"
	)
}

#[test]
fn expand_c002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c002-in.jsonld");
	println!("overriding a term");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c002-in.jsonld",
		"tests/expand/c002-out.jsonld"
	)
}

#[test]
fn expand_c003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c003-in.jsonld");
	println!("property and value with different terms mapping to the same expanded property");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c003-in.jsonld",
		"tests/expand/c003-out.jsonld"
	)
}

#[test]
fn expand_c004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c004-in.jsonld");
	println!("deep @context affects nested nodes");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c004-in.jsonld",
		"tests/expand/c004-out.jsonld"
	)
}

#[test]
fn expand_c005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c005-in.jsonld");
	println!("scoped context layers on intemediate contexts");
	println!("Expansion using a scoped context uses term scope for selecting proper term");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c005-in.jsonld",
		"tests/expand/c005-out.jsonld"
	)
}

#[test]
fn expand_c006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c006-in.jsonld");
	println!("adding new term");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c006-in.jsonld",
		"tests/expand/c006-out.jsonld"
	)
}

#[test]
fn expand_c007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c007-in.jsonld");
	println!("overriding a term");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c007-in.jsonld",
		"tests/expand/c007-out.jsonld"
	)
}

#[test]
fn expand_c008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c008-in.jsonld");
	println!("alias of @type");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c008-in.jsonld",
		"tests/expand/c008-out.jsonld"
	)
}

#[test]
fn expand_c009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c009-in.jsonld");
	println!("deep @type-scoped @context does NOT affect nested nodes");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c009-in.jsonld",
		"tests/expand/c009-out.jsonld"
	)
}

#[test]
fn expand_c010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c010-in.jsonld");
	println!("scoped context layers on intemediate contexts");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c010-in.jsonld",
		"tests/expand/c010-out.jsonld"
	)
}

#[test]
fn expand_c011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c011-in.jsonld");
	println!("orders @type terms when applying scoped contexts");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c011-in.jsonld",
		"tests/expand/c011-out.jsonld"
	)
}

#[test]
fn expand_c012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c012-in.jsonld");
	println!("deep property-term scoped @context in @type-scoped @context affects nested nodes");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c012-in.jsonld",
		"tests/expand/c012-out.jsonld"
	)
}

#[test]
fn expand_c013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c013-in.jsonld");
	println!("type maps use scoped context from type index and not scoped context from containing");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c013-in.jsonld",
		"tests/expand/c013-out.jsonld"
	)
}

#[test]
fn expand_c014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c014-in.jsonld");
	println!("type-scoped context nullification");
	println!("type-scoped context nullification");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c014-in.jsonld",
		"tests/expand/c014-out.jsonld"
	)
}

#[test]
fn expand_c015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c015-in.jsonld");
	println!("type-scoped base");
	println!("type-scoped base");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c015-in.jsonld",
		"tests/expand/c015-out.jsonld"
	)
}

#[test]
fn expand_c016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c016-in.jsonld");
	println!("type-scoped vocab");
	println!("type-scoped vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c016-in.jsonld",
		"tests/expand/c016-out.jsonld"
	)
}

#[test]
fn expand_c017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c017-in.jsonld");
	println!("multiple type-scoped contexts are properly reverted");
	println!("multiple type-scoped contexts are property reverted");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c017-in.jsonld",
		"tests/expand/c017-out.jsonld"
	)
}

#[test]
fn expand_c018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c018-in.jsonld");
	println!("multiple type-scoped types resolved against previous context");
	println!("multiple type-scoped types resolved against previous context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c018-in.jsonld",
		"tests/expand/c018-out.jsonld"
	)
}

#[test]
fn expand_c019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c019-in.jsonld");
	println!("type-scoped context with multiple property scoped terms");
	println!("type-scoped context with multiple property scoped terms");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c019-in.jsonld",
		"tests/expand/c019-out.jsonld"
	)
}

#[test]
fn expand_c020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c020-in.jsonld");
	println!("type-scoped value");
	println!("type-scoped value");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c020-in.jsonld",
		"tests/expand/c020-out.jsonld"
	)
}

#[test]
fn expand_c021() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c021-in.jsonld");
	println!("type-scoped value mix");
	println!("type-scoped value mix");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c021-in.jsonld",
		"tests/expand/c021-out.jsonld"
	)
}

#[test]
fn expand_c022() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c022-in.jsonld");
	println!("type-scoped property-scoped contexts including @type:@vocab");
	println!("type-scoped property-scoped contexts including @type:@vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c022-in.jsonld",
		"tests/expand/c022-out.jsonld"
	)
}

#[test]
fn expand_c023() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c023-in.jsonld");
	println!("composed type-scoped property-scoped contexts including @type:@vocab");
	println!("composed type-scoped property-scoped contexts including @type:@vocab");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c023-in.jsonld",
		"tests/expand/c023-out.jsonld"
	)
}

#[test]
fn expand_c024() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c024-in.jsonld");
	println!("type-scoped + property-scoped + values evaluates against previous context");
	println!("type-scoped + property-scoped + values evaluates against previous context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c024-in.jsonld",
		"tests/expand/c024-out.jsonld"
	)
}

#[test]
fn expand_c025() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c025-in.jsonld");
	println!("type-scoped + graph container");
	println!("type-scoped + graph container");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c025-in.jsonld",
		"tests/expand/c025-out.jsonld"
	)
}

#[test]
fn expand_c026() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c026-in.jsonld");
	println!("@propagate: true on type-scoped context");
	println!("type-scoped context with @propagate: true survive node-objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c026-in.jsonld",
		"tests/expand/c026-out.jsonld"
	)
}

#[test]
fn expand_c027() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c027-in.jsonld");
	println!("@propagate: false on property-scoped context");
	println!("property-scoped context with @propagate: false do not survive node-objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c027-in.jsonld",
		"tests/expand/c027-out.jsonld"
	)
}

#[test]
fn expand_c028() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c028-in.jsonld");
	println!("@propagate: false on embedded context");
	println!("embedded context with @propagate: false do not survive node-objects");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c028-in.jsonld",
		"tests/expand/c028-out.jsonld"
	)
}

#[test]
fn expand_c029() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c029-in.jsonld");
	println!("@propagate is invalid in 1.0");
	println!("@propagate is invalid in 1.0");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c029-in.jsonld",
		ErrorCode::InvalidContextEntry
	)
}

#[test]
fn expand_c030() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c030-in.jsonld");
	println!("@propagate must be boolean valued");
	println!("@propagate must be boolean valued");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c030-in.jsonld",
		ErrorCode::InvalidPropagateValue
	)
}

#[test]
fn expand_c031() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c031-in.jsonld");
	println!("@context resolutions respects relative URLs.");
	println!("URL resolution follows RFC3986");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c031-in.jsonld",
		"tests/expand/c031-out.jsonld"
	)
}

#[test]
fn expand_c032() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c032-in.jsonld");
	println!("Unused embedded context with error.");
	println!("An embedded context which is never used should still be checked.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c032-in.jsonld",
		ErrorCode::InvalidScopedContext
	)
}

#[test]
fn expand_c033() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c033-in.jsonld");
	println!("Unused context with an embedded context error.");
	println!("An unused context with an embedded context should still be checked.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c033-in.jsonld",
		ErrorCode::InvalidScopedContext
	)
}

#[test]
fn expand_c034() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c034-in.jsonld");
	println!("Remote scoped context.");
	println!("Scoped contexts may be externally loaded.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c034-in.jsonld",
		"tests/expand/c034-out.jsonld"
	)
}

#[test]
fn expand_c035() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/c035-in.jsonld");
	println!("Term scoping with embedded contexts.");
	println!("Terms should make use of @vocab relative to the scope in which the term was defined.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/c035-in.jsonld",
		"tests/expand/c035-out.jsonld"
	)
}

#[test]
fn expand_di01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di01-in.jsonld");
	println!("Expand string using default and term directions");
	println!("Strings are coerced to have @direction based on default and term direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di01-in.jsonld",
		"tests/expand/di01-out.jsonld"
	)
}

#[test]
fn expand_di02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di02-in.jsonld");
	println!("Expand string using default and term directions and languages");
	println!("Strings are coerced to have @direction based on default and term direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di02-in.jsonld",
		"tests/expand/di02-out.jsonld"
	)
}

#[test]
fn expand_di03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di03-in.jsonld");
	println!("expand list values with @direction");
	println!("List values where the term has @direction are used in expansion.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di03-in.jsonld",
		"tests/expand/di03-out.jsonld"
	)
}

#[test]
fn expand_di04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di04-in.jsonld");
	println!("simple language map with term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di04-in.jsonld",
		"tests/expand/di04-out.jsonld"
	)
}

#[test]
fn expand_di05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di05-in.jsonld");
	println!("simple language mapwith overriding term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di05-in.jsonld",
		"tests/expand/di05-out.jsonld"
	)
}

#[test]
fn expand_di06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di06-in.jsonld");
	println!("simple language mapwith overriding null direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di06-in.jsonld",
		"tests/expand/di06-out.jsonld"
	)
}

#[test]
fn expand_di07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di07-in.jsonld");
	println!("simple language map with mismatching term direction");
	println!("Term selection with language maps and @direction.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di07-in.jsonld",
		"tests/expand/di07-out.jsonld"
	)
}

#[test]
fn expand_di08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di08-in.jsonld");
	println!("@direction must be one of ltr or rtl");
	println!("Generate an error if @direction has illegal value.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di08-in.jsonld",
		ErrorCode::InvalidBaseDirection
	)
}

#[test]
fn expand_di09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/di09-in.jsonld");
	println!("@direction is incompatible with @type");
	println!("Value objects can have either @type but not @language or @direction.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/di09-in.jsonld",
		ErrorCode::InvalidValueObject
	)
}

#[test]
fn expand_ec01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/ec01-in.jsonld");
	println!("Invalid keyword in term definition");
	println!("Verifies that an exception is raised on expansion when a invalid term definition is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/ec01-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_ec02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/ec02-in.jsonld");
	println!("Term definition on @type with empty map");
	println!("Verifies that an exception is raised if @type is defined as a term with an empty map");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/ec02-in.jsonld",
		ErrorCode::KeywordRedefinition
	)
}

#[test]
fn expand_em01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/em01-in.jsonld");
	println!("Invalid container mapping");
	println!("Verifies that an exception is raised on expansion when a invalid container mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/em01-in.jsonld",
		ErrorCode::InvalidContainerMapping
	)
}

#[test]
fn expand_en01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/en01-in.jsonld");
	println!("@nest MUST NOT have a string value");
	println!("container: @nest");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/en01-in.jsonld",
		ErrorCode::InvalidNestValue
	)
}

#[test]
fn expand_en02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/en02-in.jsonld");
	println!("@nest MUST NOT have a boolen value");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/en02-in.jsonld",
		ErrorCode::InvalidNestValue
	)
}

#[test]
fn expand_en03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/en03-in.jsonld");
	println!("@nest MUST NOT have a numeric value");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/en03-in.jsonld",
		ErrorCode::InvalidNestValue
	)
}

#[test]
fn expand_en04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/en04-in.jsonld");
	println!("@nest MUST NOT have a value object value");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/en04-in.jsonld",
		ErrorCode::InvalidNestValue
	)
}

#[test]
fn expand_en05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/en05-in.jsonld");
	println!("does not allow a keyword other than @nest for the value of @nest");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/en05-in.jsonld",
		ErrorCode::InvalidNestValue
	)
}

#[test]
fn expand_en06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/en06-in.jsonld");
	println!("does not allow @nest with @reverse");
	println!("Transparent Nesting");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/en06-in.jsonld",
		ErrorCode::InvalidReverseProperty
	)
}

#[test]
fn expand_ep02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/ep02-in.jsonld");
	println!("processingMode json-ld-1.0 conflicts with @version: 1.1");
	println!("If processingMode is explicitly json-ld-1.0, it will conflict with 1.1 features.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/ep02-in.jsonld",
		ErrorCode::ProcessingModeConflict
	)
}

#[test]
fn expand_ep03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/ep03-in.jsonld");
	println!("@version must be 1.1");
	println!("If @version is specified, it must be 1.1");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/ep03-in.jsonld",
		ErrorCode::InvalidVersionValue
	)
}

#[test]
fn expand_er01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er01-in.jsonld");
	println!("Keywords cannot be aliased to other keywords");
	println!("Verifies that an exception is raised on expansion when processing an invalid context aliasing a keyword to another keyword");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er01-in.jsonld",
		ErrorCode::KeywordRedefinition
	)
}

#[test]
fn expand_er04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er04-in.jsonld");
	println!("Error dereferencing a remote context");
	println!("Verifies that an exception is raised on expansion when a context dereference results in an error");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er04-in.jsonld",
		ErrorCode::LoadingRemoteContextFailed
	)
}

#[test]
fn expand_er05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er05-in.jsonld");
	println!("Invalid remote context");
	println!("Verifies that an exception is raised on expansion when a remote context is not an object containing @context");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er05-in.jsonld",
		ErrorCode::InvalidRemoteContext
	)
}

#[test]
fn expand_er06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er06-in.jsonld");
	println!("Invalid local context");
	println!("Verifies that an exception is raised on expansion when a context is not a string or object");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er06-in.jsonld",
		ErrorCode::InvalidLocalContext
	)
}

#[test]
fn expand_er07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er07-in.jsonld");
	println!("Invalid base IRI");
	println!("Verifies that an exception is raised on expansion when a context contains an invalid @base");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er07-in.jsonld",
		ErrorCode::InvalidBaseIri
	)
}

#[test]
fn expand_er08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er08-in.jsonld");
	println!("Invalid vocab mapping");
	println!("Verifies that an exception is raised on expansion when a context contains an invalid @vocab mapping");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er08-in.jsonld",
		ErrorCode::InvalidVocabMapping
	)
}

#[test]
fn expand_er09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er09-in.jsonld");
	println!("Invalid default language");
	println!("Verifies that an exception is raised on expansion when a context contains an invalid @language");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er09-in.jsonld",
		ErrorCode::InvalidDefaultLanguage
	)
}

#[test]
fn expand_er10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er10-in.jsonld");
	println!("Cyclic IRI mapping");
	println!("Verifies that an exception is raised on expansion when a cyclic IRI mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er10-in.jsonld",
		ErrorCode::CyclicIriMapping
	)
}

#[test]
fn expand_er11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er11-in.jsonld");
	println!("Invalid term definition");
	println!("Verifies that an exception is raised on expansion when a invalid term definition is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er11-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_er12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er12-in.jsonld");
	println!("Invalid type mapping (not a string)");
	println!("Verifies that an exception is raised on expansion when a invalid type mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er12-in.jsonld",
		ErrorCode::InvalidTypeMapping
	)
}

#[test]
fn expand_er13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er13-in.jsonld");
	println!("Invalid type mapping (not absolute IRI)");
	println!("Verifies that an exception is raised on expansion when a invalid type mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er13-in.jsonld",
		ErrorCode::InvalidTypeMapping
	)
}

#[test]
fn expand_er14() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er14-in.jsonld");
	println!("Invalid reverse property (contains @id)");
	println!("Verifies that an exception is raised on expansion when a invalid reverse property is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er14-in.jsonld",
		ErrorCode::InvalidReverseProperty
	)
}

#[test]
fn expand_er15() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er15-in.jsonld");
	println!("Invalid IRI mapping (@reverse not a string)");
	println!("Verifies that an exception is raised on expansion when a invalid IRI mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er15-in.jsonld",
		ErrorCode::InvalidIriMapping
	)
}

#[test]
fn expand_er17() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er17-in.jsonld");
	println!("Invalid reverse property (invalid @container)");
	println!("Verifies that an exception is raised on expansion when a invalid reverse property is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er17-in.jsonld",
		ErrorCode::InvalidReverseProperty
	)
}

#[test]
fn expand_er18() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er18-in.jsonld");
	println!("Invalid IRI mapping (@id not a string)");
	println!("Verifies that an exception is raised on expansion when a invalid IRI mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er18-in.jsonld",
		ErrorCode::InvalidIriMapping
	)
}

#[test]
fn expand_er19() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er19-in.jsonld");
	println!("Invalid keyword alias (@context)");
	println!("Verifies that an exception is raised on expansion when a invalid keyword alias is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er19-in.jsonld",
		ErrorCode::InvalidKeywordAlias
	)
}

#[test]
fn expand_er20() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er20-in.jsonld");
	println!("Invalid IRI mapping (no vocab mapping)");
	println!("Verifies that an exception is raised on expansion when a invalid IRI mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er20-in.jsonld",
		ErrorCode::InvalidIriMapping
	)
}

#[test]
fn expand_er21() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er21-in.jsonld");
	println!("Invalid container mapping");
	println!("Verifies that an exception is raised on expansion when a invalid container mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er21-in.jsonld",
		ErrorCode::InvalidContainerMapping
	)
}

#[test]
fn expand_er22() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er22-in.jsonld");
	println!("Invalid language mapping");
	println!("Verifies that an exception is raised on expansion when a invalid language mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er22-in.jsonld",
		ErrorCode::InvalidLanguageMapping
	)
}

#[test]
fn expand_er23() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er23-in.jsonld");
	println!("Invalid IRI mapping (relative IRI in @type)");
	println!("Verifies that an exception is raised on expansion when a invalid type mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er23-in.jsonld",
		ErrorCode::InvalidTypeMapping
	)
}

#[test]
fn expand_er25() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er25-in.jsonld");
	println!("Invalid reverse property map");
	println!("Verifies that an exception is raised in Expansion when a invalid reverse property map is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er25-in.jsonld",
		ErrorCode::InvalidReversePropertyMap
	)
}

#[test]
fn expand_er26() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er26-in.jsonld");
	println!("Colliding keywords");
	println!("Verifies that an exception is raised in Expansion when colliding keywords are found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er26-in.jsonld",
		ErrorCode::CollidingKeywords
	)
}

#[test]
fn expand_er27() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er27-in.jsonld");
	println!("Invalid @id value");
	println!("Verifies that an exception is raised in Expansion when an invalid @id value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er27-in.jsonld",
		ErrorCode::InvalidIdValue
	)
}

#[test]
fn expand_er28() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er28-in.jsonld");
	println!("Invalid type value");
	println!("Verifies that an exception is raised in Expansion when an invalid type value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er28-in.jsonld",
		ErrorCode::InvalidTypeValue
	)
}

#[test]
fn expand_er29() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er29-in.jsonld");
	println!("Invalid value object value");
	println!("Verifies that an exception is raised in Expansion when an invalid value object value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er29-in.jsonld",
		ErrorCode::InvalidValueObjectValue
	)
}

#[test]
fn expand_er30() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er30-in.jsonld");
	println!("Invalid language-tagged string");
	println!("Verifies that an exception is raised in Expansion when an invalid language-tagged string value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er30-in.jsonld",
		ErrorCode::InvalidLanguageTaggedString
	)
}

#[test]
fn expand_er31() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er31-in.jsonld");
	println!("Invalid @index value");
	println!("Verifies that an exception is raised in Expansion when an invalid @index value value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er31-in.jsonld",
		ErrorCode::InvalidIndexValue
	)
}

#[test]
fn expand_er33() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er33-in.jsonld");
	println!("Invalid @reverse value");
	println!("Verifies that an exception is raised in Expansion when an invalid @reverse value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er33-in.jsonld",
		ErrorCode::InvalidReverseValue
	)
}

#[test]
fn expand_er34() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er34-in.jsonld");
	println!("Invalid reverse property value (in @reverse)");
	println!("Verifies that an exception is raised in Expansion when an invalid reverse property value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er34-in.jsonld",
		ErrorCode::InvalidReversePropertyValue
	)
}

#[test]
fn expand_er35() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er35-in.jsonld");
	println!("Invalid language map value");
	println!("Verifies that an exception is raised in Expansion when an invalid language map value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er35-in.jsonld",
		ErrorCode::InvalidLanguageMapValue
	)
}

#[test]
fn expand_er36() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er36-in.jsonld");
	println!("Invalid reverse property value (through coercion)");
	println!("Verifies that an exception is raised in Expansion when an invalid reverse property value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er36-in.jsonld",
		ErrorCode::InvalidReversePropertyValue
	)
}

#[test]
fn expand_er37() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er37-in.jsonld");
	println!("Invalid value object (unexpected keyword)");
	println!("Verifies that an exception is raised in Expansion when an invalid value object is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er37-in.jsonld",
		ErrorCode::InvalidValueObject
	)
}

#[test]
fn expand_er38() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er38-in.jsonld");
	println!("Invalid value object (@type and @language)");
	println!("Verifies that an exception is raised in Expansion when an invalid value object is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er38-in.jsonld",
		ErrorCode::InvalidValueObject
	)
}

#[test]
fn expand_er39() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er39-in.jsonld");
	println!("Invalid language-tagged value");
	println!("Verifies that an exception is raised in Expansion when an invalid language-tagged value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er39-in.jsonld",
		ErrorCode::InvalidLanguageTaggedValue
	)
}

#[test]
fn expand_er40() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er40-in.jsonld");
	println!("Invalid typed value");
	println!("Verifies that an exception is raised in Expansion when an invalid typed value is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er40-in.jsonld",
		ErrorCode::InvalidTypedValue
	)
}

#[test]
fn expand_er41() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er41-in.jsonld");
	println!("Invalid set or list object");
	println!("Verifies that an exception is raised in Expansion when an invalid set or list object is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er41-in.jsonld",
		ErrorCode::InvalidSetOrListObject
	)
}

#[test]
fn expand_er42() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er42-in.jsonld");
	println!("Keywords may not be redefined in 1.0");
	println!("Verifies that an exception is raised on expansion when processing an invalid context attempting to define @container on a keyword");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er42-in.jsonld",
		ErrorCode::KeywordRedefinition
	)
}

#[test]
fn expand_er43() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er43-in.jsonld");
	println!("Term definition with @id: @type");
	println!("Expanding term mapping to @type uses @type syntax now illegal");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er43-in.jsonld",
		ErrorCode::InvalidIriMapping
	)
}

#[test]
fn expand_er44() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er44-in.jsonld");
	println!("Redefine terms looking like compact IRIs");
	println!("Term definitions may look like compact IRIs, but must be consistent.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er44-in.jsonld",
		ErrorCode::InvalidIriMapping
	)
}

#[test]
fn expand_er48() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er48-in.jsonld");
	println!("Invalid term as relative IRI");
	println!("Verifies that a relative IRI cannot be used as a term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er48-in.jsonld",
		ErrorCode::InvalidIriMapping
	)
}

#[test]
fn expand_er49() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er49-in.jsonld");
	println!("A relative IRI cannot be used as a prefix");
	println!("Verifies that a relative IRI cannot be used as a term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er49-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_er50() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er50-in.jsonld");
	println!("Invalid reverse id");
	println!("Verifies that an exception is raised in Expansion when an invalid IRI is used for @reverse.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er50-in.jsonld",
		ErrorCode::InvalidIriMapping
	)
}

#[test]
fn expand_er51() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er51-in.jsonld");
	println!("Invalid value object value using a value alias");
	println!("Verifies that an exception is raised in Expansion when an invalid value object value is found using a value alias");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er51-in.jsonld",
		ErrorCode::InvalidValueObjectValue
	)
}

#[test]
fn expand_er52() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er52-in.jsonld");
	println!("Definition for the empty term");
	println!("Verifies that an exception is raised on expansion when a context contains a definition for the empty term");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er52-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_er53() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/er53-in.jsonld");
	println!("Invalid prefix value");
	println!("Verifies that an exception is raised on expansion when a context contains an invalid @prefix value");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/er53-in.jsonld",
		ErrorCode::InvalidPrefixValue
	)
}

#[test]
fn expand_es01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/es01-in.jsonld");
	println!("Using an array value for @context is illegal in JSON-LD 1.0");
	println!("Verifies that an exception is raised on expansion when a invalid container mapping is found");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/es01-in.jsonld",
		ErrorCode::InvalidContainerMapping
	)
}

#[test]
fn expand_es02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/es02-in.jsonld");
	println!("Mapping @container: [@list, @set] is invalid");
	println!("Testing legal combinations of @set with other container values");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/es02-in.jsonld",
		ErrorCode::InvalidContainerMapping
	)
}

#[test]
fn expand_in01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in01-in.jsonld");
	println!("Basic Included array");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in01-in.jsonld",
		"tests/expand/in01-out.jsonld"
	)
}

#[test]
fn expand_in02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in02-in.jsonld");
	println!("Basic Included object");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in02-in.jsonld",
		"tests/expand/in02-out.jsonld"
	)
}

#[test]
fn expand_in03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in03-in.jsonld");
	println!("Multiple properties mapping to @included are folded together");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in03-in.jsonld",
		"tests/expand/in03-out.jsonld"
	)
}

#[test]
fn expand_in04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in04-in.jsonld");
	println!("Included containing @included");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in04-in.jsonld",
		"tests/expand/in04-out.jsonld"
	)
}

#[test]
fn expand_in05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in05-in.jsonld");
	println!("Property value with @included");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in05-in.jsonld",
		"tests/expand/in05-out.jsonld"
	)
}

#[test]
fn expand_in06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in06-in.jsonld");
	println!("json.api example");
	println!("Tests included blocks.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in06-in.jsonld",
		"tests/expand/in06-out.jsonld"
	)
}

#[test]
fn expand_in07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in07-in.jsonld");
	println!("Error if @included value is a string");
	println!("Tests included blocks.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in07-in.jsonld",
		ErrorCode::InvalidIncludedValue
	)
}

#[test]
fn expand_in08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in08-in.jsonld");
	println!("Error if @included value is a value object");
	println!("Tests included blocks.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in08-in.jsonld",
		ErrorCode::InvalidIncludedValue
	)
}

#[test]
fn expand_in09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/in09-in.jsonld");
	println!("Error if @included value is a list object");
	println!("Tests included blocks.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/in09-in.jsonld",
		ErrorCode::InvalidIncludedValue
	)
}

#[test]
fn expand_js01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js01-in.jsonld");
	println!("Expand JSON literal (boolean true)");
	println!("Tests expanding property with @type @json to a JSON literal (boolean true).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js01-in.jsonld",
		"tests/expand/js01-out.jsonld"
	)
}

#[test]
fn expand_js02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js02-in.jsonld");
	println!("Expand JSON literal (boolean false)");
	println!("Tests expanding property with @type @json to a JSON literal (boolean false).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js02-in.jsonld",
		"tests/expand/js02-out.jsonld"
	)
}

#[test]
fn expand_js03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js03-in.jsonld");
	println!("Expand JSON literal (double)");
	println!("Tests expanding property with @type @json to a JSON literal (double).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js03-in.jsonld",
		"tests/expand/js03-out.jsonld"
	)
}

#[test]
fn expand_js04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js04-in.jsonld");
	println!("Expand JSON literal (double-zero)");
	println!("Tests expanding property with @type @json to a JSON literal (double-zero).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js04-in.jsonld",
		"tests/expand/js04-out.jsonld"
	)
}

#[test]
fn expand_js05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js05-in.jsonld");
	println!("Expand JSON literal (integer)");
	println!("Tests expanding property with @type @json to a JSON literal (integer).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js05-in.jsonld",
		"tests/expand/js05-out.jsonld"
	)
}

#[test]
fn expand_js06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js06-in.jsonld");
	println!("Expand JSON literal (object)");
	println!("Tests expanding property with @type @json to a JSON literal (object).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js06-in.jsonld",
		"tests/expand/js06-out.jsonld"
	)
}

#[test]
fn expand_js07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js07-in.jsonld");
	println!("Expand JSON literal (array)");
	println!("Tests expanding property with @type @json to a JSON literal (array).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js07-in.jsonld",
		"tests/expand/js07-out.jsonld"
	)
}

#[test]
fn expand_js08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js08-in.jsonld");
	println!("Expand JSON literal with array canonicalization");
	println!("Tests expanding JSON literal with array canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js08-in.jsonld",
		"tests/expand/js08-out.jsonld"
	)
}

#[test]
fn expand_js09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js09-in.jsonld");
	println!("Transform JSON literal with string canonicalization");
	println!("Tests expanding JSON literal with string canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js09-in.jsonld",
		"tests/expand/js09-out.jsonld"
	)
}

#[test]
fn expand_js10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js10-in.jsonld");
	println!("Expand JSON literal with structural canonicalization");
	println!("Tests expanding JSON literal with structural canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js10-in.jsonld",
		"tests/expand/js10-out.jsonld"
	)
}

#[test]
fn expand_js11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js11-in.jsonld");
	println!("Expand JSON literal with unicode canonicalization");
	println!("Tests expanding JSON literal with unicode canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js11-in.jsonld",
		"tests/expand/js11-out.jsonld"
	)
}

#[test]
fn expand_js12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js12-in.jsonld");
	println!("Expand JSON literal with value canonicalization");
	println!("Tests expanding JSON literal with value canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js12-in.jsonld",
		"tests/expand/js12-out.jsonld"
	)
}

#[test]
fn expand_js13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js13-in.jsonld");
	println!("Expand JSON literal with wierd canonicalization");
	println!("Tests expanding JSON literal with wierd canonicalization.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js13-in.jsonld",
		"tests/expand/js13-out.jsonld"
	)
}

#[test]
fn expand_js14() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js14-in.jsonld");
	println!("Expand JSON literal without expanding contents");
	println!("Tests expanding JSON literal does not expand terms inside json.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js14-in.jsonld",
		"tests/expand/js14-out.jsonld"
	)
}

#[test]
fn expand_js15() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js15-in.jsonld");
	println!("Expand JSON literal aleady in expanded form");
	println!("Tests expanding JSON literal in expanded form.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js15-in.jsonld",
		"tests/expand/js15-out.jsonld"
	)
}

#[test]
fn expand_js16() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js16-in.jsonld");
	println!("Expand JSON literal aleady in expanded form with aliased keys");
	println!("Tests expanding JSON literal in expanded form with aliased keys in value object.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js16-in.jsonld",
		"tests/expand/js16-out.jsonld"
	)
}

#[test]
fn expand_js17() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js17-in.jsonld");
	println!("Expand JSON literal (string)");
	println!("Tests expanding property with @type @json to a JSON literal (string).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js17-in.jsonld",
		"tests/expand/js17-out.jsonld"
	)
}

#[test]
fn expand_js18() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js18-in.jsonld");
	println!("Expand JSON literal (null)");
	println!("Tests expanding property with @type @json to a JSON literal (null).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js18-in.jsonld",
		"tests/expand/js18-out.jsonld"
	)
}

#[test]
fn expand_js19() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js19-in.jsonld");
	println!("Expand JSON literal with aliased @type");
	println!("Tests expanding JSON literal with aliased @type.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js19-in.jsonld",
		"tests/expand/js19-out.jsonld"
	)
}

#[test]
fn expand_js20() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js20-in.jsonld");
	println!("Expand JSON literal with aliased @value");
	println!("Tests expanding JSON literal with aliased @value.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js20-in.jsonld",
		"tests/expand/js20-out.jsonld"
	)
}

#[test]
fn expand_js21() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js21-in.jsonld");
	println!("Expand JSON literal with @context");
	println!("Tests expanding JSON literal with a @context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js21-in.jsonld",
		"tests/expand/js21-out.jsonld"
	)
}

#[test]
fn expand_js22() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js22-in.jsonld");
	println!("Expand JSON literal (null) aleady in expanded form.");
	println!("Tests expanding property with @type @json to a JSON literal (null).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js22-in.jsonld",
		"tests/expand/js22-out.jsonld"
	)
}

#[test]
fn expand_js23() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/js23-in.jsonld");
	println!("Expand JSON literal (empty array).");
	println!("Tests expanding property with @type @json to a JSON literal (empty array).");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/js23-in.jsonld",
		"tests/expand/js23-out.jsonld"
	)
}

#[test]
fn expand_l001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/l001-in.jsonld");
	println!("Language map with null value");
	println!("A language map may have a null value, which is ignored");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/l001-in.jsonld",
		"tests/expand/l001-out.jsonld"
	)
}

#[test]
fn expand_li01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li01-in.jsonld");
	println!("@list containing @list");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li01-in.jsonld",
		"tests/expand/li01-out.jsonld"
	)
}

#[test]
fn expand_li02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li02-in.jsonld");
	println!("@list containing empty @list");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li02-in.jsonld",
		"tests/expand/li02-out.jsonld"
	)
}

#[test]
fn expand_li03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li03-in.jsonld");
	println!("@list containing @list (with coercion)");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li03-in.jsonld",
		"tests/expand/li03-out.jsonld"
	)
}

#[test]
fn expand_li04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li04-in.jsonld");
	println!("@list containing empty @list (with coercion)");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li04-in.jsonld",
		"tests/expand/li04-out.jsonld"
	)
}

#[test]
fn expand_li05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li05-in.jsonld");
	println!("coerced @list containing an array");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li05-in.jsonld",
		"tests/expand/li05-out.jsonld"
	)
}

#[test]
fn expand_li06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li06-in.jsonld");
	println!("coerced @list containing an empty array");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li06-in.jsonld",
		"tests/expand/li06-out.jsonld"
	)
}

#[test]
fn expand_li07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li07-in.jsonld");
	println!("coerced @list containing deep arrays");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li07-in.jsonld",
		"tests/expand/li07-out.jsonld"
	)
}

#[test]
fn expand_li08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li08-in.jsonld");
	println!("coerced @list containing deep empty arrays");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li08-in.jsonld",
		"tests/expand/li08-out.jsonld"
	)
}

#[test]
fn expand_li09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li09-in.jsonld");
	println!("coerced @list containing multiple lists");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li09-in.jsonld",
		"tests/expand/li09-out.jsonld"
	)
}

#[test]
fn expand_li10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/li10-in.jsonld");
	println!("coerced @list containing mixed list values");
	println!("List of lists");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/li10-in.jsonld",
		"tests/expand/li10-out.jsonld"
	)
}

#[test]
fn expand_m001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m001-in.jsonld");
	println!("Adds @id to object not having an @id");
	println!("Expansion using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m001-in.jsonld",
		"tests/expand/m001-out.jsonld"
	)
}

#[test]
fn expand_m002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m002-in.jsonld");
	println!("Retains @id in object already having an @id");
	println!("Expansion using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m002-in.jsonld",
		"tests/expand/m002-out.jsonld"
	)
}

#[test]
fn expand_m003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m003-in.jsonld");
	println!("Adds @type to object not having an @type");
	println!("Expansion using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m003-in.jsonld",
		"tests/expand/m003-out.jsonld"
	)
}

#[test]
fn expand_m004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m004-in.jsonld");
	println!("Prepends @type in object already having an @type");
	println!("Expansion using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m004-in.jsonld",
		"tests/expand/m004-out.jsonld"
	)
}

#[test]
fn expand_m005() {
	let input_url = iri!("http://example.org/");
	println!("Adds expanded @id to object");
	println!("Expansion using @container: @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m005-in.jsonld",
		"tests/expand/m005-out.jsonld"
	)
}

#[test]
fn expand_m006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m006-in.jsonld");
	println!("Adds vocabulary expanded @type to object");
	println!("Expansion using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m006-in.jsonld",
		"tests/expand/m006-out.jsonld"
	)
}

#[test]
fn expand_m007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m007-in.jsonld");
	println!("Adds document expanded @type to object");
	println!("Expansion using @container: @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m007-in.jsonld",
		"tests/expand/m007-out.jsonld"
	)
}

#[test]
fn expand_m008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m008-in.jsonld");
	println!("When type is in a type map");
	println!("scoped context on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m008-in.jsonld",
		"tests/expand/m008-out.jsonld"
	)
}

#[test]
fn expand_m009() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m009-in.jsonld");
	println!("language map with @none");
	println!("index on @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m009-in.jsonld",
		"tests/expand/m009-out.jsonld"
	)
}

#[test]
fn expand_m010() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m010-in.jsonld");
	println!("language map with alias of @none");
	println!("index on @language");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m010-in.jsonld",
		"tests/expand/m010-out.jsonld"
	)
}

#[test]
fn expand_m011() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m011-in.jsonld");
	println!("id map with @none");
	println!("index on @id");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m011-in.jsonld",
		"tests/expand/m011-out.jsonld"
	)
}

#[test]
fn expand_m012() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m012-in.jsonld");
	println!("type map with alias of @none");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m012-in.jsonld",
		"tests/expand/m012-out.jsonld"
	)
}

#[test]
fn expand_m013() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m013-in.jsonld");
	println!("graph index map with @none");
	println!("index on @graph and @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m013-in.jsonld",
		"tests/expand/m013-out.jsonld"
	)
}

#[test]
fn expand_m014() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m014-in.jsonld");
	println!("graph index map with alias @none");
	println!("index on @graph and @index");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m014-in.jsonld",
		"tests/expand/m014-out.jsonld"
	)
}

#[test]
fn expand_m015() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m015-in.jsonld");
	println!("graph id index map with aliased @none");
	println!("index on @graph and @id with @none");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m015-in.jsonld",
		"tests/expand/m015-out.jsonld"
	)
}

#[test]
fn expand_m016() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m016-in.jsonld");
	println!("graph id index map with aliased @none");
	println!("index on @graph and @id with @none");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m016-in.jsonld",
		"tests/expand/m016-out.jsonld"
	)
}

#[test]
fn expand_m017() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m017-in.jsonld");
	println!("string value of type map expands to node reference");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m017-in.jsonld",
		"tests/expand/m017-out.jsonld"
	)
}

#[test]
fn expand_m018() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m018-in.jsonld");
	println!("string value of type map expands to node reference with @type: @id");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m018-in.jsonld",
		"tests/expand/m018-out.jsonld"
	)
}

#[test]
fn expand_m019() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m019-in.jsonld");
	println!("string value of type map expands to node reference with @type: @vocab");
	println!("index on @type");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m019-in.jsonld",
		"tests/expand/m019-out.jsonld"
	)
}

#[test]
fn expand_m020() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/m020-in.jsonld");
	println!("string value of type map must not be a literal");
	println!("index on @type");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/m020-in.jsonld",
		ErrorCode::InvalidTypeMapping
	)
}

#[test]
fn expand_n001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/n001-in.jsonld");
	println!("Expands input using @nest");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/n001-in.jsonld",
		"tests/expand/n001-out.jsonld"
	)
}

#[test]
fn expand_n002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/n002-in.jsonld");
	println!("Expands input using aliased @nest");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/n002-in.jsonld",
		"tests/expand/n002-out.jsonld"
	)
}

#[test]
fn expand_n003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/n003-in.jsonld");
	println!("Appends nested values when property at base and nested");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/n003-in.jsonld",
		"tests/expand/n003-out.jsonld"
	)
}

#[test]
fn expand_n004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/n004-in.jsonld");
	println!("Appends nested values from all @nest aliases");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/n004-in.jsonld",
		"tests/expand/n004-out.jsonld"
	)
}

#[test]
fn expand_n005() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/n005-in.jsonld");
	println!("Nested nested containers");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/n005-in.jsonld",
		"tests/expand/n005-out.jsonld"
	)
}

#[test]
fn expand_n006() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/n006-in.jsonld");
	println!("Arrays of nested values");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/n006-in.jsonld",
		"tests/expand/n006-out.jsonld"
	)
}

#[test]
fn expand_n007() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/n007-in.jsonld");
	println!("A nest of arrays");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/n007-in.jsonld",
		"tests/expand/n007-out.jsonld"
	)
}

#[test]
fn expand_n008() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/n008-in.jsonld");
	println!("Multiple keys may mapping to @type when nesting");
	println!("Expansion using @nest");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/n008-in.jsonld",
		"tests/expand/n008-out.jsonld"
	)
}

#[test]
fn expand_p001() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/p001-in.jsonld");
	println!("@version may be specified after first context");
	println!("If processing mode is not set through API, it is set by the first context containing @version.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/p001-in.jsonld",
		"tests/expand/p001-out.jsonld"
	)
}

#[test]
fn expand_p002() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/p002-in.jsonld");
	println!("@version setting [1.0, 1.1, 1.0]");
	println!("If processing mode is not set through API, it is set by the first context containing @version.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/p002-in.jsonld",
		"tests/expand/p002-out.jsonld"
	)
}

#[test]
fn expand_p003() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/p003-in.jsonld");
	println!("@version setting [1.1, 1.0]");
	println!("If processing mode is not set through API, it is set by the first context containing @version.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/p003-in.jsonld",
		"tests/expand/p003-out.jsonld"
	)
}

#[test]
fn expand_p004() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/p004-in.jsonld");
	println!("@version setting [1.1, 1.0, 1.1]");
	println!("If processing mode is not set through API, it is set by the first context containing @version.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/p004-in.jsonld",
		"tests/expand/p004-out.jsonld"
	)
}

#[test]
fn expand_pi01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi01-in.jsonld");
	println!("error if @version is json-ld-1.0 for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi01-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_pi02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi02-in.jsonld");
	println!("error if @container does not include @index for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi02-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_pi03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi03-in.jsonld");
	println!("error if @index is a keyword for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi03-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_pi04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi04-in.jsonld");
	println!("error if @index is not a string for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi04-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_pi05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi05-in.jsonld");
	println!("error if attempting to add property to value object for property-valued index");
	println!("Expanding index maps where index is a property.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi05-in.jsonld",
		ErrorCode::InvalidValueObject
	)
}

#[test]
fn expand_pi06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi06-in.jsonld");
	println!("property-valued index expands to property value, instead of @index (value)");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi06-in.jsonld",
		"tests/expand/pi06-out.jsonld"
	)
}

#[test]
fn expand_pi07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi07-in.jsonld");
	println!("property-valued index appends to property value, instead of @index (value)");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi07-in.jsonld",
		"tests/expand/pi07-out.jsonld"
	)
}

#[test]
fn expand_pi08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi08-in.jsonld");
	println!("property-valued index expands to property value, instead of @index (node)");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi08-in.jsonld",
		"tests/expand/pi08-out.jsonld"
	)
}

#[test]
fn expand_pi09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi09-in.jsonld");
	println!("property-valued index appends to property value, instead of @index (node)");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi09-in.jsonld",
		"tests/expand/pi09-out.jsonld"
	)
}

#[test]
fn expand_pi10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi10-in.jsonld");
	println!("property-valued index does not output property for @none");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi10-in.jsonld",
		"tests/expand/pi10-out.jsonld"
	)
}

#[test]
fn expand_pi11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pi11-in.jsonld");
	println!("property-valued index adds property to graph object");
	println!("Expanding index maps where index is a property.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pi11-in.jsonld",
		"tests/expand/pi11-out.jsonld"
	)
}

#[test]
fn expand_pr01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr01-in.jsonld");
	println!("Protect a term");
	println!("Check error when overriding a protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr01-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr02-in.jsonld");
	println!("Set a term to not be protected");
	println!("A term with @protected: false is not protected.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr02-in.jsonld",
		"tests/expand/pr02-out.jsonld"
	)
}

#[test]
fn expand_pr03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr03-in.jsonld");
	println!("Protect all terms in context");
	println!("A protected context protects all term definitions.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr03-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr04() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr04-in.jsonld");
	println!("Do not protect term with @protected: false");
	println!("A protected context does not protect terms with @protected: false.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr04-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr05-in.jsonld");
	println!("Clear active context with protected terms from an embedded context");
	println!("The Active context be set to null from an embedded context.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr05-in.jsonld",
		ErrorCode::InvalidContextNullification
	)
}

#[test]
fn expand_pr06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr06-in.jsonld");
	println!("Clear active context of protected terms from a term.");
	println!("The Active context may be set to null from a scoped context of a term.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr06-in.jsonld",
		"tests/expand/pr06-out.jsonld"
	)
}

#[test]
fn expand_pr08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr08-in.jsonld");
	println!("Term with protected scoped context.");
	println!("A scoped context can protect terms.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr08-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr09-in.jsonld");
	println!("Attempt to redefine term in other protected context.");
	println!("A protected term cannot redefine another protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr09-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr10-in.jsonld");
	println!("Simple protected and unprotected terms.");
	println!("Simple protected and unprotected terms.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr10-in.jsonld",
		"tests/expand/pr10-out.jsonld"
	)
}

#[test]
fn expand_pr11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr11-in.jsonld");
	println!("Fail to override protected term.");
	println!("Fail to override protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr11-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr12-in.jsonld");
	println!("Scoped context fail to override protected term.");
	println!("Scoped context fail to override protected term.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr12-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr13-in.jsonld");
	println!("Override unprotected term.");
	println!("Override unprotected term.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr13-in.jsonld",
		"tests/expand/pr13-out.jsonld"
	)
}

#[test]
fn expand_pr14() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr14-in.jsonld");
	println!("Clear protection with null context.");
	println!("Clear protection with null context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr14-in.jsonld",
		"tests/expand/pr14-out.jsonld"
	)
}

#[test]
fn expand_pr15() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr15-in.jsonld");
	println!("Clear protection with array with null context");
	println!("Clear protection with array with null context");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr15-in.jsonld",
		"tests/expand/pr15-out.jsonld"
	)
}

#[test]
fn expand_pr16() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr16-in.jsonld");
	println!("Override protected terms after null.");
	println!("Override protected terms after null.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr16-in.jsonld",
		"tests/expand/pr16-out.jsonld"
	)
}

#[test]
fn expand_pr17() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr17-in.jsonld");
	println!("Fail to override protected terms with type.");
	println!("Fail to override protected terms with type.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr17-in.jsonld",
		ErrorCode::InvalidContextNullification
	)
}

#[test]
fn expand_pr18() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr18-in.jsonld");
	println!("Fail to override protected terms with type+null+ctx.");
	println!("Fail to override protected terms with type+null+ctx.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr18-in.jsonld",
		ErrorCode::InvalidContextNullification
	)
}

#[test]
fn expand_pr19() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr19-in.jsonld");
	println!("Mix of protected and unprotected terms.");
	println!("Mix of protected and unprotected terms.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr19-in.jsonld",
		"tests/expand/pr19-out.jsonld"
	)
}

#[test]
fn expand_pr20() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr20-in.jsonld");
	println!("Fail with mix of protected and unprotected terms with type+null+ctx.");
	println!("Fail with mix of protected and unprotected terms with type+null+ctx.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr20-in.jsonld",
		ErrorCode::InvalidContextNullification
	)
}

#[test]
fn expand_pr21() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr21-in.jsonld");
	println!("Fail with mix of protected and unprotected terms with type+null.");
	println!("Fail with mix of protected and unprotected terms with type+null.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr21-in.jsonld",
		ErrorCode::InvalidContextNullification
	)
}

#[test]
fn expand_pr22() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr22-in.jsonld");
	println!("Check legal overriding of type-scoped protected term from nested node.");
	println!("Check legal overriding of type-scoped protected term from nested node.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr22-in.jsonld",
		"tests/expand/pr22-out.jsonld"
	)
}

#[test]
fn expand_pr23() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr23-in.jsonld");
	println!("Allows redefinition of protected alias term with same definition.");
	println!("Allows redefinition of protected alias term with same definition.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr23-in.jsonld",
		"tests/expand/pr23-out.jsonld"
	)
}

#[test]
fn expand_pr24() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr24-in.jsonld");
	println!("Allows redefinition of protected prefix term with same definition.");
	println!("Allows redefinition of protected prefix term with same definition.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr24-in.jsonld",
		"tests/expand/pr24-out.jsonld"
	)
}

#[test]
fn expand_pr25() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr25-in.jsonld");
	println!("Allows redefinition of terms with scoped contexts using same definitions.");
	println!("Allows redefinition of terms with scoped contexts using same definitions.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr25-in.jsonld",
		"tests/expand/pr25-out.jsonld"
	)
}

#[test]
fn expand_pr26() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr26-in.jsonld");
	println!("Fails on redefinition of terms with scoped contexts using different definitions.");
	println!("Fails on redefinition of terms with scoped contexts using different definitions.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr26-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr27() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr27-in.jsonld");
	println!("Allows redefinition of protected alias term with same definition modulo protected flag.");
	println!("Allows redefinition of protected alias term with same definition modulo protected flag.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr27-in.jsonld",
		"tests/expand/pr27-out.jsonld"
	)
}

#[test]
fn expand_pr28() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr28-in.jsonld");
	println!("Fails if trying to redefine a protected null term.");
	println!("A protected term with a null IRI mapping cannot be redefined.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr28-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr29() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr29-in.jsonld");
	println!("Does not expand a Compact IRI using a non-prefix term.");
	println!("Expansion of Compact IRIs considers if the term can be used as a prefix.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr29-in.jsonld",
		"tests/expand/pr29-out.jsonld"
	)
}

#[test]
fn expand_pr30() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr30-in.jsonld");
	println!("Keywords may be protected.");
	println!("Keywords may not be redefined other than to protect them.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr30-in.jsonld",
		"tests/expand/pr30-out.jsonld"
	)
}

#[test]
fn expand_pr31() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr31-in.jsonld");
	println!("Protected keyword aliases cannot be overridden.");
	println!("Keywords may not be redefined other than to protect them.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr31-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr32() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr32-in.jsonld");
	println!("Protected @type cannot be overridden.");
	println!("Keywords may not be redefined other than to protect them.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr32-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_pr33() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr33-in.jsonld");
	println!("Fails if trying to declare a keyword alias as prefix.");
	println!("Keyword aliases can not be used as prefixes.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr33-in.jsonld",
		ErrorCode::InvalidTermDefinition
	)
}

#[test]
fn expand_pr34() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr34-in.jsonld");
	println!("Ignores a non-keyword term starting with '@'");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr34-in.jsonld",
		"tests/expand/pr34-out.jsonld"
	)
}

#[test]
fn expand_pr35() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr35-in.jsonld");
	println!("Ignores a non-keyword term starting with '@' (with @vocab)");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr35-in.jsonld",
		"tests/expand/pr35-out.jsonld"
	)
}

#[test]
fn expand_pr36() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr36-in.jsonld");
	println!("Ignores a term mapping to a value in the form of a keyword.");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr36-in.jsonld",
		"tests/expand/pr36-out.jsonld"
	)
}

#[test]
fn expand_pr37() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr37-in.jsonld");
	println!("Ignores a term mapping to a value in the form of a keyword (with @vocab).");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr37-in.jsonld",
		"tests/expand/pr37-out.jsonld"
	)
}

#[test]
fn expand_pr38() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr38-in.jsonld");
	println!("Ignores a term mapping to a value in the form of a keyword (@reverse).");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr38-in.jsonld",
		"tests/expand/pr38-out.jsonld"
	)
}

#[test]
fn expand_pr39() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr39-in.jsonld");
	println!("Ignores a term mapping to a value in the form of a keyword (@reverse with @vocab).");
	println!("Terms in the form of a keyword, which are not keywords, are ignored.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr39-in.jsonld",
		"tests/expand/pr39-out.jsonld"
	)
}

#[test]
fn expand_pr40() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/pr40-in.jsonld");
	println!("Protected terms and property-scoped contexts");
	println!("Check overriding of protected term from property-scoped context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/pr40-in.jsonld",
		"tests/expand/pr40-out.jsonld"
	)
}

#[test]
fn expand_so01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so01-in.jsonld");
	println!("@import is invalid in 1.0.");
	println!("@import is invalid in 1.0.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so01-in.jsonld",
		ErrorCode::InvalidContextEntry
	)
}

#[test]
fn expand_so02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so02-in.jsonld");
	println!("@import must be a string");
	println!("@import must be a string.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so02-in.jsonld",
		ErrorCode::InvalidImportValue
	)
}

#[test]
fn expand_so03() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so03-in.jsonld");
	println!("@import overflow");
	println!("Processors must detect source contexts that include @import.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so03-in.jsonld",
		ErrorCode::InvalidContextEntry
	)
}

#[test]
fn expand_so05() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so05-in.jsonld");
	println!("@propagate: true on type-scoped context with @import");
	println!("type-scoped context with @propagate: true survive node-objects (with @import)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so05-in.jsonld",
		"tests/expand/so05-out.jsonld"
	)
}

#[test]
fn expand_so06() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so06-in.jsonld");
	println!("@propagate: false on property-scoped context with @import");
	println!("property-scoped context with @propagate: false do not survive node-objects (with @import)");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so06-in.jsonld",
		"tests/expand/so06-out.jsonld"
	)
}

#[test]
fn expand_so07() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so07-in.jsonld");
	println!("Protect all terms in sourced context");
	println!("A protected context protects all term definitions.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so07-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_so08() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so08-in.jsonld");
	println!("Override term defined in sourced context");
	println!("The containing context is merged into the source context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so08-in.jsonld",
		"tests/expand/so08-out.jsonld"
	)
}

#[test]
fn expand_so09() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so09-in.jsonld");
	println!("Override @vocab defined in sourced context");
	println!("The containing context is merged into the source context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so09-in.jsonld",
		"tests/expand/so09-out.jsonld"
	)
}

#[test]
fn expand_so10() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so10-in.jsonld");
	println!("Protect terms in sourced context");
	println!("The containing context is merged into the source context.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so10-in.jsonld",
		ErrorCode::ProtectedTermRedefinition
	)
}

#[test]
fn expand_so11() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so11-in.jsonld");
	println!("Override protected terms in sourced context");
	println!("The containing context is merged into the source context.");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so11-in.jsonld",
		"tests/expand/so11-out.jsonld"
	)
}

#[test]
fn expand_so12() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so12-in.jsonld");
	println!("@import may not be used in an imported context.");
	println!("@import only valid within a term definition.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so12-in.jsonld",
		ErrorCode::InvalidContextEntry
	)
}

#[test]
fn expand_so13() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/so13-in.jsonld");
	println!("@import can only reference a single context");
	println!("@import can only reference a single context.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/so13-in.jsonld",
		ErrorCode::InvalidRemoteContext
	)
}

#[test]
fn expand_tn01() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/tn01-in.jsonld");
	println!("@type: @none is illegal in 1.0.");
	println!("@type: @none is illegal in json-ld-1.0.");
	negative_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/tn01-in.jsonld",
		ErrorCode::InvalidTypeMapping
	)
}

#[test]
fn expand_tn02() {
	let input_url = iri!("https://w3c.github.io/json-ld-api/tests/expand/tn02-in.jsonld");
	println!("@type: @none expands strings as value objects");
	println!("@type: @none leaves inputs other than strings alone");
	positive_test(
		Options {
			processing_mode: ProcessingMode::JsonLd1_1,
			expand_context: None,
			ordered: false
		},
		input_url,
		"tests/expand/tn02-in.jsonld",
		"tests/expand/tn02-out.jsonld"
	)
}

