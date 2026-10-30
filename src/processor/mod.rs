use json_syntax::JsonValue;
use linked_data::ser::to_rdf_quads_interpretation_with;
use rdf_syntax::{GenInterpretation, IriBuf};
use rdf_syntax::{Generator, Quad, Term};

/// Drives a future that is expected to complete synchronously on the first
/// poll (i.e., backed by a sync [`ProcessingEnvironment`]).
///
/// Uses a no-op waker so no thread is ever parked. Panics if the future
/// returns [`Poll::Pending`], which would indicate a bug in the caller.
fn resolve_sync<F: std::future::Future>(f: F) -> F::Output {
	let waker = futures::task::noop_waker_ref();
	let mut cx = std::task::Context::from_waker(waker);
	let mut f = std::pin::pin!(f);
	match f.as_mut().poll(&mut cx) {
		std::task::Poll::Ready(result) => result,
		std::task::Poll::Pending => {
			panic!("async function backed by a sync loader returned Poll::Pending")
		}
	}
}

use crate::JsonLdVisit;
use crate::{
	JsonLdDirection, JsonLdDocument, JsonLdExpandedDocument, JsonLdLocatedError,
	JsonLdProcessingMode, RemoteJsonLdContext,
	algorithms::{
		AsyncJsonLdEnvironment, JsonLdCompactionOptions, JsonLdContextProcessingOptions,
		JsonLdEnvironment, JsonLdExpansionOptions, JsonLdExpansionPolicy, RdfSerializationOptions,
	},
};

mod document;

/// JSON-LD Processor options.
#[derive(Clone)]
pub struct JsonLdOptions {
	/// The base IRI to use when expanding or compacting the document.
	///
	/// If set, this overrides the input document's IRI.
	pub base: Option<IriBuf>,

	/// If set to true, the JSON-LD processor replaces arrays with just one element with that element during compaction.
	///
	/// If set to false, all arrays will remain arrays even if they have just one element.
	///
	/// Defaults to `true`.
	pub compact_arrays: bool,

	/// Determines if IRIs are compacted relative to the base option or document
	/// location when compacting.
	///
	/// Defaults to `true`.
	pub compact_to_relative: bool,

	/// A context that is used to initialize the active context when expanding a document.
	pub expand_context: Option<RemoteJsonLdContext>,

	/// If set to `true`, certain algorithm processing steps where indicated are
	/// ordered lexicographically.
	///
	/// If `false`, order is not considered in processing.
	///
	/// Defaults to `false`.
	pub ordered: bool,

	/// Sets the processing mode.
	///
	/// Defaults to `ProcessingMode::JsonLd1_1`.
	pub processing_mode: JsonLdProcessingMode,

	/// Determines how value objects containing a base direction are transformed
	/// to and from RDF.
	pub rdf_direction: Option<JsonLdDirection>,

	/// If set to `true`, the JSON-LD processor may emit blank nodes for triple
	/// predicates, otherwise they will be omitted.
	pub produce_generalized_rdf: bool,

	/// Term expansion policy, passed to the document expansion algorithm.
	pub expansion_policy: JsonLdExpansionPolicy,
}

impl JsonLdOptions {
	/// Returns these options with the `ordered` flag set to `false`.
	pub fn unordered(self) -> Self {
		Self {
			ordered: false,
			..self
		}
	}

	/// Returns these options with the `expand_context` set to the given
	/// `context`.
	pub fn with_expand_context(self, context: RemoteJsonLdContext) -> Self {
		Self {
			expand_context: Some(context),
			..self
		}
	}

	/// Builds options for the context processing algorithm from these options.
	pub fn context_processing_options(&self) -> JsonLdContextProcessingOptions {
		JsonLdContextProcessingOptions {
			processing_mode: self.processing_mode,
			..Default::default()
		}
	}

	/// Builds options for the expansion algorithm from these options.
	pub fn expansion_options(&self) -> JsonLdExpansionOptions {
		JsonLdExpansionOptions {
			processing_mode: self.processing_mode,
			ordered: self.ordered,
			policy: self.expansion_policy,
		}
	}

	/// Builds options for the compaction algorithm from these options.
	pub fn compaction_options(&self) -> JsonLdCompactionOptions {
		JsonLdCompactionOptions {
			processing_mode: self.processing_mode,
			compact_to_relative: self.compact_to_relative,
			compact_arrays: self.compact_arrays,
			ordered: self.ordered,
		}
	}

