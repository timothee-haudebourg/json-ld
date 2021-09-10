//! This example shows how to use the `reqwest::Loader` to download remote documents by
//! enabeling the `reqwest-loader` feature.
//#![feature(proc_macro_hygiene)]

extern crate iref;
extern crate tokio;
#[macro_use]
extern crate iref_enum;
#[macro_use]
extern crate static_iref;
extern crate json_ld;

use json_ld::{reqwest::Loader, Document, JsonContext, Lexicon, Object};

/// Vocabulary of the test manifest
#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
#[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
pub enum Vocab {
    #[iri("rdfs:comment")]
    Comment,
    #[iri("manifest:name")]
    Name,
    #[iri("manifest:entries")]
    Entries,
    #[iri("manifest:action")]
    Action,
    #[iri("manifest:result")]
    Result,
}

type Id = Lexicon<Vocab>;

#[tokio::main]
async fn main() {
    let mut loader = Loader::new();

    // Create the initial context.
    let context: JsonContext<Id> = JsonContext::new(None);

    // The JSON-LD document to expand.
    let doc = loader
        .load(iri!(
            "https://w3c.github.io/json-ld-api/tests/expand-manifest.jsonld"
        ))
        .await
        .unwrap();

    // Expansion.
    let expanded_doc = doc.expand(&context, &mut loader).await.unwrap();

    // Iterate through the expanded objects.
    for object in expanded_doc {
        if let Object::Node(node) = object.as_ref() {
            for entries in node.get(Vocab::Entries) {
                if let Object::List(entries) = entries.as_ref() {
                    for entry in entries {
                        if let Object::Node(entry) = entry.as_ref() {
                            let name = entry.get(Vocab::Name).next().unwrap().as_str().unwrap();
                            println!("test name: {}", name);
                        }
                    }
                }
            }
        }
    }
}
