use std::convert::TryFrom;
use iref::{Iri, IriRef, IriBuf};
use crate::{Keyword, context::{ActiveContext, Id, Key}};

pub fn expand_iri<T: Id, C: ActiveContext<T>>(active_context: &C, value: &str, document_relative: bool, vocab: bool) -> Option<Key<T>> {
	if let Ok(keyword) = Keyword::try_from(value) {
		Some(Key::Keyword(keyword))
	} else {
		// If value has the form of a keyword, a processor SHOULD generate a warning and return
		// null.
		// TODO

		if let Some(term_definition) = active_context.get(value) {
			// If active context has a term definition for value, and the associated IRI mapping
			// is a keyword, return that keyword.

			// If vocab is true and the active context has a term definition for value, return the
			// associated IRI mapping.
			if term_definition.value.is_keyword() || vocab {
				return Some(term_definition.value.clone())
			}
		}

		// If value contains a colon (:) anywhere after the first character, it is either an IRI,
		// a compact IRI, or a blank node identifier:
		if let Some(index) = value.find(':') {
			if index > 0 {
				// Split value into a prefix and suffix at the first occurrence of a colon (:).
				let (prefix, suffix) = value.split_at(index);

				// If prefix is underscore (_) or suffix begins with double-forward-slash (//),
				// return value as it is already an IRI or a blank node identifier.
				if prefix == "_" {
					return Some(Key::Id(T::from_blank_id(suffix)))
				}

				if suffix.starts_with("//") {
					if let Ok(iri) = Iri::new(value) {
						return Some(Key::Id(T::from_iri(iri)))
					} else {
						return None
					}
				}

				// If active context contains a term definition for prefix having a non-null IRI
				// mapping and the prefix flag of the term definition is true, return the result
				// of concatenating the IRI mapping associated with prefix and suffix.
				if let Some(term_definition) = active_context.get(prefix) {
					if term_definition.prefix == Some(true) {
						if let Some(iri) = term_definition.value.iri() {
							let mut result = iri.as_str().to_string();
							result.push_str(suffix);

							if let Ok(result) = Iri::new(&result) {
								return Some(Key::Id(T::from_iri(result)))
							} else {
								return None
							}
						}
					}
				}

				// If value has the form of an IRI, return value.
				if let Ok(result) = Iri::new(value) {
					return Some(Key::Id(T::from_iri(result)))
				}
			}
		}

		// If vocab is true, and active context has a vocabulary mapping, return the result of
		// concatenating the vocabulary mapping with value.
		if vocab {
			if let Some(vocabulary) = active_context.vocabulary() {
				if let Key::Id(id) = vocabulary {
					if let Some(iri) = id.iri() {
						let mut result = iri.as_str().to_string();
						result.push_str(value);

						if let Ok(result) = Iri::new(&result) {
							return Some(Key::Id(T::from_iri(result)))
						} else {
							return None
						}
					} else {
						return None
					}
				} else {
					return None
				}
			}
		}

		// Otherwise, if document relative is true set value to the result of resolving value
		// against the base IRI from active context. Only the basic algorithm in section 5.2 of
		// [RFC3986] is used; neither Syntax-Based Normalization nor Scheme-Based Normalization
		// are performed. Characters additionally allowed in IRI references are treated in the
		// same way that unreserved characters are treated in URI references, per section 6.5 of
		// [RFC3987].
		if document_relative {
			if let Ok(iri_ref) = IriRef::new(value) {
				if let Some(base_iri) = active_context.base_iri() {
					let value = iri_ref.resolved(base_iri);
					return Some(Key::Id(T::from_iri(value.as_iri())))
				} else {
					return None
				}
			} else {
				return None
			}
		}

		// Return value as is.
		None
	}
}