	pub fn rdf_serialization_options(&self) -> RdfSerializationOptions {
		RdfSerializationOptions {
			produce_generalized_rdf: self.produce_generalized_rdf,
		}
	}
}

impl Default for JsonLdOptions {
	fn default() -> Self {
		Self {
			base: None,
			compact_arrays: true,
			compact_to_relative: true,
			expand_context: None,
			ordered: false,
			processing_mode: JsonLdProcessingMode::JsonLd1_1,
			rdf_direction: None,
			produce_generalized_rdf: false,
			expansion_policy: JsonLdExpansionPolicy::default(),
		}
	}
}

/// Result returned by the [`JsonLdProcessor::expand`] function.
pub type JsonLdExpandResult = Result<JsonLdExpandedDocument, JsonLdLocatedError>;

/// Result of converting into a [`JsonLdDocument`].
pub type JsonLdIntoDocumentResult = Result<JsonLdDocument, JsonLdLocatedError>;

/// Result of the [`JsonLdProcessor::compact`] function.
pub type JsonLdCompactResult = Result<JsonValue, JsonLdLocatedError>;

/// Result of the [`JsonLdProcessor::flatten`] function.
pub type JsonLdFlattenResult = Result<JsonValue, JsonLdLocatedError>;

pub type JsonLdToRdfResult = Result<Vec<Quad<Term>>, JsonLdLocatedError>;

/// Result of the [`JsonLdProcessor::compare`] function.
pub type JsonLdCompareResult = Result<bool, JsonLdLocatedError>;

/// The `JsonLdProcessor` interface is the high-level programming structure that
/// developers use to access the JSON-LD transformation methods.
///
/// It is notably implemented for the [`JsonLdDocument`] type.
///
/// # Methods naming
///
/// Each processing function is declined in two variants:
///
///  - `_with`: allows passing custom [`JsonLdOptions`].
///  - no suffix: uses default options.
pub trait JsonLdProcessor: Sized {
	/// Compare this document against `other` using the given `options`.
	fn compare_with(
		&self,
		other: &Self,
		env: impl JsonLdEnvironment,
		options: JsonLdOptions,
	) -> JsonLdCompareResult {
		resolve_sync(self.async_compare_with(other, env.into_async_environment(), options))
	}

	/// Compare this document against `other` using default options.
	fn compare(&self, other: &Self, env: impl JsonLdEnvironment) -> JsonLdCompareResult {
		self.compare_with(other, env, JsonLdOptions::default())
	}

	/// Compare this document against `other` using the given `options`.
	#[allow(async_fn_in_trait)]
	async fn async_compare_with(
		&self,
		other: &Self,
		env: impl AsyncJsonLdEnvironment,
		options: JsonLdOptions,
	) -> JsonLdCompareResult;

	/// Compare this document against `other` using default options.
	#[allow(async_fn_in_trait)]
	async fn async_compare(
		&self,
		other: &Self,
		env: impl AsyncJsonLdEnvironment,
	) -> JsonLdCompareResult {
		self.async_compare_with(other, env, JsonLdOptions::default())
			.await
	}

	/// Expand the document using the given `options`.
	fn expand_with(
		&self,
		env: impl JsonLdEnvironment,
		options: JsonLdOptions,
	) -> JsonLdExpandResult {
		resolve_sync(self.async_expand_with(env.into_async_environment(), options))
	}

	/// Expand the document using default options.
	fn expand(&self, env: impl JsonLdEnvironment) -> JsonLdExpandResult {
		self.expand_with(env, JsonLdOptions::default())
	}

	/// Expand the document using the given `options`.
	#[allow(async_fn_in_trait)]
	async fn async_expand_with(
		&self,
		env: impl AsyncJsonLdEnvironment,
		options: JsonLdOptions,
	) -> JsonLdExpandResult;

	/// Expand the document using default options.
	#[allow(async_fn_in_trait)]
	async fn async_expand(&self, env: impl AsyncJsonLdEnvironment) -> JsonLdExpandResult {
		self.async_expand_with(env, JsonLdOptions::default()).await
	}

