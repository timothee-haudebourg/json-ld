//! This simple example shows how to compact a document using the `Document::compact` method.
use json_ld::{
	JsonContext,
	NoLoader,
	Document,
	Compact
};

#[async_std::main]
async fn main() {
	// Create the initial context.
	let context: JsonContext = JsonContext::new(None);

	// The JSON-LD document to expand.
	let doc = json::parse(r#"
		{
			"@context": {
				"name": "http://xmlns.com/foaf/0.1/name"
			},
			"@id": "https://www.rust-lang.org",
			"name": "Rust Programming Language"
		}
	"#).unwrap();

	// Expansion.
	let compact_doc = doc.compact(&context, &mut NoLoader).await.unwrap();
	println!("{}", compact_doc);
}
