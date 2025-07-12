//! JSON-LD expansion algorithms.
//!
//! See: <https://www.w3.org/TR/json-ld-api/#expansion-algorithms>
use crate::{
	algorithms::ProcessingEnvironment, Document, ExpandedDocument, IndexedObject, Object,
	ProcessedContext,
};

use super::Error;

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

impl Document {
	/// Expand this document with the default expansion options.
	///
	/// The given `loader` is used to load remote documents (such as contexts)
	/// imported by the input and required during expansion.
	///
	/// # Example
	///
	/// ```
	/// # mod json_ld { pub use json_ld_syntax as syntax; pub use json_ld_core::{RemoteDocument, ExpandedDocument, NoLoader}; pub use json_ld_expansion::Expand; };
	///
	/// use iref::IriBuf;
	/// use rdf_types::BlankIdBuf;
	/// use static_iref::iri;
	/// use json_ld::{syntax::Parse, RemoteDocument, Expand};
	///
	/// # #[async_std::test]
	/// # async fn example() {
	/// // Parse the input JSON(-LD) document.
	/// let (json, _) = json_ld::syntax::Value::parse_str(
	///   r##"
	///   {
	///     "@graph": [
	///       {
	///         "http://example.org/vocab#a": {
	///           "@graph": [
	///             {
	///               "http://example.org/vocab#b": "Chapter One"
	///             }
	///           ]
	///         }
	///       }
	///     ]
	///   }
	///   "##)
	/// .unwrap();
	///
	/// // Prepare a dummy document loader using [`json_ld::NoLoader`],
	/// // since we won't need to load any remote document while expanding this one.
	/// let mut loader = json_ld::NoLoader;
	///
	/// // The `expand` method returns an [`json_ld::ExpandedDocument`].
	/// json
	///     .expand(&mut loader)
	///     .await
	///     .unwrap();
	/// # }
	/// ```
	pub async fn expand(&self, env: impl ProcessingEnvironment) -> Result<ExpandedDocument, Error> {
		let active_context = ProcessedContext::new(self.url().map(ToOwned::to_owned));
		self.expand_with(env, &active_context, ExpansionOptions::default())
			.await
	}

	/// Expand this document with the given expansion options and active
	/// context.
	pub async fn expand_with(
		&self,
		mut env: impl ProcessingEnvironment,
		active_context: &ProcessedContext,
		options: ExpansionOptions,
	) -> Result<ExpandedDocument, Error> {
		Expander {
			base_url: self.url(),
			options,
			active_context,
			active_property: None,
		}
		.expand_document(&mut env, self.document())
		.await
	}
}

fn filter_top_level_item(item: &IndexedObject) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}
