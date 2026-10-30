use json_syntax::JsonValue;

use crate::{JsonLdLocatedError, ProcessedJsonLdContext};

/// Context embeding method.
///
/// This trait provides the `embed_context` method that can be used
/// to include a JSON-LD context to a JSON-LD document.
pub trait JsonLdEmbedContext {
	/// Embeds the given context into the document.
	fn embed_context(&mut self, context: &ProcessedJsonLdContext)
	-> Result<(), JsonLdLocatedError>;
}

impl JsonLdEmbedContext for JsonValue {
	fn embed_context(
		&mut self,
		context: &ProcessedJsonLdContext,
	) -> Result<(), JsonLdLocatedError> {
		if let Some(obj) = self.as_object_mut() {
			let json_context = json_syntax::to_value(context.unprocessed()).unwrap();

			if !obj.is_empty()
				&& !json_context.is_null()
				&& !json_context.is_empty_array_or_object()
			{
				obj.push_front("@context", json_context);
			}
		};

		Ok(())
	}
}
