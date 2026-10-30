use json_syntax::object::Key;

use crate::{
	JsonLdBacktraceBuilder, JsonLdError, JsonLdIndexed, JsonLdLocatedError, JsonLdNullable,
	JsonLdObject, JsonLdProcessingMode, JsonLdTerm, JsonLdType, JsonLdValue,
	context::{
		JsonLdContainer,
		inverse::{JsonLdLangSelection, JsonLdSelection, JsonLdTypeSelection},
	},
	object::{self, AnyJsonLdObject},
	syntax::{is_keyword, is_keyword_like},
};

use super::{Compactor, TypeLangValue};

impl<'a> Compactor<'a> {
	pub fn compact_key(
		&self,
		var: &JsonLdTerm,
		vocab: bool,
		reverse: bool,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<Option<Key>, JsonLdLocatedError> {
		Ok(self
			.compact_iri(var, vocab, reverse, location)?
			.map(Into::into))
	}

	/// Compact the given term without considering any value.
	///
	/// Calls [`compact_iri_with`] with `None` for `value`.
	pub fn compact_iri(
		&self,
		var: &JsonLdTerm,
		vocab: bool,
		reverse: bool,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<Option<String>, JsonLdLocatedError> {
		self.compact_iri_with::<JsonLdObject>(var, vocab, reverse, None, location)
	}

	/// Compact the given term considering the given value object.
	///
	/// Default value for `value` is `None` and `false` for `vocab` and `reverse`.
	pub fn compact_iri_with<O>(
		&self,
		var: &JsonLdTerm,
		vocab: bool,
		reverse: bool,
		value: Option<&JsonLdIndexed<O>>,
		location: JsonLdBacktraceBuilder<'_>,
	) -> Result<Option<String>, JsonLdLocatedError>
	where
		O: AnyJsonLdObject,
	{
		if var.is_null() {
			return Ok(None);
		}

		if vocab {
			if let Some(entry) = self.active_context.inverse().get(var) {
				// Initialize containers to an empty array.
				// This array will be used to keep track of an ordered list of preferred container
				// mapping for a term, based on what is compatible with value.
				let mut containers = Vec::new();
				let mut type_lang_value = None;

				if let Some(value) = value
					&& value.index().is_some()
					&& !value.is_graph()
				{
					containers.push(JsonLdContainer::Index);
					containers.push(JsonLdContainer::IndexSet);
				}

				let mut has_index = false;
				let mut is_simple_value = false; // value object with no type, no index, no language and no direction.

				if reverse {
					type_lang_value = Some(TypeLangValue::Type(JsonLdTypeSelection::Reverse));
					containers.push(JsonLdContainer::Set);
				} else {
					let value_ref = value.map(|v| {
						has_index = v.index().is_some();
						v.inner().as_ref()
					});

					match value_ref {
						Some(object::JsonLdObjectRef::List(list)) => {
							if !has_index {
								containers.push(JsonLdContainer::List);
							}

							let mut common_type = None;
							let mut common_lang_dir = None;

							if list.is_empty() {
								common_lang_dir = Some(JsonLdNullable::Some((
									self.active_context.default_language(),
									self.active_context.default_base_direction(),
								)))
							} else {
								for item in list {
									let mut item_type = None;
									let mut item_lang_dir = None;
									let mut is_value = false;

									match item.inner() {
										JsonLdObject::Value(value) => {
											is_value = true;
											match value {
												JsonLdValue::LangString(lang_str) => {
													item_lang_dir = Some(JsonLdNullable::Some((
														lang_str.language(),
														lang_str.direction(),
													)))
												}
												JsonLdValue::Literal(lit) => match &lit.type_ {
													Some(
														object::value::JsonLdLiteralType::Json,
													) => item_type = Some(JsonLdType::Json),
													Some(
														object::value::JsonLdLiteralType::Iri(ty),
													) => {
														item_type =
															Some(JsonLdType::Iri(ty.clone()))
													}
													None => {
														item_lang_dir = Some(JsonLdNullable::Null)
													}
												},
											}
										}
										_ => item_type = Some(JsonLdType::Id),
									}

									if common_lang_dir.is_none() {
										common_lang_dir = item_lang_dir
									} else if is_value && common_lang_dir != item_lang_dir {
										common_lang_dir = Some(JsonLdNullable::Some((None, None)))
									}

									if common_type.is_none() {
										common_type = Some(item_type)
									} else if *common_type.as_ref().unwrap() != item_type {
										common_type = Some(None)
									}

									if common_lang_dir == Some(JsonLdNullable::Some((None, None)))
										&& common_type == Some(None)
									{
										break;
									}
								}
							}

							if common_lang_dir.is_none() {
								common_lang_dir = Some(JsonLdNullable::Some((None, None)))
							}
							let common_lang_dir = common_lang_dir.unwrap();

							if common_type.is_none() {
								common_type = Some(None)
							}
							let common_type = common_type.unwrap();

							if let Some(common_type) = common_type {
								type_lang_value = Some(TypeLangValue::Type(
									JsonLdTypeSelection::Type(common_type),
								))
							} else {
								type_lang_value = Some(TypeLangValue::Lang(
									JsonLdLangSelection::Lang(common_lang_dir),
								))
							}
						}
						Some(object::JsonLdObjectRef::Node(node)) if node.is_graph() => {
							// Otherwise, if value is a graph object, prefer a mapping most
							// appropriate for the particular value.
							if has_index {
								// If value contains an @index entry, append the values
								// @graph@index and @graph@index@set to containers.
								containers.push(JsonLdContainer::GraphIndex);
								containers.push(JsonLdContainer::GraphIndexSet);
							}

							if node.id.is_some() {
								// If value contains an @id entry, append the values @graph@id and
								// @graph@id@set to containers.
								containers.push(JsonLdContainer::GraphId);
								containers.push(JsonLdContainer::GraphIdSet);
							}

							// Append the values @graph, @graph@set, and @set to containers.
							containers.push(JsonLdContainer::Graph);
							containers.push(JsonLdContainer::GraphSet);
							containers.push(JsonLdContainer::Set);

							if !has_index {
								// If value does not contain an @index entry, append the values
								// @graph@index and @graph@index@set to containers.
								containers.push(JsonLdContainer::GraphIndex);
								containers.push(JsonLdContainer::GraphIndexSet);
							}

							if node.id.is_none() {
								// If the value does not contain an @id entry, append the values
								// @graph@id and @graph@id@set to containers.
								containers.push(JsonLdContainer::GraphId);
								containers.push(JsonLdContainer::GraphIdSet);
							}

							// Append the values @index and @index@set to containers.
							containers.push(JsonLdContainer::Index);
							containers.push(JsonLdContainer::IndexSet);

							type_lang_value = Some(TypeLangValue::Type(JsonLdTypeSelection::Type(
								JsonLdType::Id,
							)))
						}
						Some(object::JsonLdObjectRef::Value(v)) => {
							// If value is a value object:
							if (v.direction().is_some() || v.language().is_some()) && !has_index {
								type_lang_value =
									Some(TypeLangValue::Lang(JsonLdLangSelection::Lang(
										JsonLdNullable::Some((v.language(), v.direction())),
									)));
								containers.push(JsonLdContainer::Language);
								containers.push(JsonLdContainer::LanguageSet)
							} else if let Some(ty) = v.typ() {
								type_lang_value = Some(TypeLangValue::Type(
									JsonLdTypeSelection::Type(ty.into_object_type()),
								))
							} else {
								is_simple_value =
									v.direction().is_none() && v.language().is_none() && !has_index
							}

							containers.push(JsonLdContainer::Set)
						}
						_ => {
							// Otherwise, set type/language to @type and set type/language value
							// to @id, and append @id, @id@set, @type, and @set@type, to containers.
							type_lang_value = Some(TypeLangValue::Type(JsonLdTypeSelection::Type(
								JsonLdType::Id,
							)));
							containers.push(JsonLdContainer::Id);
							containers.push(JsonLdContainer::IdSet);
							containers.push(JsonLdContainer::Type);
							containers.push(JsonLdContainer::SetType);

							containers.push(JsonLdContainer::Set)
						}
					}
				}

				containers.push(JsonLdContainer::Null);

				if self.options.processing_mode != JsonLdProcessingMode::JsonLd1_0 && !has_index {
					containers.push(JsonLdContainer::Index);
					containers.push(JsonLdContainer::IndexSet)
				}

				if self.options.processing_mode != JsonLdProcessingMode::JsonLd1_0
					&& is_simple_value
				{
					containers.push(JsonLdContainer::Language);
					containers.push(JsonLdContainer::LanguageSet)
				}

				let mut is_empty_list = false;
				if let Some(value) = value
					&& let object::JsonLdObjectRef::List(list) = value.inner().as_ref()
					&& list.is_empty()
				{
					is_empty_list = true;
				}

				// If type/language value is @reverse, append @reverse to preferred values.
				let selection = if is_empty_list {
					JsonLdSelection::Any
				} else {
					match type_lang_value {
						Some(TypeLangValue::Type(type_value)) => {
							let mut selection: Vec<JsonLdTypeSelection> = Vec::new();

							if type_value == JsonLdTypeSelection::Reverse {
								selection.push(JsonLdTypeSelection::Reverse);
							}

							let mut has_id_type = false;
							if let Some(value) = value
								&& let Some(id) = value.id()
								&& (type_value == JsonLdTypeSelection::Type(JsonLdType::Id)
									|| type_value == JsonLdTypeSelection::Reverse)
							{
								has_id_type = true;
								let mut vocab = false;
								let compacted_iri = self
									.compact_iri(&id.clone().into_term(), true, false, location)?
									.unwrap();
								if let Some(def) = self.active_context.get(compacted_iri.as_str())
									&& let Some(iri_mapping) = def.value()
								{
									vocab = iri_mapping == id;
								}

								if vocab {
									selection.push(JsonLdTypeSelection::Type(JsonLdType::Vocab));
									selection.push(JsonLdTypeSelection::Type(JsonLdType::Id));
								} else {
									selection.push(JsonLdTypeSelection::Type(JsonLdType::Id));
									selection.push(JsonLdTypeSelection::Type(JsonLdType::Vocab));
								}

								selection.push(JsonLdTypeSelection::Type(JsonLdType::None));
							}

							if !has_id_type {
								selection.push(type_value);
								selection.push(JsonLdTypeSelection::Type(JsonLdType::None));
							}

							selection.push(JsonLdTypeSelection::Any);

							JsonLdSelection::Type(selection)
						}
						Some(TypeLangValue::Lang(lang_value)) => {
							let mut selection = vec![
								lang_value,
								JsonLdLangSelection::Lang(JsonLdNullable::Some((None, None))),
								JsonLdLangSelection::Any,
							];

							if let JsonLdLangSelection::Lang(JsonLdNullable::Some((
								Some(_),
								Some(dir),
							))) = lang_value
							{
								selection.push(JsonLdLangSelection::Lang(JsonLdNullable::Some((
									None,
									Some(dir),
								))));
							}

							JsonLdSelection::Lang(selection)
						}
						None => JsonLdSelection::Lang(vec![
							JsonLdLangSelection::Lang(JsonLdNullable::Null),
							JsonLdLangSelection::Lang(JsonLdNullable::Some((None, None))),
							JsonLdLangSelection::Any,
						]),
					}
				};

				if let Some(term) = entry.select(&containers, &selection) {
					return Ok(Some(term.to_string()));
				}
			}

			// At this point, there is no simple term that var can be compacted to.
			// If vocab is true and active context has a vocabulary mapping:
			if let Some(vocab_mapping) = self.active_context.vocabulary() {
				// If var begins with the vocabulary mapping's value but is longer, then initialize
				// suffix to the substring of var that does not match. If suffix does not have a term
				// definition in active context, then return suffix.
				if let Some(suffix) = var.as_str().strip_prefix(vocab_mapping.as_str())
					&& !suffix.is_empty()
					&& self.active_context.get(suffix).is_none()
				{
					return Ok(Some(suffix.into()));
				}
			}
		}

		// The var could not be compacted using the active context's vocabulary mapping.
		// Try to create a compact IRI, starting by initializing compact IRI to null.
		// This variable will be used to store the created compact IRI, if any.
		let mut compact_iri = String::new();

		// For each term definition definition in active context:
		for binding in self.active_context.definitions() {
			let key = binding.term();
			let definition = binding.definition();
			// If the IRI mapping of definition is null, its IRI mapping equals var,
			// its IRI mapping is not a substring at the beginning of var,
			// or definition does not have a true prefix flag,
			// definition's key cannot be used as a prefix.
			// Continue with the next definition.
			match definition.value() {
				Some(iri_mapping) if definition.prefix() => {
					if let Some(suffix) = var.as_str().strip_prefix(iri_mapping.as_str())
						&& !suffix.is_empty()
					{
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
						let candidate_def = self.active_context.get(candidate.as_str());
						if (compact_iri.is_empty()
							|| (candidate.len() <= compact_iri.len() && candidate < compact_iri))
							&& (candidate_def.is_none()
								|| (candidate_def.is_some()
									&& (candidate_def.and_then(|def| def.value()) == Some(var))
									&& value.is_none()))
						{
							compact_iri = candidate
						}
					}
				}
				_ => (),
			}
		}

		// If compact IRI is not null, return compact IRI.
		if !compact_iri.is_empty() {
			return Ok(Some(compact_iri.as_str().into()));
		}

		// To ensure that the IRI var is not confused with a compact IRI,
		// if the IRI scheme of var matches any term in active context with prefix flag set to true,
		// and var has no IRI authority (preceded by double-forward-slash (//),
		// an IRI confused with prefix error has been detected, and processing is aborted.
		if let Some(iri) = var.as_iri()
			&& self.active_context.contains_term(iri.scheme().as_str())
		{
			return Err(JsonLdError::IriConfusedWithPrefix.at(location));
		}

		// If vocab is false,
		// transform var to a relative IRI reference using the base IRI from active context,
		// if it exists.
		if !vocab
			&& let Some(base_iri) = self.active_context.base_iri()
			&& let Some(iri) = var.as_iri()
		{
			return Ok(Some(disambiguate_keyword(
				iri.relative_to(base_iri).as_str().into(),
			)));
		}

		// Finally, return var as is.
		Ok(Some(var.to_string()))
	}
}

fn disambiguate_keyword(s: String) -> String {
	if is_keyword_like(&s) && !is_keyword(&s) {
		"./".to_string() + &s
	} else {
		s
	}
}
