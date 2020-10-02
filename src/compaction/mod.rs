use std::collections::HashSet;
use std::convert::TryFrom;
use futures::future::{BoxFuture, FutureExt};
use mown::Mown;
use json::JsonValue;
use crate::{
	Id,
	Context,
	ContextMut,
	Indexed,
	Object,
	Value,
	Node,
	Reference,
	Lenient,
	Error,
	ProcessingMode,
	context::{
		self,
		Loader,
		ProcessingStack,
		Local,
		InverseContext,
		inverse::{
			TypeSelection,
			LangSelection,
			Selection
		}
	},
	syntax::{
		Keyword,
		Container,
		ContainerType,
		Term,
		Type
	},
	util::AsJson
};

#[derive(Clone, Copy)]
pub struct Options {
	pub processing_mode: ProcessingMode,
	pub compact_to_relative: bool,
	pub compact_arrays: bool,
	pub ordered: bool
}

impl From<Options> for context::ProcessingOptions {
	fn from(options: Options) -> context::ProcessingOptions {
		let mut opt = context::ProcessingOptions::default();
		opt.processing_mode = options.processing_mode;
		opt
	}
}

impl From<crate::expansion::Options> for Options {
	fn from(options: crate::expansion::Options) -> Options {
		Options {
			processing_mode: options.processing_mode,
			ordered: options.ordered,
			..Options::default()
		}
	}
}

impl Default for Options {
	fn default() -> Options {
		Options {
			processing_mode: ProcessingMode::default(),
			compact_to_relative: false,
			compact_arrays: false,
			ordered: false
		}
	}
}

pub trait Compact<T: Id> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send;

	fn compact<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, loader: &'a mut L) -> BoxFuture<'a, Result<JsonValue, Error>> where Self: Sync, T: 'a + Sync + Send, C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			let inverse_context = InverseContext::new();
			self.compact_with(active_context, active_context, &inverse_context, None, loader, Options::default()).await
		}.boxed()
	}
}

enum TypeLangValue<'a, T: Id> {
	Type(TypeSelection<T>),
	Lang(LangSelection<'a>)
}

