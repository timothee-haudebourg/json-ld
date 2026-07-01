# Error catalog

This directory contains one JSON-LD document per [`json_ld::algorithms::Error`]
variant (`src/algorithms/error.rs`), designed to deterministically trigger
that specific error during processing. They exist to exercise the CLI's
error-reporting path (`miette` diagnostics).

Every document below triggers its error during **expansion** with **default options**:

```sh
cargo run --features cli --bin json-ld -- expand < examples/expand/error/<file>.jsonld
```

| File | `Error` variant | Trigger |
|---|---|---|
| `context-nullification.jsonld` | `InvalidContextNullification` | A `@protected` term is defined, then the context array nullifies (`null`) the active context. |
| `loading-document-failed.jsonld` | `LoadingDocumentFailed` | `@context` is a relative reference and no base IRI is available to resolve it (run without `--base`). |
| `context-loading-failed.jsonld` | `ContextLoadingFailed` | `@context` is an absolute IRI that the loader cannot dereference (no mount point registered). |
| `context-syntax.jsonld` | `ContextSyntax` | `@context` is `true`, which cannot deserialize into a context (must be `null`, a string, an object, or an array thereof). |
| `duplicate-key.jsonld` | `DuplicateKey` | The top-level object has two `@context` entries. |
| `invalid-import-value.jsonld` | `InvalidImportValue` | `@import`'s value is a relative reference that can't be resolved (no base IRI). |
| `invalid-base-iri.jsonld` | `InvalidBaseIri` | `@base`'s value is a relative reference and there is no base IRI to resolve it against. |
| `invalid-vocab-mapping.jsonld` | `InvalidVocabMapping` | `@vocab`'s value expands to a keyword (`@id`) instead of an IRI. |
| `cyclic-iri-mapping.jsonld` | `CyclicIriMapping` | Two terms use each other as a compact-IRI prefix. |
| `invalid-term-definition.jsonld` | `InvalidTermDefinition` | The empty string (`""`) is used as a term name. |
| `invalid-type-mapping.jsonld` | `InvalidTypeMapping` | A term's `@type` mapping expands to `@language`, which is not `@id`/`@vocab`/an IRI. |
| `invalid-reverse-property.jsonld` | `InvalidReverseProperty` | A term definition has both `@reverse` and `@id`. |
| `invalid-iri-mapping.jsonld` | `InvalidIriMapping` | A term's `@id` is a bare relative word with no `@vocab` or base IRI to resolve it against. |
| `invalid-keyword-alias.jsonld` | `InvalidKeywordAlias` | A term is aliased to `@context`. |
| `invalid-container-mapping.jsonld` | `InvalidContainerMapping` | `@container` combines `@list` with `@set`, an unsupported combination. |
| `protected-term-redefinition.jsonld` | `ProtectedTermRedefinition` | A `@protected` term is redefined with a different IRI mapping later in the same context array. |
| `invalid-id-value.jsonld` | `InvalidIdValue` | `@id`'s value is an object instead of a string. |
| `invalid-type-value.jsonld` | `InvalidTypeValue` | `@type`'s value is a number instead of a string (or array of strings). |
| `colliding-keywords.jsonld` | `CollidingKeywords` | A term aliased to `@id` and the literal `@id` keyword are both used on the same node. |
| `invalid-included-value.jsonld` | `InvalidIncludedValue` | `@included`'s value expands to a plain literal instead of a node object. |
| `invalid-index-value.jsonld` | `InvalidIndexValue` | `@index`'s value is a number instead of a string. |
| `invalid-reverse-value.jsonld` | `InvalidReverseValue` | `@reverse`'s value is a string instead of a map. |
| `invalid-reverse-property-map.jsonld` | `InvalidReversePropertyMap` | A key inside `@reverse` expands to a keyword (`@type`). |
| `invalid-reverse-property-value.jsonld` | `InvalidReversePropertyValue` | A property inside `@reverse` has a plain literal value instead of a node. |
| `invalid-nest-value.jsonld` | `InvalidNestValue` | The `@nest` keyword is used directly with a non-object value. |
| `invalid-set-or-list-object.jsonld` | `InvalidSetOrListObject` | A `@list` object contains a disallowed extra entry (`@language`). |
| `invalid-value-object.jsonld` | `InvalidValueObject` | A value object (`@value`) contains a disallowed extra entry. |
| `invalid-value-object-value.jsonld` | `InvalidValueObjectValue` | `@value`'s value is an object instead of a scalar or `null`. |
| `invalid-language-tagged-string.jsonld` | `InvalidLanguageTaggedString` | `@language`'s value is a number instead of a string. |
| `invalid-language-tagged-value.jsonld` | `InvalidLanguageTaggedValue` | `@value` is a number while `@language` is present (only strings can be language-tagged). |
| `invalid-base-direction.jsonld` | `InvalidBaseDirection` | `@direction`'s value is neither `"ltr"` nor `"rtl"`. |
| `invalid-typed-value.jsonld` | `InvalidTypedValue` | `@type`'s value (in a value object) is a number instead of a string. |
| `invalid-language-map-value.jsonld` | `InvalidLanguageMapValue` | A term with `@container: "@language"` has a non-string value in its language map. |

## Errors that require non-default options

Four variants are only reachable with processing options the CLI does not yet expose
(JSON-LD 1.0 mode, or a stricter expansion policy). Their example documents live in
[`requires-options/`](requires-options/README.md).

## Errors reachable only via other operations

Two variants are not expansion errors at all:

- `ConflictingIndexes` — detected during **flattening**; example in [`examples/flatten/error/`](../../../flatten/error/README.md).
- `IriConfusedWithPrefix` — detected during **compaction**; example in [`examples/compact/error/`](../../../compact/error/README.md).

## Not currently reachable

A few variants are not represented because nothing in the current
implementation constructs them:

- `InvalidProtectedValue` and `InvalidScopedContext` are defined in
  `ErrorCode`/`Error` but are dead code: no code path currently returns them.
- `InvalidRemoteContext` and `RemoteContextSyntax` both require a remote
  context document to be *successfully* loaded first (via `@context` or
  `@import`) before their specific shape is validated. The CLI's loader
  (`TokioFsLoader`) is created with no mount points in `src/bin/cli.rs`, so
  any external reference currently fails earlier with `ContextLoadingFailed`
  instead. These will become reachable once the CLI supports mounting a
  local directory (e.g. via a `--base`/`--mount` flag).
