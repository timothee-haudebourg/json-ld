use crate::{Options, TypeLangValue};
use contextual::WithContext;
use json_ld_core::{
	context::inverse::{LangSelection, Selection, TypeSelection},
	object, Container, Context, Indexed, Nullable, Object, ProcessingMode, Term, Type, Value,
};
use locspan::Meta;
use rdf_types::Vocabulary;
use std::hash::Hash;

pub struct IriConfusedWithPrefix;

/// Compact the given term without considering any value.
///
/// Calls [`compact_iri_full`] with `None` for `value`.
pub(crate) fn compact_iri<'a, I, B, M, C>(
	vocabulary: &impl Vocabulary<I, B>,
	active_context: &Context<I, B, C, M>,
	var: Meta<&Term<I, B>, &M>,
	vocab: bool,
	reverse: bool,
	options: Options,
) -> Result<Option<Meta<String, M>>, Meta<IriConfusedWithPrefix, M>>
where
	I: Clone + Hash + Eq,
	B: Clone + Hash + Eq,
	M: Clone,
{
	compact_iri_full::<I, B, M, C, Object<I, B, M>>(
		vocabulary,
		active_context,
		var,
		None,
		vocab,
		reverse,
		options,
	)
}

pub(crate) fn compact_key<'a, I, B, M, C>(
	vocabulary: &impl Vocabulary<I, B>,
	active_context: &Context<I, B, C, M>,
	var: Meta<&Term<I, B>, &M>,
	vocab: bool,
	reverse: bool,
	options: Options,
) -> Result<Option<Meta<json_syntax::object::Key, M>>, Meta<IriConfusedWithPrefix, M>>
where
	I: Clone + Hash + Eq,
	B: Clone + Hash + Eq,
	M: Clone,
{
	Ok(compact_iri(vocabulary, active_context, var, vocab, reverse, options)?.map(Meta::cast))
}

/// Compact the given term considering the given value object.
///
/// Calls [`compact_iri_full`] with `Some(value)`.
pub(crate) fn compact_iri_with<'a, I, B, M, C, O: object::Any<I, B, M>>(
	vocabulary: &impl Vocabulary<I, B>,
	active_context: &Context<I, B, C, M>,
	var: Meta<&Term<I, B>, &M>,
	value: &Indexed<O, M>,
	vocab: bool,
	reverse: bool,
	options: Options,
) -> Result<Option<Meta<String, M>>, Meta<IriConfusedWithPrefix, M>>
where
	I: Clone + Hash + Eq,
	B: Clone + Hash + Eq,
	M: Clone,
{
	compact_iri_full(
		vocabulary,
		active_context,
		var,
		Some(value),
		vocab,
		reverse,
		options,
	)
}

