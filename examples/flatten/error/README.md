# Flatten error catalog

This directory contains JSON-LD documents that trigger a [`json_ld::algorithms::Error`]
variant during **flattening** (`flatten`).

The `flatten` command is not yet wired up in `src/bin/cli.rs`, so these files cannot be
exercised through the CLI yet.

| File | `Error` variant | Trigger |
|---|---|---|
| `conflicting-indexes.jsonld` | `ConflictingIndexes` | Two node objects share the same `@id` but declare different `@index` values. |
