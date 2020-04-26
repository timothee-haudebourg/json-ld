//! This simple example shows how to use an enum type (`Foaf`) to replace IRIs (`IriBuf`) as
//! identifiers in the expanded document.
//! This can reduce the cost of storing and comparing actual IRIs.
//! The only constraint is to define convertions between the type and `Iri`. It also must implement
//! `Clone`, `PartialEq`, `Eq` and `Hash`.
//! Since the `Foaf` type does not cover every possible IRIs, we use the `Lexicon` wrapper to
//! cover the rest. The identifer type (`Id`) is then `Lexicon<Foaf>`.
//! See the `custom-ids-iref-enum.rs` example to see how to simplify the definition of `Foaf` using
//! the `iref-enum` crate.
#![feature(proc_macro_hygiene)]

extern crate async_std;
extern crate iref;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use std::convert::TryFrom;
use iref::Iri;
use json_ld::{
	JsonContext,
	NoLoader,
	Document,
	Object,
	Lexicon
};

// Parts of the FOAF vocabulary will need.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Foaf {
	Name,
	Mbox
}

// Convertion from Iri
impl<'a> TryFrom<Iri<'a>> for Foaf {
	type Error = ();

	fn try_from(iri: Iri<'a>) -> Result<Foaf, ()> {
		match iri {
			_ if iri == iri!("http://xmlns.com/foaf/0.1/name") => Ok(Foaf::Name),
			_ if iri == iri!("http://xmlns.com/foaf/0.1/mbox") => Ok(Foaf::Mbox),
			_ => Err(())
		}
	}
}

impl<'a> From<&'a Foaf> for Iri<'a> {
	fn from(term: &'a Foaf) -> Iri<'a> {
		match term {
			Foaf::Name => iri!("http://xmlns.com/foaf/0.1/name"),
			Foaf::Mbox => iri!("http://xmlns.com/foaf/0.1/mbox")
		}
	}
}

type Id = Lexicon<Foaf>;

#[async_std::main]
async fn main() {
	// Create the initial context.
	let context: JsonContext<Id> = JsonContext::new(None);

	// The JSON-LD document to expand.
	let doc = json::parse(r#"
		{
			"@context": {
				"name": "http://xmlns.com/foaf/0.1/name",
				"email": "http://xmlns.com/foaf/0.1/mbox"
			},
			"@id": "timothee.haudebourg.net",
			"name": "Timothée Haudebourg",
			"email": "author@haudebourg.net"
		}
	"#).unwrap();

	// Expansion.
	let expanded_doc = doc.expand(&context, &mut NoLoader).await.unwrap();

	// Iterate through the expanded objects.
	for object in expanded_doc {
		if let Object::Node(node) = object.as_ref() {
			println!("node: {}", node.id().unwrap()); // print the `@id`
			for name in node.get(Foaf::Name) { // <- Note how we can directly use `Foaf` here.
				println!("name: {}", name.as_str().unwrap());
			}

			for name in node.get(Foaf::Mbox) { // <- Note how we can directly use `Foaf` here.
				println!("email: {}", name.as_str().unwrap());
			}
		}
	}
}
