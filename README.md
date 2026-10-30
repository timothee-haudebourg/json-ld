# A JSON-LD implementation for Rust

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/timothee-haudebourg/json-ld/ci.yml?style=flat-square&logo=github)](https://github.com/timothee-haudebourg/json-ld/actions)
[![Crate informations](https://img.shields.io/crates/v/json-ld.svg?style=flat-square)](https://crates.io/crates/json-ld)
[![Crates.io MSRV](https://img.shields.io/crates/msrv/json-ld?style=flat-square)](https://crates.io/crates/json-ld)
[![License](https://img.shields.io/crates/l/json-ld.svg?style=flat-square)](https://github.com/timothee-haudebourg/json-ld#license)
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

The entry point for this library is the [`JsonLdProcessor`] trait
that provides an access to all the JSON-LD transformation algorithms
(context processing, expansion, compaction, etc.).
If you want to explore and/or transform [`JsonLdExpandedDocument`]s, you may
also want to check out the [`JsonLdObject`](https://docs.rs/json-ld/latest/json_ld/core/object/enum.JsonLdObject.html) type representing a JSON object.

[`JsonLdProcessor`]: https://docs.rs/json-ld/latest/json_ld/processor/trait.JsonLdProcessor.html

### Expansion

If you want to expand a JSON-LD document, first describe the document to
be expanded using either [`JsonLdDocument`] or [`RemoteJsonLdDocument`]:
  - [`JsonLdDocument`] wraps the JSON representation of the document
    alongside its remote URL.
  - [`RemoteJsonLdDocument`] may represent only an URL, letting
    some loader fetching the remote document by dereferencing the URL.

After that, you can simply use the [`JsonLdProcessor::expand`] function on
the remote document.

[`JsonLdDocument`]: https://docs.rs/json-ld/latest/json_ld/core/document/struct.JsonLdDocument.html
[`RemoteJsonLdDocument`]: https://docs.rs/json-ld/latest/json_ld/core/loader/enum.RemoteJsonLdDocument.html
[`JsonLdProcessor::expand`]: https://docs.rs/json-ld/latest/json_ld/processor/trait.JsonLdProcessor.html#tymethod.expand

#### Example

```rust
use rdf_syntax::iri;
use json_ld::{JsonLdProcessor, JsonLdDocument, json_syntax::{JsonValue, JsonParse}};

// Parse a JSON-LD document.
let (value, _) = JsonValue::parse_str(r#"{
  "@context": { "name": "http://xmlns.com/foaf/0.1/name" },
  "@id": "https://www.rust-lang.org",
  "name": "Rust Programming Language"
}"#).expect("unable to parse file");

let input = JsonLdDocument::new(
  Some(iri!("https://example.com/sample.jsonld").to_owned()),
  None,
  value,
);

// Use `NoJsonLdLoader` as we won't need to load any remote document.
let expanded = input.expand(json_ld::NoJsonLdLoader).expect("expansion failed");

assert!(!expanded.is_empty());
```

Here is another example using a file-system loader.

```rust
use rdf_syntax::iri;
use json_ld::{JsonLdProcessor, JsonLdLoader};

let mut loader = json_ld::JsonLdFsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let input = loader.load(iri!("https://example.com/sample.jsonld")).expect("loading failed");
let expanded = input.expand(&loader).expect("expansion failed");
```

### Compaction

The JSON-LD Compaction is a transformation that consists in applying a
context to a given JSON-LD document reducing its size.
There are two ways to get a compact JSON-LD document with this library
depending on your starting point:
  - If you want to get a compact representation for an arbitrary remote
    document, simply use the [`JsonLdProcessor::compact`]
    (or [`JsonLdProcessor::compact_with`]) method.
  - Otherwise to compact a [`JsonLdExpandedDocument`] you can use the
    [`JsonLdExpandedDocument::compact`] method.

[`JsonLdProcessor::compact`]: https://docs.rs/json-ld/latest/json_ld/processor/trait.JsonLdProcessor.html#tymethod.compact
[`JsonLdProcessor::compact_with`]: https://docs.rs/json-ld/latest/json_ld/processor/trait.JsonLdProcessor.html#tymethod.compact_with
[`JsonLdExpandedDocument`]: https://docs.rs/json-ld/latest/json_ld/core/document/expanded/struct.JsonLdExpandedDocument.html
[`JsonLdExpandedDocument::compact`]: https://docs.rs/json-ld/latest/json_ld/core/document/expanded/struct.JsonLdExpandedDocument.html#method.compact

#### Example

Here is an example compacting an arbitrary document using [`JsonLdProcessor::compact`].

```rust
use rdf_syntax::iri;
use json_ld::{JsonLdProcessor, JsonLdLoader, RemoteJsonLdContext, json_syntax::JsonPrint};

let mut loader = json_ld::JsonLdFsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let input = loader.load(iri!("https://example.com/sample.jsonld")).expect("loading failed");
let context = RemoteJsonLdContext::iri(iri!("https://example.com/context.jsonld").to_owned());

let compact = input.compact(context, &loader).expect("compaction failed");

println!("output: {}", compact.pretty_print());
```

### Flattening

The JSON-LD Flattening is a transformation that consists in moving nested
nodes out. The result is a list of all the nodes declared in the document.
There are two ways to flatten JSON-LD document with this library
depending on your starting point:
  - If you want to flatten an arbitrary remote document, use
    [`JsonLdProcessor::flatten`] (or [`JsonLdProcessor::flatten_with`]).
    This returns a compacted JSON-LD document.
  - To flatten an already-expanded [`JsonLdExpandedDocument`], use the
    [`JsonLdExpandedDocument::flatten`] (or
    [`JsonLdExpandedDocument::flatten_unordered`]) method, which returns a
    [`JsonLdFlattenedDocument`].

[`JsonLdProcessor::flatten`]: https://docs.rs/json-ld/latest/json_ld/processor/trait.JsonLdProcessor.html#tymethod.flatten
[`JsonLdProcessor::flatten_with`]: https://docs.rs/json-ld/latest/json_ld/processor/trait.JsonLdProcessor.html#tymethod.flatten_with
[`JsonLdExpandedDocument::flatten`]: https://docs.rs/json-ld/latest/json_ld/core/document/expanded/struct.JsonLdExpandedDocument.html#method.flatten
[`JsonLdExpandedDocument::flatten_unordered`]: https://docs.rs/json-ld/latest/json_ld/core/document/expanded/struct.JsonLdExpandedDocument.html#method.flatten_unordered
[`JsonLdFlattenedDocument`]: https://docs.rs/json-ld/latest/json_ld/core/document/flattened/type.JsonLdFlattenedDocument.html

#### Example

Here is an example flattening an arbitrary document using [`JsonLdProcessor::flatten`].

```rust
use rdf_syntax::iri;
use json_ld::{JsonLdProcessor, JsonLdLoader, json_syntax::JsonPrint};

let mut loader = json_ld::JsonLdFsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let input = loader.load(iri!("https://example.com/sample.jsonld")).expect("loading failed");
let flattened = input.flatten(None, &loader).expect("flattening failed");

println!("output: {}", flattened.pretty_print());
```

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
