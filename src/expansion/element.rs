use mown::Mown;
use futures::future::{LocalBoxFuture, FutureExt};
use iref::Iri;
use json::JsonValue;
use crate::{Keyword, as_array, Id, Key, Value, Object};
use crate::context::{MutableActiveContext, LocalContext, ContextLoader};

use super::{ExpansionError, Entry, expand_literal, expand_array, expand_value, expand_node, expand_iri};

/// https://www.w3.org/TR/json-ld11-api/#expansion-algorithm
/// The default specified value for `ordered` and `from_map` is `false`.
pub fn expand_element<'a, T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &'a C, active_property: Option<&'a str>, element: &'a JsonValue, base_url: Option<Iri<'a>>, loader: &'a mut L, ordered: bool, from_map: bool) -> LocalBoxFuture<'a, Result<Option<Vec<Object<T>>>, ExpansionError>> where C::LocalContext: From<JsonValue> {
	async move {
		// If `element` is null, return null.
		if element.is_null() {
			return Ok(None)
		}

		let active_property_definition = active_context.get_opt(active_property);

		// // If `active_property` is `@default`, initialize the `frame_expansion` flag to `false`.
		// if active_property == Some("@default") {
		// 	frame_expansion = false;
		// }

		// If `active_property` has a term definition in `active_context` with a local context,
		// initialize property-scoped context to that local context.
		let property_scoped_context = if let Some(definition) = active_property_definition {
			definition.context.as_ref()
		} else {
			None
		};

		match element {
			JsonValue::Null => unreachable!(),
			JsonValue::Array(element) => {
				return Ok(Some(expand_array(active_context, active_property, active_property_definition, element, base_url, loader, ordered, from_map).await?))
			},

			JsonValue::Object(element) => {
				// We will need to consider expanded keys, and maybe ordered keys.
				let mut entries: Vec<Entry<&'a str>> = Vec::with_capacity(element.len());
				for (key, value) in element.iter() {
					entries.push(Entry(key, value));
				}

				if ordered {
					entries.sort()
				}

				let mut value_entry: Option<&JsonValue> = None;
				let mut id_entry = None;

				for Entry(key, value) in entries.iter() {
					match expand_iri(active_context, key, false, true) {
						Key::Keyword(Keyword::Value) => {
							value_entry = Some(value)
						},
						Key::Keyword(Keyword::Id) => {
							id_entry = Some(value)
						},
						_ => ()
					}
				}

				// Otherwise element is a map.
				// If `active_context` has a `previous_context`, the active context is not
				// propagated.
				let mut active_context = Mown::Borrowed(active_context);
				if let Some(previous_context) = active_context.previous_context() {
					// If `from_map` is undefined or false, and `element` does not contain an entry
					// expanding to `@value`, and `element` does not consist of a single entry
					// expanding to `@id` (where entries are IRI expanded), set active context to
					// previous context from active context, as the scope of a term-scoped context
					// does not apply when processing new Object objects.
					if !from_map && value_entry.is_none() && !(element.len() == 1 && id_entry.is_some()) {
						active_context = Mown::Owned(previous_context.clone())
					}
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of
				// the Context Processing algorithm, passing `active_context`,
				// `property_scoped_context` as `local_context`, `base_url` from the term
				// definition for `active_property`, in `active_context` and `true` for
				// `override_protected`.
				if let Some(property_scoped_context) = property_scoped_context {
					active_context = Mown::Owned(property_scoped_context.process(active_context.as_ref(), loader, base_url, false, true, true).await?);
				}

				// If `element` contains the entry `@context`, set `active_context` to the result
				// of the Context Processing algorithm, passing `active_context`, the value of the
				// `@context` entry as `local_context` and `base_url`.
				if let Some(local_context) = element.get("@context") {
					active_context = Mown::Owned(local_context.process(active_context.as_ref(), loader, base_url, false, false, true).await?);
				}

				let mut expanded_entries = Vec::with_capacity(element.len());
				let mut type_entries = Vec::new();
				let mut list_entry = None;
				let mut set_entry = None;
				value_entry = None;
				for Entry(key, value) in entries.iter() {
					let expanded_key = expand_iri(active_context.as_ref(), key, false, true);
					match &expanded_key {
						Key::Unknown(_) => {
							warn!("failed to expand key `{}`", key)
						},
						Key::Keyword(Keyword::Value) => {
							value_entry = Some(value)
						},
						Key::Keyword(Keyword::List) if active_property.is_some() && active_property != Some("@graph") => {
							list_entry = Some(value)
						},
						Key::Keyword(Keyword::Set) => {
							set_entry = Some(value)
						},
						Key::Keyword(Keyword::Type) => {
							type_entries.push(Entry(key, value));
						},
						_ => ()
					}

					expanded_entries.push(Entry((*key, expanded_key), value))
				}

				type_entries.sort();

				// Initialize `type_scoped_context` to `active_context`.
				// This is used for expanding values that may be relevant to any previous
				// type-scoped context.
				let type_scoped_context = active_context.as_ref();
				let mut active_context = Mown::Borrowed(active_context.as_ref());

				// For each `key` and `value` in `element` ordered lexicographically by key where
				// key IRI expands to @type:
				for Entry(_, value) in &type_entries {
					// Convert `value` into an array, if necessary.
					let value = as_array(value);

					// For each `term` which is a value of `value` ordered lexicographically,
					let mut sorted_value = Vec::with_capacity(value.len());
					for term in value {
						if let Some(term) = term.as_str() {
							sorted_value.push(term);
						}
					}
					sorted_value.sort();

					// if `term` is a string, and `term`'s term definition in `type_scoped_context`
					// has a `local_context`,
					for term in sorted_value {
						if let Some(term_definition) = type_scoped_context.get(term) {
							if let Some(local_context) = &term_definition.context {
								// set `active_context` to the result of
								// Context Processing algorithm, passing `active_context`, the value of the
								// `term`'s local context as `local_context`, `base_url` from the term
								// definition for value in `active_context`, and `false` for `propagate`.
								let base_url = term_definition.base_url.as_ref().map(|url| url.as_iri());
								active_context = Mown::Owned(local_context.process(active_context.as_ref(), loader, base_url, false, false, false).await?);
							}
						}
					}
				}

				// Initialize `input_type` to expansion of the last value of the first entry in
				// `element` expanding to `@type` (if any), ordering entries lexicographically by
				// key.
				// Both the key and value of the matched entry are IRI expanded.
				let input_type = if let Some(Entry(_, value)) = type_entries.first() {
					if let Some(input_type) = as_array(value).last() {
						if let Some(input_type) = input_type.as_str() {
							Some(expand_iri(active_context.as_ref(), input_type, false, true))
						} else {
							None
						}
					} else {
						None
					}
				} else {
					None
				};

				if let Some(list_entry) = list_entry {
					for Entry((_, expanded_key), _) in expanded_entries {
						match expanded_key {
							Key::Keyword(Keyword::Index) => {
								panic!("TODO list index")
							},
							Key::Keyword(Keyword::List) => (),
							_ => {
								return Err(ExpansionError::InvalidListEntry)
							}
						}
					}

					// Initialize expanded value to the result of using this algorithm
					// recursively passing active context, active property, value for element,
					// base URL, and the frameExpansion and ordered flags, ensuring that the
					// result is an array..
					let result = if let Some(expanded_value) = expand_element(active_context.as_ref(), active_property, list_entry, base_url, loader, ordered, false).await? {
						Some(vec![Value::List(expanded_value).into()])
					} else {
						None
					};

					return Ok(result)
				} else if let Some(set_entry) = set_entry {
					for Entry((_, expanded_key), _) in expanded_entries {
						match expanded_key {
							Key::Keyword(Keyword::Index) => {
								panic!("TODO set index")
							},
							Key::Keyword(Keyword::Set) => (),
							_ => {
								return Err(ExpansionError::InvalidSetEntry)
							}
						}
					}

					// set expanded value to the result of using this algorithm recursively,
					// passing active context, active property, value for element, base URL, and
					// the frameExpansion and ordered flags.
					let result = if let Some(expanded_value) = expand_element(active_context.as_ref(), active_property, set_entry, base_url, loader, ordered, false).await? {
						Some(expanded_value)
					} else {
						Some(vec![])
					};

					return Ok(result)
				} else if let Some(value_entry) = value_entry {
					if let Some(value) = expand_value(input_type, type_scoped_context, expanded_entries, value_entry)? {
						return Ok(Some(vec![value]))
					} else {
						return Ok(None)
					}
				} else {
					if let Some((result, result_data)) = expand_node(active_context.as_ref(), type_scoped_context, active_property, expanded_entries, base_url, loader, ordered).await? {
						return Ok(Some(vec![Object::Node(result, result_data)]))
					} else {
						return Ok(None)
					}
				}
			},

			_ => {
				// If element is a scalar (bool, int, string, null),
				// If `active_property` is `null` or `@graph`, drop the free-floating scalar by
				// returning null.
				if active_property.is_none() || active_property == Some("@graph") {
					return Ok(None)
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of the
				// Context Processing algorithm, passing `active_context`, `property_scoped_context` as
				// local context, and `base_url` from the term definition for `active_property` in
				// `active context`.
				let active_context = if let Some(property_scoped_context) = property_scoped_context {
					// FIXME it is unclear what we should use as `base_url` if there is no term definition for `active_context`.
					let base_url = if let Some(definition) = active_context.get_opt(active_property) {
						if let Some(base_url) = &definition.base_url {
							Some(base_url.as_iri())
						} else {
							None
						}
					} else {
						None
					};

					let result = property_scoped_context.process(active_context, loader, base_url, false, false, true).await?;
					Mown::Owned(result)
				} else {
					Mown::Borrowed(active_context)
				};

				// Return the result of the Value Expansion algorithm, passing the `active_context`,
				// `active_property`, and `element` as value.
				let mut result = Vec::new();
				result.push(expand_literal(active_context.as_ref(), active_property, element)?);
				return Ok(Some(result))
			}
		}
	}.boxed_local()
}