/// Compact the given term.
///
/// Default value for `value` is `None` and `false` for `vocab` and `reverse`.
pub(crate) fn compact_iri_full<'a, I, B, M, C, O: object::Any<I, B, M>>(
	vocabulary: &impl Vocabulary<I, B>,
	active_context: &Context<I, B, C, M>,
	Meta(var, meta): Meta<&Term<I, B>, &M>,
	value: Option<&Indexed<O, M>>,
	vocab: bool,
	reverse: bool,
	options: Options,
) -> Result<Option<Meta<String, M>>, Meta<IriConfusedWithPrefix, M>>
where
	I: Clone + Hash + Eq,
	B: Clone + Hash + Eq,
	M: Clone,
{
	if var.is_null() {
		return Ok(None);
	}

	if vocab {
		if let Some(entry) = active_context.inverse().get(var) {
			// Initialize containers to an empty array.
			// This array will be used to keep track of an ordered list of preferred container
			// mapping for a term, based on what is compatible with value.
			let mut containers = Vec::new();
			let mut type_lang_value = None;

			if let Some(value) = value {
				if value.index().is_some() && !value.is_graph() {
					containers.push(Container::Index);
					containers.push(Container::IndexSet);
				}
			}

			let mut has_index = false;
			let mut is_simple_value = false; // value object with no type, no index, no language and no direction.

			if reverse {
				type_lang_value = Some(TypeLangValue::Type(TypeSelection::Reverse));
				containers.push(Container::Set);
			} else {
				let value_ref = value.map(|v| {
					has_index = v.index().is_some();
					v.inner().as_ref()
				});

				match value_ref {
					Some(object::Ref::List(list)) => {
						if !has_index {
							containers.push(Container::List);
						}

						let mut common_type = None;
						let mut common_lang_dir = None;

						if list.is_empty() {
							common_lang_dir = Some(Nullable::Some((
								active_context.default_language(),
								active_context.default_base_direction(),
							)))
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
												item_lang_dir = Some(Nullable::Some((
													lang_str.language(),
													lang_str.direction(),
												)))
											}
											Value::Literal(_, Some(ty)) => {
												item_type = Some(Type::Ref(ty.clone()))
											}
											Value::Literal(_, None) => {
												item_lang_dir = Some(Nullable::Null)
											}
											Value::Json(_) => item_type = Some(Type::Json),
										}
									}
									_ => item_type = Some(Type::Id),
								}

								if common_lang_dir.is_none() {
									common_lang_dir = item_lang_dir
								} else if is_value && common_lang_dir != item_lang_dir {
									common_lang_dir = Some(Nullable::Some((None, None)))
								}

								if common_type.is_none() {
									common_type = Some(item_type)
								} else if *common_type.as_ref().unwrap() != item_type {
									common_type = Some(None)
								}

								if common_lang_dir == Some(Nullable::Some((None, None)))
									&& common_type == Some(None)
								{
									break;
								}
							}
						}

						if common_lang_dir.is_none() {
							common_lang_dir = Some(Nullable::Some((None, None)))
						}
						let common_lang_dir = common_lang_dir.unwrap();

						if common_type.is_none() {
							common_type = Some(None)
						}
						let common_type = common_type.unwrap();

						if let Some(common_type) = common_type {
							type_lang_value =
								Some(TypeLangValue::Type(TypeSelection::Type(common_type)))
						} else {
							type_lang_value =
								Some(TypeLangValue::Lang(LangSelection::Lang(common_lang_dir)))
						}
					}
					Some(object::Ref::Node(node)) if node.is_graph() => {
						// Otherwise, if value is a graph object, prefer a mapping most
						// appropriate for the particular value.
						if has_index {
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

						if !has_index {
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
					}
					Some(object::Ref::Value(v)) => {
						// If value is a value object:
						if (v.direction().is_some() || v.language().is_some()) && !has_index {
							type_lang_value = Some(TypeLangValue::Lang(LangSelection::Lang(
								Nullable::Some((v.language(), v.direction())),
							)));
							containers.push(Container::Language);
							containers.push(Container::LanguageSet)
						} else if let Some(ty) = v.typ() {
							type_lang_value = Some(TypeLangValue::Type(TypeSelection::Type(
								ty.as_syntax_type().cloned(),
							)))
						} else {
							is_simple_value =
								v.direction().is_none() && v.language().is_none() && !has_index
						}

						containers.push(Container::Set)
					}
					_ => {
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

			let mut is_empty_list = false;
			if let Some(value) = value {
				if let object::Ref::List(list) = value.inner().as_ref() {
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
					Some(TypeLangValue::Type(type_value)) => {
						let mut selection: Vec<TypeSelection<I>> = Vec::new();

						if type_value == TypeSelection::Reverse {
							selection.push(TypeSelection::Reverse);
						}

						let mut has_id_type = false;
						if let Some(value) = value {
							if let Some(Meta(id, meta)) = value.id() {
								if type_value == TypeSelection::Type(Type::Id)
									|| type_value == TypeSelection::Reverse
								{
									has_id_type = true;
									let mut vocab = false;
									let Meta(compacted_iri, _) = compact_iri::<_, _, M, _>(
										vocabulary,
										active_context,
										Meta(&id.clone().into_term(), meta),
										true,
										false,
										options,
									)?
									.unwrap();
									if let Some(def) = active_context.get(compacted_iri.as_str()) {
										if let Some(iri_mapping) = &def.value {
											vocab = iri_mapping == id;
										}
									}

									if vocab {
										selection.push(TypeSelection::Type(Type::Vocab));
										selection.push(TypeSelection::Type(Type::Id));
									} else {
										selection.push(TypeSelection::Type(Type::Id));
										selection.push(TypeSelection::Type(Type::Vocab));
									}

									selection.push(TypeSelection::Type(Type::None));
								}
							}
						}

						if !has_id_type {
							selection.push(type_value);
							selection.push(TypeSelection::Type(Type::None));
						}

						selection.push(TypeSelection::Any);

						Selection::Type(selection)
					}
					Some(TypeLangValue::Lang(lang_value)) => {
						let mut selection = vec![
							lang_value,
							LangSelection::Lang(Nullable::Some((None, None))),
							LangSelection::Any,
						];

						if let LangSelection::Lang(Nullable::Some((Some(_), Some(dir)))) =
							lang_value
						{
							selection.push(LangSelection::Lang(Nullable::Some((None, Some(dir)))));
						}

						Selection::Lang(selection)
					}
					None => Selection::Lang(vec![
						LangSelection::Lang(Nullable::Null),
						LangSelection::Lang(Nullable::Some((None, None))),
						LangSelection::Any,
					]),
				}
			};

			if let Some(term) = entry.select(&containers, &selection) {
				return Ok(Some(Meta(term.to_string(), meta.clone())));
			}
		}

		// At this point, there is no simple term that var can be compacted to.
		// If vocab is true and active context has a vocabulary mapping:
		if let Some(vocab_mapping) = active_context.vocabulary() {
			// If var begins with the vocabulary mapping's value but is longer, then initialize
			// suffix to the substring of var that does not match. If suffix does not have a term
			// definition in active context, then return suffix.
			if let Some(suffix) = var
				.with(vocabulary)
				.as_str()
				.strip_prefix(vocab_mapping.with(vocabulary).as_str())
			{
				if !suffix.is_empty() && active_context.get(suffix).is_none() {
					return Ok(Some(Meta(suffix.into(), meta.clone())));
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
				if let Some(suffix) = var
					.with(vocabulary)
					.as_str()
					.strip_prefix(iri_mapping.with(vocabulary).as_str())
				{
					if !suffix.is_empty() {
						// Initialize candidate by concatenating definition key,
						// a colon (:),
						// and the substring of var that follows after the value of the definition's IRI mapping.
						let mut candidate = key.to_string();
						candidate.push(':');
						candidate.push_str(suffix);

						// If either compact IRI is null,
						// candidate is shorter or the same length but lexicographically less than
						// compact IRI and candidate does not have a term definition in active
						// context, or if that term definition has an IRI mapping that equals var
						// and value is null, set compact IRI to candidate.
						let candidate_def = active_context.get(candidate.as_str());
						if (compact_iri.is_empty()
							|| (candidate.len() <= compact_iri.len() && candidate < compact_iri))
							&& (candidate_def.is_none()
								|| (candidate_def.is_some()
									&& candidate_def
										.and_then(|def| def.value.as_ref())
										.map_or(false, |v| v == var) && value.is_none()))
						{
							compact_iri = candidate
						}
					}
				}
			}
			_ => (),
		}
	}

	// If compact IRI is not null, return compact IRI.
	if !compact_iri.is_empty() {
		return Ok(Some(Meta(compact_iri.as_str().into(), meta.clone())));
	}

	// To ensure that the IRI var is not confused with a compact IRI,
	// if the IRI scheme of var matches any term in active context with prefix flag set to true,
	// and var has no IRI authority (preceded by double-forward-slash (//),
	// an IRI confused with prefix error has been detected, and processing is aborted.
	if let Some(iri) = var.as_iri() {
		let iri = vocabulary.iri(iri).unwrap();
		if active_context.contains_key(iri.scheme().as_str()) {
			return Err(Meta(IriConfusedWithPrefix, meta.clone()));
		}
	}

	// If vocab is false,
	// transform var to a relative IRI reference using the base IRI from active context,
	// if it exists.
	if !vocab {
		if let Some(base_iri) = active_context.base_iri() {
			let base_iri = vocabulary.iri(base_iri).unwrap();
			if let Some(iri) = var.as_iri() {
				let iri = vocabulary.iri(iri).unwrap();
				return Ok(Some(Meta(
					iri.relative_to(base_iri).as_str().into(),
					meta.clone(),
				)));
			}
		}
	}

	// Finally, return var as is.
	Ok(Some(Meta(var.with(vocabulary).to_string(), meta.clone())))
}
