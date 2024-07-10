# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.21.1] - 2024-07-10

### Fixed

- [f3ad52d] Fix `context::term_definition::Type` deserialization.

## [0.21.0] - 2024-07-09

### Fixed

- [8a0d9e5] Fix `Option<Nullable<T>>` deserialization. ([#80](https://github.com/timothee-haudebourg/json-ld/issues/80))

## [0.20.0] - 2024-07-08

### Added

- [51b19b9] Add an expansion policy option for `@vocab`. ([#78](https://github.com/timothee-haudebourg/json-ld/issues/78))

## [0.19.2] - 2024-06-28

### Added

- [5a15c03] Impl `Loader` for `&L`.

## [0.19.1] - 2024-06-25

### Added

- [ce1669a] Add `syntax::from_value`, `to_value` and `Unordered` re-exports from `json-syntax`.

## [0.19.0] - 2024-06-25

### Changed

- [3f00670] Change `Loader::load` receiver from `&mut self` to `&self` ([#77](https://github.com/timothee-haudebourg/json-ld/issues/77))

### Removed

- [3f00670] Remove `Loader` type parameter ([#77](https://github.com/timothee-haudebourg/json-ld/issues/77))

## [0.18.0] - 2024-06-21

### Fixed

- [cb43034] Fix `FsLoader` to allow multiples IRIs bound to the same path.

### Removed

- [cb43034] Remove generics on error types. ([#75](https://github.com/timothee-haudebourg/json-ld/issues/75))

## [0.17.2] - 2024-06-05

### Added

- [f4aca7d] Add `map_ids` methods.
- [b022273] Add `Context::map_ids` method.
- [b022273] Add public fields to `RemoteDocument`
- [b022273] Add public fields to `Processed`

## [0.17.0] - 2024-05-20

### Added

- [8a9b3f4] Add `client` option to the `ReqwestLoader`.
- [c483397] Impl `Serialize` for `LenientLangTag`.
- [4c52747] Impl `Serialize`/`Deserialize` for `LangString`.

### Fixed

- [ef3fb9d] Fix README badges.

## [0.16.0] - 2024-04-09

### Added

- [17266de] Impl  `linked-data::LinkedData*`.
- [773fc2b] Add `serde` support for context syntax types.
- [8ff9083] Implement serialization from (interpreted) RDF
- [18fb2aa] Add `ExpandedDocument::main_node` method.
- [28edd71] Add `Document` type.
- [28edd71] Add `serde` feature.
- [b795872] Add `into_document_*` functions.
- [0997428] Add `ExpandedDocument::into_main_node`.
- [e59f3d6] Add `RemoteDocument::document_mut`
- [ece6f0d] Impl `Clone` and `Debug` for expanded docs.
- [ec7664b] Impl `PrintWithContext` for `Node`.
- [a6de88b] Add default value for `RemoteContextReference` Iri parameter.
- [9d51cae] Add `syntax::ContextDocument` type.
- [8fc4a23] Impl `Loader` for `&mut L`.
- [7fc9875] Add `MapLoader`.
- [38d3444] Impl `std::error::Error`for `SerializationError`.

### Build

- [e42003e] Upgrade `iref`, simplify API, serializer.
- [368b09e] Upgrade `rdf-types` to version 0.17.3
- [368b09e] Upgrade `grdf` to version 0.21
- [368b09e] Upgrade `nquads-syntax` to version 0.16
- [1ba0d65] Upgrade `locspan` to version 0.8
- [1ba0d65] Upgrade `rdf-types` to version 0.18
- [1ba0d65] Upgrade `json-syntax` to version 0.10
- [1ba0d65] Upgrade `grdf` to version 0.22
- [1ba0d65] Upgrade `nquads-syntax` 0.17
- [cc7691d] Upgrade `json-syntax` to version 0.12
- [78c0116] Upgrade `xsd-types`to version 0.9.1.

### Changed

- [d358b11] Change default type values for remote docs.

### Fixed

- [f9cb306] Fix warnings.
- [9f6327c] Fix warnings.
- [2354040] Fix object serialization.
- [63b573b] Fix `ContextDocument` serialization.
- [08971f1] Fix `@version` de/serialization.
- [b648e04] Fix `@version` deserialization.
- [d437133] Fix formatting.

### Removed

- [baa6e3b] Remove locspan patch.
- [f13208d] Remove `test.jsonld`
- [ab8f40a] Remove metadata from JSON-LD objects.
- [f234530] Remove dead link in README.md.
- [0b4a87b] Remove unnecessary `Send` bounds in `leader` module.

## [0.15.1] - 2023-12-06

### Added

- [2aacc36] Add support for `wasm32` target.

## [0.15.0] - 2023-06-06

### Build

- [e7f856a] Upgrade `rdf-types` to version `0.15.2`

## [0.14.2] - 2023-06-06

### Removed

- [394f75d] Remove forgotten debug `eprintln`.
- [f36a571] Remove useless `syntax::number` module.

## [0.14.1] - 2023-04-25

### Added

- [57e11e3] Add mutable accessors in `Object`.
- [57e11e3] Add `Object::as_value_mut`.
- [57e11e3] Add `Object::as_node_mut`.
- [57e11e3] Add `Object::as_list_mut`.

### Fixed

- [8d38977] Fix deprecated `clippy::derive_hash_xor_eq`.
- [4722c96] Fix formatting.

### Removed

- [dae97c2] Remove unused parameter in `invalid_iri`.

## [0.14.0] - 2023-02-28

### Build

- [8fdf15f] Upgrade `rdf-types` from `0.13` to `0.14.2`.

### Fixed

- [c00839e] Fix `json-ld` dependencies versions.

## [0.13.0] - 2023-02-28

### Added

- [1ab6833] Add a custom test to explore stack size limits.

### Build

- [df1bc4e] Upgrade `rdf-type` from `0.12.9` to `0.13.0`.

### Fixed

- [ee0bd4a] Fix `cargo test` command in `ci.yml`.

## [0.12.1] - 2023-01-10

### Added

- [ac56b2d] Impl `Error` for all the error types.

### Build

- [ac56b2d] Introduce `thiserror` at version `1.0.38`.
- [ac56b2d] Upgrade `locspan` from `0.7.12` to `0.7.13`.

### Fixed

- [8ef9a5e] Fix `cliff.toml`

## [0.12.0] - 2023-01-10

### Added

- [75c73b1] Add `expansion_policy` in `json_ld::Options`.
- [1076ada] Add `Options::with_expand_context` function.

## [0.11.0] - 2023-01-10

### Build

- [b81e53a] Upgrade `json-syntax` to 0.9.

## [0.10.0] - 2022-12-19

### Added

- [6e69524] Add RDF deserialization to the processor API

### Changed

- [d6f91f5] Move to version 0.10.0

### Fixed

- [eafe462] Fix git submodule instructions.
- [eafe462] Fixes #43
- [6e69524] Fixes #46

## [0.9.1] - 2022-12-09

### Added

- [12d5579] Add some doc.
- [911c801] Add documentation.
- [14f9fdb] Add table of contents.
- [c91e5b1] Add `hashbrown::Equivalent<Id<I, B>>` impls.
- [ae1c82f] Add `ReqwestLoader` compliant with the spec.

### Changed

- [d114466] Move to version `0.9.1`.

### Fixed

- [b0f961d] Fix formatting.
- [7a0563b] Fix formatting.
- [c4c8572] Fix typo & Makefile.
- [15a78c8] Fix clippy warnings.
- [17e712e] Fix formatting.
- [e499bfd] Fix formatting.
- [88edfe6] Fix and update `README.md`.

## [0.9.0-beta] - 2022-10-20

### Changed

- [ffb61be] Move to version 0.9.0-beta.

### Fixed

- [f340b38] Fix clippy warnings.

## [0.7.0-beta] - 2022-10-20

### Added

- [3d3c761] Add context to syntax functions.
- [784b861] Add `flatten` API.
- [f0344db] Add a `::code` method to error types.
- [0c9bab9] Add some expansion doc.
- [354a9d6] Add `default_base_url` func to the `Expand` trait.

### Changed

- [62d915a] Refactoring of expansion algo almost done.
- [8bb57a3] Refactoring of expansion algorithm done!
- [f8872ae] Refactor more of the compaction algorithm.
- [e7119e1] Refactored `compaction` module.
- [e7119e1] Refactor is complete, but not tested yet.

### Fixed

- [3102653] Fix clippy warnings.
- [ef85f1f] Fix type params order in `expansion` library.
- [59cdf04] Fix clippy warnings in `context-processing`.
- [1e7f78d] Fix the rest of clippy warnings.
- [f8872ae] Fix `Reference`/`ValidReference` bug.
- [9366eb7] Fix context definition entries size hint.
- [4070cb9] Fix compact bug.
- [3be7a05] Fix url of expected compacted document in tests.
- [bf08768] Fix CI.
- [d0b8a0f] Fix tests README.
- [6c461e8] Fix formatting
- [d78d366] Fix Spruce sponsor ([#42](https://github.com/timothee-haudebourg/json-ld/issues/42))

### Removed

- [2b9d456] Remove old expansion code.
- [6710a47] Remove old tests.
- [532c0e9] Remove `flattening` folder.
- [bc5b54d] Remove `Lexicon` datatype.
- [b1b7272] Remove traces of the `generic_json` crate.
- [deec55e] Remove `reqwest` feature (for now).

## [0.6.0] - 2022-01-14

### Added

- [cfadb4c] Add warning type.
- [eda84c9] impl `Display` for `Warning`.
- [551e240] Add the `flattening` module.

### Changed

- [7df142c] Move to version 0.6.0.

### Fixed

- [2aab37f] Fix formatting and clippy warnings.
- [e1e5f8b] Fix loaders.
- [97f2b5c] Fix formatting & clippy warnings.
- [3e56c17] Fix formatting & clippy warnings.
- [7b31108] Fix some new clippy warnings.

## [0.5.0] - 2021-11-04

### Added

- [7eb7d09] Add `Lenient::map`.
- [9fe900e] Add a compact method and example.
- [448791e] Add strict expansion mode
- [6c48bcb] Add semicolon after warn
- [a3a0caa] Add `+nightly` in workflow.
- [429f688] Add custom tests.
- [2c8a48b] Add sort-jsonld-array utility script.
- [461114f] Add CHANGELOG, move to version 0.4.0
- [6be001a] Add inline hints.

### Build

- [74b101f] Introduce key expansion policies.

### Changed

- [1749ea4] Move to 0.3.0-alpha.
- [26bb267] Move to version 0.3.0.
- [a1e1acd] Move to version 0.5.0.

### Fixed

- [85b05c7] Fix dependencies spec.
- [1706f60] Fix too strong lifetime constraint on Vocab.
- [d3995b9] Fix too strong lifetime constraint in example.
- [f0d4529] Fix iref dep.
- [86f1771] Fix iri compaction.
- [1f34815] Fix compact_property_graph.
- [67b20ab] Fixing some warnings.
- [a8a81e2] Fix type and index maps.
- [9b0e452] Fix generated options.
- [a479378] Fix expected output for custom/c038.
- [40b3be8] Fix expected output of custom/e112
- [4493bbe] Fix clippy warnings.
- [b4d8616] Fix custom test warning.
- [c1c3c06] Fix Spruce link
- [652d107] Fixing tests...
- [b260d7c] Fix clippy warnings.
- [092eb69] Fix warnings in `compaction` example.
- [46108b2] Fix doc tests.
- [0258d89] Fix test templates.
- [590c489] Fix json-ld comparison function.
- [4479778] Fix typo.
- [955a957] Fix iteration of merged contexts.
- [b274216] Fix clippy warning.
- [d4cedca] Fix README

### Removed

- [6167f19] Remove useless MappedMut.
- [cebb6af] Remove unused JsonValue in compaction example.
- [7c9fd8e] Remove the `Lenient` type.
- [56a4151] Remove remaining traces of the `Lenient` type.

## [0.2.0-alpha] - 2020-04-27

### Changed

- [425caed] Move to version 0.2.0-alpha.

### Fixed

- [0a98f9a] Fix the `included` node interface.
- [4ad5585] Fix typos in the README.

## [0.1.0-alpha] - 2020-04-27

### Build

- [598ff07] Introduce the Expanded type.

### Changed

- [60bf328] Refactoring.
- [ccd7abf] Refactoring. Automatic tests.

### Fixed

- [05aeebd] Fix the test generator. Add a README.
- [12ff709] Fix test options.
- [b42ee1a] Fix the reqwest loader and give an example.

### Removed

- [d41a51c] Remove some warnings.
- [b2ab85c] Remove debug prints.
- [e0c50a6] Remove debug print.
- [83bc6d0] Remove debug prints.
- [9c717d9] Remove more debug prints.
- [d1aeae8] Remove expand.rs test file.
- [6e501fd] Remove Term from node ids.
- [47389b7] Remove `as_json_ld` from LocalContext.
- [50ff476] Remove keyword iri.
- [87da59c] Remove useless comment.

