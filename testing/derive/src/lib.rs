//! Derive macro to generate test suite functions from a JSON-LD manifest.
//!
//! # Example
//!
//! ```ignore
//! #[json_ld_testing::test_suite("expand-manifest.jsonld")]
//! #[mount("https://w3c.github.io/json-ld-api", "json-ld-api")]
//! #[ignore_test("#t0042", see = "https://github.com/...")]
//! async fn my_test(loader: &FsLoader, entry: &ManifestEntry) {
//!   // test body.
//! }
//! ```
//!
//! This generates one `#[test]` function per manifest entry.
//! Mount paths are relative to the calling file (resolved at compile
//! time via `CARGO_MANIFEST_DIR/tests/`).
use std::path::PathBuf;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, ItemFn, LitStr, Token};

use json_ld::{
	iref::{Iri, IriBuf, IriRef},
	linked_data, rdf_types, FsLoader, Loader,
};
use json_ld_testing_core::{Manifest, SpecVersion};

const BASE_URL: &str = "https://w3c.github.io/json-ld-api/tests/";

/// A URL prefix → local directory mount point.
struct MountPoint {
	/// URL prefix.
	url: String,

	/// Path to the mounting folder, relative to the calling file.
	path: PathBuf,
}

impl MountPoint {
	fn absolute_path(&self) -> PathBuf {
		PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
			.join(&self.path)
			.to_owned()
	}
}

/// A test to ignore, with an optional reason link.
struct IgnoredTest {
	fragment: String,
	see: Option<String>,
}

/// All configuration extracted from helper attributes.
struct TestSuiteConfig {
	mounts: Vec<MountPoint>,
	ignored: Vec<IgnoredTest>,
}

/// Parse and consume `#[mount(...)]` and `#[ignore_test(...)]` attributes.
fn extract_config(item: &mut ItemFn) -> TestSuiteConfig {
	let mut mounts = Vec::new();
	let mut ignored = Vec::new();

	item.attrs.retain(|attr| {
		if attr.path().is_ident("mount") {
			if let Ok(args) =
				attr.parse_args_with(Punctuated::<LitStr, Token![,]>::parse_terminated)
			{
				let args: Vec<_> = args.into_iter().collect();
				if args.len() == 2 {
					mounts.push(MountPoint {
						url: args[0].value(),
						path: args[1].value().into(),
					});
				}
			}
			false
		} else if attr.path().is_ident("ignore_test") {
			let _ = attr.parse_args_with(|input: syn::parse::ParseStream| {
				let fragment: LitStr = input.parse()?;
				let mut see = None;

				if input.peek(Token![,]) {
					let _: Token![,] = input.parse()?;
					let ident: syn::Ident = input.parse()?;
					if ident == "see" {
						let _: Token![=] = input.parse()?;
						let url: LitStr = input.parse()?;
						see = Some(url.value());
					}
				}

				ignored.push(IgnoredTest {
					fragment: fragment.value(),
					see,
				});

				Ok(())
			});
			false
		} else {
			true
		}
	});

	TestSuiteConfig { mounts, ignored }
}

/// Resolve the manifest URI reference against the base URL.
fn resolve_manifest_uri(uri_ref: &str) -> IriBuf {
	let base = Iri::new(BASE_URL).unwrap();
	let r = IriRef::new(uri_ref).expect("invalid manifest URI reference");
	r.resolved(base)
}

/// Load and expand the manifest, returning deserialized entries.
fn load_manifest(manifest_url: &Iri, config: &TestSuiteConfig) -> Manifest {
	// Setup loader.
	let mut loader = FsLoader::new();
	for mount in &config.mounts {
		let url = IriBuf::new(mount.url.clone())
			.unwrap_or_else(|e| panic!("invalid mount URL `{}`: {e}", mount.url));
		loader.mount(url, &mount.absolute_path());
	}

	let doc = loader
		.load(manifest_url)
		.unwrap_or_else(|e| panic!("failed to load manifest `{manifest_url}`: {e}"));

	let expanded = json_ld::JsonLdProcessor::expand(&doc, &loader)
		.unwrap_or_else(|e| panic!("failed to expand manifest: {e}"));

	let quads = linked_data::ser::to_rdf_quads(&expanded)
		.unwrap_or_else(|e| panic!("failed to serialize manifest to RDF: {e}"));

	let manifest_term = rdf_types::Term::iri(
		manifest_url
			.as_iri_ref()
			.resolved(Iri::new(BASE_URL).unwrap()),
	);

	let quads_copy = quads.clone();
	match linked_data::de::from_rdf_quads::<Manifest>(quads, [manifest_term.clone()]) {
		Ok(m) => m,
		Err(e) => {
			for quad in &quads_copy {
				eprintln!("{quad}");
			}
			eprintln!("manifest_term: {manifest_term}");
			panic!("failed to deserialize manifest: {e}");
		}
	}
}

