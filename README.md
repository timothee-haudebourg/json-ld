# A JSON-LD implementation for Rust

This repository holds the (very) early development of a Rust crate implementing
the [JSON-LD](https://www.w3.org/TR/json-ld/) data serialization format.

# Running tests

The early development currently passes the
[expansion test suite](https://w3c.github.io/json-ld-api/tests/expand-manifest.html).

The test suite can be imported using the `generate-expand-tests` example:
```
$ git submodule init
$ git submodule update
$ cargo run --example generate-expand-tests > tests/expand.rs
```

This will checkout the [JSON-LD test suite](https://github.com/w3c/json-ld-api/) included in a submodule,
and write the associated Rust test file `tests/expand.rs`.
Then use `cargo test` to run the tests.
All the tests should pass except for the expansion test `0122` (see [#480](https://github.com/w3c/json-ld-api/issues/480#) on the `json-ld-api` repository).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
