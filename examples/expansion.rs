//! This simple example shows how to expand a document using the `Document::expand` method.

extern crate async_std;
extern crate iref;
extern crate json_ld;

use iref::IriBuf;
use json_ld::{Document, JsonContext, NoLoader, Object, Reference};

#[async_std::main]
async fn main() {
	// The JSON-LD document to expand.
	let doc = json::parse(
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

	// Expansion.
	let expanded_doc = doc.expand::<JsonContext, _>(&mut NoLoader).await.unwrap();

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
