# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.11.0]
- Upgraded `json_syntax`: 0.8.17 -> 0.9
  This version adds a better support for `serde` and `serde_json`,
  but changes the behavior of the `json` macro without metadata annotations to
  simplify type inference.

## [0.10.0]
### Changed
- Use `rdf_types::Literal` instead of `json_ld::rdf::Value`.

### Added
- `JsonLdProcessor::to_rdf*` functions for RDF serialization.
- `toRdf` test suite.

## [0.9.1]
### Changed
- All the library has been refactored. Please take a look at the `README.md`
  for a new introduction to the API.
- `json-syntax` library is now used in place of `generic-json`, dropping the
  support for `serde-json`. `json-syntax` should add back support for it in the
  future.
- Improved the `ReqwestLoader` so it can deal with redirections and `Link`
  headers.

### Added
- `JsonLdProcessor` trait.
- The `locspan` library is used everywhere to keep track of code mapping info.

## [0.6.1]
- Relax the `K: JsonFrom<J>` bound into `K: Json` from the `AsJson` trait definition. Fixes #33.

## [0.6.0]
### Changed
- Associate a unique identifier to each loaded document through the `Loader` trait.
- Locate errors using its source (a `loader::Id`) and its metadata.
- Locate warnings using its source (a `loader::Id`) and its metadata.
- The `request::Loader` not longer panic.

### Added
- `Warning` type to enumerate possible warnings.
- `Loc` type to locate errors and warnings.
- `loader::Id` type to identify source files.
- `Loader::id`, `Loader::iri`.
- Compaction API.

## [0.5.0] - 2021-11-04
### Changed
- Abstract the JSON implementation.
  The JSON type (formerly `json::JsonValue`) is now a type parameter.
  It can theoretically be replaced by any type you want, as long as
  it implements the `generic_json::Json` trait.
  As of now, only the `ijson::IValue` type implements this trait.
  If the https://github.com/serde-rs/json/pull/814 PR is merged,
  then `serde_json::Value` should follow.

## Added
- `Object::into_node`, `into_value`, `into_list`, `as_node`, `as_value`, `as_list`.
- `Indexed<Object>::into_indexed_node`, `into_indexed_value`, `into_indexed_list`.
- `Node::properties`, `reverse_properties`.
- `PartialEq<str>` impl for `Property`.
- More documentation.

## [0.4.0] - 2021-09-15
### Added 
- `policy` option in the context processing `Options` struct controlling how undefined keys are expanded.

### Removed
- Unused `strict` field in the context processing `Options` struct.