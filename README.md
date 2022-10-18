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

[<center><svg class="fill-current w-full" width="138" height="40" viewBox="0 0 138 40" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M16.2738 3.50217C18.5233 -0.434076 24.147 -0.434065 26.3965 3.50218L41.8786 30.5935C44.1281 34.5297 41.3162 39.45 36.8173 39.45L5.85304 39.45C1.35407 39.45 -1.45777 34.5297 0.791713 30.5934L16.2738 3.50217Z" fill="#4C49E4"></path><path d="M20.7055 10.1551C22.8206 6.44484 28.1085 6.44485 30.2237 10.1551L42.1653 31.1019C44.2805 34.8122 41.6365 39.45 37.4062 39.45L13.523 39.45C9.29264 39.45 6.64869 34.8122 8.76385 31.1019L20.7055 10.1551Z" fill="#3376E7"></path><path d="M27.2642 16.7277C28.8315 13.989 32.7497 13.989 34.317 16.7277L43.7942 33.288C45.3615 36.0266 43.4023 39.45 40.2678 39.45H21.3135C18.1789 39.45 16.2198 36.0266 17.7871 33.288L27.2642 16.7277Z" fill="#26F3A8"></path><path d="M57.5048 30.9899C63.1979 30.9899 65.7249 28.5772 65.7249 25.0705C65.7249 21.4796 63.1107 19.8805 59.3347 19.123L56.5753 18.5619C54.6001 18.1411 53.9611 17.4117 53.9611 16.4018C53.9611 15.1954 55.0358 14.2977 57.4757 14.2977C60.0899 14.2977 61.2227 15.5601 61.3389 17.0751H65.3764C65.3473 13.4 62.0941 10.8471 57.5338 10.8471C52.8283 10.8471 49.8655 13.0914 49.8655 16.4018C49.8655 19.9646 52.3635 21.3954 55.8781 22.1248L58.8409 22.742C60.758 23.1347 61.6294 23.8361 61.6294 25.1546C61.6294 26.6415 60.4094 27.5673 57.5048 27.5673C55.0068 27.5673 53.4092 26.4732 53.2349 24.2569H49.1974C49.4008 28.3247 52.2183 30.9899 57.5048 30.9899Z" fill="white"></path><path d="M67.333 37.1618H71.2253V29.1664H71.3415C72.0967 30.1202 73.3457 30.9899 75.5242 30.9899C79.4454 30.9899 81.7111 27.7917 81.7111 23.4714C81.7111 18.9827 79.3293 15.8968 75.5242 15.8968C73.3457 15.8968 72.0967 16.7665 71.3415 17.7203H71.2253L70.8186 16.1493H67.333V37.1618ZM74.4204 27.6795C71.9514 27.6795 71.08 25.7438 71.08 23.4714C71.08 21.171 72.0095 19.2072 74.4204 19.2072C76.8893 19.2072 77.7607 21.171 77.7607 23.4714C77.7607 25.7438 76.9184 27.6795 74.4204 27.6795Z" fill="white"></path><path d="M83.2627 30.7374H86.9516V23.5836C86.9516 20.9465 88.2877 19.4036 90.6114 19.4036H91.9766V15.9248H90.8729C88.9558 15.9248 87.6196 16.9909 86.9516 18.3656H86.8354L86.5449 16.1493H83.2627V30.7374Z" fill="white"></path><path d="M102.846 16.1493V24.4813C102.846 26.3329 102.033 27.4831 100.057 27.4831C98.0531 27.4831 97.2398 26.3329 97.2398 24.4813V16.1493H93.3476V25.0705C93.3476 28.2125 95.4389 30.9899 100.028 30.9899H100.057C104.618 30.9899 106.738 28.2125 106.738 25.0705V16.1493H102.846Z" fill="white"></path><path d="M115.616 30.9899C119.885 30.9899 122.093 28.5211 122.325 25.2388H118.404C118.23 26.6976 117.213 27.6795 115.528 27.6795C113.321 27.6795 112.217 26.1926 112.217 23.4433C112.217 20.7221 113.321 19.2072 115.528 19.2072C117.184 19.2072 118.23 20.2171 118.375 21.704H122.296C122.093 18.4497 119.885 15.8968 115.528 15.8968C110.794 15.8968 108.238 19.0108 108.238 23.4433C108.238 27.932 110.881 30.9899 115.616 30.9899Z" fill="white"></path><path d="M130.323 30.9899C134.216 30.9899 136.685 28.8297 137.091 26.1926H133.286C132.996 27.0904 132.066 27.8759 130.323 27.8759C128.087 27.8759 127.041 26.4732 127.012 24.4813H137.295V23.0225C137.295 19.0388 134.767 15.8968 130.265 15.8968C125.414 15.8968 123.033 19.3194 123.033 23.4714C123.033 27.6795 125.647 30.9899 130.323 30.9899ZM127.041 21.9845C127.128 20.3293 128.087 19.0388 130.265 19.0388C132.299 19.0388 133.257 20.3293 133.315 21.9845H127.041Z" fill="white"></path></svg></center>](https://www.spruceid.com/)

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
