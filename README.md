# A JSON-LD implementation for Rust

<table><tr>
	<td><a href="https://docs.rs/json-ld">Documentation</a></td>
	<td><a href="https://crates.io/crates/json-ld">Crate informations</a></td>
	<td><a href="https://github.com/timothee-haudebourg/json-ld">Repository</a></td>
</tr></table>

This crate is a Rust implementation of the
[JSON-LD](https://www.w3.org/TR/json-ld/)
data interchange format.

[Linked Data (LD)](https://www.w3.org/standards/semanticweb/data)
is a [World Wide Web Consortium (W3C)](https://www.w3.org/)
initiative built upon standard Web technologies to create an
interrelated network of datasets across the Web.
The [JavaScript Object Notation (JSON)](https://tools.ietf.org/html/rfc7159) is
a widely used, simple, unstructured data serialization format to describe
data objects in a human readable way.
JSON-LD brings these two technologies together, adding semantics to JSON
to create a lightweight data serialization format that can organize data and
help Web applications to inter-operate at a large scale.

This crate aims to provide a set of types to build and process expanded
JSON-LD documents.
With the help of the [`json`](https://crates.io/crates/json)
crate it can also expand, compact and flatten JSON-LD documents of any kind.

### Basic Usage

JSON-LD documents are represented by the `Document` trait,
implemented for instance by the `json::JsonValue` type.
This trait represent *compact* JSON-LD documents that must be expanded in order
to be processed.
Expansion is done asynchronously through the `Document::expand` method by
specifying an initial [context](https://www.w3.org/TR/json-ld11/#the-context),
and document loader
(which may be needed to load remote documents during expansion).

```rust
use async_std::task;
use iref::IriBuf;
use json_ld::{context, NoLoader, Document, Object, Reference};
use serde_json::Value;

#[async_std::main]
async fn main() -> Result<(), json_ld::Error> {
  // The JSON-LD document to expand.
  let doc: Value = serde_json::from_str(r#"
    {
      "@context": {
        "name": "http://xmlns.com/foaf/0.1/name"
      },
      "@id": "https://www.rust-lang.org",
      "name": "Rust Programming Language"
    }
  "#).unwrap();

  // JSON document loader.
  let mut loader = NoLoader::<Value>::new();

  // Expansion.
  let expanded_doc = doc.expand::<context::Json<Value>, _>(&mut loader).await?;

  // Reference to the `name` property.
  let name_property = Reference::Id(IriBuf::new("http://xmlns.com/foaf/0.1/name").unwrap());

  // Iterate through the expanded objects.
  for object in expanded_doc {
    if let Object::Node(node) = object.as_ref() {
      println!("node: {}", node.id().unwrap()); // print the `@id`
      for name in node.get(&name_property) { // get the names.
        println!("name: {}", name.as_str().unwrap());
      }
    }
  }

  Ok(())
}
```

This crate provides multiple loader implementations:
  - `NoLoader` that always fail. Useful when it is known in advance that the
    document expansion will not require external resources.
  - `FsLoader` to load remote resources from the file system through a
    mount point system.
  - `reqwest::Loader` provided by the `reqwest-loader` feature that uses the
    [`reqwest`](https://crates.io/crates/reqwest) crate to load remote documents.
  Note that `reqwest` requires the
  [`tokio`](https://crates.io/crates/tokio) runtime to work.

#### Compaction

The `Document` trait also provides a `Document::compact` function to compact a document using a given context.

```rust
#[async_std::main]
async fn main() -> Result<(), json_ld::Error> {
  // Input JSON-LD document to compact.
  let input: Value = serde_json::from_str(r#"
    [{
      "http://xmlns.com/foaf/0.1/name": ["Timothée Haudebourg"],
      "http://xmlns.com/foaf/0.1/homepage": [{"@id": "https://haudebourg.net/"}]
    }]
  "#).unwrap();

  // JSON-LD context.
  let context: Value = serde_json::from_str(r#"
    {
      "name": "http://xmlns.com/foaf/0.1/name",
      "homepage": {"@id": "http://xmlns.com/foaf/0.1/homepage", "@type": "@id"}
    }
  "#).unwrap();

  // JSON document loader.
  let mut loader = NoLoader::<Value>::new();

  // Process the context.
  let processed_context = context.process::<context::Json<Value>, _>(&mut loader, None).await?;

  // Compact the input document.
  let output = input.compact(&processed_context, &mut loader).await.unwrap();
  println!("{}", serde_json::to_string_pretty(&output).unwrap());

  Ok(())
}
```

#### Flattening

Flattening is not yet implemented, but will be in the future.

### Custom identifiers

Storing and comparing IRIs can be costly.
This is why while JSON-LD uses IRIs to identify nodes and properties, this implementation
allows you to use different data types, as long as they can be easily converted
into IRIs (they implement the `Id` trait).
One usage example is through the `Vocab` trait and `Lexicon` wrapper that can
transform any `enum` type into an identifier type.

```rust
use iref_enum::IriEnum;
use json_ld::Lexicon;

// Vocabulary used in the implementation.
#[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
pub enum MyVocab {
  #[iri("manifest:name")] Name,
  #[iri("manifest:entries")] Entries,
  #[iri("manifest:action")] Action,
  #[iri("manifest:result")] Result,
}

// A fully functional identifier type.
pub type Id = Lexicon<MyVocab>;

fn handle_node(node: &json_ld::Node<Value, Id>) {
  for name in node.get(MyVocab::Name) { // <- NOTE: we can directly use `MyVocab` here.
    println!("node name: {}", name.as_str().unwrap());
  }
}
```

Note that we use the [`iref-enum`](https://crates.io/crates/iref-enum)
crate that provides the `IriEnum` derive macro which automatically generate
conversions between the `MyVocab` and `iref::Iri` types.

### RDF Serialization/Deserialization

This is not and will not be handled directly by this crate.

## Running the tests

The implementation currently passes the
[expansion test suite](https://w3c.github.io/json-ld-api/tests/expand-manifest.html).
It can be imported using the `generate-expand-tests` example:
```
$ git submodule init
$ git submodule update
$ cargo run --example generate-expand-tests > tests/expand.rs
$ cargo run --example generate-compact-tests > tests/compact.rs
```

This will checkout the [JSON-LD test suite](https://github.com/w3c/json-ld-api/) included in a submodule,
and write the associated Rust test file `tests/expand.rs`.
Then use `cargo test` to run the tests.
All the tests should pass except for the compaction test `p004`
(see [#517](https://github.com/w3c/json-ld-api/issues/517#) on the `json-ld-api` repository).

## Sponsoring

![](https://uploads-ssl.webflow.com/5f37276ebba6e91b4cdefcea/5f398730ecda61a7494906ba_Spruce_Logo_Horizontal.png)

Many thanks to [Spruce](https://www.spruceid.com/) for sponsoring this project!

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
