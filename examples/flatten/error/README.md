# Flatten error catalog

This directory contains JSON-LD documents that trigger a [`json_ld::algorithms::Error`]
variant during **flattening** (`flatten`).

| File | `Error` variant | Trigger |
|---|---|---|
| `conflicting-indexes.jsonld` | `ConflictingIndexes` | Two node objects share the same `@id` but declare different `@index` values. |
