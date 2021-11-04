# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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