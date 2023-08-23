use crate::{
	expand_element, expand_iri, expand_literal, filter_top_level_item, ActiveProperty, Error,
	Expanded, ExpandedEntry, LiteralValue, Loader, Options, Policy, Warning, WarningHandler,
};
use contextual::WithContext;
use json_ld_context_processing::{ContextLoader, Options as ProcessingOptions, Process};
use json_ld_core::{
	future::{BoxFuture, FutureExt},
	object, object::value::Literal, Container, Context, Id, Indexed, IndexedObject, LangString,
	Node, Object, ProcessingMode, Term, Type, Value,
};
use json_ld_syntax::{ContainerKind, Keyword, LenientLanguageTagBuf, Nullable};
use json_syntax::object::Entry;
use locspan::{At, Meta, Stripped};
use mown::Mown;
use rdf_types::VocabularyMut;
use std::collections::HashSet;
use std::hash::Hash;

/// Convert a term to a node id, if possible.
/// Return `None` if the term is `null`.
pub(crate) fn node_id_of_term<T, B, M>(
	Meta(term, meta): Meta<Term<T, B>, M>,
) -> Option<Meta<Id<T, B>, M>> {
	match term {
		Term::Null => None,
		Term::Id(prop) => Some(Meta(prop, meta)),
		Term::Keyword(kw) => Some(Meta(Id::Invalid(kw.into_str().to_string()), meta)),
	}
}

