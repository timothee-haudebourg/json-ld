use iref::IriBuf;
use json_syntax::JsonValue;
use linked_data::ser::to_rdf_quads_interpretation_with;
use rdf_types::{interpretation::GeneratorInterpretation, Generator, Quad};

use crate::{
	algorithms::{
		CompactionOptions, ContextProcessingOptions, ExpansionOptions, ExpansionPolicy,
		ProcessingEnvironment, RdfSerializationOptions,
	},
	Direction, Document, Error, ExpandedDocument, ProcessingMode, Relabel, RemoteContext,
};

mod remote_document;

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
	pub expand_context: Option<RemoteContext>,

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
	pub processing_mode: ProcessingMode,

	/// Determines how value objects containing a base direction are transformed
	/// to and from RDF.
	pub rdf_direction: Option<Direction>,

	/// If set to `true`, the JSON-LD processor may emit blank nodes for triple
	/// predicates, otherwise they will be omitted.
	pub produce_generalized_rdf: bool,

	/// Term expansion policy, passed to the document expansion algorithm.
	pub expansion_policy: ExpansionPolicy,
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
	pub fn with_expand_context(self, context: RemoteContext) -> Self {
		Self {
			expand_context: Some(context),
			..self
		}
	}

	/// Builds options for the context processing algorithm from these options.
	pub fn context_processing_options(&self) -> ContextProcessingOptions {
		ContextProcessingOptions {
			processing_mode: self.processing_mode,
			..Default::default()
		}
	}

	/// Builds options for the expansion algorithm from these options.
	pub fn expansion_options(&self) -> ExpansionOptions {
		ExpansionOptions {
			processing_mode: self.processing_mode,
			ordered: self.ordered,
			policy: self.expansion_policy,
		}
	}

	/// Builds options for the compaction algorithm from these options.
	pub fn compaction_options(&self) -> CompactionOptions {
		CompactionOptions {
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
			processing_mode: ProcessingMode::JsonLd1_1,
			rdf_direction: None,
			produce_generalized_rdf: false,
			expansion_policy: ExpansionPolicy::default(),
		}
	}
}

/// Result returned by the [`JsonLdProcessor::expand`] function.
pub type ExpandResult = Result<ExpandedDocument, Error>;

/// Result returned by the [`JsonLdProcessor::into_document`] function.
pub type IntoDocumentResult = Result<Document, Error>;

/// Result of the [`JsonLdProcessor::compact`] function.
pub type CompactResult = Result<JsonValue, Error>;

/// Result of the [`JsonLdProcessor::flatten`] function.
pub type FlattenResult = Result<JsonValue, Error>;

/// Result of the [`JsonLdProcessor::compare`] function.
pub type CompareResult = Result<bool, Error>;

/// The `JsonLdProcessor` interface is the high-level programming structure that
/// developers use to access the JSON-LD transformation methods.
///
/// It is notably implemented for the [`RemoteDocument`](crate::RemoteDocument)
/// type.
///
/// # Methods naming
///
/// Each processing function is declined in two variants:
///
///  - `_with`: allows passing custom [`Options`].
///  - no suffix: uses default options.
pub trait JsonLdProcessor: Sized {
	/// Compare this document against `other` using the given `options`.
	#[allow(async_fn_in_trait)]
	async fn compare_with(
		&self,
		other: &Self,
		env: impl ProcessingEnvironment,
		options: JsonLdOptions,
	) -> CompareResult;

	/// Compare this document against `other` using default options.
	#[allow(async_fn_in_trait)]
	async fn compare(&self, other: &Self, env: impl ProcessingEnvironment) -> CompareResult {
		self.compare_with(other, env, JsonLdOptions::default())
			.await
	}

	/// Expand the document using the given `options`.
	#[allow(async_fn_in_trait)]
	async fn expand_with(
		&self,
		env: impl ProcessingEnvironment,
		options: JsonLdOptions,
	) -> ExpandResult;

	/// Expand the document using default options.
	#[allow(async_fn_in_trait)]
	async fn expand(&self, env: impl ProcessingEnvironment) -> ExpandResult {
		self.expand_with(env, JsonLdOptions::default()).await
	}

	/// Compact the document relative to `context` using the given `options`.
	#[allow(async_fn_in_trait)]
	async fn compact_with(
		&self,
		context: RemoteContext,
		env: impl ProcessingEnvironment,
		options: JsonLdOptions,
	) -> CompactResult;

	/// Compact the document relative to `context` using default options.
	#[allow(async_fn_in_trait)]
	async fn compact(
		&self,
		context: RemoteContext,
		env: impl ProcessingEnvironment,
	) -> CompactResult {
		self.compact_with(context, env, JsonLdOptions::default())
			.await
	}

	/// Flatten the document using the given `options`.
	///
	/// An optional `context` can be given to compact the result.
	#[allow(async_fn_in_trait)]
	async fn flatten_with(
		&self,
		context: Option<RemoteContext>,
		env: impl ProcessingEnvironment,
		options: JsonLdOptions,
	) -> FlattenResult;

	/// Flatten the document using default options.
	#[allow(async_fn_in_trait)]
	async fn flatten(
		&self,
		context: Option<RemoteContext>,
		env: impl ProcessingEnvironment,
	) -> FlattenResult {
		self.flatten_with(context, env, JsonLdOptions::default())
			.await
	}

	/// Serialize the document to RDF using the given `options`.
	#[allow(async_fn_in_trait)]
	async fn to_rdf_with(
		&self,
		env: impl ProcessingEnvironment,
		mut generator: impl Generator,
		options: JsonLdOptions,
	) -> Result<Vec<Quad>, Error> {
		let rdf_serialization_options = options.rdf_serialization_options();
		let mut expanded = JsonLdProcessor::expand_with(self, env, options).await?;
		expanded.relabel(&mut generator);
		let interpretation = GeneratorInterpretation::new(generator);
		Ok(
			to_rdf_quads_interpretation_with(&expanded, interpretation, rdf_serialization_options)
				.unwrap(),
		)
	}
}
