//! Derive macro to generate test suite functions from a JSON-LD manifest.
//!
//! # Example
//!
//! ```ignore
//! #[json_ld_testing::test_suite("expand-manifest.jsonld")]
//! #[mount("https://w3c.github.io/json-ld-api", "json-ld-api")]
//! #[ignore_test("#t0042", note = "https://github.com/...")]
//! async fn my_test(loader: &FsLoader, entry: &ManifestEntry) {
//!   // test body.
//! }
//! ```
//!
//! This generates one `#[test]` function per manifest entry.
//! Mount paths are relative to the calling file (resolved at compile
//! time via `CARGO_MANIFEST_DIR/tests/`).
use std::borrow::Cow;
use std::path::PathBuf;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn, LitStr, Token};

use json_ld::{
	iref::{iri, Iri, IriBuf, IriRef},
	linked_data,
	rdf_syntax::{GrdfQuad, GroundTerm, LiteralType, Quad},
	syntax::CompactIri,
	IriRefBuf, JsonLdFsLoader, JsonLdLoader, JsonLdNode,
};

const MF_ENTRIES: &Iri = iri!("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#entries");

fn resolve_iri_ref(
	base_iri: Option<&Iri>,
	prefixes: &[ResolvedPrefixBinding],
	iri_ref: &IriRef,
) -> IriBuf {
	match CompactIri::new(iri_ref) {
		Ok(compact) => compact.resolve(prefix_resolver(prefixes)).unwrap(),
		Err(_) => match base_iri {
			Some(base_iri) => iri_ref.resolved(base_iri),
			None => panic!("missing base URL"),
		},
	}
}

/// A URL prefix → local directory mount point.
struct MountPoint {
	/// URL prefix, resolved against the prefixes declared via
	/// `#[prefix(...)]`, or against the `#[base = "..."]` URL otherwise.
	///
	/// If absent (`#[mount("path")]`), the `#[base = "..."]` URL is used
	/// as-is.
	url: Option<IriRefBuf>,

	/// Path to the mounting folder, relative to the calling file.
	path: PathBuf,
}

impl MountPoint {
	fn resolve(
		&self,
		base_iri: Option<&Iri>,
		prefixes: &[ResolvedPrefixBinding],
	) -> ResolvedMountPoint {
		let url = match &self.url {
			Some(url) => resolve_iri_ref(base_iri, prefixes, url),
			None => base_iri
				.expect("mount point has no URL and no `#[base = \"...\"]` was provided")
				.to_owned(),
		};

		ResolvedMountPoint {
			url,
			path: self.path.clone(),
		}
	}
}

struct ResolvedMountPoint {
	url: IriBuf,
	path: PathBuf,
}

impl ResolvedMountPoint {
	fn absolute_path(&self) -> PathBuf {
		PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
			.join(&self.path)
	}
}

/// A test to ignore, with an optional reason link.
struct IgnoredTest {
	iri_ref: IriRefBuf,
	note: Option<String>,
}

struct ResolvedIgnoredTest {
	iri: IriBuf,
	note: Option<String>,
}

struct IgnoreIf {
	path: Vec<IriRefBuf>,
	value: ConditionValue,
	note: Option<String>,
}

fn prefix_resolver<'a>(
	prefixes: &'a [ResolvedPrefixBinding],
) -> impl FnOnce(&str) -> Option<Cow<'a, Iri>> {
	|prefix| {
		prefixes
			.iter()
			.find(|binding| binding.prefix == prefix)
			.map(|b| Cow::Borrowed(b.iri.as_iri()))
	}
}

impl IgnoreIf {
	fn resolve(
		&self,
		base_iri: Option<&Iri>,
		prefixes: &[ResolvedPrefixBinding],
	) -> ResolvedIgnoreIf {
		let path = self
			.path
			.iter()
			.map(|iri_ref| resolve_iri_ref(base_iri, prefixes, iri_ref))
			.collect();

		ResolvedIgnoreIf {
			path,
			value: self.value.clone(),
			note: self.note.clone(),
		}
	}
}