// default value for `value` is `None` and `false` for `vocab` and `reverse`.
fn compact_iri<T: Id, C: Context<T>>(active_context: &C, inverse_context: &InverseContext<T>, var: Lenient<Term<T>>, value: Option<&Indexed<Object<T>>>, vocab: bool, reverse: bool, options: Options) -> Result<JsonValue, Error> {
	if var == Lenient::Ok(Term::Null) {
		return Ok(JsonValue::Null)
	}

	if vocab {
		if let Lenient::Ok(var) = &var {
			if let Some(entry) = inverse_context.get(var) {
				// let default_lang_dir = (active_context.default_language(), active_context.default_base_direction());

				// Initialize containers to an empty array.
				// This array will be used to keep track of an ordered list of preferred container
				// mapping for a term, based on what is compatible with value.
				let mut containers = Vec::new();
				let mut type_lang_value = None;
				// let mut type_selection: Vec<TypeSelection<T>> = Vec::new();
				// let mut lang_selection: Vec<LangSelection> = Vec::new();
				// let mut select_by_type = false;

				if let Some(value) = value {
					if value.index().is_some() && !value.is_graph() {
						containers.push(Container::Type);
						containers.push(Container::IndexSet);
					}
				}

				if reverse {
					type_lang_value = Some(TypeLangValue::Type(TypeSelection::Reverse));
					containers.push(Container::Set);
				} else {
					let mut has_index = false;
					let mut is_simple_value = false; // value object with no type, no index, no language and no direction.

					if let Some(value) = value {
						has_index = value.index().is_some();

						match value.inner() {
							Object::List(list) => {
								if value.index().is_none() {
									containers.push(Container::List);
								}

								let mut common_type = None;
								let mut common_lang_dir = None;

								if list.is_empty() {
									common_lang_dir = Some(Some((active_context.default_language(), active_context.default_base_direction())))
								} else {
									for item in list {
										let mut item_type = None;
										let mut item_lang_dir = None;
										let mut is_value = false;

										match item.inner() {
											Object::Value(value) => {
												is_value = true;
												match value {
													Value::LangString(lang_str) => {
														item_lang_dir = Some((lang_str.language(), lang_str.direction()))
													},
													Value::Literal(_, Some(ty)) => {
														item_type = Some(Type::Ref(ty.clone()))
													},
													Value::Literal(_, None) => {
														item_type = None
													},
													Value::Json(_) => {
														item_type = Some(Type::Json)
													}
												}
											},
											_ => {
												item_type = Some(Type::Id)
											}
										}

										if common_lang_dir.is_none() {
											common_lang_dir = Some(item_lang_dir)
										} else if is_value && *common_lang_dir.as_ref().unwrap() != item_lang_dir {
											common_lang_dir = Some(None)
										}

										if common_type.is_none() {
											common_type = Some(item_type)
										} else if *common_type.as_ref().unwrap() != item_type {
											common_type = Some(None)
										}

										if common_lang_dir == Some(None) && common_type == Some(None) {
											break
										}
									}

									if common_lang_dir.is_none() {
										common_lang_dir = Some(None)
									}
									let common_lang_dir = common_lang_dir.unwrap();

									if common_type.is_none() {
										common_type = Some(None)
									}
									let common_type = common_type.unwrap();

									if let Some(common_type) = common_type {
										type_lang_value = Some(TypeLangValue::Type(TypeSelection::Type(common_type)))
									} else if let Some(common_lang_dir) = common_lang_dir {
										type_lang_value = Some(TypeLangValue::Lang(LangSelection::Lang(common_lang_dir.0, common_lang_dir.1)))
									} else {
										type_lang_value = Some(TypeLangValue::Lang(LangSelection::Lang(None, None)))
									}
								}
							},
							Object::Node(node) if node.is_graph() => {
								// Otherwise, if value is a graph object, prefer a mapping most
								// appropriate for the particular value.
								if value.index().is_some() {
									// If value contains an @index entry, append the values
									// @graph@index and @graph@index@set to containers.
									containers.push(Container::GraphIndex);
									containers.push(Container::GraphIndexSet);
								}

								if node.id().is_some() {
									// If value contains an @id entry, append the values @graph@id and
									// @graph@id@set to containers.
									containers.push(Container::GraphId);
									containers.push(Container::GraphIdSet);
								}

								// Append the values @graph, @graph@set, and @set to containers.
								containers.push(Container::Graph);
								containers.push(Container::GraphSet);
								containers.push(Container::Set);

								if value.index().is_none() {
									// If value does not contain an @index entry, append the values
									// @graph@index and @graph@index@set to containers.
									containers.push(Container::GraphIndex);
									containers.push(Container::GraphIndexSet);
								}

								if node.id().is_none() {
									// If the value does not contain an @id entry, append the values
									// @graph@id and @graph@id@set to containers.
									containers.push(Container::GraphId);
									containers.push(Container::GraphIdSet);
								}

								// Append the values @index and @index@set to containers.
								containers.push(Container::Index);
								containers.push(Container::IndexSet);

								type_lang_value = Some(TypeLangValue::Type(TypeSelection::Type(Type::Id)))
							},
							Object::Value(v) => {
								// If value is a value object:
								if (v.direction().is_some() || v.language().is_some()) && value.index().is_none() {
									type_lang_value = Some(TypeLangValue::Lang(LangSelection::Lang(v.language(), v.direction())));
									containers.push(Container::Language);
									containers.push(Container::LanguageSet)
								} else if let Some(ty) = v.typ() {
									type_lang_value = Some(TypeLangValue::Type(TypeSelection::Type(ty.map(|ty| (*ty).clone()))))
								} else {
									is_simple_value = v.direction().is_none() && v.language().is_none() && value.index().is_none()
								}

								containers.push(Container::Set)
							},
							Object::Node(node) => {
								// Otherwise, set type/language to @type and set type/language value
								// to @id, and append @id, @id@set, @type, and @set@type, to containers.
								type_lang_value = Some(TypeLangValue::Type(TypeSelection::Type(Type::Id)));
								containers.push(Container::Id);
								containers.push(Container::IdSet);
								containers.push(Container::Type);
								containers.push(Container::SetType);

								containers.push(Container::Set)
							}
						}
					}

					containers.push(Container::None);

					if options.processing_mode != ProcessingMode::JsonLd1_0 && !has_index {
						containers.push(Container::Index);
						containers.push(Container::IndexSet)
					}

					if options.processing_mode != ProcessingMode::JsonLd1_0 && is_simple_value {
						containers.push(Container::Language);
						containers.push(Container::LanguageSet)
					}

					// If type/language value is null, set type/language value to @null. This is the
					// key under which null values are stored in the inverse context entry.
					// if type_selection.is_empty() {
					// 	type_selection.push(TypeSelection::Null)
					// }
					// if lang_selection.is_empty() {
					// 	lang_selection.push(LangSelection::Null)
					// }
					// TODO what?

					let mut is_empty_list = false;
					if let Some(value) = value {
						if let Object::List(list) = value.inner() {
							if list.is_empty() {
								is_empty_list = true;
							}
						}
					}

					// If type/language value is @reverse, append @reverse to preferred values.
					let selection = if is_empty_list {
						Selection::Any
					} else {
						match type_lang_value {
							Some(TypeLangValue::Type(mut type_value)) => {
								let mut selection: Vec<TypeSelection<T>> = Vec::new();

								if type_value == TypeSelection::Reverse {
									selection.push(TypeSelection::Reverse);
								}

								let mut has_id_type = false;
								if let Some(value) = value {
									if let Some(id) = value.id() {
										if type_value == TypeSelection::Type(Type::Id) || type_value == TypeSelection::Reverse {
											has_id_type = true;
											let mut vocab = false;
											if let Lenient::Ok(id) = id {
												let term_id = Lenient::Ok(Term::Ref(id.clone()));
												let compacted_iri = compact_iri(active_context, inverse_context, term_id, None, false, false, options)?;
												if let Some(def) = active_context.get(compacted_iri.as_str().unwrap()) {
													if let Some(iri_mapping) = &def.value {
														vocab = iri_mapping == id;
													}
												}
											}

											if vocab {
												selection.push(TypeSelection::Type(Type::Vocab));
												selection.push(TypeSelection::Type(Type::Id));
												selection.push(TypeSelection::Type(Type::None));
											} else {
												selection.push(TypeSelection::Type(Type::Id));
												selection.push(TypeSelection::Type(Type::Vocab));
												selection.push(TypeSelection::Type(Type::None));
											}
										}
									}
								}

								if !has_id_type {
									selection.push(type_value);
									selection.push(TypeSelection::Type(Type::None));
								}

								selection.push(TypeSelection::Any);

								Selection::Type(selection)
							},
							Some(TypeLangValue::Lang(lang_value)) => {
								let mut selection = Vec::new();

								selection.push(lang_value);
								selection.push(LangSelection::Lang(None, None));

								selection.push(LangSelection::Any);

								if let LangSelection::Lang(Some(_), Some(dir)) = lang_value {
									selection.push(LangSelection::Lang(None, Some(dir)));
								}

								Selection::Lang(selection)
							},
							None => {
								let mut selection = Vec::new();
								selection.push(LangSelection::Lang(None, None));
								selection.push(LangSelection::Any);
								Selection::Lang(selection)
							}
						}
					};

					if let Some(term) = inverse_context.select(var, &containers, &selection) {
						return Ok(term.into())
					}
				}
			}
		}

		// At this point, there is no simple term that var can be compacted to.
		// If vocab is true and active context has a vocabulary mapping:
		if let Some(vocab_mapping) = active_context.vocabulary() {
			// If var begins with the vocabulary mapping's value but is longer, then initialize
			// suffix to the substring of var that does not match. If suffix does not have a term
			// definition in active context, then return suffix.
			if let Some(suffix) = var.as_str().strip_prefix(vocab_mapping.as_str()) {
				if !suffix.is_empty() {
					if active_context.get(suffix).is_none() {
						return Ok(suffix.into())
					}
				}
			}
		}
	}

	// The var could not be compacted using the active context's vocabulary mapping.
	// Try to create a compact IRI, starting by initializing compact IRI to null.
	// This variable will be used to store the created compact IRI, if any.
	let mut compact_iri = String::new();

	// For each term definition definition in active context:
	for (key, definition) in active_context.definitions() {
		// If the IRI mapping of definition is null, its IRI mapping equals var,
		// its IRI mapping is not a substring at the beginning of var,
		// or definition does not have a true prefix flag,
		// definition's key cannot be used as a prefix.
		// Continue with the next definition.
		match definition.value.as_ref() {
			Some(iri_mapping) if definition.prefix => {
				if let Some(suffix) = var.as_str().strip_prefix(iri_mapping.as_str()) {
					if !suffix.is_empty() {
						// Initialize candidate by concatenating definition key,
						// a colon (:),
						// and the substring of var that follows after the value of the definition's IRI mapping.
						let candidate = key.clone() + ":" + suffix;

						// If either compact IRI is null,
						// candidate is shorter or the same length but lexicographically less than
						// compact IRI and candidate does not have a term definition in active
						// context, or if that term definition has an IRI mapping that equals var
						// and value is null, set compact IRI to candidate.
						let candidate_def = active_context.get(&candidate);
						if compact_iri.is_empty()
						|| (candidate.len() <= compact_iri.len() && candidate < compact_iri && candidate_def.is_none())
						|| (candidate_def.is_some() && candidate_def.map_or(None, |def| def.value.as_ref()).map_or(false, |v| v.as_str() == var.as_str()) && value.is_none()) {
							compact_iri = candidate
						}
					}
				}
			},
			_ => ()
		}
	}

	// If compact IRI is not null, return compact IRI.
	if !compact_iri.is_empty() {
		return Ok(compact_iri.into())
	}

	// To ensure that the IRI var is not confused with a compact IRI,
	// if the IRI scheme of var matches any term in active context with prefix flag set to true,
	// and var has no IRI authority (preceded by double-forward-slash (//),
	// an IRI confused with prefix error has been detected, and processing is aborted.
	// TODO

	// If vocab is false,
	// transform var to a relative IRI reference using the base IRI from active context,
	// if it exists.
	if !vocab {
		if let Some(base_iri) = active_context.base_iri() {
			if let Some(iri) = var.as_iri() {
				return Ok(iri.relative_to(base_iri).as_str().into())
			}
		}
	}

	// Finally, return var as is.
	Ok(var.as_str().into())
}

