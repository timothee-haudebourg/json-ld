use std::collections::HashMap;

use iref::{Iri, IriRef};
use rdf_types::BlankId;

use crate::{
	algorithms::{
		context_processing::{merged::Merged, ContextProcessor, TargetProcessedContext},
		error::Error,
		warning::Warning,
		ProcessingEnvironment,
	},
	context::{NormalTermDefinition, TypeTermDefinition},
	syntax::{
		context::{EntryValueRef, ExpandedTermDefinitionRef, IdRef, KeyOrKeyword, KeyOrKeywordRef},
		CompactIri, Container, ContainerItem, ExpandableRef, Keyword,
	},
	Id, LenientLangTag, Nullable, ProcessingMode, Term, Type, ValidId,
};

fn is_gen_delim(c: char) -> bool {
	matches!(c, ':' | '/' | '?' | '#' | '[' | ']' | '@')
}

// Checks if the input term is an IRI ending with a gen-delim character, or a blank node identifier.
fn is_gen_delim_or_blank(t: &Term) -> bool {
	match t {
		Term::Id(Id::Valid(ValidId::BlankId(_))) => true,
		Term::Id(Id::Valid(ValidId::Iri(iri))) => {
			if let Some(c) = iri.chars().last() {
				is_gen_delim(c)
			} else {
				false
			}
		}
		_ => false,
	}
}

/// Checks if the the given character is included in the given string anywhere but at the first or last position.
fn contains_between_boundaries(id: &str, c: char) -> bool {
	if let Some(i) = id.find(c) {
		let j = id.rfind(c).unwrap();
		i > 0 && j < id.len() - 1
	} else {
		false
	}
}

#[derive(Default)]
pub struct DefinedTerms(HashMap<KeyOrKeyword, DefinedTerm>);

impl DefinedTerms {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn begin(&mut self, key: &KeyOrKeyword) -> Result<bool, Error> {
		match self.0.get(key) {
			Some(d) => {
				if d.pending {
					Err(Error::CyclicIriMapping)
				} else {
					Ok(false)
				}
			}
			None => {
				self.0.insert(key.clone(), DefinedTerm { pending: true });

				Ok(true)
			}
		}
	}

	pub fn end(&mut self, key: &KeyOrKeyword) {
		self.0.get_mut(key).unwrap().pending = false
	}
}

pub struct DefinedTerm {
	pending: bool,
}

// /// Environment of a context term definition.
// pub struct TermDefiner<'a> {
// 	pub defined: &'a mut DefinedTerms,
// 	pub local_context: &'a Merged<'a>,
// 	pub base_url: Option<&'a Iri>,
// 	pub remote_contexts: ProcessingStack,
// 	pub options: ContextProcessingOptions,
// }

// impl<'a> TermDefiner<'a> {
// 	pub fn reborrow(&mut self) -> TermDefiner<'_> {
// 		TermDefiner {
// 			defined: self.defined,
// 			local_context: self.local_context,
// 			base_url: self.base_url,
// 			remote_contexts: self.remote_contexts.clone(),
// 			options: self.options,
// 		}
// 	}

// 	pub fn for_recursive_definition(&mut self) -> TermDefiner<'_> {
// 		TermDefiner {
// 			defined: self.defined,
// 			local_context: self.local_context,
// 			base_url: None,
// 			remote_contexts: self.remote_contexts.clone(),
// 			options: self.options.with_no_override(),
// 		}
// 	}

// 	pub async fn process_context(
// 		&mut self,
// 		context: &Context,
// 		active_context: &ProcessedContext,
// 		options: ContextProcessingOptions,
// 	) -> Result<ProcessedContext, Error> {
// 		let env = ContextProcessor {
// 			active_context,
// 			remote_contexts: self.remote_contexts.clone(),
// 			base_url: self.base_url,
// 			options,
// 		};

// 		Box::pin(context.process_in(env)).await
// 	}
// }