struct ResolvedIgnoreIf {
	path: Vec<IriBuf>,
	value: ConditionValue,
	note: Option<String>,
}

impl ResolvedIgnoreIf {
	pub fn matches(&self, entry: &JsonLdNode) -> bool {
		let (last, init) = self
			.path
			.split_last()
			.expect("#[ignore_if(...)] requires at least one path segment");

		let mut current = entry;
		for iri in init {
			match current.get(iri).next().and_then(|object| object.as_node()) {
				Some(node) => current = node,
				None => return false,
			}
		}

		let found = current.get(last).next();

		match (&self.value, found) {
			(ConditionValue::Bool(expected), Some(object)) => object.as_bool() == Some(*expected),
			(ConditionValue::String(expected), Some(object)) => {
				object.as_str() == Some(expected.as_str())
			}
			(_, None) => false,
		}
	}
}

#[derive(Clone)]
enum ConditionValue {
	Bool(bool),
	String(String),
}

/// All configuration extracted from helper attributes.
struct TestSuiteConfig {
	base_iri: Option<IriBuf>,
	mounts: Vec<MountPoint>,
	prefixes: Vec<PrefixBinding>,
	ignored: Vec<IgnoredTest>,
	ignore_if: Vec<IgnoreIf>,
}

impl TestSuiteConfig {
	/// Resolves every `#[prefix(...)]` binding's IRI reference against the
	/// `#[base = "..."]` URL.
	fn resolve_prefixes(&self) -> Vec<ResolvedPrefixBinding> {
		self.prefixes
			.iter()
			.map(|prefix| prefix.resolve(self.base_iri.as_deref()))
			.collect()
	}

	fn resolve_mounts(&self, prefixes: &[ResolvedPrefixBinding]) -> Vec<ResolvedMountPoint> {
		self.mounts
			.iter()
			.map(|mount| mount.resolve(self.base_iri.as_deref(), prefixes))
			.collect()
	}

	fn resolve_ignored_tests(&self, base_url: &Iri) -> Vec<ResolvedIgnoredTest> {
		self.ignored
			.iter()
			.map(|ignored| ResolvedIgnoredTest {
				iri: ignored.iri_ref.resolved(base_url),
				note: ignored.note.clone(),
			})
			.collect()
	}

	/// Resolves every `#[ignore_if(...)]` condition's path segments into
	/// full URIs, using the prefixes declared via `#[prefix(...)]` and the
	/// `#[base = "..."]` URL.
	fn resolve_ignore_if(&self, prefixes: &[ResolvedPrefixBinding]) -> Vec<ResolvedIgnoreIf> {
		self.ignore_if
			.iter()
			.map(|condition| condition.resolve(self.base_iri.as_deref(), prefixes))
			.collect()
	}
}

/// A URL prefix binding declared via `#[prefix("name" = "iri-ref")]`.
///
/// The IRI may be relative; it is resolved against the `#[base = "..."]`
/// URL by [`PrefixBinding::resolve`].
struct PrefixBinding {
	prefix: String,
	iri: IriRefBuf,
}

impl PrefixBinding {
	fn resolve(&self, base_iri: Option<&Iri>) -> ResolvedPrefixBinding {
		let iri = match base_iri {
			Some(base_iri) => self.iri.resolved(base_iri),
			None => panic!(
				"prefix `{}` has a relative URI but no `#[base = \"...\"]` was provided",
				self.prefix
			),
		};

		ResolvedPrefixBinding {
			prefix: self.prefix.clone(),
			iri,
		}
	}
}

struct ResolvedPrefixBinding {
	prefix: String,
	iri: IriBuf,
}