/// Expand a node object.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn expand_node<'a, T, B, M, N, L: Loader<T, M> + ContextLoader<T, M>, W>(
	vocabulary: &'a mut N,
	active_context: &'a Context<T, B, M>,
	type_scoped_context: &'a Context<T, B, M>,
	active_property: ActiveProperty<'a, M>,
	expanded_entries: Vec<ExpandedEntry<'a, T, B, M>>,
	base_url: Option<&'a T>,
	loader: &'a mut L,
	options: Options,
	warnings: W,
) -> Result<(Option<Indexed<Node<T, B, M>, M>>, W), Meta<Error<M, L::ContextError>, M>>
where
	N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
	T: Clone + Eq + Hash + Sync + Send,
	B: Clone + Eq + Hash + Sync + Send,
	M: Clone + Sync + Send,
	L: Sync + Send,
	L::Output: Into<json_syntax::Value<M>>,
	L::ContextError: Send,
	W: 'a + Send + WarningHandler<B, N, M>,
{
	// Initialize two empty maps, `result` and `nests`.
	// let mut result = Indexed::new(Node::new(), None);
	// let mut has_value_object_entries = false;

	let (result, has_value_object_entries, warnings) = expand_node_entries(
		vocabulary,
		Indexed::new_entry(Node::new(), None),
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

	if has_value_object_entries && result.is_empty() && result.id_entry().is_none() {
		return Ok((None, warnings));
	}

	// If active property is null or @graph, drop free-floating
	// values as follows:
	if active_property.is_none() || active_property == Keyword::Graph {
		// If `result` is a map which is empty,
		// [or contains only the entries `@value` or `@list` (does not apply here)]
		// set `result` to null.
		// Otherwise, if result is a map whose only entry is @id, set result to null.
		if result.is_empty() && result.index().is_none() {
			// both cases are covered by checking `is_empty`.
			return Ok((None, warnings));
		}
	}

	Ok((Some(result), warnings))
}

/// Type returned by the `expand_node_entries` function.
///
/// It is a tuple containing both the node being expanded
/// and a boolean flag set to `true` if the node contains
/// value object entries (in practice, if it has a `@language` entry).
type ExpandedNode<T, B, M, W> = (Indexed<Node<T, B, M>, M>, bool, W);

/// Result of the `expand_node_entries` function.
type NodeEntriesExpensionResult<T, B, M, L, W> =
	Result<ExpandedNode<T, B, M, W>, Meta<Error<M, <L as ContextLoader<T, M>>::ContextError>, M>>;

#[allow(clippy::too_many_arguments)]
fn expand_node_entries<'a, T, B, M, N, L: Loader<T, M> + ContextLoader<T, M>, W>(
	vocabulary: &'a mut N,
	mut result: Indexed<Node<T, B, M>, M>,
	mut has_value_object_entries: bool,
	active_context: &'a Context<T, B, M>,
	type_scoped_context: &'a Context<T, B, M>,
	active_property: ActiveProperty<'a, M>,
	expanded_entries: Vec<ExpandedEntry<'a, T, B, M>>,
	base_url: Option<&'a T>,
	loader: &'a mut L,
	options: Options,
	mut warnings: W,
) -> BoxFuture<'a, NodeEntriesExpensionResult<T, B, M, L, W>>
where
	N: Send + Sync + VocabularyMut<Iri = T, BlankId = B>,
	T: Clone + Eq + Hash + Sync + Send,
	B: Clone + Eq + Hash + Sync + Send,
	M: Clone + Sync + Send,
	L: Sync + Send,
	L::Output: Into<json_syntax::Value<M>>,
	L::ContextError: Send,
	W: 'a + Send + WarningHandler<B, N, M>,
{
	async move {
		// For each `key` and `value` in `element`, ordered lexicographically by key
		// if `ordered` is `true`:
		for ExpandedEntry(Meta(key, key_metadata), expanded_key, value) in expanded_entries {
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
					if active_property == Keyword::Reverse {
						return Err(Error::InvalidReversePropertyMap.at(key_metadata.clone()));
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
						return Err(Error::CollidingKeywords.at(key_metadata.clone()));
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
								result.set_id(
									node_id_of_term(expand_iri(
										vocabulary,
										active_context,
										Meta(Nullable::Some(str_value.into()), value.metadata()),
										true,
										false,
										&mut warnings,
									))
									.map(|t| {
										json_ld_syntax::Entry::new_with(key_metadata.clone(), t)
									}),
								)
							} else {
								return Err(Error::InvalidIdValue.at(value.metadata().clone()));
							}
						}
						// If expanded property is @type:
						Keyword::Type => {
							// If value is neither a string nor an array of strings, an
							// invalid type value error has been detected and processing
							// is aborted.
							let Meta(value, value_metadata) =
								json_syntax::Value::force_as_array(value);
							// Set `expanded_value` to the result of IRI expanding each
							// of its values using `type_scoped_context` for active
							// context, and true for document relative.
							for ty in value {
								if let Some(str_ty) = ty.as_str() {
									if let Ok(ty) = expand_iri(
										vocabulary,
										type_scoped_context,
										Meta(Nullable::Some(str_ty.into()), ty.metadata()),
										true,
										true,
										&mut warnings,
									)
									.try_cast()
									{
										result
											.type_entry_or_default(
												key_metadata.clone(),
												value_metadata.clone(),
											)
											.push(ty)
									} else {
										return Err(
											Error::InvalidTypeValue.at(ty.metadata().clone())
										);
									}
								} else {
									return Err(Error::InvalidTypeValue.at(ty.metadata().clone()));
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
							let (expanded_value, w) = expand_element(
								vocabulary,
								active_context,
								ActiveProperty::Some(Meta("@graph", key_metadata)),
								value,
								base_url,
								loader,
								options,
								false,
								warnings,
							)
							.await?;
							warnings = w;

							result.set_graph(Some(json_ld_syntax::Entry::new_with(
								key_metadata.clone(),
								Meta(
									expanded_value
										.into_iter()
										.filter(filter_top_level_item)
										.map(Stripped)
										.collect(),
									value.metadata().clone(),
								),
							)));
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
							let (expanded_value, w) = expand_element(
								vocabulary,
								active_context,
								ActiveProperty::Some(Meta("@included", key_metadata)),
								value,
								base_url,
								loader,
								options,
								false,
								warnings,
							)
							.await?;
							warnings = w;
							let mut expanded_nodes = Vec::new();
							for Meta(obj, meta) in expanded_value.into_iter() {
								match obj.try_cast::<Node<T, B, M>>() {
									Ok(node) => expanded_nodes.push(Meta(node, meta)),
									Err(_) => {
										return Err(Error::InvalidIncludedValue.at(
											value.metadata().clone(), // TODO take the metadata of the expanded value `obj`.
										));
									}
								}
							}

							if let Some(included) = result.included_entry_mut() {
								included.extend(expanded_nodes.into_iter().map(Stripped));
							} else {
								result.set_included(Some(json_ld_syntax::Entry::new_with(
									key_metadata.clone(),
									Meta(
										expanded_nodes.into_iter().map(Stripped).collect(),
										value.metadata().clone(),
									),
								)));
							}
						}
						// If expanded property is @language:
						Keyword::Language => has_value_object_entries = true,
						// If expanded property is @direction:
						Keyword::Direction => has_value_object_entries = true,
						// If expanded property is @index:
						Keyword::Index => {
							if let Some(index) = value.as_str() {
								result.set_index(Some(json_ld_syntax::Entry::new_with(
									key_metadata.clone(),
									Meta(index.to_string(), value.metadata().clone()),
								)))
							} else {
								// If value is not a string, an invalid @index value
								// error has been detected and processing is aborted.
								return Err(Error::InvalidIndexValue.at(value.metadata().clone()));
							}
						}
						// If expanded property is @reverse:
						Keyword::Reverse => {
							// If value is not a map, an invalid @reverse value error
							// has been detected and processing is aborted.
							if let Some(value) = value.as_object() {
								let mut reverse_entries: Vec<&Entry<M>> = value.iter().collect();

								if options.ordered {
									reverse_entries.sort_by_key(|entry| entry.key.value())
								}

								for Entry {
									key: Meta(reverse_key, reverse_key_metadata),
									value: reverse_value,
								} in reverse_entries
								{
									match expand_iri(
										vocabulary,
										active_context,
										Meta(
											Nullable::Some(reverse_key.as_str().into()),
											reverse_key_metadata,
										),
										false,
										true,
										&mut warnings,
									) {
										Meta(Term::Keyword(_), meta) => {
											return Err(Error::InvalidReversePropertyMap.at(meta))
										}
										Meta(Term::Id(Id::Invalid(_)), meta)
											if options.policy == Policy::Strictest =>
										{
											return Err(Error::KeyExpansionFailed.at(meta))
										}
										Meta(Term::Id(reverse_prop), meta)
											if reverse_prop
												.with(&*vocabulary)
												.as_str()
												.contains(':') || options.policy
												== Policy::Relaxed =>
										{
											let (reverse_expanded_value, w) = expand_element(
												vocabulary,
												active_context,
												ActiveProperty::Some(Meta(
													reverse_key.as_ref(),
													reverse_key_metadata,
												)),
												reverse_value,
												base_url,
												loader,
												options,
												false,
												warnings,
											)
											.await?;
											warnings = w;

											let is_double_reversed =
												if let Some(reverse_key_definition) =
													active_context.get(reverse_key.as_str())
												{
													reverse_key_definition.reverse_property()
												} else {
													false
												};

											if is_double_reversed {
												result.insert_all(
													Meta(reverse_prop, meta),
													reverse_expanded_value.into_iter(),
												)
											} else {
												let mut reverse_expanded_nodes = Vec::new();
												for Meta(object, meta) in reverse_expanded_value {
													match object.try_cast::<Node<T, B, M>>() {
														Ok(node) => reverse_expanded_nodes
															.push(Meta(node, meta)),
														Err(_) => {
															return Err(
																Error::InvalidReversePropertyValue
																	.at(reverse_value
																		.metadata()
																		.clone()),
															)
														}
													}
												}

												result
													.reverse_properties_or_default(
														key_metadata.clone(),
														reverse_value.metadata().clone(),
													)
													.insert_all(
														Meta(reverse_prop, meta),
														reverse_expanded_nodes.into_iter(),
													)
											}
										}
										_ => {
											if options.policy.is_strict() {
												return Err(Error::KeyExpansionFailed
													.at(reverse_key_metadata.clone()));
											}
											// otherwise the key is just dropped.
										}
									}
								}
							} else {
								return Err(Error::InvalidReverseValue.at(value.metadata().clone()));
							}
						}
						// If expanded property is @nest
						Keyword::Nest => {
							let nesting_key = key;
							// Recursively repeat steps 3, 8, 13, and 14 using `nesting_key` for active property,
							// and nested value for element.
							let Meta(value, _) = json_syntax::Value::force_as_array(value);
							for nested_value in value {
								// Step 3 again.
								let mut property_scoped_base_url = None;
								let property_scoped_context = match active_context.get(nesting_key)
								{
									Some(definition) => {
										if let Some(base_url) = definition.base_url() {
											property_scoped_base_url = Some(base_url.clone());
										}

										definition.context()
									}
									None => None,
								};

								// Step 8 again.
								let active_context = match property_scoped_context {
									Some(property_scoped_context) => {
										let options: ProcessingOptions = options.into();
										Mown::Owned(
											property_scoped_context
												.value
												.process_with(
													vocabulary,
													active_context,
													loader,
													property_scoped_base_url,
													options.with_override(),
												)
												.await
												.map_err(Meta::cast)?
												.into_processed(),
										)
									}
									None => Mown::Borrowed(active_context),
								};

								// Steps 13 and 14 again.
								if let Some(nested_value) = nested_value.as_object() {
									let mut nested_entries: Vec<&Entry<M>> = Vec::new();

									for entry in nested_value.iter() {
										nested_entries.push(entry)
									}

									if options.ordered {
										nested_entries.sort_by_key(|entry| entry.key.value());
									}

									let nested_expanded_entries = nested_entries
										.into_iter()
										.map(
											|Entry {
											     key: Meta(key, key_metadata),
											     value,
											 }| {
												let Meta(expanded_key, _) = expand_iri(
													vocabulary,
													active_context.as_ref(),
													Meta(
														Nullable::Some(key.as_str().into()),
														key_metadata,
													),
													false,
													true,
													&mut warnings,
												);
												ExpandedEntry(
													Meta(key, key_metadata),
													expanded_key,
													value,
												)
											},
										)
										.collect();

									let (new_result, new_has_value_object_entries, w) =
										expand_node_entries(
											vocabulary,
											result,
											has_value_object_entries,
											active_context.as_ref(),
											type_scoped_context,
											active_property,
											nested_expanded_entries,
											base_url,
											loader,
											options,
											warnings,
										)
										.await?;

									warnings = w;
									result = new_result;
									has_value_object_entries = new_has_value_object_entries;
								} else {
									return Err(
										Error::InvalidNestValue.at(nested_value.metadata().clone())
									);
								}
							}
						}
						Keyword::Value => {
							return Err(Error::InvalidNestValue.at(key_metadata.clone()))
						}
						_ => (),
					}
				}

				Term::Id(Id::Invalid(_)) if options.policy == Policy::Strictest => {
					return Err(Error::KeyExpansionFailed.at(key_metadata.clone()))
				}

				Term::Id(prop)
					if prop.with(&*vocabulary).as_str().contains(':')
						|| options.policy == Policy::Relaxed =>
				{
					let mut container_mapping = Container::new();

					let key_definition = active_context.get(key);
					let mut is_reverse_property = false;
					let mut is_json = false;

					if let Some(key_definition) = key_definition {
						is_reverse_property = key_definition.reverse_property();

						// Initialize container mapping to key's container mapping in active context.
						container_mapping = key_definition.container();

						// If key's term definition in `active_context` has a type mapping of `@json`,
						// set expanded value to a new map,
						// set the entry `@value` to `value`, and set the entry `@type` to `@json`.
						if key_definition.typ() == Some(&Type::Json) {
							is_json = true;
						}
					}

					let mut expanded_value = if is_json {
						Expanded::Object(Meta(
							Object::Value(Value::Json(value.clone())).into(),
							value.metadata().clone(),
						))
					} else {
						match value.as_object() {
							Some(value) if container_mapping.contains(ContainerKind::Language) => {
								// Otherwise, if container mapping includes @language and value is a map then
								// value is expanded from a language map as follows:
								// Initialize expanded value to an empty array.
								let mut expanded_value = Vec::new();

								// Initialize direction to the default base direction from active context.
								let mut direction = active_context.default_base_direction();

								// If key's term definition in active context has a
								// direction mapping, update direction with that value.
								if let Some(key_definition) = key_definition {
									if let Some(key_direction) = key_definition.direction() {
										direction = key_direction.option()
									}
								}

								// For each key-value pair language-language value in
								// value, ordered lexicographically by language if ordered is true:
								let mut language_entries: Vec<&Entry<M>> =
									Vec::with_capacity(value.len());
								for language_entry in value.iter() {
									language_entries.push(language_entry);
								}

								if options.ordered {
									language_entries.sort_by_key(|entry| entry.key.value());
								}

								for Entry {
									key: Meta(language, language_metadata),
									value: language_value,
								} in language_entries
								{
									// If language value is not an array set language value to
									// an array containing only language value.
									let Meta(language_value, _) =
										json_syntax::Value::force_as_array(language_value);

									// For each item in language value:
									for Meta(item, item_metadata) in language_value {
										match item {
											// If item is null, continue to the next entry in
											// language value.
											json_syntax::Value::Null => (),
											json_syntax::Value::String(item) => {
												// If language is @none, or expands to
												// @none, remove @language from v.
												let language = if expand_iri(
													vocabulary,
													active_context,
													Meta(
														Nullable::Some(language.as_str().into()),
														language_metadata,
													),
													false,
													true,
													&mut warnings,
												)
												.into_value() == Term::Keyword(
													Keyword::None,
												) {
													None
												} else {
													let (language, error) =
														LenientLanguageTagBuf::new(
															language.to_string(),
														);

													if let Some(error) = error {
														warnings.handle(
															vocabulary,
															Meta::new(
																Warning::MalformedLanguageTag(
																	language.to_string().clone(),
																	error,
																),
																language_metadata.clone(),
															),
														)
													}

													Some(language)
												};

												// initialize a new map v consisting of two
												// key-value pairs: (@value-item) and
												// (@language-language).
												if let Ok(v) = LangString::new(
													item.clone(),
													language,
													direction,
												) {
													// If item is neither @none nor well-formed
													// according to section 2.2.9 of [BCP47],
													// processors SHOULD issue a warning.
													// TODO warning

													// Append v to expanded value.
													expanded_value.push(Meta(
														Object::Value(Value::LangString(v)).into(),
														item_metadata.clone(),
													))
												} else {
													expanded_value.push(Meta(
														Object::Value(Value::Literal(
															Literal::String(item.clone()),
															None,
														))
														.into(),
														item_metadata.clone(),
													))
												}
											}
											_ => {
												// item must be a string, otherwise an
												// invalid language map value error has
												// been detected and processing is aborted.
												return Err(Error::InvalidLanguageMapValue
													.at(item_metadata.clone()));
											}
										}
									}
								}

								Expanded::Array(expanded_value)
							}
							Some(value)
								if container_mapping.contains(ContainerKind::Index)
									|| container_mapping.contains(ContainerKind::Type)
									|| container_mapping.contains(ContainerKind::Id) =>
							{
								// Otherwise, if container mapping includes @index, @type, or @id and value
								// is a map then value is expanded from a map as follows:

								// Initialize expanded value to an empty array.
								let mut expanded_value: Vec<IndexedObject<T, B, M>> = Vec::new();

								// Initialize `index_key` to the key's index mapping in
								// `active_context`, or @index, if it does not exist.
								let index_key = if let Some(key_definition) = key_definition {
									if let Some(index) = key_definition.index() {
										index.as_str()
									} else {
										"@index"
									}
								} else {
									"@index"
								};

								// For each key-value pair index-index value in value,
								// ordered lexicographically by index if ordered is true:
								let mut entries: Vec<&Entry<M>> = Vec::with_capacity(value.len());
								for entry in value.iter() {
									entries.push(entry)
								}

								if options.ordered {
									entries.sort_by_key(|entry| entry.key.value());
								}

								for Entry {
									key: Meta(index, index_metadata),
									value: index_value,
								} in entries
								{
									// If container mapping includes @id or @type,
									// initialize `map_context` to the `previous_context`
									// from `active_context` if it exists, otherwise, set
									// `map_context` to `active_context`.
									let mut map_context = Mown::Borrowed(active_context);
									if container_mapping.contains(ContainerKind::Type)
										|| container_mapping.contains(ContainerKind::Id)
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
									if container_mapping.contains(ContainerKind::Type) {
										if let Some(index_definition) =
											map_context.get(index.as_str())
										{
											if let Some(local_context) = index_definition.context()
											{
												let base_url = index_definition.base_url().cloned();
												map_context = Mown::Owned(
													local_context
														.process_with(
															vocabulary,
															map_context.as_ref(),
															loader,
															base_url,
															options.into(),
														)
														.await
														.map_err(Meta::cast)?
														.into_processed(),
												)
											}
										}
									}

									// Otherwise, set map context to active context.
									// TODO What?

									// Initialize `expanded_index` to the result of IRI
									// expanding index.
									let expanded_index = match expand_iri(
										vocabulary,
										active_context,
										Meta(Nullable::Some(index.as_str().into()), index_metadata),
										false,
										true,
										&mut warnings,
									) {
										Meta(Term::Null | Term::Keyword(Keyword::None), _) => None,
										Meta(key, meta) => Some(Meta(key, meta)),
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
									let (expanded_index_value, w) = expand_element(
										vocabulary,
										map_context.as_ref(),
										ActiveProperty::Some(Meta(key, key_metadata)),
										index_value,
										base_url,
										loader,
										options,
										true,
										warnings,
									)
									.await
									.map_err(Meta::cast)?;
									warnings = w;
									// For each item in index value:
									for mut item in expanded_index_value {
										// If container mapping includes @graph,
										// and item is not a graph object, set item to
										// a new map containing the key-value pair
										// @graph-item, ensuring that the value is
										// represented using an array.
										if container_mapping.contains(ContainerKind::Graph)
											&& !item.is_graph()
										{
											let item_metadata = item.metadata().clone();
											let mut node = Node::new();
											let mut graph = HashSet::new();
											graph.insert(Stripped(item));
											node.set_graph(Some(json_ld_syntax::Entry::new_with(
												item_metadata.clone(),
												Meta(graph, item_metadata.clone()),
											)));
											item = Meta(Object::node(node).into(), item_metadata);
										}

										if expanded_index.is_some() {
											// If `container_mapping` includes @index,
											// index key is not @index, and expanded index is
											// not @none:
											// TODO the @none part.
											if container_mapping.contains(ContainerKind::Index)
												&& index_key != "@index"
											{
												// Initialize re-expanded index to the result
												// of calling the Value Expansion algorithm,
												// passing the active context, index key as
												// active property, and index as value.
												let re_expanded_index = expand_literal(
													vocabulary,
													active_context,
													ActiveProperty::Some(Meta(
														index_key,
														index_metadata,
													)),
													Meta(
														LiteralValue::Inferred(
															index.as_str().into(),
														),
														index_metadata,
													),
													&mut warnings,
												)
												.map_err(Meta::cast)?;

												// Initialize expanded index key to the result
												// of IRI expanding index key.
												let expanded_index_key = match expand_iri(
													vocabulary,
													active_context,
													Meta(
														Nullable::Some(index_key.into()),
														index_metadata,
													),
													false,
													true,
													&mut warnings,
												)
												.into_value()
												{
													Term::Id(prop) => prop,
													_ => continue,
												};

												// Add the key-value pair (expanded index
												// key-index property values) to item.
												if let Object::Node(node) =
													item.value_mut().inner_mut()
												{
													node.insert(
														Meta(
															expanded_index_key,
															index_metadata.clone(),
														),
														re_expanded_index,
													);
												} else {
													// If item is a value object, it MUST NOT
													// contain any extra properties; an invalid
													// value object error has been detected and
													// processing is aborted.
													return Err(Error::Value(
														crate::InvalidValue::ValueObject,
													)
													.at(index_value.metadata().clone()));
												}
											} else if container_mapping
												.contains(ContainerKind::Index) && item
												.index()
												.is_none()
											{
												// Otherwise, if container mapping includes
												// @index, item does not have an entry @index,
												// and expanded index is not @none, add the
												// key-value pair (@index-index) to item.
												item.set_index(Some(
													json_ld_syntax::Entry::new_with(
														index_metadata.clone(),
														Meta(
															(*index).to_string(),
															index_metadata.clone(),
														),
													),
												))
											} else if container_mapping.contains(ContainerKind::Id)
												&& item.id().is_none()
											{
												// Otherwise, if container mapping includes
												// @id item does not have the entry @id,
												// and expanded index is not @none, add the
												// key-value pair (@id-expanded index) to
												// item, where expanded index is set to the
												// result of IRI expanding index using true for
												// document relative and false for vocab.
												if let Object::Node(ref mut node) = **item {
													node.set_id(
														node_id_of_term(expand_iri(
															vocabulary,
															active_context,
															Meta(
																Nullable::Some(
																	index.as_str().into(),
																),
																index_metadata,
															),
															true,
															false,
															&mut warnings,
														))
														.map(|t| {
															json_ld_syntax::Entry::new_with(
																index_metadata.clone(),
																t,
															)
														}),
													)
												}
											} else if container_mapping
												.contains(ContainerKind::Type)
											{
												// Otherwise, if container mapping includes
												// @type and expanded index is not @none,
												// initialize types to a new array consisting
												// of expanded index followed by any existing
												// values of @type in item. Add the key-value
												// pair (@type-types) to item.
												if let Ok(typ) = expanded_index
													.clone()
													.unwrap()
													.try_cast::<Id<T, B>>()
												{
													if let Object::Node(ref mut node) = **item {
														node.type_entry_or_default(
															key_metadata.clone(),
															typ.metadata().clone(),
														)
														.insert(0, typ);
													}
												} else {
													return Err(Error::InvalidTypeValue
														.at(index_value.metadata().clone()));
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
								let (e, w) = expand_element(
									vocabulary,
									active_context,
									ActiveProperty::Some(Meta(key, key_metadata)),
									value,
									base_url,
									loader,
									options,
									false,
									warnings,
								)
								.await
								.map_err(Meta::cast)?;
								warnings = w;
								e
							}
						}
					};

					// If container mapping includes @list and expanded value is
					// not already a list object, convert expanded value to a list
					// object by first setting it to an array containing only
					// expanded value if it is not already an array, and then by
					// setting it to a map containing the key-value pair
					// @list-expanded value.
					if container_mapping.contains(ContainerKind::List) && !expanded_value.is_list()
					{
						expanded_value = Expanded::Object(Meta(
							Object::List(object::List::new_with(
								key_metadata.clone(),
								Meta(
									expanded_value.into_iter().collect(),
									value.metadata().clone(),
								),
							))
							.into(),
							value.metadata().clone(),
						));
					}

					// If container mapping includes @graph, and includes neither
					// @id nor @index, convert expanded value into an array, if
					// necessary, then convert each value ev in expanded value
					// into a graph object:
					if container_mapping.contains(ContainerKind::Graph)
						&& !container_mapping.contains(ContainerKind::Id)
						&& !container_mapping.contains(ContainerKind::Index)
					{
						expanded_value = Expanded::Array(
							expanded_value
								.into_iter()
								.map(|ev| {
									let ev_metadata = ev.metadata().clone();
									let mut node = Node::new();
									let mut graph = HashSet::new();
									graph.insert(Stripped(ev));
									node.set_graph(Some(json_ld_syntax::Entry::new_with(
										ev_metadata.clone(),
										Meta(graph, ev_metadata.clone()),
									)));
									Meta(Object::node(node).into(), ev_metadata)
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
							for Meta(object, meta) in expanded_value {
								match object.try_cast::<Node<T, B, M>>() {
									Ok(node) => reverse_expanded_nodes.push(Meta(node, meta)),
									Err(_) => {
										return Err(Error::InvalidReversePropertyValue
											.at(value.metadata().clone()))
									}
								}
							}

							result
								.reverse_properties_or_default(
									key_metadata.clone(),
									value.metadata().clone(),
								)
								.insert_all(
									Meta(prop, key_metadata.clone()),
									reverse_expanded_nodes.into_iter(),
								);
						} else {
							// Otherwise, key is not a reverse property use add value
							// to add expanded value to the expanded property entry in
							// result using true for as array.
							result.insert_all(
								Meta(prop, key_metadata.clone()),
								expanded_value.into_iter(),
							);
						}
					}
				}

				Term::Id(_) => {
					if options.policy.is_strict() {
						return Err(Error::KeyExpansionFailed.at(key_metadata.clone()));
					}
					// non-keyword properties that does not include a ':' are skipped.
				}
			}
		}

		Ok((result, has_value_object_entries, warnings))
	}
	.boxed()
}
