# A JSON-LD implementation for Rust

<table><tr>
	<td><a href="https://docs.rs/json-ld">Documentation</a></td>
	<td><a href="https://crates.io/crates/json-ld">Crate informations</a></td>
	<td><a href="https://github.com/timothee-haudebourg/json-ld">Repository</a></td>
</tr></table>

This crate is a Rust implementation of the
[JSON-LD](https://www.w3.org/TR/json-ld/)
data interchange format.

[Linked Data (LD)](https://www.w3.org/standards/semanticweb/data)
is a [World Wide Web Consortium (W3C)](https://www.w3.org/)
initiative built upon standard Web technologies to create an
interrelated network of datasets across the Web.
The [JavaScript Object Notation (JSON)](https://tools.ietf.org/html/rfc7159) is
a widely used, simple, unstructured data serialization format to describe
data objects in a human readable way.
JSON-LD brings these two technologies together, adding semantics to JSON
to create a lightweight data serialization format that can organize data and
help Web applications to inter-operate at a large scale.

## State of the crate

This new version of the crate includes many breaking changes
that are not yet documented. Some functions may still be renamed,
which is why the latest release is still flagged as `beta`.
Don't hesitate to contact me for any question about how to use this new API
while I write the documentation.

## Sponsor

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