/// Parses the body of a `#[ignore_test(...)]` or `#[ignore_if(...)]`
/// attribute's trailing `, note = "..."` part, if present.
fn parse_note(input: syn::parse::ParseStream) -> syn::Result<Option<String>> {
	let mut note = None;

	if input.peek(Token![,]) {
		let _: Token![,] = input.parse()?;
		let ident: syn::Ident = input.parse()?;
		if ident == "note" {
			let _: Token![=] = input.parse()?;
			let url: LitStr = input.parse()?;
			note = Some(url.value());
		}
	}

	Ok(note)
}

/// Parse and consume `#[base = "..."]`, `#[mount(...)]`, `#[prefix(...)]`,
/// `#[ignore_test(...)]` and `#[ignore_if(...)]` attributes.
fn extract_config(item: &mut ItemFn) -> TestSuiteConfig {
	let mut base_iri = None;
	let mut mounts = Vec::new();
	let mut prefixes = Vec::new();
	let mut ignored = Vec::new();
	let mut ignore_if = Vec::new();

	item.attrs.retain(|attr| {
		if attr.path().is_ident("base") {
			if let syn::Meta::NameValue(meta) = &attr.meta {
				if let syn::Expr::Lit(syn::ExprLit {
					lit: syn::Lit::Str(s),
					..
				}) = &meta.value
				{
					base_iri = Some(
						IriBuf::new(s.value())
							.unwrap_or_else(|_| panic!("invalid base URI: {}", s.value())),
					);
				}
			}
			false
		} else if attr.path().is_ident("mount") {
			let _ = attr.parse_args_with(|input: syn::parse::ParseStream| {
				let first: LitStr = input.parse()?;

				// `#[mount("path")]` mounts the `#[base = "..."]` URL itself;
				// `#[mount("url", "path")]` mounts an explicit URL.
				let (url, path) = if input.peek(Token![,]) {
					let _: Token![,] = input.parse()?;
					let path: LitStr = input.parse()?;
					(Some(first), path)
				} else {
					(None, first)
				};

				let url = url
					.map(|url| {
						IriRefBuf::from_string(url.value())
							.map_err(|_| syn::Error::new(url.span(), "invalid IRI reference"))
					})
					.transpose()?;

				mounts.push(MountPoint {
					url,
					path: path.value().into(),
				});

				Ok(())
			});
			false
		} else if attr.path().is_ident("prefix") {
			attr.parse_args_with(|input: syn::parse::ParseStream| {
				let prefix: LitStr = input.parse()?;
				let _: Token![=] = input.parse()?;
				let iri: LitStr = input.parse()?;

				// The IRI may be relative; it is resolved against
				// `#[base = "..."]` later, by `PrefixBinding::resolve`.
				let iri = IriRefBuf::from_string(iri.value())
					.map_err(|_| syn::Error::new(iri.span(), "invalid IRI reference"))?;

				prefixes.push(PrefixBinding {
					prefix: prefix.value(),
					iri,
				});

				Ok(())
			})
			.unwrap_or_else(|e| panic!("invalid `#[prefix(...)]` attribute: {e}"));
			false
		} else if attr.path().is_ident("ignore_test") {
			let _ = attr.parse_args_with(|input: syn::parse::ParseStream| {
				let iri_ref: LitStr = input.parse()?;
				let note = parse_note(input)?;

				ignored.push(IgnoredTest {
					iri_ref: IriRefBuf::from_string(iri_ref.value()).unwrap(),
					note,
				});

				Ok(())
			});
			false
		} else if attr.path().is_ident("ignore_if") {
			let _ = attr.parse_args_with(|input: syn::parse::ParseStream| {
				// Path: one or more dot-separated compact IRI string literals,
				// e.g. `"test:option"."test:normative"`.
				let mut path = Vec::new();
				loop {
					let segment: LitStr = input.parse()?;
					let iri_ref = IriRefBuf::from_string(segment.value())
						.map_err(|_| syn::Error::new(segment.span(), "invalid IRI reference"))?;
					path.push(iri_ref);

					if input.peek(Token![.]) {
						let _: Token![.] = input.parse()?;
					} else {
						break;
					}
				}

				let _: Token![==] = input.parse()?;

				let value = if input.peek(syn::LitBool) {
					let b: syn::LitBool = input.parse()?;
					ConditionValue::Bool(b.value)
				} else {
					let s: LitStr = input.parse()?;
					ConditionValue::String(s.value())
				};

				let note = parse_note(input)?;

				ignore_if.push(IgnoreIf { path, value, note });

				Ok(())
			});
			false
		} else {
			true
		}
	});

	TestSuiteConfig {
		base_iri,
		mounts,
		prefixes,
		ignored,
		ignore_if,
	}
}

