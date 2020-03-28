#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate tokio;
extern crate iref;
extern crate json_ld;

use tokio::runtime::Runtime;
use iref::Iri;
use json_ld::{
	context::{
		ActiveContext,
		ContextLoader,
		JsonLdContextLoader,
		Context,
		load_remote_json_ld_document
	},
	PrettyPrint
};

const URL: &str = "https://w3c.github.io/json-ld-api/tests/expand-manifest.jsonld";
const VERBOSITY: usize = 2;

#[test]
pub fn expand() {
	stderrlog::new().verbosity(VERBOSITY).init().unwrap();

	let mut runtime = Runtime::new().unwrap();

	let url = Iri::new(URL).unwrap();

	let mut loader = JsonLdContextLoader::new();

	let doc = runtime.block_on(load_remote_json_ld_document(url))
		.expect("unable to load the test suite");

	let active_context: Context<json_ld::DefaultKey> = Context::new(url, url);
	let expanded_doc = runtime.block_on(json_ld::expand(&active_context, None, &doc, Some(url), &mut loader, false, false))
		.expect("expansion failed");

	if let Some(doc) = expanded_doc {
		for item in &doc {
			println!("{}", PrettyPrint::new(item));
		}
	}
}
