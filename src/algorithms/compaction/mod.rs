//! Compaction algorithm.
//!
//! See: <https://www.w3.org/TR/json-ld-api/#compaction-algorithms>
mod collection;
mod document;
mod iri;
mod object;
mod options;

pub use options::*;

use crate::{
	algorithms::ProcessingEnvironment,
	context::{
		inverse::{LangSelection, TypeSelection},
		RawProcessedContext,
	},
	syntax::Keyword,
	Error, Indexed, ProcessedContext, Term,
};

/// Document that can be compacted.
pub trait Compact {
	/// Compacts the input document with the given options.
	#[allow(async_fn_in_trait)]
	async fn compact_with(
		&self,
		env: impl ProcessingEnvironment,
		context: &ProcessedContext<'_>,
		options: CompactionOptions,
	) -> Result<json_syntax::Value, Error>;

	/// Compacts the input document with the default options.
	#[allow(async_fn_in_trait)]
	async fn compact(
		&self,
		env: impl ProcessingEnvironment,
		context: &ProcessedContext<'_>,
	) -> Result<json_syntax::Value, Error> {
		self.compact_with(env, context, CompactionOptions::default())
			.await
	}
}

/// Compactor.
struct Compactor<'a> {
	pub options: CompactionOptions,
	pub active_context: &'a RawProcessedContext,
	pub type_scoped_context: &'a RawProcessedContext,
	pub active_property: Option<&'a str>,
}

impl<'a> Compactor<'a> {
	pub fn new(active_context: &'a RawProcessedContext, options: CompactionOptions) -> Self {
		Self {
			options,
			active_context,
			type_scoped_context: active_context,
			active_property: None,
		}
	}

	pub fn with_active_context<'b>(
		&'b self,
		active_context: &'b RawProcessedContext,
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
		type_scoped_context: &'b RawProcessedContext,
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
		env: &mut impl ProcessingEnvironment,
		compactor: &Compactor,
	) -> Result<json_syntax::Value, Error>;
}

enum TypeLangValue<'a> {
	Type(TypeSelection),
	Lang(LangSelection<'a>),
}

/// Type that can be compacted with an index.
trait CompactIndexedFragment {
	#[allow(async_fn_in_trait)]
	#[allow(clippy::too_many_arguments)]
	async fn compact_indexed_fragment(
		&self,
		env: &mut impl ProcessingEnvironment,
		compactor: &Compactor<'_>,
		index: Option<&str>,
	) -> Result<json_syntax::Value, Error>;
}

impl<T: CompactIndexedFragment> CompactFragment for Indexed<T> {
	async fn compact_fragment(
		&self,
		env: &mut impl ProcessingEnvironment,
		compactor: &Compactor<'_>,
	) -> Result<json_syntax::Value, Error> {
		self.inner()
			.compact_indexed_fragment(env, compactor, self.index())
			.await
	}
}

/// Context embeding method.
///
/// This trait provides the `embed_context` method that can be used
/// to include a JSON-LD context to a JSON-LD document.
/// It is used at the end of compaction algorithm to embed to
/// context used to compact the document into the compacted output.
pub trait EmbedContext {
	/// Embeds the given context into the document.
	fn embed_context(
		&mut self,
		context: &ProcessedContext,
		options: CompactionOptions,
	) -> Result<(), Error>;
}

impl EmbedContext for json_syntax::Value {
	fn embed_context(
		&mut self,
		context: &ProcessedContext,
		options: CompactionOptions,
	) -> Result<(), Error> {
		let value = self.take();

		let obj = match value {
			json_syntax::Value::Array(array) => {
				let mut obj = json_syntax::Object::new();

				if !array.is_empty() {
					let compactor = Compactor {
						options,
						active_context: context,
						type_scoped_context: context,
						active_property: None,
					};

					let key = compactor.compact_iri(&Term::Keyword(Keyword::Graph), true, false)?;

					obj.insert(key.unwrap(), array.into());
				}

				Some(obj)
			}
			json_syntax::Value::Object(obj) => Some(obj),
			_null => None,
		};

		if let Some(mut obj) = obj {
			// let json_context = IntoJson::into_json(context.unprocessed().clone());
			let json_context = json_syntax::to_value(context.unprocessed()).unwrap();

			if !obj.is_empty()
				&& !json_context.is_null()
				&& !json_context.is_empty_array_or_object()
			{
				obj.push_front("@context", json_context);
			}

			*self = obj.into()
		};

		Ok(())
	}
}
