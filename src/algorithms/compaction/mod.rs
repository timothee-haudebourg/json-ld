//! Compaction algorithm.
//!
//! See: <https://www.w3.org/TR/json-ld-api/#compaction-algorithms>
mod collection;
mod document;
mod iri;
mod object;
mod options;

use json_syntax::{JsonObject, JsonValue};
pub use options::*;

use crate::{
	JsonLdBacktraceBuilder, JsonLdIndexed, JsonLdLocatedError, JsonLdProcessedContext, JsonLdTerm,
	algorithms::AsyncJsonLdEnvironment,
	context::{
		RawJsonLdProcessedContext,
		inverse::{JsonLdLangSelection, JsonLdTypeSelection},
	},
	syntax::JsonLdKeyword,
};

/// Document that can be compacted.
pub trait JsonLdCompact {
	/// Compacts the input document with the given options.
	#[allow(async_fn_in_trait)]
	async fn compact_with(
		&self,
		env: impl AsyncJsonLdEnvironment,
		context: &JsonLdProcessedContext<'_>,
		options: JsonLdCompactionOptions,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError>;

	/// Compacts the input document with the default options.
	#[allow(async_fn_in_trait)]
	async fn compact(
		&self,
		env: impl AsyncJsonLdEnvironment,
		context: &JsonLdProcessedContext<'_>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.compact_with(env, context, JsonLdCompactionOptions::default(), location)
			.await
	}
}

impl<T: JsonLdCompact> JsonLdCompact for std::sync::Arc<T> {
	async fn compact_with(
		&self,
		env: impl AsyncJsonLdEnvironment,
		context: &JsonLdProcessedContext<'_>,
		options: JsonLdCompactionOptions,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		T::compact_with(self, env, context, options, location).await
	}

	async fn compact(
		&self,
		env: impl AsyncJsonLdEnvironment,
		context: &JsonLdProcessedContext<'_>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		T::compact(self, env, context, location).await
	}
}

/// Compactor.
struct Compactor<'a> {
	pub options: JsonLdCompactionOptions,
	pub active_context: &'a RawJsonLdProcessedContext,
	pub type_scoped_context: &'a RawJsonLdProcessedContext,
	pub active_property: Option<&'a str>,
}

impl Compactor<'_> {
	/// Wraps a compacted top-level array into a node object, using the
	/// (possibly aliased) `@graph` key.
	///
	/// An empty array becomes an empty object. Any other value is returned
	/// unchanged. This is the last compaction step before the context is
	/// embedded into the result.
	fn wrap_in_graph(&self, compact: JsonValue) -> Result<JsonValue, JsonLdLocatedError> {
		let JsonValue::Array(array) = compact else {
			return Ok(compact);
		};

		let mut obj = JsonObject::new();

		if !array.is_empty() {
			// No meaningful source location available at this post-processing step.
			let key = self.compact_iri(
				&JsonLdTerm::Keyword(JsonLdKeyword::Graph),
				true,
				false,
				JsonLdBacktraceBuilder::Root,
			)?;

			obj.insert(key.unwrap(), array.into());
		}

		Ok(obj.into())
	}
}

impl<'a> Compactor<'a> {
	pub fn new(
		active_context: &'a RawJsonLdProcessedContext,
		options: JsonLdCompactionOptions,
	) -> Self {
		Self {
			options,
			active_context,
			type_scoped_context: active_context,
			active_property: None,
		}
	}

	pub fn with_active_context<'b>(
		&'b self,
		active_context: &'b RawJsonLdProcessedContext,
	) -> Compactor<'b> {
		Compactor {
			options: self.options,
			active_context,
			type_scoped_context: self.type_scoped_context,
			active_property: self.active_property,
		}
	}

	pub fn with_type_scoped_context<'b>(
		&'b self,
		type_scoped_context: &'b RawJsonLdProcessedContext,
	) -> Compactor<'b> {
		Compactor {
			options: self.options,
			active_context: self.active_context,
			type_scoped_context,
			active_property: self.active_property,
		}
	}

	pub fn with_active_property<'b>(&'b self, active_property: Option<&'b str>) -> Compactor<'b> {
		Compactor {
			options: self.options,
			active_context: self.active_context,
			type_scoped_context: self.type_scoped_context,
			active_property,
		}
	}
}

trait CompactFragment {
	#[allow(async_fn_in_trait)]
	async fn compact_fragment(
		&self,
		env: &impl AsyncJsonLdEnvironment,
		compactor: &Compactor,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError>;
}

enum TypeLangValue<'a> {
	Type(JsonLdTypeSelection),
	Lang(JsonLdLangSelection<'a>),
}

/// Type that can be compacted with an index.
trait CompactIndexedFragment {
	#[allow(async_fn_in_trait)]
	#[allow(clippy::too_many_arguments)]
	async fn compact_indexed_fragment(
		&self,
		env: &impl AsyncJsonLdEnvironment,
		compactor: &Compactor<'_>,
		index: Option<&str>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError>;
}

impl<T: CompactIndexedFragment> CompactFragment for JsonLdIndexed<T> {
	async fn compact_fragment(
		&self,
		env: &impl AsyncJsonLdEnvironment,
		compactor: &Compactor<'_>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<JsonValue, JsonLdLocatedError> {
		self.inner()
			.compact_indexed_fragment(env, compactor, self.index(), location)
			.await
	}
}