impl<'a> ContextProcessor<'a> {
	/// Follows the `https://www.w3.org/TR/json-ld11-api/#create-term-definition` algorithm.
	/// Default value for `base_url` is `None`. Default values for `protected` and `override_protected` are `false`.
	pub async fn define(
		&self,
		env: &mut impl ProcessingEnvironment,
		result: &mut TargetProcessedContext<'_>,
		local_context: &Merged<'_>,
		term: KeyOrKeywordRef<'_>,
		protected: bool,
	) -> Result<(), Error> {
		let term = term.to_owned();
		if result.defined.begin(&term)? {
			if term.is_empty() {
				return Err(Error::InvalidTermDefinition);
			}

			// Initialize `value` to a copy of the value associated with the entry `term` in
			// `local_context`.
			if let Some(value) = local_context.get(&term) {
				// Set the value associated with defined's term entry to false.
				// This indicates that the term definition is now being created but is not yet
				// complete.
				// Done with `defined.begin`.
				match value {
					// If term is @type, ...
					EntryValueRef::Type(d) => {
						// ... and processing mode is json-ld-1.0, a keyword
						// redefinition error has been detected and processing is aborted.
						if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
							return Err(Error::KeywordRedefinition);
						}

						let previous_definition = result.value.set_type(None);

						// At this point, `value` MUST be a map with only either or both of the
						// following entries:
						// An entry for @container with value @set.
						// An entry for @protected.
						// Any other value means that a keyword redefinition error has been detected
						// and processing is aborted.
						// Checked during parsing.
						let mut definition = TypeTermDefinition {
							container: d.container,
							..Default::default()
						};

						if let Some(protected) = d.protected {
							if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
								return Err(Error::InvalidTermDefinition);
							}

							definition.protected = protected
						}

						// If override protected is false and previous_definition exists and is protected;
						if !self.options.override_protected {
							if let Some(previous_definition) = previous_definition {
								if previous_definition.protected {
									// If `definition` is not the same as `previous_definition`
									// (other than the value of protected), a protected term
									// redefinition error has been detected, and processing is aborted.
									if definition.modulo_protected_field()
										!= previous_definition.modulo_protected_field()
									{
										return Err(Error::ProtectedTermRedefinition);
									}

									// Set `definition` to `previous definition` to retain the value of
									// protected.
									definition.protected = true;
								}
							}
						}

						result.value.set_type(Some(definition));
					}
					EntryValueRef::Definition(d) => {
						let key = term.as_key().unwrap();
						// Initialize `previous_definition` to any existing term definition for `term` in
						// `active_context`, removing that term definition from active context.
						let previous_definition = result.value.set_normal(key.clone(), None);

						let simple_term = !d.map(|d| d.is_expanded()).unwrap_or(false);
						let value = ExpandedTermDefinitionRef::from(d);

						// Create a new term definition, `definition`, initializing `prefix` flag to
						// `false`, `protected` to `protected`, and `reverse_property` to `false`.
						let mut definition = NormalTermDefinition {
							protected,
							..Default::default()
						};

						// If the @protected entry in value is true set the protected flag in
						// definition to true.
						if let Some(protected) = value.protected {
							// If processing mode is json-ld-1.0, an invalid term definition has
							// been detected and processing is aborted.
							if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
								return Err(Error::InvalidTermDefinition);
							}

							definition.protected = protected;
						}

						// If value contains the entry @type:
						if let Some(type_) = value.type_ {
							// Set `typ` to the result of IRI expanding type, using local context,
							// and defined.
							let typ = self
								.expand_iri_recursive(
									env,
									result,
									local_context,
									type_.cast(),
									false,
									true,
								)
								.await?;

							// If the expanded type is @json or @none, and processing mode is
							// json-ld-1.0, an invalid type mapping error has been detected and
							// processing is aborted.
							if self.options.processing_mode == ProcessingMode::JsonLd1_0
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
						}

						// If `value` contains the entry @reverse:
						if let Some(reverse_value) = value.reverse {
							// If `value` contains `@id` or `@nest`, entries, an invalid reverse
							// property error has been detected and processing is aborted.
							if value.id.is_some() || value.nest.is_some() {
								return Err(Error::InvalidReverseProperty);
							}

							// If the value associated with the @reverse entry is a string having
							// the form of a keyword, return; processors SHOULD generate a warning.
							if reverse_value.is_keyword_like() {
								env.warn(Warning::KeywordLikeValue(reverse_value.to_string()));
								return Ok(());
							}

							// Otherwise, set the IRI mapping of definition to the result of IRI
							// expanding the value associated with the @reverse entry, using
							// local context, and defined.
							// If the result does not have the form of an IRI or a blank node
							// identifier, an invalid IRI mapping error has been detected and
							// processing is aborted.
							match self
								.expand_iri_recursive(
									env,
									result,
									local_context,
									Nullable::Some(reverse_value.as_str().into()),
									false,
									true,
								)
								.await?
							{
								Term::Id(mapping) if mapping.is_valid() => {
									definition.value = Some(Term::Id(mapping))
								}
								_ => return Err(Error::InvalidIriMapping),
							}

							// If `value` contains an `@container` entry, set the `container`
							// mapping of `definition` to an array containing its value;
							// if its value is neither `@set`, nor `@index`, nor null, an
							// invalid reverse property error has been detected (reverse properties
							// only support set- and index-containers) and processing is aborted.
							if let Some(container_value) = value.container {
								if matches!(container_value, Container::Set | Container::Index) {
									definition.container = container_value
								} else {
									return Err(Error::InvalidReverseProperty);
								}
							}

							// Set the `reverse_property` flag of `definition` to `true`.
							definition.reverse_property = true;

							// Set the term definition of `term` in `active_context` to
							// `definition` and the value associated with `defined`'s entry `term`
							// to `true` and return.
							result.value.set_normal(key.to_owned(), Some(definition));
							result.defined.end(&term);
							return Ok(());
						}

						match value.id {
							// If `value` contains the entry `@id` and its value does not equal `term`:
							Some(id_value)
								if id_value.cast::<KeyOrKeywordRef>()
									!= Nullable::Some(key.into()) =>
							{
								match id_value {
									// If the `@id` entry of value is `null`, the term is not used for IRI
									// expansion, but is retained to be able to detect future redefinitions
									// of this term.
									Nullable::Null => (),
									Nullable::Some(id_value) => {
										// Otherwise:
										// If the value associated with the `@id` entry is not a
										// keyword, but has the form of a keyword, return;
										// processors SHOULD generate a warning.
										if id_value.is_keyword_like() && !id_value.is_keyword() {
											debug_assert!(
												Keyword::try_from(id_value.as_str()).is_err()
											);
											env.warn(Warning::KeywordLikeValue(
												id_value.to_string(),
											));
											return Ok(());
										}

										// Otherwise, set the IRI mapping of `definition` to the result
										// of IRI expanding the value associated with the `@id` entry,
										// using `local_context`, and `defined`.
										definition.value = match self
											.expand_iri_recursive(
												env,
												result,
												local_context,
												Nullable::Some(id_value.into()),
												false,
												true,
											)
											.await?
										{
											Term::Keyword(Keyword::Context) => {
												// if it equals `@context`, an invalid keyword alias error has
												// been detected and processing is aborted.
												return Err(Error::InvalidKeywordAlias);
											}
											Term::Id(prop) if !prop.is_valid() => {
												// If the resulting IRI mapping is neither a keyword,
												// nor an IRI, nor a blank node identifier, an
												// invalid IRI mapping error has been detected and processing
												// is aborted;
												return Err(Error::InvalidIriMapping);
											}
											value => Some(value),
										};

										// If `term` contains a colon (:) anywhere but as the first or
										// last character of `term`, or if it contains a slash (/)
										// anywhere:
										if contains_between_boundaries(key.as_str(), ':')
											|| key.as_str().contains('/')
										{
											// Set the value associated with `defined`'s `term` entry
											// to `true`.
											result.defined.end(&term);

											// If the result of IRI expanding `term` using
											// `local_context`, and `defined`, is not the same as the
											// IRI mapping of definition, an invalid IRI mapping error
											// has been detected and processing is aborted.
											let expanded_term = self
												.expand_iri_recursive(
													env,
													result,
													local_context,
													Nullable::Some((&term).into()),
													false,
													true,
												)
												.await?;
											if definition.value != Some(expanded_term) {
												return Err(Error::InvalidIriMapping);
											}
										}

										// If `term` contains neither a colon (:) nor a slash (/),
										// simple term is true, and if the IRI mapping of definition
										// is either an IRI ending with a gen-delim character,
										// or a blank node identifier, set the `prefix` flag in
										// `definition` to true.
										if !key.as_str().contains(':')
											&& !key.as_str().contains('/') && simple_term
											&& is_gen_delim_or_blank(
												definition.value.as_ref().unwrap(),
											) {
											definition.prefix = true;
										}
									}
								}
							}
							Some(Nullable::Some(IdRef::Keyword(Keyword::Type))) => {
								// Otherwise, if `term` is ``@type`, set the IRI mapping of definition to
								// `@type`.
								definition.value = Some(Term::Keyword(Keyword::Type))
							}
							_ => {
								// Otherwise if the `term` contains a colon (:) anywhere after the first
								// character.
								if let KeyOrKeyword::Key(term) = &term {
									if let Ok(compact_iri) = CompactIri::new(term.as_str()) {
										// If `term` is a compact IRI with a prefix that is an entry in local
										// context a dependency has been found.
										// Use this algorithm recursively passing `active_context`,
										// `local_context`, the prefix as term, and `defined`.
										Box::pin(self.for_recursive_definition().define(
											env,
											result,
											local_context,
											KeyOrKeywordRef::Key(compact_iri.prefix().into()),
											false,
										))
										.await?;

										// If `term`'s prefix has a term definition in `active_context`, set the
										// IRI mapping of `definition` to the result of concatenating the value
										// associated with the prefix's IRI mapping and the term's suffix.
										if let Some(prefix_definition) =
											result.value.get(compact_iri.prefix())
										{
											let mut result = String::new();

											if let Some(prefix_key) = prefix_definition.value() {
												if let Some(prefix_iri) = prefix_key.as_iri() {
													result = prefix_iri.to_owned().into_string()
												}
											}

											result.push_str(compact_iri.suffix());

											if let Ok(iri) = Iri::new(result.as_str()) {
												definition.value =
													Some(Term::Id(Id::iri(iri.to_owned())))
											} else {
												return Err(Error::InvalidIriMapping);
											}
										}
									}

									// not a compact IRI
									if definition.value.is_none() {
										if let Ok(blank_id) = BlankId::new(term.as_str()) {
											definition.value =
												Some(Term::Id(Id::blank(blank_id.to_owned())))
										} else if let Ok(iri_ref) = IriRef::new(term.as_str()) {
											match iri_ref.as_iri() {
												Some(iri) => {
													definition.value =
														Some(Term::Id(Id::iri(iri.to_owned())))
												}
												None => {
													if iri_ref.as_str().contains('/') {
														// Term is a relative IRI reference.
														// Set the IRI mapping of definition to the result of IRI expanding
														// term.
														match result.value.expand_iri_with(
															Nullable::Some(ExpandableRef::String(
																iri_ref.as_str(),
															)),
															false,
															true,
															|w| env.warn(w),
														) {
															Term::Id(Id::Valid(ValidId::Iri(
																id,
															))) => definition.value = Some(id.into()),
															// If the resulting IRI mapping is not an IRI, an invalid IRI mapping
															// error has been detected and processing is aborted.
															_ => {
																return Err(
																	Error::InvalidIriMapping,
																)
															}
														}
													}
												}
											}
										}

										// not a compact IRI, IRI, IRI reference or blank node id.
										if definition.value.is_none() {
											if let Some(context_vocabulary) =
												result.value.vocabulary()
											{
												// Otherwise, if `active_context` has a vocabulary mapping, the IRI mapping
												// of `definition` is set to the result of concatenating the value
												// associated with the vocabulary mapping and `term`.
												// If it does not have a vocabulary mapping, an invalid IRI mapping error
												// been detected and processing is aborted.
												if let Some(vocabulary_iri) =
													context_vocabulary.as_iri()
												{
													let mut result =
														vocabulary_iri.to_owned().into_string();
													result.push_str(key.as_str());
													if let Ok(iri) = Iri::new(result.as_str()) {
														definition.value =
															Some(Term::from(iri.to_owned()))
													} else {
														return Err(Error::InvalidIriMapping);
													}
												} else {
													return Err(Error::InvalidIriMapping);
												}
											} else {
												// If it does not have a vocabulary mapping, an invalid IRI mapping error
												// been detected and processing is aborted.
												return Err(Error::InvalidIriMapping);
											}
										}
									}
								}
							}
						}

						// If value contains the entry @container:
						if let Some(container_value) = value.container {
							// If the container value is @graph, @id, or @type, or is otherwise not a
							// string, generate an invalid container mapping error and abort processing
							// if processing mode is json-ld-1.0.
							if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
								match container_value {
									Container::Index
									| Container::Language
									| Container::List
									| Container::Set => (),
									_ => return Err(Error::InvalidContainerMapping),
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
							definition.container = container_value;

							// Set the container mapping of definition to container coercing to an
							// array, if necessary.
							// already done.

							// If the `container` mapping of definition includes `@type`:
							if definition.container.contains(ContainerItem::Type) {
								if let Some(typ) = &definition.typ {
									// If type mapping in definition is neither `@id` nor `@vocab`,
									// an invalid type mapping error has been detected and processing
									// is aborted.
									match typ {
										Type::Id | Type::Vocab => (),
										_ => return Err(Error::InvalidTypeMapping),
									}
								} else {
									// If type mapping in definition is undefined, set it to @id.
									definition.typ = Some(Type::Id)
								}
							}
						}

						// If value contains the entry @index:
						if let Some(index_value) = value.index {
							// If processing mode is json-ld-1.0 or container mapping does not include
							// `@index`, an invalid term definition has been detected and processing
							// is aborted.
							if !definition.container.contains(ContainerItem::Index)
								|| self.options.processing_mode == ProcessingMode::JsonLd1_0
							{
								return Err(Error::InvalidTermDefinition);
							}

							// Initialize `index` to the value associated with the `@index` entry,
							// which MUST be a string expanding to an IRI.
							// Otherwise, an invalid term definition has been detected and processing
							// is aborted.
							match result.value.expand_iri_with(
								Nullable::Some(index_value.as_str().into()),
								false,
								true,
								|w| env.warn(w),
							) {
								Term::Id(Id::Valid(ValidId::Iri(_))) => (),
								_ => return Err(Error::InvalidTermDefinition),
							}

							definition.index = Some(index_value.to_owned())
						}

						// If `value` contains the entry `@context`:
						if let Some(context) = value.context {
							// If processing mode is json-ld-1.0, an invalid term definition has been
							// detected and processing is aborted.
							if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
								return Err(Error::InvalidTermDefinition);
							}

							// Initialize `context` to the value associated with the @context entry,
							// which is treated as a local context.
							// done.

							// Invoke the Context Processing algorithm using the `active_context`,
							// `context` as local context, `base_url`, and `true` for override
							// protected.
							// If any error is detected, an invalid scoped context error has been
							// detected and processing is aborted.
							Box::pin(self.with_override().process(env, context)).await?;

							// Set the local context of definition to context, and base URL to base URL.
							definition.context = Some(Box::new(context.clone()));
							definition.base_url = self.base_url.map(ToOwned::to_owned);
						}

						// If `value` contains the entry `@language` and does not contain the entry
						// `@type`:
						if value.type_.is_none() {
							if let Some(language_value) = value.language {
								// Initialize `language` to the value associated with the `@language`
								// entry, which MUST be either null or a string.
								// If `language` is not well-formed according to section 2.2.9 of
								// [BCP47], processors SHOULD issue a warning.
								// Otherwise, an invalid language mapping error has been detected and
								// processing is aborted.
								// Set the `language` mapping of definition to `language`.
								definition.language =
									Some(language_value.map(LenientLangTag::to_owned));
							}

							// If `value` contains the entry `@direction` and does not contain the
							// entry `@type`:
							if let Some(direction_value) = value.direction {
								// Initialize `direction` to the value associated with the `@direction`
								// entry, which MUST be either null, "ltr", or "rtl".
								definition.direction = Some(direction_value);
							}
						}

						// If value contains the entry @nest:
						if let Some(nest_value) = value.nest {
							// If processing mode is json-ld-1.0, an invalid term definition has been
							// detected and processing is aborted.
							if self.options.processing_mode == ProcessingMode::JsonLd1_0 {
								return Err(Error::InvalidTermDefinition);
							}

							definition.nest = Some(nest_value.clone());
						}

						// If value contains the entry @prefix:
						if let Some(prefix_value) = value.prefix {
							// If processing mode is json-ld-1.0, or if `term` contains a colon (:) or
							// slash (/), an invalid term definition has been detected and processing
							// is aborted.
							if key.as_str().contains(':')
								|| key.as_str().contains('/')
								|| self.options.processing_mode == ProcessingMode::JsonLd1_0
							{
								return Err(Error::InvalidTermDefinition);
							}

							// Set the `prefix` flag to the value associated with the @prefix entry,
							// which MUST be a boolean.
							// Otherwise, an invalid @prefix value error has been detected and
							// processing is aborted.
							definition.prefix = prefix_value;

							// If the `prefix` flag of `definition` is set to `true`, and its IRI
							// mapping is a keyword, an invalid term definition has been detected and
							// processing is aborted.
							if definition.prefix && definition.value.as_ref().unwrap().is_keyword()
							{
								return Err(Error::InvalidTermDefinition);
							}
						}

						// If value contains any entry other than @id, @reverse, @container, @context,
						// @direction, @index, @language, @nest, @prefix, @protected, or @type, an
						// invalid term definition error has been detected and processing is aborted.
						if value.propagate.is_some() {
							return Err(Error::InvalidTermDefinition);
						}

						// If override protected is false and previous_definition exists and is protected;
						if !self.options.override_protected {
							if let Some(previous_definition) = previous_definition {
								if previous_definition.protected {
									// If `definition` is not the same as `previous_definition`
									// (other than the value of protected), a protected term
									// redefinition error has been detected, and processing is aborted.
									if definition.modulo_protected_field()
										!= previous_definition.modulo_protected_field()
									{
										return Err(Error::ProtectedTermRedefinition);
									}

									// Set `definition` to `previous definition` to retain the value of
									// protected.
									definition.protected = true;
								}
							}
						}

						// Set the term definition of `term` in `active_context` to `definition` and
						// set the value associated with `defined`'s entry term to true.
						result.value.set_normal(key.to_owned(), Some(definition));
					}
					_ => {
						// Otherwise, since keywords cannot be overridden, term MUST NOT be a keyword and
						// a keyword redefinition error has been detected and processing is aborted.
						return Err(Error::KeywordRedefinition);
					}
				}
			}

			result.defined.end(&term);
		}

		Ok(())
	}
}
