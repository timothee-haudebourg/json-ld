#!/bin/sh
cargo run --example=generate-expand-tests > tests/expand.rs
cargo run --example=generate-compact-tests > tests/compact.rs
cargo run --example=generate-flatten-tests > tests/flatten.rs
cargo run --example=generate-to_rdf-tests > tests/to_rdf.rs