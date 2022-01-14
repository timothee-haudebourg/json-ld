//! This simple example shows how to expand a document using the `Document::expand` method.

extern crate async_std;
extern crate iref;
extern crate json_ld;

use iref::IriBuf;
use json_ld::{context, Document, NoLoader, Object, Reference};
use serde_json::Value;

#[async_std::main]
async fn main() {
	// The JSON-LD document to expand.
	let doc: Value = serde_json::from_str(
		r#"
		{
			"@context": {
				"name": "http://xmlns.com/foaf/0.1/name"
			},
			"@id": "https://www.rust-lang.org",
			"name": "Rust Programming Language"
		}
	"#,
	)
	.unwrap();

	// JSON document loader.
	let mut loader = NoLoader::<Value>::new();

	// Expansion.
	let expanded_doc = doc
		.expand::<context::Json<Value>, _>(&mut loader)
		.await
		.unwrap();

	// Reference to the `name` property.
	let name_property = Reference::Id(IriBuf::new("http://xmlns.com/foaf/0.1/name").unwrap());

	// Iterate through the expanded objects.
	for object in expanded_doc {
		if let Object::Node(node) = object.as_ref() {
			println!("node: {}", node.id().unwrap()); // print the `@id`
			for name in node.get(&name_property) {
				// get the names.
				println!("name: {}", name.as_str().unwrap());
			}
		}
	}
}