	/// Compact the document relative to `context` using the given `options`.
	fn compact_with(
		&self,
		context: RemoteJsonLdContext,
		env: impl JsonLdEnvironment,
		options: JsonLdOptions,
	) -> JsonLdCompactResult {
		resolve_sync(self.async_compact_with(context, env.into_async_environment(), options))
	}

	/// Compact the document relative to `context` using default options.
	fn compact(
		&self,
		context: RemoteJsonLdContext,
		env: impl JsonLdEnvironment,
	) -> JsonLdCompactResult {
		self.compact_with(context, env, JsonLdOptions::default())
	}

	/// Compact the document relative to `context` using the given `options`.
	#[allow(async_fn_in_trait)]
	async fn async_compact_with(
		&self,
		context: RemoteJsonLdContext,
		env: impl AsyncJsonLdEnvironment,
		options: JsonLdOptions,
	) -> JsonLdCompactResult;

	/// Compact the document relative to `context` using default options.
	#[allow(async_fn_in_trait)]
	async fn async_compact(
		&self,
		context: RemoteJsonLdContext,
		env: impl AsyncJsonLdEnvironment,
	) -> JsonLdCompactResult {
		self.async_compact_with(context, env, JsonLdOptions::default())
			.await
	}

	/// Flatten the document using the given `options`.
	///
	/// An optional `context` can be given to compact the result.
	fn flatten_with(
		&self,
		context: Option<RemoteJsonLdContext>,
		env: impl JsonLdEnvironment,
		options: JsonLdOptions,
	) -> JsonLdFlattenResult {
		resolve_sync(self.async_flatten_with(context, env.into_async_environment(), options))
	}

	/// Flatten the document using default options.
	fn flatten(
		&self,
		context: Option<RemoteJsonLdContext>,
		env: impl JsonLdEnvironment,
	) -> JsonLdFlattenResult {
		self.flatten_with(context, env, JsonLdOptions::default())
	}

	/// Flatten the document using the given `options`.
	///
	/// An optional `context` can be given to compact the result.
	#[allow(async_fn_in_trait)]
	async fn async_flatten_with(
		&self,
		context: Option<RemoteJsonLdContext>,
		env: impl AsyncJsonLdEnvironment,
		options: JsonLdOptions,
	) -> JsonLdFlattenResult;

	/// Flatten the document using default options.
	#[allow(async_fn_in_trait)]
	async fn async_flatten(
		&self,
		context: Option<RemoteJsonLdContext>,
		env: impl AsyncJsonLdEnvironment,
	) -> JsonLdFlattenResult {
		self.async_flatten_with(context, env, JsonLdOptions::default())
			.await
	}

	/// Serialize the document to RDF using the given `options`.
	fn to_rdf_with(
		&self,
		env: impl JsonLdEnvironment,
		generator: impl Generator,
		options: JsonLdOptions,
	) -> JsonLdToRdfResult {
		resolve_sync(self.async_to_rdf_with(env.into_async_environment(), generator, options))
	}

	/// Serialize the document to RDF using default options.
	fn to_rdf(&self, env: impl JsonLdEnvironment, generator: impl Generator) -> JsonLdToRdfResult {
		self.to_rdf_with(env, generator, JsonLdOptions::default())
	}

	/// Serialize the document to RDF using the given `options`.
	#[allow(async_fn_in_trait)]
	async fn async_to_rdf_with(
		&self,
		env: impl AsyncJsonLdEnvironment,
		mut generator: impl Generator,
		options: JsonLdOptions,
	) -> JsonLdToRdfResult {
		let rdf_serialization_options = options.rdf_serialization_options();
		let mut expanded = JsonLdProcessor::async_expand_with(self, env, options).await?;
		expanded.relabel(&mut generator);
		let interpretation = GenInterpretation::new(generator);
		Ok(
			to_rdf_quads_interpretation_with(&expanded, interpretation, rdf_serialization_options)
				.unwrap(),
		)
	}

	/// Serialize the document to RDF using default options.
	#[allow(async_fn_in_trait)]
	async fn async_to_rdf(
		&self,
		env: impl AsyncJsonLdEnvironment,
		generator: impl Generator,
	) -> JsonLdToRdfResult {
		self.async_to_rdf_with(env, generator, JsonLdOptions::default())
			.await
	}
}
