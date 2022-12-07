# A JSON-LD implementation for Rust

<table><tr>
	<td><a href="https://docs.rs/json-ld">Documentation</a></td>
	<td><a href="https://crates.io/crates/json-ld">Crate informations</a></td>
	<td><a href="https://github.com/timothee-haudebourg/json-ld">Repository</a></td>
</tr></table>

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

[`JsonLdProcessor`]: crate::JsonLdProcessor

### Introduction

Before diving into the processing function usage, here are some must-know
design choices of this library.

#### Code mapping and metadata

One important feature of this library is the preservation of the code
mapping information extracted from any source JSON document through the
diverse transformation algorithms. This is done using:
  - The [`locspan`](crates.io/crates/locspan) parsing utility library that
    provides the [`Meta`] type associating a value to some metadata. The
    metadata is intended to be code mapping information, but you ultimately
    can decide what it is.
  - The [`json_syntax`](https://crates.io/crates/json-syntax) library that
    parse JSON documents while preserving the code mapping information
    using the [`Meta`] type.

This is particularly useful to provide useful error messages that can
pinpoint the source of the error in the original source file.

##### Example

Here is a example usage of the [`Meta`] that may come in handy when using
this library.

```rust
use locspan::Meta;

// build a value associated with its metadata.
let value_with_metadata = Meta("value", "metadata");

// get a reference to the value.
let value = value_with_metadata.value();

// get a reference to the metadata.
let metadata = value_with_metadata.metadata();

// deconstruct.
let Meta(value, metadata) = value_with_metadata;
```

[`Meta`]: https://docs.rs/locspan/latest/locspan/struct.Meta.html

#### IRIs and Blank Node Identifiers

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

### Expansion

If you want to expand a JSON-LD document, first describe the document to
be expanded using either [`RemoteDocument`] or [`RemoteDocumentReference`]:
  - [`RemoteDocument`] wraps the JSON representation of the document
    alongside its remote URL.
  - [`RemoteDocumentReference`] may represent only an URL, letting
    some loader fetching the remote document by dereferencing the URL.

After that, you can simply use the [`JsonLdProcessor::expand`] function on
the remote document.

[`RemoteDocument`]: crate::RemoteDocument
[`RemoteDocumentReference`]: crate::RemoteDocumentReference
[`JsonLdProcessor::expand`]: JsonLdProcessor::expand

#### Example

```rust
use iref::IriBuf;
use static_iref::iri;
use locspan::Span;
use json_ld::{JsonLdProcessor, Options, RemoteDocument, syntax::{Value, Parse}};

// Create a "remote" document by parsing a file manually.
let input = RemoteDocument::new(
  // We use `IriBuf` as IRI type.
  Some(iri!("https://example.com/sample.jsonld").to_owned()),

  // Parse the file.
  Value::parse_str(
    &std::fs::read_to_string("examples/sample.jsonld")
      .expect("unable to read file"),
    |span| span // keep the source `Span` of each element as metadata.
  ).expect("unable to parse file")
);

// Use `NoLoader` as we won't need to load any remote document.
let mut loader = json_ld::NoLoader::<IriBuf, Span>::new();

// Expand the "remote" document.
let expanded = input
  .expand(&mut loader, Options::<_, _>::default())
  .await
  .expect("expansion failed");

for node in expanded.into_value() {
  if let Some(id) = node.id() {
    println!("node id: {}", id);
  }
}
```

Here is another example using `RemoteDocumentReference`.

```rust
use static_iref::iri;
use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, syntax::{Value, Parse}};

let input = RemoteDocumentReference::Reference(iri!("https://example.com/sample.jsonld").to_owned());

// Use `FsLoader` to redirect any URL starting with `https://example.com/` to
// the local `example` directory. No HTTP query.
let mut loader = json_ld::FsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let expanded = input.expand(&mut loader, Options::<_, _>::default())
  .await
  .expect("expansion failed");
```

Lastly, the same example replacing [`IriBuf`] with the lightweight
[`rdf_types::vocabulary::Index`] type.

[`IriBuf`]: https://docs.rs/iref/latest/iref/struct.IriBuf.html

```rust
use rdf_types::IriVocabularyMut;
// Creates the vocabulary that will map each `rdf_types::vocabulary::Index`
// to an actual `IriBuf`.
let mut vocabulary: rdf_types::IndexVocabulary = rdf_types::IndexVocabulary::new();

let iri_index = vocabulary.insert(iri!("https://example.com/sample.jsonld"));
let input = RemoteDocumentReference::Reference(iri_index);

// Use `FsLoader` to redirect any URL starting with `https://example.com/` to
// the local `example` directory. No HTTP query.
let mut loader = json_ld::FsLoader::default();
loader.mount(vocabulary.insert(iri!("https://example.com/")), "examples");

let expanded = input
  .expand_with(&mut vocabulary, &mut loader, Options::<_, _>::default())
  .await
  .expect("expansion failed");
```

### Compaction

The JSON-LD Compaction is a transformation that consists in applying a
context to a given JSON-LD document reducing its size.
There are two ways to get a compact JSON-LD document with this library
depending on your starting point:
  - If you want to get a compact representation for an arbitrary remote
    document, simply use the [`JsonLdProcessor::compact`]
    (or [`JsonLdProcessor::compact_with`]) method.
  - Otherwise to compact an [`ExpandedDocument`] you can use the
    [`Compact::compact`] method.

[`JsonLdProcessor::compact`]: crate::JsonLdProcessor::compact
[`JsonLdProcessor::compact_with`]: crate::JsonLdProcessor::compact_with
[`ExpandedDocument`]: crate::ExpandedDocument
[`Compact::compact`]: crate::Compact::compact

#### Example

Here is an example compaction an arbitrary [`RemoteDocumentReference`]
using [`JsonLdProcessor::compact`].

```rust
use static_iref::iri;
use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, syntax::Print};

