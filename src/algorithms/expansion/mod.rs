//! JSON-LD expansion algorithms.
//!
//! See: <https://www.w3.org/TR/json-ld-api/#expansion-algorithms>
use crate::{
	Document, ExpandedDocument, IndexedObject, Object,
	algorithms::{AsyncProcessingEnvironment, JsonLdSourceRef},
	context::RawProcessedContext,
};

use super::{JsonLdError, JsonLdLocated, JsonLdLocationStack};

mod array;
mod document;
mod element;
mod environment;
mod expanded;
mod iri;
mod literal;
mod node;
mod options;
mod value;

use element::*;
use environment::*;
use expanded::*;
use literal::*;
use node::node_id_of_term;
pub use options::*;

/// Low-level expansion trait.
///
/// This provides direct access to the expansion algorithm with full control
/// over the active context and options. For the high-level processor entry
/// point, use [`JsonLdProcessor::expand_with`](crate::JsonLdProcessor::expand_with).
pub trait Expand {
	/// Expand this document with the default expansion options.
	#[allow(async_fn_in_trait)]
	async fn expand(
		&self,
		env: impl AsyncProcessingEnvironment,
	) -> Result<ExpandedDocument, JsonLdLocated<JsonLdError>>;

	/// Expand this document with the given expansion options and active
	/// context.
	#[allow(async_fn_in_trait)]
	async fn expand_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		active_context: &RawProcessedContext,
		options: ExpansionOptions,
	) -> Result<ExpandedDocument, JsonLdLocated<JsonLdError>>;
}

impl Expand for Document {
	async fn expand(
		&self,
		env: impl AsyncProcessingEnvironment,
	) -> Result<ExpandedDocument, JsonLdLocated<JsonLdError>> {
		let active_context = RawProcessedContext::new(self.url().map(ToOwned::to_owned));
		self.expand_with(env, &active_context, ExpansionOptions::default())
			.await
	}

	async fn expand_with(
		&self,
		env: impl AsyncProcessingEnvironment,
		active_context: &RawProcessedContext,
		options: ExpansionOptions,
	) -> Result<ExpandedDocument, JsonLdLocated<JsonLdError>> {
		let loc = JsonLdLocationStack::new().file(JsonLdSourceRef::Compact(self.url()));
		Expander {
			base_url: self.url(),
			options,
			active_context,
			active_property: None,
		}
		.expand_document(&env, self.document(), loc)
		.await
	}
}

fn filter_top_level_item(item: &IndexedObject) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}