impl<T: Sync + Send + Id> Compact<T> for Reference<T> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			match self {
				Reference::Id(id) => {
					Ok(id.as_iri().as_str().into())
				},
				Reference::Blank(id) => {
					Ok(id.as_str().into())
				}
			}
		}.boxed()
	}
}

impl<T: Sync + Send + Id, V: Sync + Send + Compact<T>> Compact<T> for Lenient<V> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			match self {
				Lenient::Ok(value) => value.compact_with(active_context, type_scoped_context, inverse_context, active_property, loader, options).await,
				Lenient::Unknown(u) => Ok(u.as_str().into())
			}
		}.boxed()
	}
}

pub trait CompactIndexed<T: Id> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send;
}

impl<T: Sync + Send + Id, V: Sync + Send + CompactIndexed<T>> Compact<T> for Indexed<V> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		self.inner().compact_indexed_with(self.index(), active_context, type_scoped_context, inverse_context, active_property, loader, options)
	}
}

impl<T: Sync + Send + Id> CompactIndexed<T> for Object<T> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		match self {
			Object::Value(value) => {
				value.compact_indexed_with(index, active_context, type_scoped_context, inverse_context, active_property, loader, options)
			},
			Object::Node(node) => {
				let mut active_context = active_context;
				if let Some(previous_context) = active_context.previous_context() {
					active_context = previous_context;
				}

				node.compact_indexed_with(index, active_context, type_scoped_context, inverse_context, active_property, loader, options)
			},
			Object::List(list) => async move {
				// If the term definition for active property in active context has a local context:
				let mut active_context = Mown::Borrowed(active_context);
				let mut inverse_context = Mown::Borrowed(inverse_context);
				if let Some(active_property) = active_property {
					if let Some(active_property_definition) = active_context.get(active_property.as_str()) {
						if let Some(local_context) = &active_property_definition.context {
							active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?);
							inverse_context = Mown::Owned(active_context.invert())
						}
					}
				}

