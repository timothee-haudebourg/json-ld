use futures::future::BoxFuture;
use std::collections::HashMap;
use generic_json::ValueRef;
use json_ld_core::{
	Id,
	Context,
	context::TermDefinition,
	ProcessingMode,
	syntax::{Term, Keyword, is_keyword, is_keyword_like},
	Loc
};
use iref::Iri;
use crate::{
	Process,
	ProcessingStack,
	ProcessingOptions,
	Loader,
	LocWarning,
	Warning,
	Error
};
use super::{
	JsonContext,
	LocalContextObject,
	WrappedValue,
	expand_iri
};

/// Follows the `https://www.w3.org/TR/json-ld11-api/#create-term-definition` algorithm.
/// Default value for `base_url` is `None`. Default values for `protected` and `override_protected` are `false`.
pub fn define<
	'a,
	J: JsonContext,
	T: Id + Send + Sync,
	C: ProcessMeta<T>,
	L: Loader + Send + Sync,
>(
	active_context: &'a mut Context<T, C>,
	local_context: &'a LocalContextObject<'a, J::Object>,
	term: &'a str,
	term_metadata: &'a C::MetaData,
	defined: &'a mut HashMap<String, bool>,
	remote_contexts: ProcessingStack,
	loader: &'a mut L,
	base_url: Option<Iri<'a>>,
	protected: bool,
	options: ProcessingOptions,
	warnings: &'a mut Vec<LocWarning<C::Source, C::MetaData>>,
) -> BoxFuture<'a, Result<(), Error>>
where
	L::Output: Into<J>,
{
	let source = base_url.and_then(|iri| loader.source(iri));
	async move {
		match defined.get(term) {
			// If defined contains the entry term and the associated value is true (indicating
			// that the term definition has already been created), return.
			Some(true) => Ok(()),
			// Otherwise, if the value is false, a cyclic IRI mapping error has been detected and processing is aborted.
			Some(false) => Err(Error::CyclicIriMapping),
			None => {
				if term.is_empty() {
					return Err(Error::InvalidTermDefinition);
				}

				// Initialize `value` to a copy of the value associated with the entry `term` in
				// `local_context`.
				if let Some(value) = local_context.get(term) {
					// Set the value associated with defined's term entry to false.
					// This indicates that the term definition is now being created but is not yet
					// complete.
					defined.insert(term.to_string(), false);

					// If term is @type, ...
					if term == "@type" {
						// ... and processing mode is json-ld-1.0, a keyword
						// redefinition error has been detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::KeywordRedefinition);
						}

						// At this point, `value` MUST be a map with only either or both of the
						// following entries:
						// An entry for @container with value @set.
						// An entry for @protected.
						// Any other value means that a keyword redefinition error has been detected
						// and processing is aborted.
						if let ValueRef::Object(value) = value.as_value_ref() {
							if value.is_empty() {
								return Err(Error::KeywordRedefinition);
							}

							for (key, value) in value.iter() {
								match key.as_ref() {
									"@container" if value.as_str() == Some("@set") => (),
									"@protected" => (),
									_ => return Err(Error::KeywordRedefinition),
								}
							}
						} else {
							return Err(Error::KeywordRedefinition);
						}
					} else {
						// Otherwise, since keywords cannot be overridden, term MUST NOT be a keyword and
						// a keyword redefinition error has been detected and processing is aborted.
						if is_keyword(term) {
							return Err(Error::KeywordRedefinition);
						} else {
							// If term has the form of a keyword (i.e., it matches the ABNF rule "@"1*ALPHA
							// from [RFC5234]), return; processors SHOULD generate a warning.
							if is_keyword_like(term) {
								warnings.push(Warning::KeywordLikeTerm(term.to_string()).located(source, term_metadata.clone()));
								return Ok(());
							}
						}
					}

					// Initialize `previous_definition` to any existing term definition for `term` in
					// `active_context`, removing that term definition from active context.
					let previous_definition = active_context.set(term, None);

					let mut simple_term = true;
					let value: WrappedValue<'_, J> = match value.as_value_ref() {
						ValueRef::Null => {
							// If `value` is null, convert it to a map consisting of a single entry
							// whose key is @id and whose value is null.
							WrappedValue::WrappedNull
						}
						ValueRef::String(str_value) => {
							// Otherwise, if value is a string, convert it to a map consisting of a
							// single entry whose key is @id and whose value is value. Set simple
							// term to true (it already is).
							WrappedValue::Wrapped(str_value, value.metadata())
						}
						ValueRef::Object(value) => {
							simple_term = false;
							WrappedValue::Unwrapped(value)
						}
						_ => return Err(Error::InvalidTermDefinition),
					};

					// Create a new term definition, `definition`, initializing `prefix` flag to
					// `false`, `protected` to `protected`, and `reverse_property` to `false`.
					let mut definition = TermDefinition::<T, C> {
						protected,
						..Default::default()
					};

					// If the @protected entry in value is true set the protected flag in
					// definition to true.
					if let Some(protected_value) = value.get("@protected") {
						if let Some(b) = protected_value.as_bool() {
							definition.protected = b;
						} else {
							// If the value of @protected is not a boolean, an invalid @protected
							// value error has been detected.
							return Err(Error::InvalidProtectedValue);
						}

						// If processing mode is json-ld-1.0, an invalid term definition has
						// been detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::InvalidTermDefinition);
						}
					}

					// If value contains the entry @type:
					if let Some(type_value) = value.get("@type") {
						// Initialize `typ` to the value associated with the `@type` entry, which
						// MUST be a string. Otherwise, an invalid type mapping error has been
						// detected and processing is aborted.
						if let Some(typ) = type_value.as_str() {
							// Set `typ` to the result of IRI expanding type, using local context,
							// and defined.
							let typ = expand_iri(
								active_context,
								typ,
								source,
								type_value.metadata(),
								false,
								true,
								local_context,
								defined,
								remote_contexts.clone(),
								loader,
								options,
								warnings,
							)
							.await?;
							// If the expanded type is @json or @none, and processing mode is
							// json-ld-1.0, an invalid type mapping error has been detected and
							// processing is aborted.
							if options.processing_mode == ProcessingMode::JsonLd1_0
								&& (typ == Term::Keyword(Keyword::Json)
									|| typ == Term::Keyword(Keyword::None))
							{
								return Err(Error::InvalidTypeMapping);
							}

							if let Ok(typ) = typ.try_into() {
								// Set the type mapping for definition to type.
								definition.typ = Some(typ);
							} else {
								return Err(Error::InvalidTypeMapping);
							}
						} else {
							return Err(Error::InvalidTypeMapping);
						}
					}

					// If `value` contains the entry @reverse:
					if let Some(reverse_value) = value.get("@reverse") {
						// If `value` contains `@id` or `@nest`, entries, an invalid reverse
						// property error has been detected and processing is aborted.
						if value.id().is_some() || value.get("@nest").is_some() {
							return Err(Error::InvalidReverseProperty);
						}

						let reverse_value_metadata = reverse_value.metadata();
						if let Some(reverse_value) = reverse_value.as_str() {
							// If the value associated with the @reverse entry is a string having
							// the form of a keyword, return; processors SHOULD generate a warning.
							if is_keyword_like(reverse_value) {
								warnings.push(Warning::KeywordLikeValue(reverse_value.into()).located(source, reverse_value_metadata.clone()));
								return Ok(());
							}

							// Otherwise, set the IRI mapping of definition to the result of IRI
							// expanding the value associated with the @reverse entry, using
							// local context, and defined.
							// If the result does not have the form of an IRI or a blank node
							// identifier, an invalid IRI mapping error has been detected and
							// processing is aborted.
							match expand_iri(
								active_context,
								reverse_value,
								source,
								reverse_value_metadata,
								false,
								true,
								local_context,
								defined,
								remote_contexts,
								loader,
								options,
								warnings,
							)
							.await?
							{
								Term::Ref(mapping) if mapping.is_valid() => {
									definition.value = Some(Term::Ref(mapping))
								}
								_ => return Err(Error::InvalidIriMapping),
							}

							// If `value` contains an `@container` entry, set the `container`
							// mapping of `definition` to an array containing its value;
							// if its value is neither `@set`, nor `@index`, nor null, an
							// invalid reverse property error has been detected (reverse properties
							// only support set- and index-containers) and processing is aborted.
							if let Some(container_value) = value.get("@container") {
								match container_value.as_value_ref() {
									ValueRef::Null => (),
									ValueRef::String(_) => {
										if let Ok(container_value) = ContainerType::try_from(
											container_value.as_str().unwrap(),
										) {
											match container_value {
												ContainerType::Set | ContainerType::Index => {
													definition.container.add(container_value);
												}
												_ => {
													return Err(
														ErrorCode::InvalidReverseProperty.into()
													)
												}
											}
										} else {
											return Err(ErrorCode::InvalidReverseProperty.into());
										}
									}
									_ => return Err(ErrorCode::InvalidReverseProperty.into()),
								};
							}

							// Set the `reverse_property` flag of `definition` to `true`.
							definition.reverse_property = true;

							// Set the term definition of `term` in `active_context` to
							// `definition` and the value associated with `defined`'s entry `term`
							// to `true` and return.
							active_context.set(term, Some(definition));
							defined.insert(term.to_string(), true);
							return Ok(());
						} else {
							// If the value associated with the `@reverse` entry is not a string,
							// an invalid IRI mapping error has been detected and processing is
							// aborted.
							return Err(ErrorCode::InvalidIriMapping.into());
						}
					}

					// If `value` contains the entry `@id` and its value does not equal `term`:
					match value.id() {
						Some(id_value) if id_value.as_str() != Some(term) => {
							// If the `@id` entry of value is `null`, the term is not used for IRI
							// expansion, but is retained to be able to detect future redefinitions
							// of this term.
							if !id_value.is_null() {
								// Otherwise:
								let id_value_metadata = id_value.metadata().unwrap();
								if let Some(id_value) = id_value.as_str() {
									// If the value associated with the `@id` entry is not a
									// keyword, but has the form of a keyword, return;
									// processors SHOULD generate a warning.
									if is_keyword_like(id_value) && !is_keyword(id_value) {
										warnings.push(Loc::new(
											Warning::KeywordLikeValue(id_value.into()),
											source,
											id_value_metadata.clone(),
										));
										return Ok(());
									}

									// Otherwise, set the IRI mapping of `definition` to the result
									// of IRI expanding the value associated with the `@id` entry,
									// using `local_context`, and `defined`.
									definition.value = match expand_iri(
										active_context,
										id_value,
										source,
										id_value_metadata,
										false,
										true,
										local_context,
										defined,
										remote_contexts.clone(),
										loader,
										options,
										warnings,
									)
									.await?
									{
										Term::Keyword(Keyword::Context) => {
											// if it equals `@context`, an invalid keyword alias error has
											// been detected and processing is aborted.
											return Err(ErrorCode::InvalidKeywordAlias.into());
										}
										Term::Ref(prop) if !prop.is_valid() => {
											// If the resulting IRI mapping is neither a keyword,
											// nor an IRI, nor a blank node identifier, an
											// invalid IRI mapping error has been detected and processing
											// is aborted;
											return Err(ErrorCode::InvalidIriMapping.into());
										}
										value => Some(value),
									};

									// If `term` contains a colon (:) anywhere but as the first or
									// last character of `term`, or if it contains a slash (/)
									// anywhere:
									if contains_between_boundaries(term, ':') || term.contains('/')
									{
										// Set the value associated with `defined`'s `term` entry
										// to `true`.
										defined.insert(term.to_string(), true);

										// If the result of IRI expanding `term` using
										// `local_context`, and `defined`, is not the same as the
										// IRI mapping of definition, an invalid IRI mapping error
										// has been detected and processing is aborted.
										let expanded_term = expand_iri(
											active_context,
											term,
											source,
											term_metadata,
											false,
											true,
											local_context,
											defined,
											remote_contexts.clone(),
											loader,
											options,
											warnings,
										)
										.await?;
										if definition.value != Some(expanded_term) {
											return Err(ErrorCode::InvalidIriMapping.into());
										}
									}

									// If `term` contains neither a colon (:) nor a slash (/),
									// simple term is true, and if the IRI mapping of definition
									// is either an IRI ending with a gen-delim character,
									// or a blank node identifier, set the `prefix` flag in
									// `definition` to true.
									if !term.contains(':')
										&& !term.contains('/') && simple_term
										&& is_gen_delim_or_blank(definition.value.as_ref().unwrap())
									{
										definition.prefix = true;
									}
								} else {
									// If the value associated with the `@id` entry is not a
									// string, an invalid IRI mapping error has been detected and
									// processing is aborted.
									return Err(ErrorCode::InvalidIriMapping.into());
								}
							}
						}
						_ => {
							if contains_after_first(term, ':') {
								// Otherwise if the `term` contains a colon (:) anywhere after the first
								// character:
								let i = term.find(':').unwrap();
								let (prefix, suffix) = term.split_at(i);
								let suffix = &suffix[1..suffix.len()];

								// If `term` is a compact IRI with a prefix that is an entry in local
								// context a dependency has been found.
								// Use this algorithm recursively passing `active_context`,
								// `local_context`, the prefix as term, and `defined`.
								define(
									active_context,
									local_context,
									prefix,
									term_metadata,
									defined,
									remote_contexts.clone(),
									loader,
									None,
									false,
									options.with_no_override(),
									warnings,
								)
								.await?;

								// If `term`'s prefix has a term definition in `active_context`, set the
								// IRI mapping of `definition` to the result of concatenating the value
								// associated with the prefix's IRI mapping and the term's suffix.
								if let Some(prefix_definition) = active_context.get(prefix) {
									let mut result = String::new();

									if let Some(prefix_key) = &prefix_definition.value {
										if let Some(prefix_iri) = prefix_key.as_iri() {
											result = prefix_iri.as_str().to_string()
										}
									}

									result.push_str(suffix);

									if let Ok(iri) = Iri::new(result.as_str()) {
										definition.value = Some(Term::<T>::from(T::from_iri(iri)))
									} else {
										return Err(ErrorCode::InvalidIriMapping.into());
									}
								} else {
									// Otherwise, `term` is an IRI or blank node identifier.
									// Set the IRI mapping of `definition` to `term`.
									if prefix == "_" {
										// blank node
										definition.value = Some(BlankId::new(suffix).into())
									} else if let Ok(iri) = Iri::new(term) {
										definition.value = Some(Term::<T>::from(T::from_iri(iri)))
									} else {
										return Err(ErrorCode::InvalidIriMapping.into());
									}
								}
							} else if term.contains('/') {
								// Term is a relative IRI reference.
								// Set the IRI mapping of definition to the result of IRI expanding
								// term.
								match expansion::expand_iri(
									source,
									active_context,
									term,
									term_metadata,
									false,
									true,
									warnings,
								) {
									Term::Ref(Reference::Id(id)) => {
										definition.value = Some(id.into())
									}
									// If the resulting IRI mapping is not an IRI, an invalid IRI mapping
									// error has been detected and processing is aborted.
									_ => return Err(ErrorCode::InvalidIriMapping.into()),
								}
							} else if term == "@type" {
								// Otherwise, if `term` is ``@type`, set the IRI mapping of definition to
								// `@type`.
								definition.value = Some(Term::Keyword(Keyword::Type))
							} else if let Some(vocabulary) = active_context.vocabulary() {
								// Otherwise, if `active_context` has a vocabulary mapping, the IRI mapping
								// of `definition` is set to the result of concatenating the value
								// associated with the vocabulary mapping and `term`.
								// If it does not have a vocabulary mapping, an invalid IRI mapping error
								// been detected and processing is aborted.
								if let Some(vocabulary_iri) = vocabulary.as_iri() {
									let mut result = vocabulary_iri.as_str().to_string();
									result.push_str(term);
									if let Ok(iri) = Iri::new(result.as_str()) {
										definition.value = Some(Term::<T>::from(T::from_iri(iri)))
									} else {
										return Err(ErrorCode::InvalidIriMapping.into());
									}
								} else {
									return Err(ErrorCode::InvalidIriMapping.into());
								}
							} else {
								// If it does not have a vocabulary mapping, an invalid IRI mapping error
								// been detected and processing is aborted.
								return Err(ErrorCode::InvalidIriMapping.into());
							}
						}
					}

					// If value contains the entry @container:
					if let Some(container_value) = value.get("@container") {
						// If the container value is @graph, @id, or @type, or is otherwise not a
						// string, generate an invalid container mapping error and abort processing
						// if processing mode is json-ld-1.0.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							match container_value.as_str() {
								Some("@graph") | Some("@id") | Some("@type") | None => {
									return Err(ErrorCode::InvalidContainerMapping.into())
								}
								_ => (),
							}
						}

						// Initialize `container` to the value associated with the `@container`
						// entry, which MUST be either `@graph`, `@id`, `@index`, `@language`,
						// `@list`, `@set`, `@type`, or an array containing exactly any one of
						// those keywords, an array containing `@graph` and either `@id` or
						// `@index` optionally including `@set`, or an array containing a
						// combination of `@set` and any of `@index`, `@graph`, `@id`, `@type`,
						// `@language` in any order.
						// Otherwise, an invalid container mapping has been detected and processing
						// is aborted.
						let (container_value, _) = as_array(&*container_value);
						for entry in container_value {
							if let Some(entry) = entry.as_str() {
								match ContainerType::try_from(entry) {
									Ok(c) => {
										if !definition.container.add(c) {
											return Err(ErrorCode::InvalidContainerMapping.into());
										}
									}
									Err(_) => return Err(ErrorCode::InvalidContainerMapping.into()),
								}
							} else {
								return Err(ErrorCode::InvalidContainerMapping.into());
							}
						}

						// Set the container mapping of definition to container coercing to an
						// array, if necessary.
						// already done.

						// If the `container` mapping of definition includes `@type`:
						if definition.container.contains(ContainerType::Type) {
							if let Some(typ) = &definition.typ {
								// If type mapping in definition is neither `@id` nor `@vocab`,
								// an invalid type mapping error has been detected and processing
								// is aborted.
								match typ {
									Type::Id | Type::Vocab => (),
									_ => return Err(ErrorCode::InvalidTypeMapping.into()),
								}
							} else {
								// If type mapping in definition is undefined, set it to @id.
								definition.typ = Some(Type::Id)
							}
						}
					}

					// If value contains the entry @index:
					if let Some(index_value) = value.get("@index") {
						// If processing mode is json-ld-1.0 or container mapping does not include
						// `@index`, an invalid term definition has been detected and processing
						// is aborted.
						if !definition.container.contains(ContainerType::Index)
							|| options.processing_mode == ProcessingMode::JsonLd1_0
						{
							return Err(ErrorCode::InvalidTermDefinition.into());
						}

						// Initialize `index` to the value associated with the `@index` entry,
						// which MUST be a string expanding to an IRI.
						// Otherwise, an invalid term definition has been detected and processing
						// is aborted.
						if let Some(index) = index_value.as_str() {
							match expansion::expand_iri(
								source,
								active_context,
								index,
								index_value.metadata(),
								false,
								true,
								warnings,
							) {
								Term::Ref(Reference::Id(_)) => (),
								_ => return Err(ErrorCode::InvalidTermDefinition.into()),
							}

							definition.index = Some(index.to_string())
						} else {
							return Err(ErrorCode::InvalidTermDefinition.into());
						}
					}

					// If `value` contains the entry `@context`:
					if let Some(context) = value.get("@context") {
						// If processing mode is json-ld-1.0, an invalid term definition has been
						// detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidTermDefinition.into());
						}

						// Initialize `context` to the value associated with the @context entry,
						// which is treated as a local context.
						// done.

						// Invoke the Context Processing algorithm using the `active_context`,
						// `context` as local context, `base_url`, and `true` for override
						// protected.
						// If any error is detected, an invalid scoped context error has been
						// detected and processing is aborted.
						process_context(
							active_context,
							&*context,
							remote_contexts.clone(),
							loader,
							base_url,
							options.with_override(),
							warnings,
						)
						.await
						.map_err(|_| Error::from(ErrorCode::InvalidScopedContext))?;

						// Set the local context of definition to context, and base URL to base URL.
						definition.context = Some(C::LocalContext::from((*context).clone()));
						definition.base_url = base_url.as_ref().map(|url| url.into());
					}

					// If `value` contains the entry `@language` and does not contain the entry
					// `@type`:
					if value.get("@type").is_none() {
						if let Some(language_value) = value.get("@language") {
							// Initialize `language` to the value associated with the `@language`
							// entry, which MUST be either null or a string.
							// If `language` is not well-formed according to section 2.2.9 of
							// [BCP47], processors SHOULD issue a warning.
							// Otherwise, an invalid language mapping error has been detected and
							// processing is aborted.
							// Set the `language` mapping of definition to `language`.
							definition.language = Some(match language_value.as_value_ref() {
								ValueRef::Null => Nullable::Null,
								ValueRef::String(lang_str) => {
									let lang_str: &str = lang_str.as_ref();
									match LanguageTagBuf::parse_copy(lang_str) {
										Ok(lang) => Nullable::Some(lang.into()),
										Err(err) => {
											warnings.push(Loc::new(
												Warning::MalformedLanguageTag(
													lang_str.to_string(),
													err,
												),
												source,
												language_value.metadata().clone(),
											));
											Nullable::Some(lang_str.to_string().into())
										}
									}
								}
								_ => return Err(ErrorCode::InvalidLanguageMapping.into()),
							});
						}

						// If `value` contains the entry `@direction` and does not contain the
						// entry `@type`:
						if let Some(direction_value) = value.get("@direction") {
							// Initialize `direction` to the value associated with the `@direction`
							// entry, which MUST be either null, "ltr", or "rtl".
							definition.direction = Some(match direction_value.as_str() {
								Some("ltr") => Nullable::Some(Direction::Ltr),
								Some("rtl") => Nullable::Some(Direction::Rtl),
								_ => {
									if direction_value.is_null() {
										Nullable::Null
									} else {
										// Otherwise, an invalid base direction error has been
										// detected and processing is aborted.
										return Err(ErrorCode::InvalidBaseDirection.into());
									}
								}
							});
						}
					}

					// If value contains the entry @nest:
					if let Some(nest_value) = value.get("@nest") {
						// If processing mode is json-ld-1.0, an invalid term definition has been
						// detected and processing is aborted.
						if options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(ErrorCode::InvalidTermDefinition.into());
						}

						// Initialize `nest` value in `definition` to the value associated with the
						// `@nest` entry, which MUST be a string and MUST NOT be a keyword other
						// than @nest.
						if let Some(nest_value) = nest_value.as_str() {
							if is_keyword(nest_value) && nest_value != "@nest" {
								return Err(ErrorCode::InvalidNestValue.into());
							}

							definition.nest = Some(nest_value.to_string());
						} else {
							// Otherwise, an invalid @nest value error has been detected and
							// processing is aborted.
							return Err(ErrorCode::InvalidNestValue.into());
						}
					}

					// If value contains the entry @prefix:
					if let Some(prefix_value) = value.get("@prefix") {
						// If processing mode is json-ld-1.0, or if `term` contains a colon (:) or
						// slash (/), an invalid term definition has been detected and processing
						// is aborted.
						if term.contains(':')
							|| term.contains('/') || options.processing_mode
							== ProcessingMode::JsonLd1_0
						{
							return Err(ErrorCode::InvalidTermDefinition.into());
						}

						// Set the `prefix` flag to the value associated with the @prefix entry,
						// which MUST be a boolean.
						// Otherwise, an invalid @prefix value error has been detected and
						// processing is aborted.
						if let Some(prefix) = prefix_value.as_bool() {
							definition.prefix = prefix
						} else {
							return Err(ErrorCode::InvalidPrefixValue.into());
						}

						// If the `prefix` flag of `definition` is set to `true`, and its IRI
						// mapping is a keyword, an invalid term definition has been detected and
						// processing is aborted.
						if definition.prefix && definition.value.as_ref().unwrap().is_keyword() {
							return Err(ErrorCode::InvalidTermDefinition.into());
						}
					}

					// If value contains any entry other than @id, @reverse, @container, @context,
					// @direction, @index, @language, @nest, @prefix, @protected, or @type, an
					// invalid term definition error has been detected and processing is aborted.
					for (key, _) in value.iter() {
						match key.as_ref() {
							"@id" | "@reverse" | "@container" | "@context" | "@direction"
							| "@index" | "@language" | "@nest" | "@prefix" | "@protected"
							| "@type" => (),
							_ => return Err(ErrorCode::InvalidTermDefinition.into()),
						}
					}

					// If override protected is false and previous_definition exists and is protected;
					if !options.override_protected {
						if let Some(previous_definition) = previous_definition {
							if previous_definition.protected {
								// If `definition` is not the same as `previous_definition`
								// (other than the value of protected), a protected term
								// redefinition error has been detected, and processing is aborted.
								if definition != previous_definition {
									return Err(ErrorCode::ProtectedTermRedefinition.into());
								}

								// Set `definition` to `previous definition` to retain the value of
								// protected.
								definition.protected = true;
							}
						}
					}

					// Set the term definition of `term` in `active_context` to `definition` and
					// set the value associated with `defined`'s entry term to true.
					active_context.set(term, Some(definition));
					defined.insert(term.to_string(), true);
				}

				// if the term is not in `local_context`.
				Ok(())
			}
		}
	}
	.boxed()
}