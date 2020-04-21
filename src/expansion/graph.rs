use std::collections::HashSet;
use std::convert::TryInto;
use futures::future::{LocalBoxFuture, FutureExt};
use mown::Mown;
use iref::Iri;
use json::JsonValue;
use crate::{
	Error,
	ErrorCode,
	ProcessingMode,
	Keyword,
	Container,
	ContainerType,
	LangString,
	Id,
	Term,
	Indexed,
	object::*,
	MutableActiveContext,
	LocalContext,
	ContextLoader,
	ProcessingStack
};
use crate::util::as_array;
use super::{Expanded, Entry, ExpansionOptions, expand_element, expand_literal, expand_iri, filter_top_level_item};

pub async fn expand_graph<T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &C, type_scoped_context: &C, active_property: Option<&str>, expanded_entries: Vec<Entry<'_, (&str, Term<T>)>>, base_url: Option<Iri<'_>>, loader: &mut L, options: ExpansionOptions) -> Result<Option<Indexed<Graph<T>>>, Error> where C::LocalContext: From<JsonValue> {
	// Initialize two empty maps, `result` and `nests`.
	let mut result = Indexed::new(Graph::new(), None);
	let mut has_value_object_entries = false;

	expand_graph_entries(&mut result, &mut has_value_object_entries, active_context, type_scoped_context, active_property, expanded_entries, base_url, loader, options).await?;

	// If result contains the entry @value:
	// The result must not contain any entries other than @direction, @index,
	// @language, @type, and @value.

	// Otherwise, if result contains the entry @type and its
	// associated value is not an array, set it to an array
	// containing only the associated value.
	// FIXME TODO

	// Otherwise, if result contains the entry @set or @list:
	// FIXME TODO

	if has_value_object_entries && result.is_empty() && result.id.is_none() {
		return Ok(None)
	}

	// If active property is null or @graph, drop free-floating
	// values as follows:
	if active_property == None || active_property == Some("@graph") {
		// If `result` is a map which is empty, or contains only the entries `@value`
		// or `@list`, set `result` to null.
		// => drop values

		// Otherwise, if result is a map whose only entry is @id, set result to null.
		if result.is_empty() {
			return Ok(None)
		}
	}

	Ok(Some(result))
}

fn expand_graph_entries<'a, T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(result: &'a mut Indexed<Node<T>>, has_value_object_entries: &'a mut bool, active_context: &'a C, type_scoped_context: &'a C, active_property: Option<&'a str>, expanded_entries: Vec<Entry<'a, (&'a str, Term<T>)>>, base_url: Option<Iri<'a>>, loader: &'a mut L, options: ExpansionOptions) -> LocalBoxFuture<'a, Result<(), Error>> where C::LocalContext: From<JsonValue> {
	async move {
		// For each `key` and `value` in `element`, ordered lexicographically by key
		// if `ordered` is `true`:
		for Entry((key, expanded_key), value) in expanded_entries {
			match expanded_key {
				Term::Null | Term::Unknown(_) => (),

				// If key is @context, continue to the next key.
				Term::Keyword(Keyword::Context) => (),
				// Initialize `expanded_property` to the result of IRI expanding `key`.

				// If `expanded_property` is `null` or it neither contains a colon (:)
				// nor it is a keyword, drop key by continuing to the next key.
				// (already done)

				// If `expanded_property` is a keyword:
				Term::Keyword(expanded_property) => {
					// If `active_property` equals `@reverse`, an invalid reverse property
					// map error has been detected and processing is aborted.
					if active_property == Some("@reverse") {
						return Err(ErrorCode::InvalidReverseProperty.into())
					}

					// If `result` already has an `expanded_property` entry, other than
					// `@included` or `@type` (unless processing mode is json-ld-1.0), a
					// colliding keywords error has been detected and processing is
					// aborted.
					if let Some(expanded_property) = result.expanded_property.as_ref() {
						if options.processing_mode != ProcessingMode::JsonLd1_0 && *expanded_property != Term::Keyword(Keyword::Included) && *expanded_property != Term::Keyword(Keyword::Type) {
							return Err(ErrorCode::CollidingKeywords.into())
						}
					}

					match expanded_property {
						// If `expanded_property` is @id:
						Keyword::Id => {
							// If `value` is not a string, an invalid @id value error has
							// been detected and processing is aborted.
							if let Some(value) = value.as_str() {
								// Otherwise, set `expanded_value` to the result of IRI
								// expanding value using true for document relative and
								// false for vocab.
								result.id = Some(expand_iri(active_context, value, true, false))
							} else {
								return Err(ErrorCode::InvalidIdValue.into())
							}
						},
						// If expanded property is @graph
						Keyword::Graph => {
							// Set `expanded_value` to the result of using this algorithm
							// recursively passing `active_context`, `@graph` for active
							// property, `value` for element, `base_url`, and the
							// `frame_expansion` and `ordered` flags, ensuring that
							// `expanded_value` is an array of one or more maps.
							let expanded_value = expand_element(active_context, Some("@graph"), value, base_url, loader, options).await?;
							result.graph = Some(expanded_value.into_iter().filter(filter_top_level_item).collect());
						},
						// If expanded property is @index:
						Keyword::Index => {
							if let Some(value) = value.as_str() {
								result.set_index(Some(value.to_string()))
							} else {
								// If value is not a string, an invalid @index value
								// error has been detected and processing is aborted.
								return Err(ErrorCode::InvalidIndexValue.into())
							}
						},
						_ => {
							panic!("TODO error")
						}
					}
				}
			}
		};

		Ok(())
	}.boxed_local()
}