let input = RemoteDocumentReference::Reference(iri!("https://example.com/sample.jsonld").to_owned());

let context = RemoteDocumentReference::Reference(iri!("https://example.com/context.jsonld").to_owned());

// Use `FsLoader` to redirect any URL starting with `https://example.com/` to
// the local `example` directory. No HTTP query.
let mut loader = json_ld::FsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let compact = input
  .compact(context, &mut loader, Options::<_, _>::default())
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
    document, simply use the [`JsonLdProcessor::flatten`]
    (or [`JsonLdProcessor::flatten_with`]) method.
    This will return a JSON-LD document.
  - Otherwise to compact an [`ExpandedDocument`] you can use the
    [`Flatten::flatten`] (or [`Flatten::flatten_with`]) method.
    This will return the list of nodes as a [`FlattenedDocument`].

Flattening requires assigning an identifier to nested anonymous nodes,
which is why the flattening functions take an [`rdf_types::MetaGenerator`]
as parameter. This generator is in charge of creating new fresh identifiers
(with their metadata). The most common generator is
[`rdf_types::generator::Blank`] that creates blank node identifiers.

[`JsonLdProcessor::flatten`]: crate::JsonLdProcessor::flatten
[`JsonLdProcessor::flatten_with`]: crate::JsonLdProcessor::flatten_with
[`Flatten::flatten`]: crate::Flatten::flatten
[`Flatten::flatten_with`]: crate::Flatten::flatten_with
[`FlattenedDocument`]: crate::FlattenedDocument
[`rdf_types::MetaGenerator`]: https://docs.rs/rdf-types/latest/rdf_types/generator/trait.MetaGenerator.html
[`rdf_types::generator::Blank`]: https://docs.rs/rdf-types/latest/rdf_types/generator/struct.Blank.html

#### Example

Here is an example compaction an arbitrary [`RemoteDocumentReference`]
using [`JsonLdProcessor::flatten`].

