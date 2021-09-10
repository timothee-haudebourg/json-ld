use crate::{
	syntax::{is_keyword_like, Keyword, Term},
	BlankId, Context, Id, Reference,
};
use iref::{Iri, IriRef};
use std::convert::TryFrom;

// Default value for `document_relative` is `false` and for `vocab` is `true`.
pub fn expand_iri<T: Id, C: Context<T>>(
	active_context: &C,
	value: &str,
	document_relative: bool,
	vocab: bool,
) -> Term<T> {
	if let Ok(keyword) = Keyword::try_from(value) {
		Term::Keyword(keyword)
	} else {
		// If value has the form of a keyword, a processor SHOULD generate a warning and return
		// null.
		if is_keyword_like(value) {
			return Term::Null;
		}

		if let Some(term_definition) = active_context.get(value) {
			// If active context has a term definition for value, and the associated IRI mapping
			// is a keyword, return that keyword.
			if let Some(value) = &term_definition.value {
				if value.is_keyword() {
					return Term::from(value.clone());
				}
			}

			// If vocab is true and the active context has a term definition for value, return the
			// associated IRI mapping.
			if vocab {
				if let Some(mapped_value) = &term_definition.value {
					return mapped_value.clone();
				} else {
					return Reference::Invalid(value.to_string()).into();
				}
			}
		}

		// If value contains a colon (:) anywhere after the first character, it is either an IRI,
		// a compact IRI, or a blank node identifier:
		if let Some(index) = value.find(':') {
			if index > 0 {
				// Split value into a prefix and suffix at the first occurrence of a colon (:).
				let (prefix, suffix) = value.split_at(index);
				let suffix = &suffix[1..suffix.len()];

				// If prefix is underscore (_) or suffix begins with double-forward-slash (//),
				// return value as it is already an IRI or a blank node identifier.
				if prefix == "_" {
					return Term::from(BlankId::new(suffix));
				}

				if suffix.starts_with("//") {
					if let Ok(iri) = Iri::new(value) {
						return Term::from(T::from_iri(iri));
					} else {
						return Reference::Invalid(value.to_string()).into();
					}
				}

				// If active context contains a term definition for prefix having a non-null IRI
				// mapping and the prefix flag of the term definition is true, return the result
				// of concatenating the IRI mapping associated with prefix and suffix.
				if let Some(term_definition) = active_context.get(prefix) {
					if term_definition.prefix {
						if let Some(mapping) = &term_definition.value {
							let mut result = mapping.as_str().to_string();
							result.push_str(suffix);

							if let Ok(result) = Iri::new(&result) {
								return Term::from(T::from_iri(result));
							} else {
								if let Ok(blank) = BlankId::try_from(result.as_ref()) {
									return Term::from(blank);
								} else {
									return Reference::Invalid(result).into();
								}
							}
						}
					}
				}

				// If value has the form of an IRI, return value.
				if let Ok(result) = Iri::new(value) {
					return Term::from(T::from_iri(result));
				}
			}
		}

		// If vocab is true, and active context has a vocabulary mapping, return the result of
		// concatenating the vocabulary mapping with value.
		if vocab {
			if let Some(vocabulary) = active_context.vocabulary() {
				if let Term::Ref(mapping) = vocabulary {
					let mut result = mapping.as_str().to_string();
					result.push_str(value);

					if let Ok(result) = Iri::new(&result) {
						return Term::from(T::from_iri(result)).into();
					} else {
						if let Ok(blank) = BlankId::try_from(result.as_ref()) {
							return Term::from(blank);
						} else {
							return Reference::Invalid(result).into();
						}
					}
				} else {
					return Reference::Invalid(value.to_string()).into();
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
					return Term::from(T::from_iri(value.as_iri()));
				} else {
					return Reference::Invalid(value.to_string()).into();
				}
			} else {
				return Reference::Invalid(value.to_string()).into();
			}
		}

		// Return value as is.
		Reference::Invalid(value.to_string()).into()
	}
}
