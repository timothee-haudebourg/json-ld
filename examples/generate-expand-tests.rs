#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate tokio;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use tokio::runtime::Runtime;
use iref::Iri;
use json_ld::{
	context::{
		ActiveContext,
		JsonLdContextLoader,
		Context,
		load_remote_json_ld_document
	},
	Object,
	Node,
	Value,
	Property,
	VocabId,
	Key
};

const URL: &str = "https://w3c.github.io/json-ld-api/tests/expand-manifest.jsonld";
const VERBOSITY: usize = 2;

const MF_NAME: Iri<'static> = iri!("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#name");
const MF_ENTRIES: Iri<'static> = iri!("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#entries");
const MF_ACTION: Iri<'static> = iri!("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#action");
const MF_RESULT: Iri<'static> = iri!("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#result");

const RDFS_COMMENT: Iri<'static> = iri!("http://www.w3.org/2000/01/rdf-schema#comment");

const VOCAB_POSITIVE_EVAL_TEST: Iri<'static> = iri!("https://w3c.github.io/json-ld-api/tests/vocab#PositiveEvaluationTest");
const VOCAB_NEGATIVE_EVAL_TEST: Iri<'static> = iri!("https://w3c.github.io/json-ld-api/tests/vocab#NegativeEvaluationTest");

const VOCAB_OPTION: Iri<'static> = iri!("https://w3c.github.io/json-ld-api/tests/vocab#option");
const VOCAB_SPEC_VERSION: Iri<'static> = iri!("https://w3c.github.io/json-ld-api/tests/vocab#specVersion");

/// Vocabulary of the test manifest
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vocab {
	Name,
	Entries,
	Action,
	Result,
	PositiveEvalTest,
	NegativeEvalTest,
	Comment,
	Option,
	SpecVersion
}

impl json_ld::Vocab for Vocab {
	fn from_iri(iri: Iri) -> Option<Vocab> {
		use Vocab::*;
		match iri {
			_ if iri == MF_NAME => Some(Name),
			_ if iri == MF_ENTRIES => Some(Entries),
			_ if iri == MF_ACTION => Some(Action),
			_ if iri == MF_RESULT => Some(Result),
			_ if iri == VOCAB_POSITIVE_EVAL_TEST => Some(PositiveEvalTest),
			_ if iri == VOCAB_NEGATIVE_EVAL_TEST => Some(NegativeEvalTest),
			_ if iri == VOCAB_OPTION => Some(Option),
			_ if iri == VOCAB_SPEC_VERSION => Some(SpecVersion),
			_ if iri == RDFS_COMMENT => Some(Comment),
			_ => None
		}
	}

	fn iri(&self) -> Iri {
		use Vocab::*;
		match self {
			Name => MF_NAME,
			Entries => MF_ENTRIES,
			Action => MF_ACTION,
			Result => MF_RESULT,
			PositiveEvalTest => VOCAB_POSITIVE_EVAL_TEST,
			NegativeEvalTest => VOCAB_NEGATIVE_EVAL_TEST,
			Option => VOCAB_OPTION,
			SpecVersion => VOCAB_SPEC_VERSION,
			Comment => RDFS_COMMENT
		}
	}
}

pub type Id = VocabId<Vocab>;

const NAME: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::Name));
const ENTRIES: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::Entries));
const ACTION: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::Action));
const RESULT: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::Result));
// const POSITIVE: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::PositiveEvalTest));
// const NEGATIVE: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::NegativeEvalTest));
const OPTION: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::Option));
const SPEC_VERSION: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::SpecVersion));
const COMMENT: &'static Property<Id> = &Property::Id(VocabId::Id(Vocab::Comment));

