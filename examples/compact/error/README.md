# Compact error catalog

This directory contains JSON-LD documents that trigger a [`json_ld::algorithms::Error`]
variant during **compaction** (`compact`).

The `compact` command is not yet wired up in `src/bin/cli.rs`, so these files cannot be
exercised through the CLI yet.

| File | `Error` variant | Trigger |
|---|---|---|
| `iri-confused-with-prefix.jsonld` | `IriConfusedWithPrefix` | A node's `@id` has a scheme (`http`) that is also defined as a context term. Compact this document against itself as its own context to trigger the error. |