/// Extract a test ID from a manifest entry's IRI fragment.
fn test_id(entry: &json_ld_testing_core::ManifestEntry) -> String {
	entry
		.id
		.fragment()
		.map(|f| f.as_str().to_owned())
		.unwrap_or_else(|| {
			entry
				.id
				.path()
				.segments()
				.last()
				.map(|s| s.as_str().to_owned())
				.unwrap_or_else(|| "unknown".to_owned())
		})
}

/// Check if an entry should be skipped based on its options.
fn should_skip(entry: &json_ld_testing_core::ManifestEntry) -> Option<&'static str> {
	if let Some(ref opts) = entry.options {
		if opts.normative == Some(false) {
			return Some("non-normative test");
		}
		if opts.spec_version == Some(SpecVersion::JsonLd1_0) {
			return Some("unsupported spec version (json-ld-1.0)");
		}
	}
	None
}

/// Attribute macro that generates test functions from a JSON-LD test manifest.
///
/// The macro loads the manifest at compile time, deserializes the test
/// entries, and generates one `#[tokio::test]` function per entry.
///
/// # Attributes
///
/// - `#[mount("https://example.org/", "local/path")]` — Mount a local
///   directory as a URL prefix for the document loader. Paths are relative
///   to the calling file (resolved via `CARGO_MANIFEST_DIR/tests/`).
/// - `#[ignore_test("#fragment", see = "https://issue-link")]` — Skip a
///   specific test by its IRI fragment.
#[proc_macro_attribute]
pub fn test_suite(args: TokenStream, input: TokenStream) -> TokenStream {
	let manifest_ref = parse_macro_input!(args as LitStr).value();
	let mut item = parse_macro_input!(input as ItemFn);
	let config = extract_config(&mut item);

	let manifest_url = resolve_manifest_uri(&manifest_ref);
	let manifest = load_manifest(&manifest_url, &config);

	let fn_name = &item.sig.ident;

	// Build mount statements for the runtime loader using absolute paths
	// resolved at compile time.
	let mount_stmts: Vec<_> = config
		.mounts
		.iter()
		.map(|m| {
			let url = &m.url;
			let path = m.absolute_path();
			let path_str = path.to_string_lossy();
			quote! {
				loader.mount(
					json_ld::iref::iri!(#url).to_owned(),
					#path_str,
				);
			}
		})
		.collect();

	let mut test_fns = Vec::new();

	for entry in &manifest.entries {
		let id = test_id(entry);
		let test_fn_name = format_ident!("{}_{}", fn_name, id);
		let fragment = format!("#{id}");

		// Check #[ignore_test] list.
		if let Some(ignored) = config.ignored.iter().find(|i| i.fragment == fragment) {
			let reason = match &ignored.see {
				Some(url) => format!("ignored, see {url}"),
				None => "ignored".to_owned(),
			};
			test_fns.push(quote! {
				#[test]
				#[ignore = #reason]
				fn #test_fn_name() {}
			});
			continue;
		}

		// Check if entry should be skipped by its options.
		if let Some(reason) = should_skip(entry) {
			test_fns.push(quote! {
				#[test]
				#[ignore = #reason]
				fn #test_fn_name() {}
			});
			continue;
		}

		let entry_tokens = entry.to_token_stream();

		test_fns.push(quote! {
			#[test]
			fn #test_fn_name() {
				let mut loader = json_ld::FsLoader::new();
				#(#mount_stmts)*
				let entry = #entry_tokens;
				#fn_name(&loader, &entry)
			}
		});
	}

	let output = quote! {
		#item

		#(#test_fns)*
	};

	output.into()
}
