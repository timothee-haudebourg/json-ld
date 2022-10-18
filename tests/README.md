# JSON-LD test suite.

Test suite for the [`json-ld` crate](https://crates.io/crates/json-ld).

## Usage

To run the tests for the first time use the following commands in
a shell:
```
git submodules update
cargo test
```

This will clone the [W3C JSON-LD API repository](https://github.com/w3c/json-ld-api)
containing the official test suite,
generate the associated Rust tests using the procedural macros provided by the
[`json-ld-testing` crate]() and run the tests.

Afterward a simple `cargo test` will rerun the tests.

### Known issues

#### Test `flatten_tin03` fails sometimes.

This is due to the non determinism of the flattening algorithm when it comes
to assigning a fresh blank node identifier to anonymous nodes.
The output document to this test contains two generated blank node identifiers.
Sometime it matches the expected output, some other times the two blank node identifiers
are swapped and the test fails spuriously.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