/// Load and expand the manifest, returning deserialized entries.
fn load_manifest(manifest_url: &Iri, mounts: &[ResolvedMountPoint]) -> JsonLdNode {
	// Setup loader.
	let mut loader = JsonLdFsLoader::new();
	for mount in mounts {
		loader.mount(mount.url.clone(), mount.absolute_path());
	}

	let doc = loader
		.load(manifest_url)
		.unwrap_or_else(|e| panic!("failed to load manifest `{manifest_url}`: {e}"));

	json_ld::JsonLdProcessor::expand(&doc, &loader)
		.unwrap_or_else(|e| panic!("failed to expand manifest: {e}"))
		.into_main_node()
		.expect("expect only one manifest")
}

fn test_fn_name(fn_name: &syn::Ident, id: &Iri) -> syn::Ident {
	let suffix = id
		.fragment()
		.map(|f| f.as_str().to_owned())
		.unwrap_or_else(|| {
			id.path()
				.segments()
				.next_back()
				.map(|s| s.as_str().to_owned())
				.unwrap_or_else(|| "unknown".to_owned())
		});

	format_ident!("{fn_name}_{suffix}")
}

/// Returns the manifest entry's own identifier as an RDF term, to be used
/// as the root resource when deserializing the entry's quads back.
fn entry_root_term(entry: &json_ld::JsonLdNode) -> json_ld::rdf_syntax::Term {
	match entry.id.as_ref().expect("missing id") {
		json_ld::Lenient::Valid(id) => json_ld::rdf_syntax::Term::id(id.clone()),
		json_ld::Lenient::Invalid(_) => panic!("invalid manifest entry id"),
	}
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
/// - `#[ignore_test("#fragment", note = "https://issue-link")]` — Skip a
///   specific test by its IRI fragment.
#[proc_macro_attribute]
pub fn test_suite(args: TokenStream, input: TokenStream) -> TokenStream {
	let manifest_iri_ref = IriRefBuf::from_string(parse_macro_input!(args as LitStr).value())
		.expect("invalid manifest URL");

	let mut item = parse_macro_input!(input as ItemFn);
	let config = extract_config(&mut item);
	let prefixes = config.resolve_prefixes();

	let manifest_url = resolve_iri_ref(config.base_iri.as_deref(), &prefixes, &manifest_iri_ref);
	let ignore_if = config.resolve_ignore_if(&prefixes);
	let mounts = config.resolve_mounts(&prefixes);

	let manifest = load_manifest(&manifest_url, &mounts);

	// Resolve `#[ignore_test(...)]` fragments against the manifest's own
	// (expanded) `@id`.
	let manifest_base: IriBuf = manifest
		.id
		.as_ref()
		.and_then(|id| id.as_iri())
		.map(ToOwned::to_owned)
		.unwrap_or_else(|| manifest_url.clone());
	let ignored = config.resolve_ignored_tests(&manifest_base);

	let fn_name = &item.sig.ident;

	// Build mount statements for the runtime loader using absolute paths
	// resolved at compile time.
	let mount_stmts: Vec<_> = mounts
		.iter()
		.map(|m| {
			let url = m.url.as_str();
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

	let entries = manifest
		.get(MF_ENTRIES)
		.filter_map(|entries| entries.as_list())
		.flatten()
		.filter_map(|entry| entry.as_node());

	for entry in entries {
		let id = entry
			.id
			.as_ref()
			.expect("missing id")
			.as_iri()
			.expect("invalid id");

		let test_fn_name = test_fn_name(fn_name, id);

		// Check #[ignore] list.
		if let Some(ignored) = ignored.iter().find(|i| i.iri == id) {
			let reason = ignored.note.as_ref().map(|url| {
				let reason = format!("see {url}");
				quote! { = #reason }
			});
			test_fns.push(quote! {
				#[test]
				#[ignore #reason]
				fn #test_fn_name() {}
			});
			continue;
		}

		// Check #[ignore_if] list.
		if let Some(ignored) = ignore_if.iter().find(|i| i.matches(entry)) {
			let reason = ignored.note.as_ref().map(|url| {
				let reason = format!("see {url}");
				quote! { = #reason }
			});
			test_fns.push(quote! {
				#[test]
				#[ignore #reason]
				fn #test_fn_name() {}
			});
			continue;
		}

		let quads = linked_data::ser::to_rdf_quads(entry)
			.unwrap()
			.into_iter()
			.map(quad_to_tokens);

		let root = rdf_term_to_tokens(entry_root_term(entry));

		test_fns.push(quote! {
			#[test]
			fn #test_fn_name() {
				let mut loader = json_ld::JsonLdFsLoader::new();
				#(#mount_stmts)*
				let entry = json_ld::linked_data::de::from_rdf_quads(
					[#(#quads),*],
					[#root],
				).unwrap();
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

/// Generates tokens reconstructing a `GrdfQuad` as a Rust expression.
fn quad_to_tokens(quad: GrdfQuad) -> proc_macro2::TokenStream {
	let Quad(subject, predicate, object, graph) = quad;

	let subject = rdf_term_to_tokens(subject);
	let predicate = rdf_term_to_tokens(predicate);
	let object = rdf_term_to_tokens(object);
	let graph = match graph {
		Some(g) => {
			let g = rdf_term_to_tokens(g);
			quote!(Some(#g))
		}
		None => quote!(None),
	};

	quote! {
		::json_ld::rdf_syntax::GrdfQuad::new(#subject, #predicate, #object, #graph)
	}
}

/// Generates tokens reconstructing a lexical RDF `Term` (not to be confused
/// with [`json_ld::Term`], the JSON-LD expanded term type) as a Rust
/// expression.
fn rdf_term_to_tokens(term: json_ld::rdf_syntax::Term) -> proc_macro2::TokenStream {
	match term {
		json_ld::rdf_syntax::Term::BlankId(id) => {
			let id = id.as_str();
			quote! {
				::json_ld::rdf_syntax::Term::BlankId(
					::json_ld::rdf_syntax::blankid!(#id).to_owned()
				)
			}
		}
		json_ld::rdf_syntax::Term::Ground(GroundTerm::Iri(iri)) => {
			let iri = iri.as_str();
			quote! {
				::json_ld::rdf_syntax::Term::iri(::json_ld::iref::iri!(#iri).to_owned())
			}
		}
		json_ld::rdf_syntax::Term::Ground(GroundTerm::Literal(literal)) => {
			let value = &literal.value;
			let r#type = literal_type_to_tokens(literal.r#type);
			quote! {
				::json_ld::rdf_syntax::Term::literal(
					::json_ld::rdf_syntax::Literal::new(#value, #r#type)
				)
			}
		}
	}
}

/// Generates tokens reconstructing a `LiteralType` as a Rust expression
/// (either an `IriBuf` or a `LangTagBuf`, both convertible to `LiteralType`
/// via `Into`, matching `Literal::new`'s second parameter).
fn literal_type_to_tokens(ty: LiteralType) -> proc_macro2::TokenStream {
	match ty {
		LiteralType::Any(iri) => {
			let iri = iri.as_str();
			quote! { ::json_ld::iref::iri!(#iri).to_owned() }
		}
		LiteralType::LangString(tag) => {
			let tag = tag.as_str();
			quote! {
				::json_ld::rdf_syntax::LangTagBuf::new(#tag.to_string()).unwrap()
			}
		}
	}
}
