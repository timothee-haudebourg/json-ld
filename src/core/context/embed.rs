use crate::{JsonLdLocatedError, syntax::JsonLdContext};

/// Context embeding method.
///
/// This trait provides the `embed_context` method that can be used
/// to include a JSON-LD context to a JSON-LD document.
pub trait JsonLdEmbedContext {
	/// Embeds the given context into the document.
	fn embed_context(&mut self, context: &JsonLdContext) -> Result<(), JsonLdLocatedError>;
}

#[cfg(feature = "serde")]
impl JsonLdEmbedContext for json_syntax::JsonValue {
	fn embed_context(&mut self, context: &JsonLdContext) -> Result<(), JsonLdLocatedError> {
		if let Some(obj) = self.as_object_mut() {
			let json_context = json_syntax::to_value(context).unwrap();

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
