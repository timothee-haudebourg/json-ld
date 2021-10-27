# A JSON-LD implementation for Rust

<table><tr>
	<td><a href="https://docs.rs/json-ld">Documentation</a></td>
	<td><a href="https://crates.io/crates/json-ld">Crate informations</a></td>
	<td><a href="https://github.com/timothee-haudebourg/json-ld">Repository</a></td>
</tr></table>

{{readme}}

## Running the tests

The implementation currently passes the
[expansion test suite](https://w3c.github.io/json-ld-api/tests/expand-manifest.html).
It can be imported using the `generate-expand-tests` example:
```
$ git submodule init
$ git submodule update
$ cargo run --example generate-expand-tests > tests/expand.rs
$ cargo run --example generate-compact-tests > tests/compact.rs
```

This will checkout the [JSON-LD test suite](https://github.com/w3c/json-ld-api/) included in a submodule,
and write the associated Rust test file `tests/expand.rs`.
Then use `cargo test` to run the tests.
All the tests should pass except for the compaction test `p004`
(see [#517](https://github.com/w3c/json-ld-api/issues/517#) on the `json-ld-api` repository).

## Sponsoring

![](https://uploads-ssl.webflow.com/5f37276ebba6e91b4cdefcea/5f398730ecda61a7494906ba_Spruce_Logo_Horizontal.png)

Many thanks to [Spruce](https://www.spruceid.com/) for sponsoring this project!

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
