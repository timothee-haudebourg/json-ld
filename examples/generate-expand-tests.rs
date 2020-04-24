#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate tokio;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use std::convert::TryInto;
use tokio::runtime::Runtime;
use iref::Iri;
use json_ld::{
	ErrorCode,
	object::*,
	Reference,
	VocabId,
	ProcessingMode,
	Document,
	Context,
	context::JsonContext,
	Loader,
	FsLoader
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
const VOCAB_PROCESSING_MODE: Iri<'static> = iri!("https://w3c.github.io/json-ld-api/tests/vocab#processingMode");
const VOCAB_EXPAND_CONTEXT: Iri<'static> = iri!("https://w3c.github.io/json-ld-api/tests/vocab#expandContext");
const VOCAB_BASE: Iri<'static> = iri!("https://w3c.github.io/json-ld-api/tests/vocab#base");

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
	SpecVersion,
	ProcessingMode,
	ExpandContext,
	Base
}

impl json_ld::Vocab for Vocab {
	fn from_iri(iri: Iri) -> Option<Vocab> {
		use Vocab::*;
		match iri {
			_ if iri == RDFS_COMMENT => Some(Comment),
			_ if iri == MF_NAME => Some(Name),
			_ if iri == MF_ENTRIES => Some(Entries),
			_ if iri == MF_ACTION => Some(Action),
			_ if iri == MF_RESULT => Some(Result),
			_ if iri == VOCAB_POSITIVE_EVAL_TEST => Some(PositiveEvalTest),
			_ if iri == VOCAB_NEGATIVE_EVAL_TEST => Some(NegativeEvalTest),
			_ if iri == VOCAB_OPTION => Some(Option),
			_ if iri == VOCAB_SPEC_VERSION => Some(SpecVersion),
			_ if iri == VOCAB_PROCESSING_MODE => Some(ProcessingMode),
			_ if iri == VOCAB_EXPAND_CONTEXT => Some(ExpandContext),
			_ if iri == VOCAB_BASE => Some(Base),
			_ => None
		}
	}

	fn as_iri(&self) -> Iri {
		use Vocab::*;
		match self {
			Comment => RDFS_COMMENT,
			Name => MF_NAME,
			Entries => MF_ENTRIES,
			Action => MF_ACTION,
			Result => MF_RESULT,
			PositiveEvalTest => VOCAB_POSITIVE_EVAL_TEST,
			NegativeEvalTest => VOCAB_NEGATIVE_EVAL_TEST,
			Option => VOCAB_OPTION,
			SpecVersion => VOCAB_SPEC_VERSION,
			ProcessingMode => VOCAB_PROCESSING_MODE,
			ExpandContext => VOCAB_EXPAND_CONTEXT,
			Base => VOCAB_BASE
		}
	}
}

pub type Id = VocabId<Vocab>;

const COMMENT: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::Comment));
const NAME: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::Name));
const ENTRIES: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::Entries));
const ACTION: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::Action));
const RESULT: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::Result));
const POSITIVE: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::PositiveEvalTest));
const NEGATIVE: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::NegativeEvalTest));
const OPTION: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::Option));
const SPEC_VERSION: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::SpecVersion));
const PROCESSING_MODE: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::ProcessingMode));
const EXPAND_CONTEXT: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::ExpandContext));
const BASE: &'static Reference<Id> = &Reference::Id(VocabId::Id(Vocab::Base));

fn main() {
	stderrlog::new().verbosity(VERBOSITY).init().unwrap();

	let mut runtime = Runtime::new().unwrap();
	let url = Iri::new(URL).unwrap();

	let mut loader = FsLoader::new();
	loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");

	let doc = runtime.block_on(loader.load(url))
		.expect("unable to load the test suite");

	let context: JsonContext<Id> = JsonContext::new(url, url);
	let expanded_doc = runtime.block_on(doc.expand(&context, &mut loader))
		.expect("expansion failed");

	println!(include_str!("template/header.rs"));

	for item in &expanded_doc {
		// println!("{}", PrettyPrint::new(item));
		if let Object::Node(item) = item.as_ref() {
			for entries in item.get(ENTRIES) {
				if let Object::List(entries) = entries.as_ref() {
					for entry in entries {
						if let Object::Node(entry) = entry.as_ref() {
							generate_test(entry);
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

fn generate_test(entry: &Node<Id>) {
	let name = entry.get(NAME).next().unwrap().as_str().unwrap();
	let url = entry.get(ACTION).next().unwrap().as_iri().unwrap();
	let mut base_url = url;
	let func_name = func_name(url.path().file_name().unwrap());

	let mut processing_mode = ProcessingMode::JsonLd1_1;
	let mut context_url = "None".to_string();

	for option in entry.get(OPTION) {
		if let Object::Node(option) = option.as_ref() {
			for spec_version in option.get(SPEC_VERSION) {
				if let Some(spec_version) = spec_version.as_str() {
					if spec_version != "json-ld-1.1" {
						info!("skipping {} test {}", spec_version, url);
						return
					}
				}
			}

			for mode in option.get(PROCESSING_MODE) {
				processing_mode = mode.as_str().unwrap().try_into().unwrap();
			}

			for expand_context in option.get(EXPAND_CONTEXT) {
				if let Some(url) = expand_context.as_iri() {
					context_url = format!("Some(iri!(\"{}\"))", url)
				}
			}

			for base in option.get(BASE) {
				if let Some(url) = base.as_iri() {
					base_url = url
				}
			}
		}
	}

	let mut comments = String::new();
	for comment in entry.get(COMMENT) {
		comments += format!("\n\tprintln!(\"{}\");", comment.as_str().unwrap()).as_str()
	}

	if entry.has_type(POSITIVE) {
		let output_url = entry.get(RESULT).next().unwrap().as_iri().unwrap();

		println!(
			include_str!("template/test-positive.rs"),
			func_name,
			url,
			base_url,
			output_url,
			name,
			comments,
			processing_mode,
			context_url
		);
	} else if entry.has_type(NEGATIVE) {
		let error_code: ErrorCode = entry.get(RESULT).next().unwrap().as_str().unwrap().try_into().unwrap();

		println!(
			include_str!("template/test-negative.rs"),
			func_name,
			url,
			base_url,
			name,
			comments,
			processing_mode,
			context_url,
			error_code
		);
	} else {
		panic!("cannot decide how to evaluate test result")
	}
}
