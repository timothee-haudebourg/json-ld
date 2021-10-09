use super::{
	expand_array, expand_iri, expand_literal, expand_node, expand_value, Entry, Expanded,
	ExpandedEntry, LiteralValue, Options,
};
use crate::util::as_array;
use crate::{
	context::{ContextMut, Loader, Local, ProcessingOptions},
	object::*,
	syntax::{Keyword, Term},
	Error, ErrorCode, Id, Indexed,
};
use cc_traits::{CollectionRef, Get, KeyedRef, Len, MapIter};
use futures::future::{FutureExt, LocalBoxFuture};
use generic_json::{Json, JsonClone, JsonHash, ValueRef};
use iref::Iri;
use mown::Mown;

/// https://www.w3.org/TR/json-ld11-api/#expansion-algorithm
/// The default specified value for `ordered` and `from_map` is `false`.
pub fn expand_element<'a, J: JsonHash + JsonClone, T: Id, C: ContextMut<T>, L: Loader>(
	active_context: &'a C,
	active_property: Option<&'a str>,
	element: &'a J,
	base_url: Option<Iri<'a>>,
	loader: &'a mut L,
	options: Options,
	from_map: bool,
) -> LocalBoxFuture<'a, Result<Expanded<J, T>, Error>>
where
	C::LocalContext: From<L::Output> + From<J>,
	L::Output: Into<J>,
{
	async move {
		// If `element` is null, return null.
		if element.is_null() {
			return Ok(Expanded::Null);
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

		match element.as_value_ref() {
			ValueRef::Null => unreachable!(),
			ValueRef::Array(element) => {
				expand_array(
					active_context,
					active_property,
					active_property_definition,
					element,
					base_url,
					loader,
					options,
					from_map,
				)
				.await
			}

			ValueRef::Object(element) => {
				// We will need to consider expanded keys, and maybe ordered keys.
				let mut entries: Vec<Entry<J>> = Vec::with_capacity(element.len());
				for (key, value) in element.iter() {
					entries.push(Entry(key, value));
				}

				if options.ordered {
					entries.sort()
				}

				let mut value_entry1 = None;
				let mut id_entry = None;

				for Entry(key, value) in entries.iter() {
					match expand_iri(active_context, key.as_ref(), false, true) {
						Term::Keyword(Keyword::Value) => value_entry1 = Some(value.clone()),
						Term::Keyword(Keyword::Id) => id_entry = Some(value.clone()),
						_ => (),
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
					if !from_map
						&& value_entry1.is_none()
						&& !(element.len() == 1 && id_entry.is_some())
					{
						active_context = Mown::Owned(previous_context.clone())
					}
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of
				// the Context Processing algorithm, passing `active_context`,
				// `property_scoped_context` as `local_context`, `base_url` from the term
				// definition for `active_property`, in `active_context` and `true` for
				// `override_protected`.
				if let Some(property_scoped_context) = property_scoped_context {
					let options: ProcessingOptions = options.into();
					active_context = Mown::Owned(
						property_scoped_context
							.process_with(
								active_context.as_ref(),
								loader,
								property_scoped_base_url,
								options.with_override(),
							)
							.await?
							.into_inner(),
					);
				}

				// If `element` contains the entry `@context`, set `active_context` to the result
				// of the Context Processing algorithm, passing `active_context`, the value of the
				// `@context` entry as `local_context` and `base_url`.
				if let Some(local_context) = element.get("@context") {
					active_context = Mown::Owned(
						local_context
							.process_with(active_context.as_ref(), loader, base_url, options.into())
							.await?
							.into_inner(),
					);
				}

				let mut type_entries: Vec<Entry<J>> = Vec::new();
				for entry @ Entry(key, _) in entries.iter() {
					let expanded_key =
						expand_iri(active_context.as_ref(), key.as_ref(), false, true);
					if let Term::Keyword(Keyword::Type) = expanded_key {
						type_entries.push(entry.clone());
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
					let (value, len) = as_array(&**value);

					// For each `term` which is a value of `value` ordered lexicographically,
					let mut sorted_value = Vec::with_capacity(len);
					for term in value {
						if term.is_string() {
							sorted_value.push(term);
						}
					}
					sorted_value.sort_unstable_by(|a, b| a.as_str().cmp(&b.as_str()));

					// if `term` is a string, and `term`'s term definition in `type_scoped_context`
					// has a `local_context`,
					for term in sorted_value {
						let term = term.as_str().unwrap();
						if let Some(term_definition) = type_scoped_context.get(term) {
							if let Some(local_context) = &term_definition.context {
								// set `active_context` to the result of
								// Context Processing algorithm, passing `active_context`, the value of the
								// `term`'s local context as `local_context`, `base_url` from the term
								// definition for value in `active_context`, and `false` for `propagate`.
								let base_url =
									term_definition.base_url.as_ref().map(|url| url.as_iri());
								let options: ProcessingOptions = options.into();
								active_context = Mown::Owned(
									local_context
										.process_with(
											active_context.as_ref(),
											loader,
											base_url,
											options.without_propagation(),
										)
										.await?
										.into_inner(),
								);
							}
						}
					}
				}

				// Initialize `input_type` to expansion of the last value of the first entry in
				// `element` expanding to `@type` (if any), ordering entries lexicographically by
				// key.
				// Both the key and value of the matched entry are IRI expanded.
				let input_type = if let Some(Entry(_, value)) = type_entries.first() {
					let (value, _) = as_array(&**value);
					if let Some(input_type) = value.last() {
						input_type.as_str().map(|input_type| {
							expand_iri(active_context.as_ref(), input_type, false, true)
						})
					} else {
						None
					}
				} else {
					None
				};

				let mut expanded_entries: Vec<ExpandedEntry<J, Term<T>>> =
					Vec::with_capacity(element.len());
				let mut list_entry = None;
				let mut set_entry = None;
				let mut value_entry = None;
				for Entry(key, value) in entries {
					let expanded_key =
						expand_iri(active_context.as_ref(), key.as_ref(), false, true);
					match &expanded_key {
						Term::Keyword(Keyword::Value) => value_entry = Some(value.clone()),
						Term::Keyword(Keyword::List)
							if active_property.is_some() && active_property != Some("@graph") =>
						{
							list_entry = Some(value.clone())
						}
						Term::Keyword(Keyword::Set) => set_entry = Some(value.clone()),
						_ => (),
					}

					expanded_entries.push(ExpandedEntry(
						J::Object::upcast_key_ref(key),
						expanded_key,
						J::Object::upcast_item_ref(value),
					))
				}

				if let Some(list_entry) = list_entry {
					// List objects.
					let mut index = None;
					for ExpandedEntry(_, expanded_key, value) in expanded_entries {
						match expanded_key {
							Term::Keyword(Keyword::Index) => match value.as_str() {
								Some(value) => index = Some(value.to_string()),
								None => return Err(ErrorCode::InvalidIndexValue.into()),
							},
							Term::Keyword(Keyword::List) => (),
							_ => return Err(ErrorCode::InvalidSetOrListObject.into()),
						}
					}

					// Initialize expanded value to the result of using this algorithm
					// recursively passing active context, active property, value for element,
					// base URL, and the frameExpansion and ordered flags, ensuring that the
					// result is an array..
					let mut result = Vec::new();
					let (list_entry, _) = as_array(&*list_entry);
					for item in list_entry {
						result.extend(
							expand_element(
								active_context.as_ref(),
								active_property,
								&*item,
								base_url,
								loader,
								options,
								false,
							)
							.await?,
						)
					}

					Ok(Expanded::Object(Indexed::new(Object::List(result), index)))
				} else if let Some(set_entry) = set_entry {
					// Set objects.
					// let mut index = None;
					for ExpandedEntry(_, expanded_key, value) in expanded_entries {
						match expanded_key {
							Term::Keyword(Keyword::Index) => {
								match value.as_str() {
									Some(_value) => {
										panic!("TODO expand set @index");
										// index = Some(value.to_string())
									}
									None => return Err(ErrorCode::InvalidIndexValue.into()),
								}
							}
							Term::Keyword(Keyword::Set) => (),
							_ => return Err(ErrorCode::InvalidSetOrListObject.into()),
						}
					}

					// set expanded value to the result of using this algorithm recursively,
					// passing active context, active property, value for element, base URL, and
					// the frameExpansion and ordered flags.
					expand_element(
						active_context.as_ref(),
						active_property,
						&*set_entry,
						base_url,
						loader,
						options,
						false,
					)
					.await
				} else if let Some(value_entry) = value_entry {
					// Value objects.
					if let Some(value) = expand_value(
						input_type,
						type_scoped_context,
						expanded_entries,
						&*value_entry,
					)? {
						Ok(Expanded::Object(value))
					} else {
						Ok(Expanded::Null)
					}
				} else {
					// Node objects.
					if let Some(result) = expand_node(
						active_context.as_ref(),
						type_scoped_context,
						active_property,
						expanded_entries,
						base_url,
						loader,
						options,
					)
					.await?
					{
						Ok(result.cast::<Object<J, T>>().into())
					} else {
						Ok(Expanded::Null)
					}
				}
			}

			_ => {
				// Literals.

				// If element is a scalar (bool, int, string, null),
				// If `active_property` is `null` or `@graph`, drop the free-floating scalar by
				// returning null.
				if active_property.is_none() || active_property == Some("@graph") {
					return Ok(Expanded::Null);
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of the
				// Context Processing algorithm, passing `active_context`, `property_scoped_context` as
				// local context, and `base_url` from the term definition for `active_property` in
				// `active context`.
				let active_context = if let Some(property_scoped_context) = property_scoped_context
				{
					// FIXME it is unclear what we should use as `base_url` if there is no term definition for `active_context`.
					let base_url = active_context
						.get_opt(active_property)
						.map(|definition| {
							definition
								.base_url
								.as_ref()
								.map(|base_url| base_url.as_iri())
						})
						.flatten();

					let result = property_scoped_context
						.process_with(active_context, loader, base_url, options.into())
						.await?
						.into_inner();
					Mown::Owned(result)
				} else {
					Mown::Borrowed(active_context)
				};

				// Return the result of the Value Expansion algorithm, passing the `active_context`,
				// `active_property`, and `element` as value.
				return Ok(Expanded::Object(expand_literal(
					active_context.as_ref(),
					active_property,
					LiteralValue::Given(element),
				)?));
			}
		}
	}
	.boxed_local()
}
