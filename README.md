# A JSON-LD implementation for Rust

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/spruceid/json-ld/ci.yml?style=flat-square&logo=github)](https://github.com/spruceid/json-ld/actions)
[![Crate informations](https://img.shields.io/crates/v/json-ld.svg?style=flat-square)](https://crates.io/crates/json-ld)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/json-ld?style=flat-square)](https://crates.io/crates/json-ld)
[![License](https://img.shields.io/crates/l/json-ld.svg?style=flat-square)](https://github.com/spruceid/json-ld#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/json-ld)


<!-- cargo-rdme start -->

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

## Usage

The entry point for this library is the `JsonLdProcessor` trait
that provides an access to all the JSON-LD transformation algorithms
(context processing, expansion, compaction, etc.).
If you want to explore and/or transform `ExpandedDocument`s, you may also
want to check out the [`Object`] type representing a JSON object.


### Expansion

If you want to expand a JSON-LD document, first describe the document to
be expanded using either `RemoteDocument` or `RemoteDocumentReference`:
  - `RemoteDocument` wraps the JSON representation of the document
    alongside its remote URL.
  - `RemoteDocumentReference` may represent only an URL, letting
    some loader fetching the remote document by dereferencing the URL.

After that, you can simply use the [`JsonLdProcessor::expand`] function on
the remote document.

[`JsonLdProcessor::expand`]: JsonLdProcessor::expand

#### Example

```rust
use iref::IriBuf;
use static_iref::iri;
use json_ld::{JsonLdProcessor, Options, RemoteDocument, syntax::{Value, Parse}};

// Create a "remote" document by parsing a file manually.
let input = RemoteDocument::new(
  // We use `IriBuf` as IRI type.
  Some(iri!("https://example.com/sample.jsonld").to_owned()),

  // Optional content type.
  Some("application/ld+json".parse().unwrap()),
  
  // Parse the file.
  Value::parse_str(r#"
    {
      "@context": {
        "name": "http://xmlns.com/foaf/0.1/name"
      },
      "@id": "https://www.rust-lang.org",
      "name": "Rust Programming Language"
    }"#).expect("unable to parse file").0
);

// Use `NoLoader` as we won't need to load any remote document.
let mut loader = json_ld::NoLoader;

// Expand the "remote" document.
let expanded = input
  .expand(&mut loader)
  .await
  .expect("expansion failed");

for object in expanded {
  if let Some(id) = object.id() {
    let name = object.as_node().unwrap()
      .get_any(&iri!("http://xmlns.com/foaf/0.1/name")).unwrap()
      .as_str().unwrap();

    println!("id: {id}");
    println!("name: {name}");
  }
}
```

Here is another example using `RemoteDocumentReference`.

```rust
use static_iref::iri;
use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference};

let input = RemoteDocumentReference::iri(iri!("https://example.com/sample.jsonld").to_owned());

// Use `FsLoader` to redirect any URL starting with `https://example.com/` to
// the local `example` directory. No HTTP query.
let mut loader = json_ld::FsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let expanded = input.expand(&mut loader)
  .await
  .expect("expansion failed");
```

Lastly, the same example replacing [`IriBuf`] with the lightweight
[`rdf_types::vocabulary::Index`] type.

[`IriBuf`]: https://docs.rs/iref/latest/iref/struct.IriBuf.html

```rust
use rdf_types::{Subject, vocabulary::{IriVocabularyMut, IndexVocabulary}};
use contextual::WithContext;
// Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
// to an actual `IriBuf`.
let mut vocabulary: IndexVocabulary = IndexVocabulary::new();

let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
let input = RemoteDocumentReference::iri(iri_index);

// Use `FsLoader` to redirect any URL starting with `https://example.com/` to
// the local `example` directory. No HTTP query.
let mut loader = json_ld::FsLoader::default();
loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");

let expanded = input
  .expand_with(&mut vocabulary, &mut loader)
  .await
  .expect("expansion failed");

// `foaf:name` property identifier.
let name_id = Subject::Iri(vocabulary.insert(iri!("http://xmlns.com/foaf/0.1/name")));

for object in expanded {
  if let Some(id) = object.id() {
    let name = object.as_node().unwrap()
      .get_any(&name_id).unwrap()
      .as_value().unwrap()
      .as_str().unwrap();

    println!("id: {}", id.with(&vocabulary));
    println!("name: {name}");
  }
}
```

### Compaction

The JSON-LD Compaction is a transformation that consists in applying a
context to a given JSON-LD document reducing its size.
There are two ways to get a compact JSON-LD document with this library
depending on your starting point:
  - If you want to get a compact representation for an arbitrary remote
    document, simply use the `JsonLdProcessor::compact`
    (or `JsonLdProcessor::compact_with`) method.
  - Otherwise to compact an `ExpandedDocument` you can use the
    `Compact::compact` method.


#### Example

Here is an example compaction an arbitrary `RemoteDocumentReference`
using `JsonLdProcessor::compact`.

```rust
use static_iref::iri;
use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, RemoteContextReference, syntax::Print};

let input = RemoteDocumentReference::iri(iri!("https://example.com/sample.jsonld").to_owned());

let context = RemoteContextReference::iri(iri!("https://example.com/context.jsonld").to_owned());

// Use `FsLoader` to redirect any URL starting with `https://example.com/` to
// the local `example` directory. No HTTP query.
let mut loader = json_ld::FsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let compact = input
  .compact(context, &mut loader)
  .await
  .expect("compaction failed");

println!("output: {}", compact.pretty_print());
```

### Flattening

The JSON-LD Flattening is a transformation that consists in moving nested
nodes out. The result is a list of all the nodes declared in the document.
There are two ways to flatten JSON-LD document with this library
depending on your starting point:
  - If you want to get a compact representation for an arbitrary remote
    document, simply use the `JsonLdProcessor::flatten`
    (or `JsonLdProcessor::flatten_with`) method.
    This will return a JSON-LD document.
  - Otherwise to compact an `ExpandedDocument` you can use the
    `Flatten::flatten` (or `Flatten::flatten_with`) method.
    This will return the list of nodes as a `FlattenedDocument`.

Flattening requires assigning an identifier to nested anonymous nodes,
which is why the flattening functions take an [`rdf_types::MetaGenerator`]
as parameter. This generator is in charge of creating new fresh identifiers
(with their metadata). The most common generator is
[`rdf_types::generator::Blank`] that creates blank node identifiers.

[`rdf_types::MetaGenerator`]: https://docs.rs/rdf-types/latest/rdf_types/generator/trait.MetaGenerator.html
[`rdf_types::generator::Blank`]: https://docs.rs/rdf-types/latest/rdf_types/generator/struct.Blank.html

#### Example

Here is an example compaction an arbitrary `RemoteDocumentReference`
using `JsonLdProcessor::flatten`.

```rust
use static_iref::iri;
use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, syntax::Print};

let input = RemoteDocumentReference::iri(iri!("https://example.com/sample.jsonld").to_owned());

// Use `FsLoader` to redirect any URL starting with `https://example.com/` to
// the local `example` directory. No HTTP query.
let mut loader = json_ld::FsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let mut generator = rdf_types::generator::Blank::new();

let nodes = input
  .flatten(&mut generator, &mut loader)
  .await
  .expect("flattening failed");

println!("output: {}", nodes.pretty_print());
```

## Fast IRIs and Blank Node Identifiers

This library gives you the opportunity to use any datatype you want to
represent IRIs an Blank Node Identifiers. Most types have them
parameterized.
To avoid unnecessary allocations and expensive comparisons, it is highly
recommended to use a cheap, lightweight datatype such as
[`rdf_types::vocabulary::Index`]. This type will represent each distinct
IRI/blank node identifier with a unique index. In this case a
[`rdf_types::IndexVocabulary`] that maps each index back/to its
original IRI/Blank identifier representation can be passed to every
function.

You can also use your own index type, with your own
[`rdf_types::Vocabulary`] implementation.

[`rdf_types::vocabulary::Index`]: https://docs.rs/rdf-types/latest/rdf_types/vocabulary/struct.Index.html
[`rdf_types::IndexVocabulary`]: https://docs.rs/rdf-types/latest/rdf_types/vocabulary/struct.IndexVocabulary.html
[`rdf_types::Vocabulary`]: https://docs.rs/rdf-types/latest/rdf_types/vocabulary/trait.Vocabulary.html

### Displaying vocabulary-dependent values

Since using vocabularies separates IRIs and Blank ids from their textual
representation, it complicates displaying data using them.
Fortunately many types defined by `json-ld` implement the
[`contextual::DisplayWithContext`] trait that allow displaying value with
a "context", which here would be the vocabulary.
By importing the [`contextual::WithContext`] which provides the `with`
method you can display such value like this:
```rust
use static_iref::iri;
use rdf_types::vocabulary::{IriVocabularyMut, IndexVocabulary};
use contextual::WithContext;

let mut vocabulary: IndexVocabulary = IndexVocabulary::new();
let i = vocabulary.insert(iri!("https://docs.rs/contextual"));
let value = rdf_types::Subject::Iri(i);

println!("{}", value.with(&vocabulary))
```

[`contextual::DisplayWithContext`]: https://docs.rs/contextual/latest/contextual/trait.DisplayWithContext.html
[`contextual::WithContext`]: https://docs.rs/contextual/latest/contextual/trait.WithContext.html

<!-- cargo-rdme end -->

## Testing

To run the tests for the first time use the following commands in a shell:
```sh
git submodule init
git submodule update
cargo test
```

This will clone the
[W3C JSON-LD API repository](https://github.com/w3c/json-ld-api) containing the
official test suite, generate the associated Rust tests using the procedural
macros provided by the [`json-ld-testing`](crates/testing) crate and run the
tests.

Afterward a simple `cargo test` will rerun the tests.

## Sponsor

<p align="center">
<img src="data:image/svg+xml;charset=utf-8;base64,PHN2ZyB3aWR0aD0iMTI2IiBoZWlnaHQ9IjExNSIgdmlld0JveD0iMCAwIDI1MiAyMzAiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+DQo8cGF0aCBkPSJNOTIuNDM0OSAxNi43NjgzQzEwNS4yMTIgLTUuNTg5NDYgMTM3LjE1NCAtNS41ODk0MSAxNDkuOTMxIDE2Ljc2ODNMMjM3Ljg2OSAxNzAuNjQ2QzI1MC42NDYgMTkzLjAwNCAyMzQuNjc1IDIyMC45NTEgMjA5LjEyMSAyMjAuOTUxSDMzLjI0NUM3LjY5MTEyIDIyMC45NTEgLTguMjgwMSAxOTMuMDA0IDQuNDk2OSAxNzAuNjQ2TDkyLjQzNDkgMTYuNzY4M1oiIGZpbGw9IiMxRDQ0ODYiLz4NCjxnIGZpbHRlcj0idXJsKCNmaWx0ZXIwX2RfMzUyMl8xNzEpIj4NCjxwYXRoIGQ9Ik0xMTcuNTM4IDU0LjUzNzlDMTI5LjU1MiAzMy40NjM5IDE1OS41ODcgMzMuNDYzOSAxNzEuNjAxIDU0LjUzNzlMMjM5LjQyOSAxNzMuNTE1QzI1MS40NDMgMTk0LjU4OSAyMzYuNDI1IDIyMC45MzIgMjEyLjM5NyAyMjAuOTMySDc2Ljc0MTVDNTIuNzEzMyAyMjAuOTMyIDM3LjY5NTcgMTk0LjU4OSA0OS43MDk4IDE3My41MTVMMTE3LjUzOCA1NC41Mzc5WiIgZmlsbD0iIzRFNkFDNSIvPg0KPC9nPg0KPGcgZmlsdGVyPSJ1cmwoI2ZpbHRlcjFfZF8zNTIyXzE3MSkiPg0KPHBhdGggZD0iTTE1My4yNDIgOTEuNzgyQzE2Mi4wMTIgNzYuMjEzNiAxODMuOTM4IDc2LjIxMzYgMTkyLjcwOCA5MS43ODJMMjQ3LjA2NSAxODUuOTIyQzI1NS44MzUgMjAxLjQ5IDI0My41NDcgMjIwLjk1MSAyMjYuMDA3IDIyMC45NTFIMTE5Ljk0M0MxMDIuNDAzIDIyMC45NTEgOTEuNDQwNyAyMDEuNDkgMTAwLjIxMSAxODUuOTIyTDE1My4yNDIgOTEuNzgyWiIgZmlsbD0iIzhFQTFFMiIvPg0KPC9nPg0KPGRlZnM+DQo8ZmlsdGVyIGlkPSJmaWx0ZXIwX2RfMzUyMl8xNzEiIHg9IjE2LjQ4MTQiIHk9IjE4LjczMjQiIHdpZHRoPSIyMjcuMTc2IiBoZWlnaHQ9IjIxMC4xOTkiIGZpbHRlclVuaXRzPSJ1c2VyU3BhY2VPblVzZSIgY29sb3ItaW50ZXJwb2xhdGlvbi1maWx0ZXJzPSJzUkdCIj4NCjxmZUZsb29kIGZsb29kLW9wYWNpdHk9IjAiIHJlc3VsdD0iQmFja2dyb3VuZEltYWdlRml4Ii8+DQo8ZmVDb2xvck1hdHJpeCBpbj0iU291cmNlQWxwaGEiIHR5cGU9Im1hdHJpeCIgdmFsdWVzPSIwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAxMjcgMCIgcmVzdWx0PSJoYXJkQWxwaGEiLz4NCjxmZU9mZnNldCBkeD0iLTE1IiBkeT0iLTYiLz4NCjxmZUdhdXNzaWFuQmx1ciBzdGREZXZpYXRpb249IjciLz4NCjxmZUNvbXBvc2l0ZSBpbjI9ImhhcmRBbHBoYSIgb3BlcmF0b3I9Im91dCIvPg0KPGZlQ29sb3JNYXRyaXggdHlwZT0ibWF0cml4IiB2YWx1ZXM9IjAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAuMTQgMCIvPg0KPGZlQmxlbmQgbW9kZT0ibm9ybWFsIiBpbjI9IkJhY2tncm91bmRJbWFnZUZpeCIgcmVzdWx0PSJlZmZlY3QxX2Ryb3BTaGFkb3dfMzUyMl8xNzEiLz4NCjxmZUJsZW5kIG1vZGU9Im5vcm1hbCIgaW49IlNvdXJjZUdyYXBoaWMiIGluMj0iZWZmZWN0MV9kcm9wU2hhZG93XzM1MjJfMTcxIiByZXN1bHQ9InNoYXBlIi8+DQo8L2ZpbHRlcj4NCjxmaWx0ZXIgaWQ9ImZpbHRlcjFfZF8zNTIyXzE3MSIgeD0iNzUuMTI0IiB5PSI2NS4xMDU3IiB3aWR0aD0iMTc2Ljg0MyIgaGVpZ2h0PSIxNjQuODQ1IiBmaWx0ZXJVbml0cz0idXNlclNwYWNlT25Vc2UiIGNvbG9yLWludGVycG9sYXRpb24tZmlsdGVycz0ic1JHQiI+DQo8ZmVGbG9vZCBmbG9vZC1vcGFjaXR5PSIwIiByZXN1bHQ9IkJhY2tncm91bmRJbWFnZUZpeCIvPg0KPGZlQ29sb3JNYXRyaXggaW49IlNvdXJjZUFscGhhIiB0eXBlPSJtYXRyaXgiIHZhbHVlcz0iMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMCAwIDAgMTI3IDAiIHJlc3VsdD0iaGFyZEFscGhhIi8+DQo8ZmVPZmZzZXQgZHg9Ii0xMCIgZHk9Ii0zIi8+DQo8ZmVHYXVzc2lhbkJsdXIgc3RkRGV2aWF0aW9uPSI2Ii8+DQo8ZmVDb21wb3NpdGUgaW4yPSJoYXJkQWxwaGEiIG9wZXJhdG9yPSJvdXQiLz4NCjxmZUNvbG9yTWF0cml4IHR5cGU9Im1hdHJpeCIgdmFsdWVzPSIwIDAgMCAwIDAuMTEzNzI1IDAgMCAwIDAgMC4yNzA1ODggMCAwIDAgMCAwLjUzMzMzMyAwIDAgMCAwLjUgMCIvPg0KPGZlQmxlbmQgbW9kZT0ibm9ybWFsIiBpbjI9IkJhY2tncm91bmRJbWFnZUZpeCIgcmVzdWx0PSJlZmZlY3QxX2Ryb3BTaGFkb3dfMzUyMl8xNzEiLz4NCjxmZUJsZW5kIG1vZGU9Im5vcm1hbCIgaW49IlNvdXJjZUdyYXBoaWMiIGluMj0iZWZmZWN0MV9kcm9wU2hhZG93XzM1MjJfMTcxIiByZXN1bHQ9InNoYXBlIi8+DQo8L2ZpbHRlcj4NCjwvZGVmcz4NCjwvc3ZnPg==">
</p>

Many thanks to [SpruceID](https://www.spruceid.com/) for sponsoring this project!

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