fn main() {
	let destination = std::env::args().nth(1).expect("no destination given");
	let target = Path::new(destination.as_str());
	stderrlog::new().verbosity(VERBOSITY).init().unwrap();

	let mut runtime = Runtime::new().unwrap();

	let url = Iri::new(URL).unwrap();

	let mut loader = JsonLdContextLoader::new();

	let doc = runtime.block_on(load_remote_json_ld_document(url))
		.expect("unable to load the test suite");

	let active_context: Context<Id> = Context::new(url, url);
	let expanded_doc = runtime.block_on(json_ld::expand(&active_context, None, &doc, Some(url), &mut loader))
		.expect("expansion failed");

	println!("#![feature(proc_macro_hygiene)]

extern crate tokio;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use std::fs::File;
use std::io::{{Read, BufReader}};
use tokio::runtime::Runtime;
use iref::{{Iri, IriBuf}};
use json_ld::{{
	context::{{
		ActiveContext,
		JsonLdContextLoader,
		Context,
	}},
	AsJson,
	json_ld_eq
}};

fn positive_test(input_url: Iri, input_filename: &str, output_filename: &str) {{
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

	let input_context: Context<IriBuf> = Context::new(input_url, input_url);
	let result = runtime.block_on(json_ld::expand(&input_context, None, &input, Some(input_url), &mut loader)).unwrap();

	let result_json = result.as_json();
	let success = json_ld_eq(&result_json, &output);

	if !success {{
		println!(\"output=\n{{}}\", result_json.pretty(2));
		println!(\"\nexpected=\n{{}}\", output.pretty(2));
	}}

	assert!(success)
}}
");

	for item in &expanded_doc {
		// println!("{}", PrettyPrint::new(item));
		if let Object::Node(item, _) = item {
			for entries in item.get(ENTRIES) {
				if let Object::Value(Value::List(entries), _) = entries {
					for entry in entries {
						if let Object::Node(entry, _) = entry {
							generate_test(&target, &mut runtime, entry);
						}
					}
				}
			}
		}
	}

	info!("done.");
}

fn func_name(id: &str) -> String {
	let mut name = "expand_".to_string();

	for c in id.chars() {
		match c {
			'.' | '-' => break,
			_ => name.push(c)
		}
	}

	name
}

fn generate_test(target: &Path, runtime: &mut Runtime, entry: &Node<Id>) {
	let name = entry.get(NAME).next().unwrap().as_str().unwrap();
	let url = entry.get(ACTION).next().unwrap().as_iri().unwrap();

	let mut input_filename: PathBuf = target.into();
	input_filename.push(url.path().file_name().unwrap());

	if !input_filename.exists() {
		let doc = runtime.block_on(load_remote_json_ld_document(url))
			.expect("unable to load test document");

		info!("writing to {}", input_filename.to_str().unwrap());
		let mut input_file = File::create(&input_filename).unwrap();
		input_file.write_all(doc.pretty(2).as_bytes()).unwrap();
	}

	let func_name = func_name(url.path().file_name().unwrap());

	for option in entry.get(OPTION) {
		if let Object::Node(option, _) = option {
			for spec_version in option.get(SPEC_VERSION) {
				if let Some(spec_version) = spec_version.as_str() {
					if spec_version != "json-ld-1.1" {
						info!("skipping {} test {}", spec_version, url);
						return
					}
				}
			}
		}
	}

	if entry.types().contains(&Key::Prop(Property::Id(VocabId::Id(Vocab::PositiveEvalTest)))) {
		let output_url = entry.get(RESULT).next().unwrap().as_iri().unwrap();

		let mut output_filename: PathBuf = target.into();
		output_filename.push(output_url.path().file_name().unwrap());

		if !output_filename.exists() {
			let output_doc = runtime.block_on(load_remote_json_ld_document(output_url))
				.expect("unable to load result document");

			info!("writing to {}", output_filename.to_str().unwrap());
			let mut output_file = File::create(&output_filename).unwrap();
			output_file.write_all(output_doc.pretty(2).as_bytes()).unwrap();
		}

		let mut comments = String::new();
		for comment in entry.get(COMMENT) {
			comments += format!("\n\tprintln!(\"{}\");", comment.as_str().unwrap()).as_str()
		}

		println!("#[test]
fn {}() {{
	let input_url = iri!(\"{}\");
	println!(\"{}\");{}
	positive_test(input_url, \"{}\", \"{}\")
}}
",
			func_name, url, name, comments, input_filename.to_str().unwrap(), output_filename.to_str().unwrap()
		);
	} else if entry.types().contains(&Key::Prop(Property::Id(VocabId::Id(Vocab::NegativeEvalTest)))) {
		warn!("ignoring negative example {}", url);
	} else {
		panic!("cannot decide how to evaluate test result")
	}
}
