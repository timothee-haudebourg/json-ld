# Expansion error catalog — requires non-default options

These documents trigger a [`json_ld::algorithms::Error`] variant during **expansion**,
but only when non-default processing options are used that the CLI does not yet expose.

```sh
cargo run --features cli --bin json-ld -- expand < examples/expand/error/requires-options/<file>.jsonld
```

The above command will **not** error with default options; see the "Required option" column
for what is needed.

| File | `Error` variant | Required option | Trigger |
|---|---|---|---|
| `invalid-context-entry.jsonld` | `InvalidContextEntry` | JSON-LD 1.0 processing mode | `@propagate` is used inside a context definition, which is only invalid under 1.0. |
| `processing-mode-conflict.jsonld` | `ProcessingModeConflict` | JSON-LD 1.0 processing mode | `@version: 1.1` appears in a context while the processor runs in 1.0 mode. |
| `key-expansion-failed.jsonld` | `KeyExpansionFailed` | `Strict` or `Strictest` expansion policy | An undefined, colon-less key is used; the default `Standard` policy silently drops such keys instead of erroring. |
| `keyword-redefinition.jsonld` | `KeywordRedefinition` | JSON-LD 1.0 processing mode | `@type` appears as a term definition in the context; this is only a keyword redefinition error under 1.0. |