				Ok(self.as_json())
			}.boxed()
		}
	}
}

impl<T: Sync + Send + Id> CompactIndexed<T> for Value<T> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			// If the term definition for active property in active context has a local context:
			let mut active_context = Mown::Borrowed(active_context);
			let mut inverse_context = Mown::Borrowed(inverse_context);
			if let Some(active_property) = active_property {
				if let Some(active_property_definition) = active_context.get(active_property.as_str()) {
					if let Some(local_context) = &active_property_definition.context {
						active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?);
						inverse_context = Mown::Owned(active_context.invert())
					}
				}
			}

			// TODO
			Ok(self.as_json())
		}.boxed()
	}
}

impl<T: Sync + Send + Id> CompactIndexed<T> for Node<T> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			// If the term definition for active property in active context has a local context:
			let mut active_context = Mown::Borrowed(active_context);
			let mut inverse_context = Mown::Borrowed(inverse_context);
			if let Some(active_property) = active_property {
				if let Some(active_property_definition) = active_context.get(active_property.as_str()) {
					if let Some(local_context) = &active_property_definition.context {
						active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?);
						inverse_context = Mown::Owned(active_context.invert());
					}
				}
			}

			let inside_reverse = active_property == Some(&Term::Keyword(Keyword::Reverse));
			let mut result = json::object::Object::new();

			if !self.types().is_empty() {
				// If element has an @type entry, create a new array compacted types initialized by
				// transforming each expanded type of that entry into its compacted form by IRI
				// compacting expanded type. Then, for each term in compacted types ordered
				// lexicographically:
				let mut compacted_types = Vec::new();
				for ty in self.types() {
					compacted_types.push(ty.compact_with(active_context.as_ref(), type_scoped_context, inverse_context.as_ref(), active_property, loader, options).await?)
				}

				if options.ordered {
					compacted_types.sort_by(|a, b| {
						a.as_str().unwrap().cmp(b.as_str().unwrap())
					});
				}

				for term in &compacted_types {
					if let Some(term_definition) = type_scoped_context.get(term.as_str().unwrap()) {
						if let Some(local_context) = &term_definition.context {
							active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, term_definition.base_url(), options.into()).await?);
						}
					}
				}

				inverse_context = Mown::Owned(active_context.invert());
			}

			// For each key expanded property and value expanded value in element, ordered
			// lexicographically by expanded property if ordered is true:
			let mut expanded_entries: Vec<_> = self.properties.iter().collect();
			if options.ordered {
				expanded_entries.sort_by(|(a, _), (b, _)| {
					a.as_str().cmp(b.as_str())
				})
			}

			// If expanded property is @id:
			if let Some(id) = &self.id {
				// If expanded value is a string, then initialize compacted value by IRI
				// compacting expanded value with vocab set to false.
				let compacted_value = compact_iri(active_context.as_ref(), inverse_context.as_ref(), id.clone().map(|r| Term::Ref(r.clone())), None, false, false, options)?;

				// Initialize alias by IRI compacting expanded property.
				let alias = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Lenient::Ok(Term::Keyword(Keyword::Id)), None, false, false, options)?;

				// Add an entry alias to result whose value is set to compacted value and continue
				// to the next expanded property.
				if let Some(key) = alias.as_str() {
					result.insert(key, compacted_value);
				}
			}

			// If expanded property is @type:
			if !self.types.is_empty() {
				// If expanded value is a string,
				// then initialize compacted value by IRI compacting expanded value using
				// type-scoped context for active context.
				// TODO

				// Otherwise, expanded value must be a @type array:
				// TODO

				// Initialize alias by IRI compacting expanded property.
				// TODO

				// Initialize as array to true if processing mode is json-ld-1.1 and the
				// container mapping for alias in the active context includes @set,
				// otherwise to the negation of compactArrays.
				// TODO

				// Use add value to add compacted value to the alias entry in result using as array.
				// TODO
			}

			// If expanded property is @reverse:
			if !self.reverse_properties.is_empty() {
				// TODO
			}

			// If expanded property is @index and active property has a container mapping in
			// active context that includes @index,
			if let Some(index) = index {
				let mut index_container = false;
				if let Some(active_property) = active_property {
					if let Some(active_property_definition) = active_context.get(active_property.as_str()) {
						if active_property_definition.container.contains(ContainerType::Index) {
							// then the compacted result will be inside of an @index container,
							// drop the @index entry by continuing to the next expanded property.
							index_container = true;
						}
					}
				}

				if !index_container {
					panic!("TODO")
				}
			}

			for (expanded_property, expanded_value) in expanded_entries {
				// TODO
			}

			// TODO

			Ok(JsonValue::Object(result))
		}.boxed()
	}
}

impl<T: Sync + Send + Id> Compact<T> for HashSet<Indexed<Object<T>>> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			let mut result = Vec::new();

			for item in self {
				match item.compact_with(active_context, type_scoped_context, inverse_context, active_property, loader, options).await? {
					JsonValue::Null => (),
					compacted_item => result.push(compacted_item)
				}
			}

			let mut list_or_set = false;
			if let Some(active_property) = active_property {
				if let Some(active_property_definition) = active_context.get(active_property.as_str()) {
					list_or_set = active_property_definition.container.contains(ContainerType::List) || active_property_definition.container.contains(ContainerType::Set);
				}
			}

			if result.is_empty() {
				return Ok(JsonValue::Object(json::object::Object::new()))
			}

			if result.len() > 1
			|| !options.compact_arrays
			|| active_property == Some(&Term::Keyword(Keyword::Graph)) || active_property == Some(&Term::Keyword(Keyword::Set))
			|| list_or_set {
				return Ok(JsonValue::Array(result))
			}

			return Ok(result.into_iter().next().unwrap())
		}.boxed()
	}
}
