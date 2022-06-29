use super::{
	expand_element, expand_iri, expand_literal, filter_top_level_item, ActiveProperty, Entry,
	Expanded, ExpandedEntry, JsonExpand, LiteralValue, Options, Policy,
};
use crate::util::as_array;
use crate::{
	context::{ContextMut, Loader, Local, ProcessingOptions},
	object::*,
	syntax::{Container, ContainerType, Keyword, Term, Type},
	Error, ErrorCode, Id, Indexed, LangString, Loc, ProcessingMode, Reference, Warning,
};
use cc_traits::{Len, MapIter};
use futures::future::{BoxFuture, FutureExt};
use generic_json::{Json, Key, ValueRef};
use iref::Iri;
use langtag::LanguageTagBuf;
use mown::Mown;
use std::{collections::HashSet, convert::TryInto};

/// Convert a term to a node id, if possible.
/// Return `None` if the term is `null`.
pub fn node_id_of_term<T: Id>(term: Term<T>) -> Option<Reference<T>> {
	match term {
		Term::Null => None,
		Term::Ref(prop) => Some(prop),
		Term::Keyword(kw) => Some(Reference::Invalid(kw.into_str().to_string())),
	}
}

/// Expand a node object.
pub(crate) async fn expand_node<
	'a,
	J: JsonExpand,
	T: 'a + Id + Send + Sync,
	C: ContextMut<T> + Send + Sync,
	L: Loader + Send + Sync,
>(
	active_context: &'a C,
	type_scoped_context: &'a C,
	active_property: ActiveProperty<'a, J>,
	expanded_entries: Vec<ExpandedEntry<'a, J, Term<T>>>,
	base_url: Option<Iri<'a>>,
	loader: &'a mut L,
	options: Options,
	warnings: &'a mut Vec<Loc<Warning, J::MetaData>>,
) -> Result<Option<Indexed<Node<J, T>>>, Loc<Error, J::MetaData>>
where
	C::LocalContext: From<L::Output> + From<J>,
	L::Output: Into<J>,
{
	// Initialize two empty maps, `result` and `nests`.
	// let mut result = Indexed::new(Node::new(), None);
	// let mut has_value_object_entries = false;

	let (result, has_value_object_entries) = expand_node_entries(
		Indexed::new(Node::new(), None),
		false,
		active_context,
		type_scoped_context,
		active_property,
		expanded_entries,
		base_url,
		loader,
		options,
		warnings,
	)
	.await?;

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
		return Ok(None);
	}

	// If active property is null or @graph, drop free-floating
	// values as follows:
	if active_property.is_none() || active_property == Some("@graph") {
		// If `result` is a map which is empty,
		// [or contains only the entries `@value` or `@list` (does not apply here)]
		// set `result` to null.
		// Otherwise, if result is a map whose only entry is @id, set result to null.
		if result.is_empty() && result.index().is_none() {
			// both cases are covered by checking `is_empty`.
			return Ok(None);
		}
	}

	Ok(Some(result))
}

/// Type returned by the `expand_node_entries` function.
///
/// It is a tuple containing both the node being expanded
/// and a boolean flag set to `true` if the node contains
/// value object entries (in practice, if it has a `@language` entry).
type ExpandedNode<J, T> = (Indexed<Node<J, T>>, bool);

/// Result of the `expand_node_entries` function.
type NodeEntriesExpensionResult<J, T> =
	Result<ExpandedNode<J, T>, Loc<Error, <J as Json>::MetaData>>;

fn expand_node_entries<
	'a,
	J: JsonExpand,
	T: 'a + Id + Send + Sync,
	C: ContextMut<T> + Send + Sync,
	L: Loader + Send + Sync,
