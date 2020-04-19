use mown::Mown;
use futures::future::{LocalBoxFuture, FutureExt};
use iref::Iri;
use json::JsonValue;
use crate::{
	Error,
	ErrorCode,
	Keyword,
	Id,
	Term,
	Value,
	Object,
	MutableActiveContext,
	LocalContext,
	ContextLoader,
	ContextProcessingOptions
};
use crate::util::as_array;
use super::{Expanded, Entry, ExpansionOptions, expand_literal, expand_array, expand_value, expand_node, expand_iri};

/// https://www.w3.org/TR/json-ld11-api/#expansion-algorithm
/// The default specified value for `ordered` and `from_map` is `false`.
pub fn expand_element<'a, T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &'a C, active_property: Option<&'a str>, element: &'a JsonValue, base_url: Option<Iri<'a>>, loader: &'a mut L, options: ExpansionOptions) -> LocalBoxFuture<'a, Result<Expanded<T>, Error>> where C::LocalContext: From<JsonValue> {
	async move {
		// If `element` is null, return null.
		if element.is_null() {
			return Ok(Expanded::Null)
		}

		let active_property_definition = active_context.get_opt(active_property);

		// // If `active_property` is `@default`, initialize the `frame_expansion` flag to `false`.
		// if active_property == Some("@default") {
		// 	frame_expansion = false;
		// }

		// If `active_property` has a term definition in `active_context` with a local context,
		// initialize property-scoped context to that local context.
		let mut property_scoped_base_url = None;
		let property_scoped_context = if let Some(definition) = active_property_definition {
			if let Some(base_url) = &definition.base_url {
				property_scoped_base_url = Some(base_url.as_iri());
			}

			definition.context.as_ref()
		} else {
			None
		};

		match element {
			JsonValue::Null => unreachable!(),
			JsonValue::Array(element) => {
				expand_array(active_context, active_property, active_property_definition, element, base_url, loader, options).await
			},

			JsonValue::Object(element) => {
				// We will need to consider expanded keys, and maybe ordered keys.
				let mut entries: Vec<Entry<&'a str>> = Vec::with_capacity(element.len());
				for (key, value) in element.iter() {
					entries.push(Entry(key, value));
				}

				if options.ordered {
					entries.sort()
				}

				let mut value_entry: Option<&JsonValue> = None;
				let mut id_entry = None;

				for Entry(key, value) in entries.iter() {
					match expand_iri(active_context, key, false, true) {
						Term::Keyword(Keyword::Value) => {
							value_entry = Some(value)
						},
						Term::Keyword(Keyword::Id) => {
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
					if value_entry.is_none() && !(element.len() == 1 && id_entry.is_some()) {
						active_context = Mown::Owned(previous_context.clone())
					}
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of
				// the Context Processing algorithm, passing `active_context`,
				// `property_scoped_context` as `local_context`, `base_url` from the term
				// definition for `active_property`, in `active_context` and `true` for
				// `override_protected`.
				if let Some(property_scoped_context) = property_scoped_context {
					let options: ContextProcessingOptions = options.into();
					active_context = Mown::Owned(property_scoped_context.process_with(active_context.as_ref(), loader, property_scoped_base_url, options.with_override()).await?);
				}

				// If `element` contains the entry `@context`, set `active_context` to the result
				// of the Context Processing algorithm, passing `active_context`, the value of the
				// `@context` entry as `local_context` and `base_url`.
				if let Some(local_context) = element.get("@context") {
					active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), loader, base_url, options.into()).await?);
				}

				let mut type_entries = Vec::new();
				for Entry(key, value) in entries.iter() {
					let expanded_key = expand_iri(active_context.as_ref(), key, false, true);
					match &expanded_key {
						Term::Keyword(Keyword::Type) => {
							type_entries.push(Entry(key, value));
						},
						_ => ()
					}
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
								let options: ContextProcessingOptions = options.into();
								active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), loader, base_url, options.without_propagation()).await?);
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

				let mut expanded_entries = Vec::with_capacity(element.len());
				let mut list_entry = None;
				let mut set_entry = None;
				value_entry = None;
				for Entry(key, value) in entries.iter() {
					let expanded_key = expand_iri(active_context.as_ref(), key, false, true);
					match &expanded_key {
						Term::Unknown(_) => {
							warn!("failed to expand key `{}`", key)
						},
						Term::Keyword(Keyword::Value) => {
							value_entry = Some(value)
						},
						Term::Keyword(Keyword::List) if active_property.is_some() && active_property != Some("@graph") => {
							list_entry = Some(value)
						},
						Term::Keyword(Keyword::Set) => {
							set_entry = Some(value)
						},
						_ => ()
					}

					expanded_entries.push(Entry((*key, expanded_key), value))
				}

				if let Some(list_entry) = list_entry {
					for Entry((_, expanded_key), _) in expanded_entries {
						match expanded_key {
							Term::Keyword(Keyword::Index) => {
								panic!("TODO list index")
							},
							Term::Keyword(Keyword::List) => (),
							_ => {
								return Err(ErrorCode::InvalidSetOrListObject.into())
							}
						}
					}

					// Initialize expanded value to the result of using this algorithm
					// recursively passing active context, active property, value for element,
					// base URL, and the frameExpansion and ordered flags, ensuring that the
					// result is an array..
					let mut result = Vec::new();
					for item in as_array(list_entry) {
						result.extend(expand_element(active_context.as_ref(), active_property, item, base_url, loader, options).await?)
					}

					Ok(Expanded::Object(Value::List(result).into()))
				} else if let Some(set_entry) = set_entry {
					for Entry((_, expanded_key), _) in expanded_entries {
						match expanded_key {
							Term::Keyword(Keyword::Index) => {
								panic!("TODO set index")
							},
							Term::Keyword(Keyword::Set) => (),
							_ => {
								return Err(ErrorCode::InvalidSetOrListObject.into())
							}
						}
					}

					// set expanded value to the result of using this algorithm recursively,
					// passing active context, active property, value for element, base URL, and
					// the frameExpansion and ordered flags.
					expand_element(active_context.as_ref(), active_property, set_entry, base_url, loader, options).await
				} else if let Some(value_entry) = value_entry {
					if let Some(value) = expand_value(input_type, type_scoped_context, expanded_entries, value_entry)? {
						Ok(Expanded::Object(value.into()))
					} else {
						Ok(Expanded::Null)
					}
				} else {
					if let Some((result, result_data)) = expand_node(active_context.as_ref(), type_scoped_context, active_property, expanded_entries, base_url, loader, options).await? {
						Ok(Expanded::Object(Object::Node(result, result_data)))
					} else {
						Ok(Expanded::Null)
					}
				}
			},

			_ => {
				// If element is a scalar (bool, int, string, null),
				// If `active_property` is `null` or `@graph`, drop the free-floating scalar by
				// returning null.
				if active_property.is_none() || active_property == Some("@graph") {
					return Ok(Expanded::Null)
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

					let result = property_scoped_context.process_with(active_context, loader, base_url, options.into()).await?;
					Mown::Owned(result)
				} else {
					Mown::Borrowed(active_context)
				};

				// Return the result of the Value Expansion algorithm, passing the `active_context`,
				// `active_property`, and `element` as value.
				return Ok(Expanded::Object(expand_literal(active_context.as_ref(), active_property, element)?))
			}
		}
	}.boxed_local()
}