```rust
use static_iref::iri;
use json_ld::{JsonLdProcessor, Options, RemoteDocumentReference, syntax::Print};
use locspan::{Location, Span};

let input = RemoteDocumentReference::Reference(iri!("https://example.com/sample.jsonld").to_owned());

// Use `FsLoader` to redirect any URL starting with `https://example.com/` to
// the local `example` directory. No HTTP query.
let mut loader = json_ld::FsLoader::default();
loader.mount(iri!("https://example.com/").to_owned(), "examples");

let mut generator = rdf_types::generator::Blank::new().with_metadata(
  // Each blank id will be associated to the document URL with a dummy span.
  Location::new(iri!("https://example.com/").to_owned(), Span::default())
);

let nodes = input
  .flatten(&mut generator, &mut loader, Options::<_, _>::default())
  .await
  .expect("flattening failed");

println!("output: {}", nodes.pretty_print());
```

<!-- cargo-rdme end -->

## Sponsor

![](data:image/svg+xml;base64,PHN2ZyBjbGFzcz0iZmlsbC1jdXJyZW50IHctZnVsbCIgd2lkdGg9IjEzOCIgaGVpZ2h0PSI0MCIgdmlld0JveD0iMCAwIDEzOCA0MCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cGF0aCBkPSJNMTYuMjczOCAzLjUwMjE3QzE4LjUyMzMgLTAuNDM0MDc2IDI0LjE0NyAtMC40MzQwNjUgMjYuMzk2NSAzLjUwMjE4TDQxLjg3ODYgMzAuNTkzNUM0NC4xMjgxIDM0LjUyOTcgNDEuMzE2MiAzOS40NSAzNi44MTczIDM5LjQ1TDUuODUzMDQgMzkuNDVDMS4zNTQwNyAzOS40NSAtMS40NTc3NyAzNC41Mjk3IDAuNzkxNzEzIDMwLjU5MzRMMTYuMjczOCAzLjUwMjE3WiIgZmlsbD0iIzRDNDlFNCI%2BPC9wYXRoPjxwYXRoIGQ9Ik0yMC43MDU1IDEwLjE1NTFDMjIuODIwNiA2LjQ0NDg0IDI4LjEwODUgNi40NDQ4NSAzMC4yMjM3IDEwLjE1NTFMNDIuMTY1MyAzMS4xMDE5QzQ0LjI4MDUgMzQuODEyMiA0MS42MzY1IDM5LjQ1IDM3LjQwNjIgMzkuNDVMMTMuNTIzIDM5LjQ1QzkuMjkyNjQgMzkuNDUgNi42NDg2OSAzNC44MTIyIDguNzYzODUgMzEuMTAxOUwyMC43MDU1IDEwLjE1NTFaIiBmaWxsPSIjMzM3NkU3Ij48L3BhdGg%2BPHBhdGggZD0iTTI3LjI2NDIgMTYuNzI3N0MyOC44MzE1IDEzLjk4OSAzMi43NDk3IDEzLjk4OSAzNC4zMTcgMTYuNzI3N0w0My43OTQyIDMzLjI4OEM0NS4zNjE1IDM2LjAyNjYgNDMuNDAyMyAzOS40NSA0MC4yNjc4IDM5LjQ1SDIxLjMxMzVDMTguMTc4OSAzOS40NSAxNi4yMTk4IDM2LjAyNjYgMTcuNzg3MSAzMy4yODhMMjcuMjY0MiAxNi43Mjc3WiIgZmlsbD0iIzI2RjNBOCI%2BPC9wYXRoPjxwYXRoIGQ9Ik01Ny41MDQ4IDMwLjk4OTlDNjMuMTk3OSAzMC45ODk5IDY1LjcyNDkgMjguNTc3MiA2NS43MjQ5IDI1LjA3MDVDNjUuNzI0OSAyMS40Nzk2IDYzLjExMDcgMTkuODgwNSA1OS4zMzQ3IDE5LjEyM0w1Ni41NzUzIDE4LjU2MTlDNTQuNjAwMSAxOC4xNDExIDUzLjk2MTEgMTcuNDExNyA1My45NjExIDE2LjQwMThDNTMuOTYxMSAxNS4xOTU0IDU1LjAzNTggMTQuMjk3NyA1Ny40NzU3IDE0LjI5NzdDNjAuMDg5OSAxNC4yOTc3IDYxLjIyMjcgMTUuNTYwMSA2MS4zMzg5IDE3LjA3NTFINjUuMzc2NEM2NS4zNDczIDEzLjQgNjIuMDk0MSAxMC44NDcxIDU3LjUzMzggMTAuODQ3MUM1Mi44MjgzIDEwLjg0NzEgNDkuODY1NSAxMy4wOTE0IDQ5Ljg2NTUgMTYuNDAxOEM0OS44NjU1IDE5Ljk2NDYgNTIuMzYzNSAyMS4zOTU0IDU1Ljg3ODEgMjIuMTI0OEw1OC44NDA5IDIyLjc0MkM2MC43NTggMjMuMTM0NyA2MS42Mjk0IDIzLjgzNjEgNjEuNjI5NCAyNS4xNTQ2QzYxLjYyOTQgMjYuNjQxNSA2MC40MDk0IDI3LjU2NzMgNTcuNTA0OCAyNy41NjczQzU1LjAwNjggMjcuNTY3MyA1My40MDkyIDI2LjQ3MzIgNTMuMjM0OSAyNC4yNTY5SDQ5LjE5NzRDNDkuNDAwOCAyOC4zMjQ3IDUyLjIxODMgMzAuOTg5OSA1Ny41MDQ4IDMwLjk4OTlaIiBmaWxsPSJ3aGl0ZSI%2BPC9wYXRoPjxwYXRoIGQ9Ik02Ny4zMzMgMzcuMTYxOEg3MS4yMjUzVjI5LjE2NjRINzEuMzQxNUM3Mi4wOTY3IDMwLjEyMDIgNzMuMzQ1NyAzMC45ODk5IDc1LjUyNDIgMzAuOTg5OUM3OS40NDU0IDMwLjk4OTkgODEuNzExMSAyNy43OTE3IDgxLjcxMTEgMjMuNDcxNEM4MS43MTExIDE4Ljk4MjcgNzkuMzI5MyAxNS44OTY4IDc1LjUyNDIgMTUuODk2OEM3My4zNDU3IDE1Ljg5NjggNzIuMDk2NyAxNi43NjY1IDcxLjM0MTUgMTcuNzIwM0g3MS4yMjUzTDcwLjgxODYgMTYuMTQ5M0g2Ny4zMzNWMzcuMTYxOFpNNzQuNDIwNCAyNy42Nzk1QzcxLjk1MTQgMjcuNjc5NSA3MS4wOCAyNS43NDM4IDcxLjA4IDIzLjQ3MTRDNzEuMDggMjEuMTcxIDcyLjAwOTUgMTkuMjA3MiA3NC40MjA0IDE5LjIwNzJDNzYuODg5MyAxOS4yMDcyIDc3Ljc2MDcgMjEuMTcxIDc3Ljc2MDcgMjMuNDcxNEM3Ny43NjA3IDI1Ljc0MzggNzYuOTE4NCAyNy42Nzk1IDc0LjQyMDQgMjcuNjc5NVoiIGZpbGw9IndoaXRlIj48L3BhdGg%2BPHBhdGggZD0iTTgzLjI2MjcgMzAuNzM3NEg4Ni45NTE2VjIzLjU4MzZDODYuOTUxNiAyMC45NDY1IDg4LjI4NzcgMTkuNDAzNiA5MC42MTE0IDE5LjQwMzZIOTEuOTc2NlYxNS45MjQ4SDkwLjg3MjlDODguOTU1OCAxNS45MjQ4IDg3LjYxOTYgMTYuOTkwOSA4Ni45NTE2IDE4LjM2NTZIODYuODM1NEw4Ni41NDQ5IDE2LjE0OTNIODMuMjYyN1YzMC43Mzc0WiIgZmlsbD0id2hpdGUiPjwvcGF0aD48cGF0aCBkPSJNMTAyLjg0NiAxNi4xNDkzVjI0LjQ4MTNDMTAyLjg0NiAyNi4zMzI5IDEwMi4wMzMgMjcuNDgzMSAxMDAuMDU3IDI3LjQ4MzFDOTguMDUzMSAyNy40ODMxIDk3LjIzOTggMjYuMzMyOSA5Ny4yMzk4IDI0LjQ4MTNWMTYuMTQ5M0g5My4zNDc2VjI1LjA3MDVDOTMuMzQ3NiAyOC4yMTI1IDk1LjQzODkgMzAuOTg5OSAxMDAuMDI4IDMwLjk4OTlIMTAwLjA1N0MxMDQuNjE4IDMwLjk4OTkgMTA2LjczOCAyOC4yMTI1IDEwNi43MzggMjUuMDcwNVYxNi4xNDkzSDEwMi44NDZaIiBmaWxsPSJ3aGl0ZSI%2BPC9wYXRoPjxwYXRoIGQ9Ik0xMTUuNjE2IDMwLjk4OTlDMTE5Ljg4NSAzMC45ODk5IDEyMi4wOTMgMjguNTIxMSAxMjIuMzI1IDI1LjIzODhIMTE4LjQwNEMxMTguMjMgMjYuNjk3NiAxMTcuMjEzIDI3LjY3OTUgMTE1LjUyOCAyNy42Nzk1QzExMy4zMjEgMjcuNjc5NSAxMTIuMjE3IDI2LjE5MjYgMTEyLjIxNyAyMy40NDMzQzExMi4yMTcgMjAuNzIyMSAxMTMuMzIxIDE5LjIwNzIgMTE1LjUyOCAxOS4yMDcyQzExNy4xODQgMTkuMjA3MiAxMTguMjMgMjAuMjE3MSAxMTguMzc1IDIxLjcwNEgxMjIuMjk2QzEyMi4wOTMgMTguNDQ5NyAxMTkuODg1IDE1Ljg5NjggMTE1LjUyOCAxNS44OTY4QzExMC43OTQgMTUuODk2OCAxMDguMjM4IDE5LjAxMDggMTA4LjIzOCAyMy40NDMzQzEwOC4yMzggMjcuOTMyIDExMC44ODEgMzAuOTg5OSAxMTUuNjE2IDMwLjk4OTlaIiBmaWxsPSJ3aGl0ZSI%2BPC9wYXRoPjxwYXRoIGQ9Ik0xMzAuMzIzIDMwLjk4OTlDMTM0LjIxNiAzMC45ODk5IDEzNi42ODUgMjguODI5NyAxMzcuMDkxIDI2LjE5MjZIMTMzLjI4NkMxMzIuOTk2IDI3LjA5MDQgMTMyLjA2NiAyNy44NzU5IDEzMC4zMjMgMjcuODc1OUMxMjguMDg3IDI3Ljg3NTkgMTI3LjA0MSAyNi40NzMyIDEyNy4wMTIgMjQuNDgxM0gxMzcuMjk1VjIzLjAyMjVDMTM3LjI5NSAxOS4wMzg4IDEzNC43NjcgMTUuODk2OCAxMzAuMjY1IDE1Ljg5NjhDMTI1LjQxNCAxNS44OTY4IDEyMy4wMzMgMTkuMzE5NCAxMjMuMDMzIDIzLjQ3MTRDMTIzLjAzMyAyNy42Nzk1IDEyNS42NDcgMzAuOTg5OSAxMzAuMzIzIDMwLjk4OTlaTTEyNy4wNDEgMjEuOTg0NUMxMjcuMTI4IDIwLjMyOTMgMTI4LjA4NyAxOS4wMzg4IDEzMC4yNjUgMTkuMDM4OEMxMzIuMjk5IDE5LjAzODggMTMzLjI1NyAyMC4zMjkzIDEzMy4zMTUgMjEuOTg0NUgxMjcuMDQxWiIgZmlsbD0id2hpdGUiPjwvcGF0aD48L3N2Zz4%3D)

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