>(
	mut result: Indexed<Node<J, T>>,
	mut has_value_object_entries: bool,
	active_context: &'a C,
	type_scoped_context: &'a C,
	active_property: ActiveProperty<'a, J>,
	expanded_entries: Vec<ExpandedEntry<'a, J, Term<T>>>,
	base_url: Option<Iri<'a>>,
	loader: &'a mut L,
	options: Options,
	warnings: &'a mut Vec<Loc<Warning, J::MetaData>>,
) -> BoxFuture<'a, NodeEntriesExpensionResult<J, T>>
where
	C::LocalContext: From<L::Output> + From<J> + Send + Sync,
	L::Output: Into<J>,
{
	let source = loader.id_opt(base_url);
	async move {
		// For each `key` and `value` in `element`, ordered lexicographically by key
		// if `ordered` is `true`:
		for ExpandedEntry(key, expanded_key, value) in expanded_entries {
			match expanded_key {
				Term::Null => (),

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
						return Err(ErrorCode::InvalidReversePropertyMap
							.located(source, key.metadata().clone()));
					}

					// If `result` already has an `expanded_property` entry, other than
					// `@included` or `@type` (unless processing mode is json-ld-1.0), a
					// colliding keywords error has been detected and processing is
					// aborted.
					if (options.processing_mode == ProcessingMode::JsonLd1_0
						|| (expanded_property != Keyword::Included
							&& expanded_property != Keyword::Type))
						&& result.has_key(&Term::Keyword(expanded_property))
					{
						return Err(
							ErrorCode::CollidingKeywords.located(source, key.metadata().clone())
						);
					}

					match expanded_property {
						// If `expanded_property` is @id:
						Keyword::Id => {
							// If `value` is not a string, an invalid @id value error has
							// been detected and processing is aborted.
							if let Some(str_value) = value.as_str() {
								// Otherwise, set `expanded_value` to the result of IRI
								// expanding value using true for document relative and
								// false for vocab.
								result.id = node_id_of_term(expand_iri(
									source,
									active_context,
									str_value,
									value.metadata(),
									true,
									false,
									warnings,
								))
							} else {
								return Err(ErrorCode::InvalidIdValue
									.located(source, value.metadata().clone()));
							}
						}
						// If expanded property is @type:
						Keyword::Type => {
							// If value is neither a string nor an array of strings, an
							// invalid type value error has been detected and processing
							// is aborted.
							let (value, _) = as_array(&*value);
							// Set `expanded_value` to the result of IRI expanding each
							// of its values using `type_scoped_context` for active
							// context, and true for document relative.
							for ty in value {
								if let Some(str_ty) = ty.as_str() {
									if let Ok(ty) = expand_iri(
										source,
										type_scoped_context,
										str_ty,
										ty.metadata(),
										true,
										true,
										warnings,
									)
									.try_into()
									{
										result.types.push(ty)
									} else {
										return Err(ErrorCode::InvalidTypeValue
											.located(source, ty.metadata().clone()));
									}
								} else {
									return Err(ErrorCode::InvalidTypeValue
										.located(source, ty.metadata().clone()));
								}
							}
						}
						// If expanded property is @graph
						Keyword::Graph => {
							// Set `expanded_value` to the result of using this algorithm
							// recursively passing `active_context`, `@graph` for active
							// property, `value` for element, `base_url`, and the
							// `frame_expansion` and `ordered` flags, ensuring that
							// `expanded_value` is an array of one or more maps.
							let expanded_value = expand_element(
								active_context,
								ActiveProperty::Some("@graph", key.metadata()),
								&*value,
								base_url,
								loader,
								options,
								false,
								warnings,
							)
							.await?;
							result.graph = Some(
								expanded_value
									.into_iter()
									.filter(filter_top_level_item)
									.collect(),
							);
						}
						// If expanded property is @included:
						Keyword::Included => {
							// If processing mode is json-ld-1.0, continue with the next
							// key from element.
							if options.processing_mode == ProcessingMode::JsonLd1_0 {
								continue;
							}

							// Set `expanded_value` to the result of using this algorithm
							// recursively passing `active_context`, `active_property`,
							// `value` for element, `base_url`, and the `frame_expansion`
							// and `ordered` flags, ensuring that the result is an array.
							let expanded_value = expand_element(
								active_context,
								ActiveProperty::Some("@included", key.metadata()),
								&*value,
								base_url,
								loader,
								options,
								false,
								warnings,
							)
							.await?;
							let mut expanded_nodes = Vec::new();
							for obj in expanded_value.into_iter() {
								match obj.try_cast::<Node<J, T>>() {
									Ok(node) => expanded_nodes.push(node),
									Err(_) => {
										return Err(ErrorCode::InvalidIncludedValue.located(
											source,
											value.metadata().clone(), // TODO take the metadata of the expanded value `obj`.
										));
									}
								}
							}

							if let Some(included) = &mut result.included {
								included.extend(expanded_nodes.into_iter());
							} else {
								result.included = Some(expanded_nodes.into_iter().collect());
							}
						}
						// If expanded property is @language:
						Keyword::Language => has_value_object_entries = true,
						// If expanded property is @direction:
						Keyword::Direction => has_value_object_entries = true,
						// If expanded property is @index:
						Keyword::Index => {
							if let Some(value) = value.as_str() {
								result.set_index(Some(value.to_string()))
							} else {
								// If value is not a string, an invalid @index value
								// error has been detected and processing is aborted.
								return Err(ErrorCode::InvalidIndexValue
									.located(source, value.metadata().clone()));
							}
						}
						// If expanded property is @reverse:
						Keyword::Reverse => {
							// If value is not a map, an invalid @reverse value error
							// has been detected and processing is aborted.
							if let Some(value) = value.as_object() {
								let mut reverse_entries: Vec<Entry<J>> =
									Vec::with_capacity(value.len());
								for (reverse_key, reverse_value) in value.iter() {
									reverse_entries.push(Entry(reverse_key, reverse_value));
								}

								if options.ordered {
									reverse_entries.sort();
								}

								for Entry(reverse_key, reverse_value) in reverse_entries {
									match expand_iri(
										source,
										active_context,
										reverse_key.as_ref(),
										reverse_key.metadata(),
										false,
										true,
										warnings,
									) {
										Term::Keyword(_) => {
											return Err(ErrorCode::InvalidReversePropertyMap
												.located(source, reverse_key.metadata().clone()))
										}
										Term::Ref(Reference::Invalid(_))
											if options.policy == Policy::Strictest =>
										{
											return Err(ErrorCode::KeyExpansionFailed
												.located(source, reverse_key.metadata().clone()))
										}
										Term::Ref(reverse_prop)
											if reverse_prop.as_str().contains(':')
												|| options.policy == Policy::Relaxed =>
										{
											let reverse_expanded_value = expand_element(
												active_context,
												ActiveProperty::Some(
													reverse_key.as_ref(),
													reverse_key.metadata(),
												),
												&*reverse_value,
												base_url,
												loader,
												options,
												false,
												warnings,
											)
											.await?;

											let is_double_reversed =
												if let Some(reverse_key_definition) =
													active_context.get(reverse_key.as_ref())
												{
													reverse_key_definition.reverse_property
												} else {
													false
												};

											if is_double_reversed {
												result.insert_all(
													reverse_prop,
													reverse_expanded_value.into_iter(),
												)
											} else {
												let mut reverse_expanded_nodes = Vec::new();
												for object in reverse_expanded_value {
													match object.try_cast::<Node<J, T>>() {
														Ok(node) => {
															reverse_expanded_nodes.push(node)
														}
														Err(_) => return Err(
															ErrorCode::InvalidReversePropertyValue
																.located(
																	source,
																	reverse_value
																		.metadata()
																		.clone(),
																),
														),
													}
												}

												result.insert_all_reverse(
													reverse_prop,
													reverse_expanded_nodes.into_iter(),
												)
											}
										}
										_ => {
											if options.policy.is_strict() {
												return Err(ErrorCode::KeyExpansionFailed.located(
													source,
													reverse_key.metadata().clone(),
												));
											}
											// otherwise the key is just dropped.
										}
									}
								}
							} else {
								return Err(ErrorCode::InvalidReverseValue
									.located(source, value.metadata().clone()));
							}
						}
						// If expanded property is @nest
						Keyword::Nest => {
							let nesting_key = key;
							// Recursively repeat steps 3, 8, 13, and 14 using `nesting_key` for active property,
							// and nested value for element.
							let (value, _) = as_array(&*value);
							for nested_value in value {
								// Step 3 again.
								let mut property_scoped_base_url = None;
								let property_scoped_context =
									match active_context.get(nesting_key.as_ref()) {
										Some(definition) => {
											if let Some(base_url) = &definition.base_url {
												property_scoped_base_url = Some(base_url.as_iri());
											}

											definition.context.as_ref()
										}
										None => None,
									};

								// Step 8 again.
								let active_context = match property_scoped_context {
									Some(property_scoped_context) => {
										let options: ProcessingOptions = options.into();
										Mown::Owned(
											property_scoped_context
												.process_with(
													active_context,
													loader,
													property_scoped_base_url,
													options.with_override(),
												)
												.await
												.map_err(|e| {
													e.with_metadata(nesting_key.metadata().clone())
												})?
												.into_inner(),
										)
									}
									None => Mown::Borrowed(active_context),
								};

								// Steps 13 and 14 again.
								if let Some(nested_value) = nested_value.as_object() {
									let mut nested_entries: Vec<Entry<J>> = Vec::new();

									for (key, value) in nested_value.iter() {
										nested_entries.push(Entry(key, value))
									}

									if options.ordered {
										nested_entries.sort();
									}

									let nested_expanded_entries =
										nested_entries.into_iter().map(|Entry(key, value)| {
											let expanded_key = expand_iri(
												source,
												active_context.as_ref(),
												key.as_ref(),
												key.metadata(),
												false,
												true,
												warnings,
											);
											ExpandedEntry(key, expanded_key, value)
										});

									let (new_result, new_has_value_object_entries) =
										expand_node_entries(
											result,
											has_value_object_entries,
											active_context.as_ref(),
											type_scoped_context,
											active_property,
											nested_expanded_entries.collect(),
											base_url,
											loader,
											options,
											warnings,
										)
										.await?;

									result = new_result;
									has_value_object_entries = new_has_value_object_entries;
								} else {
									return Err(ErrorCode::InvalidNestValue
										.located(source, nested_value.metadata().clone()));
								}
							}
						}
						Keyword::Value => {
							return Err(
								ErrorCode::InvalidNestValue.located(source, key.metadata().clone())
							)
						}
						_ => (),
					}
				}

				Term::Ref(Reference::Invalid(_)) if options.policy == Policy::Strictest => {
					return Err(
						ErrorCode::KeyExpansionFailed.located(source, key.metadata().clone())
					)
				}

				Term::Ref(prop)
					if prop.as_str().contains(':') || options.policy == Policy::Relaxed =>
				{
					let mut container_mapping = Mown::Owned(Container::new());

					let key_definition = active_context.get(key.as_ref());
					let mut is_reverse_property = false;
					let mut is_json = false;

					if let Some(key_definition) = key_definition {
						is_reverse_property = key_definition.reverse_property;

						// Initialize container mapping to key's container mapping in active context.
						container_mapping = Mown::Borrowed(&key_definition.container);

						// If key's term definition in `active_context` has a type mapping of `@json`,
						// set expanded value to a new map,
						// set the entry `@value` to `value`, and set the entry `@type` to `@json`.
						if key_definition.typ == Some(Type::Json) {
							is_json = true;
						}
					}

					let mut expanded_value = if is_json {
						Expanded::Object(Object::Value(Value::Json((*value).clone())).into())
					} else {
						match value.as_object() {
							Some(value) if container_mapping.contains(ContainerType::Language) => {
								// Otherwise, if container mapping includes @language and value is a map then
								// value is expanded from a language map as follows:
								// Initialize expanded value to an empty array.
								let mut expanded_value = Vec::new();

								// Initialize direction to the default base direction from active context.
								let mut direction = active_context.default_base_direction();

								// If key's term definition in active context has a
								// direction mapping, update direction with that value.
								if let Some(key_definition) = key_definition {
									if let Some(key_direction) = key_definition.direction {
										direction = key_direction.option()
									}
								}

								// For each key-value pair language-language value in
								// value, ordered lexicographically by language if ordered is true:
								let mut language_entries: Vec<Entry<J>> =
									Vec::with_capacity(value.len());
								for (language, language_value) in value.iter() {
									language_entries.push(Entry(language, language_value));
								}

								if options.ordered {
									language_entries.sort();
								}

								for Entry(language, language_value) in language_entries {
									let language_metadata = language.metadata();
									let language: &str = &*language;
									// If language value is not an array set language value to
									// an array containing only language value.
									let (language_value, _) = as_array(&*language_value);

									// For each item in language value:
									for item in language_value {
										match item.as_value_ref() {
											// If item is null, continue to the next entry in
											// language value.
											ValueRef::Null => (),
											ValueRef::String(item) => {
												// If language is @none, or expands to
												// @none, remove @language from v.
												let language = if expand_iri(
													source,
													active_context,
													language,
													language_metadata,
													false,
													true,
													warnings,
												) == Term::Keyword(Keyword::None)
												{
													None
												} else {
													match LanguageTagBuf::parse_copy(language) {
														Ok(lang) => Some(lang.into()),
														Err(err) => {
															warnings.push(Loc::new(
																Warning::MalformedLanguageTag(
																	language.to_string().clone(),
																	err,
																),
																source,
																language_metadata.clone(),
															));
															Some(language.to_string().into())
														}
													}
												};

												// initialize a new map v consisting of two
												// key-value pairs: (@value-item) and
												// (@language-language).
												if let Ok(v) = LangString::new(
													LiteralString::Expanded(item.clone()),
													language,
													direction,
												) {
													// If item is neither @none nor well-formed
													// according to section 2.2.9 of [BCP47],
													// processors SHOULD issue a warning.
													// TODO warning

													// Append v to expanded value.
													expanded_value.push(
														Object::Value(Value::LangString(v)).into(),
													)
												} else {
													expanded_value.push(
														Object::Value(Value::Literal(
															Literal::String(
																LiteralString::Expanded(
																	item.clone(),
																),
															),
															None,
														))
														.into(),
													)
												}
											}
											_ => {
												// item must be a string, otherwise an
												// invalid language map value error has
												// been detected and processing is aborted.
												return Err(ErrorCode::InvalidLanguageMapValue
													.located(source, item.metadata().clone()));
											}
										}
									}
								}

								Expanded::Array(expanded_value)
							}
							Some(value)
								if container_mapping.contains(ContainerType::Index)
									|| container_mapping.contains(ContainerType::Type)
									|| container_mapping.contains(ContainerType::Id) =>
							{
								// Otherwise, if container mapping includes @index, @type, or @id and value
								// is a map then value is expanded from a map as follows:

								// Initialize expanded value to an empty array.
								let mut expanded_value: Vec<Indexed<Object<J, T>>> = Vec::new();

								// Initialize `index_key` to the key's index mapping in
								// `active_context`, or @index, if it does not exist.
								let index_key = if let Some(key_definition) = key_definition {
									if let Some(index) = &key_definition.index {
										index.as_str()
									} else {
										"@index"
									}
								} else {
									"@index"
								};

								// For each key-value pair index-index value in value,
								// ordered lexicographically by index if ordered is true:
								let mut entries: Vec<Entry<J>> = Vec::with_capacity(value.len());
								for (key, value) in value.iter() {
									entries.push(Entry(key, value))
								}

								if options.ordered {
									entries.sort();
								}

								for Entry(index, index_value) in entries {
									// If container mapping includes @id or @type,
									// initialize `map_context` to the `previous_context`
									// from `active_context` if it exists, otherwise, set
									// `map_context` to `active_context`.
									let mut map_context = Mown::Borrowed(active_context);
									if container_mapping.contains(ContainerType::Type)
										|| container_mapping.contains(ContainerType::Id)
									{
										if let Some(previous_context) =
											active_context.previous_context()
										{
											map_context = Mown::Borrowed(previous_context)
										}
									}

									// If container mapping includes @type and
									// index's term definition in map context has a
									// local context, update map context to the result of
									// the Context Processing algorithm, passing
									// map context as active context the value of the
									// index's local context as local context and base URL
									// from the term definition for index in map context.
									if container_mapping.contains(ContainerType::Type) {
										if let Some(index_definition) =
											map_context.get(index.as_ref())
										{
											if let Some(local_context) = &index_definition.context {
												let base_url = index_definition
													.base_url
													.as_ref()
													.map(|url| url.as_iri());
												map_context = Mown::Owned(
													local_context
														.process_with(
															map_context.as_ref(),
															loader,
															base_url,
															options.into(),
														)
														.await
														.map_err(|e| {
															e.with_metadata(
																index.metadata().clone(),
															)
														})?
														.into_inner(),
												)
											}
										}
									}

									// Otherwise, set map context to active context.
									// TODO What?

									// Initialize `expanded_index` to the result of IRI
									// expanding index.
									let expanded_index = match expand_iri(
										source,
										active_context,
										index.as_ref(),
										index.metadata(),
										false,
										true,
										warnings,
									) {
										Term::Null | Term::Keyword(Keyword::None) => None,
										key => Some(key),
									};

									// If index value is not an array set index value to
									// an array containing only index value.
									// let index_value = as_array(index_value);

									// Initialize index value to the result of using this
									// algorithm recursively, passing map context as
									// active context, key as active property,
									// index value as element, base URL, and the
									// frameExpansion and ordered flags.
									// And `true` for `from_map`.
									let expanded_index_value = expand_element(
										map_context.as_ref(),
										ActiveProperty::Some(key.as_ref(), key.metadata()),
										&*index_value,
										base_url,
										loader,
										options,
										true,
										warnings,
									)
									.await?;
									// For each item in index value:
									for mut item in expanded_index_value {
										// If container mapping includes @graph,
										// and item is not a graph object, set item to
										// a new map containing the key-value pair
										// @graph-item, ensuring that the value is
										// represented using an array.
										if container_mapping.contains(ContainerType::Graph)
											&& !item.is_graph()
										{
											let mut node = Node::new();
											let mut graph = HashSet::new();
											graph.insert(item);
											node.graph = Some(graph);
											item = Object::Node(node).into();
										}

										if expanded_index.is_some() {
											// If `container_mapping` includes @index,
											// index key is not @index, and expanded index is
											// not @none:
											// TODO the @none part.
											if container_mapping.contains(ContainerType::Index)
												&& index_key != "@index"
											{
												// Initialize re-expanded index to the result
												// of calling the Value Expansion algorithm,
												// passing the active context, index key as
												// active property, and index as value.
												let re_expanded_index = expand_literal(
													source,
													active_context,
													ActiveProperty::Some(
														index_key,
														index.metadata(),
													),
													LiteralValue::Inferred(
														(**index).into(),
														index.metadata().clone(),
													),
													warnings,
												)
												.map_err(|e| {
													e.located(source, index.metadata().clone())
												})?;

												// Initialize expanded index key to the result
												// of IRI expanding index key.
												let expanded_index_key = match expand_iri(
													source,
													active_context,
													index_key,
													index.metadata(),
													false,
													true,
													warnings,
												) {
													Term::Ref(prop) => prop,
													_ => continue,
												};

												// Add the key-value pair (expanded index
												// key-index property values) to item.
												if let Object::Node(ref mut node) = *item {
													node.insert(
														expanded_index_key,
														re_expanded_index,
													);
												} else {
													// If item is a value object, it MUST NOT
													// contain any extra properties; an invalid
													// value object error has been detected and
													// processing is aborted.
													return Err(ErrorCode::InvalidValueObject
														.located(
															source,
															index_value.metadata().clone(),
														));
												}
											} else if container_mapping
												.contains(ContainerType::Index) && item
												.index()
												.is_none()
											{
												// Otherwise, if container mapping includes
												// @index, item does not have an entry @index,
												// and expanded index is not @none, add the
												// key-value pair (@index-index) to item.
												item.set_index(Some((*index).to_string()))
											} else if container_mapping.contains(ContainerType::Id)
												&& item.id().is_none()
											{
												// Otherwise, if container mapping includes
												// @id item does not have the entry @id,
												// and expanded index is not @none, add the
												// key-value pair (@id-expanded index) to
												// item, where expanded index is set to the
												// result of IRI expanding index using true for
												// document relative and false for vocab.
												if let Object::Node(ref mut node) = *item {
													node.id = node_id_of_term(expand_iri(
														source,
														active_context,
														index.as_ref(),
														index.metadata(),
														true,
														false,
														warnings,
													));
												}
											} else if container_mapping
												.contains(ContainerType::Type)
											{
												// Otherwise, if container mapping includes
												// @type and expanded index is not @none,
												// initialize types to a new array consisting
												// of expanded index followed by any existing
												// values of @type in item. Add the key-value
												// pair (@type-types) to item.
												if let Ok(typ) =
													expanded_index.clone().unwrap().try_into()
												{
													if let Object::Node(ref mut node) = *item {
														node.types.insert(0, typ);
													}
												} else {
													return Err(ErrorCode::InvalidTypeValue
														.located(
															source,
															index_value.metadata().clone(),
														));
												}
											}
										}

										// Append item to expanded value.
										expanded_value.push(item)
									}
								}

								Expanded::Array(expanded_value)
							}
							_ => {
								// Otherwise, initialize expanded value to the result of using this
								// algorithm recursively, passing active context, key for active property,
								// value for element, base URL, and the frameExpansion and ordered flags.
								expand_element(
									active_context,
									ActiveProperty::Some(key.as_ref(), key.metadata()),
									&*value,
									base_url,
									loader,
									options,
									false,
									warnings,
								)
								.await?
							}
						}
					};

					// If container mapping includes @list and expanded value is
					// not already a list object, convert expanded value to a list
					// object by first setting it to an array containing only
					// expanded value if it is not already an array, and then by
					// setting it to a map containing the key-value pair
					// @list-expanded value.
					if container_mapping.contains(ContainerType::List) && !expanded_value.is_list()
					{
						expanded_value = Expanded::Object(
							Object::List(expanded_value.into_iter().collect()).into(),
						);
					}

					// If container mapping includes @graph, and includes neither
					// @id nor @index, convert expanded value into an array, if
					// necessary, then convert each value ev in expanded value
					// into a graph object:
					if container_mapping.contains(ContainerType::Graph)
						&& !container_mapping.contains(ContainerType::Id)
						&& !container_mapping.contains(ContainerType::Index)
					{
						expanded_value = Expanded::Array(
							expanded_value
								.into_iter()
								.map(|ev| {
									let mut node = Node::new();
									let mut graph = HashSet::new();
									graph.insert(ev);
									node.graph = Some(graph);
									Object::Node(node).into()
								})
								.collect(),
						);
					}

					if !expanded_value.is_null() {
						// If the term definition associated to key indicates that it
						// is a reverse property:
						if is_reverse_property {
							// We must filter out anything that is not an object.
							let mut reverse_expanded_nodes = Vec::new();
							for object in expanded_value {
								match object.try_cast::<Node<J, T>>() {
									Ok(node) => reverse_expanded_nodes.push(node),
									Err(_) => {
										return Err(ErrorCode::InvalidReversePropertyValue
											.located(source, value.metadata().clone()))
									}
								}
							}

							result.insert_all_reverse(prop, reverse_expanded_nodes.into_iter());
						} else {
							// Otherwise, key is not a reverse property use add value
							// to add expanded value to the expanded property entry in
							// result using true for as array.
							result.insert_all(prop, expanded_value.into_iter());
						}
					}
				}

				Term::Ref(_) => {
					if options.policy.is_strict() {
						return Err(
							ErrorCode::KeyExpansionFailed.located(source, key.metadata().clone())
						);
					}
					// non-keyword properties that does not include a ':' are skipped.
				}
			}
		}

		Ok((result, has_value_object_entries))
	}
	.boxed()
}
