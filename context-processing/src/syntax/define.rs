use super::{expand_iri_with, expand_iri_simple, Merged};
use crate::{Error, Loader, LocWarning, Process, ProcessingOptions, ProcessingStack, Warning};
use futures::future::{BoxFuture, FutureExt};
use iref::Iri;
use json_ld_core::{context::TermDefinition, Context, Id, ProcessingMode, Reference, Term, Type};
use json_ld_syntax::{
	context::{
		AnyContextEntry,
		EntryRef,
		ExpandedTermDefinitionRef,
		IdRef,
		KeyOrKeyword,
		KeyOrKeywordRef,
		KeyRef,
		Key
	},
	ExpandableRef,
	Container, ContainerType, Keyword, Nullable,
	LenientLanguageTag,
};
use locspan::{At, Loc, Location, BorrowStripped};
use std::collections::HashMap;

fn is_gen_delim(c: char) -> bool {
	matches!(c, ':' | '/' | '?' | '#' | '[' | ']' | '@')
}

// Checks if the input term is an IRI ending with a gen-delim character, or a blank node identifier.
fn is_gen_delim_or_blank<T: Id>(t: &Term<T>) -> bool {
	match t {
		Term::Ref(Reference::Blank(_)) => true,
		Term::Ref(Reference::Id(id)) => {
			if let Some(c) = id.as_iri().as_str().chars().last() {
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

pub struct DefinedTerms<C: AnyContextEntry>(HashMap<KeyOrKeyword, DefinedTerm<C::Source, C::Span>>);

impl<C: AnyContextEntry> Default for DefinedTerms<C> {
	fn default() -> Self {
		Self(HashMap::new())
	}
}

impl<C: AnyContextEntry> DefinedTerms<C> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn begin(
		&mut self,
		key: &KeyOrKeyword,
		loc: &Location<C::Source, C::Span>,
	) -> Result<bool, Error>
	where
		C::Source: Clone,
		C::Span: Clone,
	{
		match self.0.get(key) {
			Some(d) => {
				if d.pending {
					Err(Error::CyclicIriMapping)
				} else {
					Ok(false)
				}
			}
			None => {
				self.0.insert(
					key.clone(),
					DefinedTerm {
						pending: true,
						location: loc.clone(),
					},
				);

				Ok(true)
			}
		}
	}

	pub fn end(&mut self, key: &KeyOrKeyword) {
		self.0.get_mut(&key).unwrap().pending = false
	}
}

pub struct DefinedTerm<S, P> {
	pending: bool,
	location: Location<S, P>,
}

/// Follows the `https://www.w3.org/TR/json-ld11-api/#create-term-definition` algorithm.
/// Default value for `base_url` is `None`. Default values for `protected` and `override_protected` are `false`.
pub fn define<
	'a,
	T: Id + Send + Sync,
	C: Process<T>
		+ AnyContextEntry<Source = <C as Process<T>>::Source, Span = <C as Process<T>>::Span>,
	L: Loader + Send + Sync,
>(
	active_context: &'a mut Context<T, C>,
	local_context: &'a Merged<'a, C>,
	Loc(term, loc): Loc<KeyOrKeywordRef, <C as Process<T>>::Source, <C as Process<T>>::Span>,
	defined: &'a mut DefinedTerms<C>,
	remote_contexts: ProcessingStack,
	loader: &'a mut L,
	base_url: Option<Iri<'a>>,
	protected: bool,
	options: ProcessingOptions,
	warnings: &'a mut Vec<LocWarning<T, C>>,
) -> BoxFuture<'a, Result<(), Error>>
where
	L::Output: Into<C>
{
	let term = term.to_owned();
	async move {
		if defined.begin(&term, &loc)? {
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
					EntryRef::Type(_) => {
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
						// Checked during parsing.
					}
					EntryRef::Definition(d) => {
						let key = term.as_key().unwrap();
						// Initialize `previous_definition` to any existing term definition for `term` in
						// `active_context`, removing that term definition from active context.
						let previous_definition = active_context.set(key.clone(), None);

						let simple_term = !d
							.definition
							.value()
							.as_ref()
							.map(|d| d.is_expanded())
							.unwrap_or(false);
						let value: ExpandedTermDefinitionRef<C> = d.definition.into();

						// Create a new term definition, `definition`, initializing `prefix` flag to
						// `false`, `protected` to `protected`, and `reverse_property` to `false`.
						let mut definition = TermDefinition::<T, C> {
							protected,
							..Default::default()
						};

						// If the @protected entry in value is true set the protected flag in
						// definition to true.
						if let Some(protected) = value.protected {
							// If processing mode is json-ld-1.0, an invalid term definition has
							// been detected and processing is aborted.
							if options.processing_mode == ProcessingMode::JsonLd1_0 {
								return Err(Error::InvalidTermDefinition);
							}

							definition.protected = *protected.value();
						}

						// If value contains the entry @type:
						if let Some(Loc(type_, type_loc)) = &value.type_ {
							// Set `typ` to the result of IRI expanding type, using local context,
							// and defined.
							let typ = expand_iri_with(
								active_context,
								Loc(type_.cast(), type_loc.clone()),
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
						}

						// If `value` contains the entry @reverse:
						if let Some(Loc(reverse_value, reverse_loc)) = value.reverse {
							// If `value` contains `@id` or `@nest`, entries, an invalid reverse
							// property error has been detected and processing is aborted.
							if value.id.is_some() || value.nest.is_some() {
								return Err(Error::InvalidReverseProperty);
							}

							// If the value associated with the @reverse entry is a string having
							// the form of a keyword, return; processors SHOULD generate a warning.
							if reverse_value.is_keyword_like() {
								warnings.push(
									Warning::KeywordLikeValue(reverse_value.to_string())
										.at(reverse_loc),
								);
								return Ok(());
							}

							// Otherwise, set the IRI mapping of definition to the result of IRI
							// expanding the value associated with the @reverse entry, using
							// local context, and defined.
							// If the result does not have the form of an IRI or a blank node
							// identifier, an invalid IRI mapping error has been detected and
							// processing is aborted.
							match expand_iri_with(
								active_context,
								Loc(
									Nullable::Some(ExpandableRef::Key(reverse_value)),
									reverse_loc,
								),
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
							if let Some(Loc(container_value, _container_loc)) = value.container {
								match container_value {
									Nullable::Null => (),
									Nullable::Some(container_value) => {
										if matches!(
											container_value,
											Container::Set | Container::Index
										) {
											definition.container = container_value
										} else {
											return Err(Error::InvalidReverseProperty.into());
										}
									}
								};
							}

							// Set the `reverse_property` flag of `definition` to `true`.
							definition.reverse_property = true;

							// Set the term definition of `term` in `active_context` to
							// `definition` and the value associated with `defined`'s entry `term`
							// to `true` and return.
							active_context.set(key.to_owned(), Some(definition));
							defined.end(&term);
							return Ok(());
						}

						// If `value` contains the entry `@id` and its value does not equal `term`:
						match value.id {
							Some(Loc(id_value, id_loc))
								if id_value.cast::<KeyOrKeywordRef>() != Nullable::Some(key.into()) =>
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
											warnings.push(
												Warning::KeywordLikeValue(id_value.to_string())
													.at(id_loc),
											);
											return Ok(());
										}

										// Otherwise, set the IRI mapping of `definition` to the result
										// of IRI expanding the value associated with the `@id` entry,
										// using `local_context`, and `defined`.
										definition.value = match expand_iri_with(
											active_context,
											Loc(Nullable::Some(id_value.into()), id_loc),
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
												return Err(Error::InvalidKeywordAlias);
											}
											Term::Ref(prop) if !prop.is_valid() => {
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
											defined.end(&term);

											// If the result of IRI expanding `term` using
											// `local_context`, and `defined`, is not the same as the
											// IRI mapping of definition, an invalid IRI mapping error
											// has been detected and processing is aborted.
											let expanded_term = expand_iri_with(
												active_context,
												Loc(Nullable::Some((&term).into()), loc.clone()),
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
												return Err(Error::InvalidIriMapping.into());
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
							Some(Loc(Nullable::Some(IdRef::CompactIri(compact_iri)), id_loc)) => {
								// Otherwise if the `term` contains a colon (:) anywhere after the first
								// character.

								// If `term` is a compact IRI with a prefix that is an entry in local
								// context a dependency has been found.
								// Use this algorithm recursively passing `active_context`,
								// `local_context`, the prefix as term, and `defined`.
								define(
									active_context,
									local_context,
									Loc(KeyOrKeywordRef::Key(KeyRef::Term(compact_iri.prefix())), id_loc),
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
								let prefix = Key::Term(compact_iri.prefix().to_string());
								if let Some(prefix_definition) = active_context.get(&prefix) {
									let mut result = String::new();

									if let Some(prefix_key) = &prefix_definition.value {
										if let Some(prefix_iri) = prefix_key.as_iri() {
											result = prefix_iri.as_str().to_string()
										}
									}

									result.push_str(compact_iri.suffix());

									if let Ok(iri) = Iri::new(result.as_str()) {
										definition.value = Some(Term::Ref(Reference::Id(T::from_iri(iri))))
									} else {
										return Err(Error::InvalidIriMapping.into());
									}
								} else if let Ok(iri) = Iri::new(compact_iri.as_str()) {
									definition.value = Some(Term::Ref(Reference::Id(T::from_iri(iri))))
								} else {
									return Err(Error::InvalidIriMapping.into());
								}
							}
							Some(Loc(Nullable::Some(IdRef::Blank(id)), _id_loc)) => {
								definition.value = Some(Term::Ref(Reference::Blank(id.to_owned())))
							}
							Some(Loc(Nullable::Some(IdRef::Iri(iri)), _id_loc)) => {
								definition.value = Some(Term::Ref(Reference::Id(T::from_iri(iri))))
							}
							Some(Loc(Nullable::Some(IdRef::Keyword(Keyword::Type)), _id_loc)) => {
								// Otherwise, if `term` is ``@type`, set the IRI mapping of definition to
								// `@type`.
								definition.value = Some(Term::Keyword(Keyword::Type))
							}
							_ => {
								if let Some(vocabulary) = active_context.vocabulary() {
									// Otherwise, if `active_context` has a vocabulary mapping, the IRI mapping
									// of `definition` is set to the result of concatenating the value
									// associated with the vocabulary mapping and `term`.
									// If it does not have a vocabulary mapping, an invalid IRI mapping error
									// been detected and processing is aborted.
									if let Some(vocabulary_iri) = vocabulary.as_iri() {
										let mut result = vocabulary_iri.as_str().to_string();
										result.push_str(key.as_str());
										if let Ok(iri) = Iri::new(result.as_str()) {
											definition.value =
												Some(Term::<T>::from(T::from_iri(iri)))
										} else {
											return Err(Error::InvalidIriMapping.into());
										}
									} else {
										return Err(Error::InvalidIriMapping.into());
									}
								} else {
									// If it does not have a vocabulary mapping, an invalid IRI mapping error
									// been detected and processing is aborted.
									return Err(Error::InvalidIriMapping.into());
								}
							}
						}

						// If value contains the entry @container:
						if let Some(Loc(container_value, _container_loc)) = value.container {
							// If the container value is @graph, @id, or @type, or is otherwise not a
							// string, generate an invalid container mapping error and abort processing
							// if processing mode is json-ld-1.0.
							if options.processing_mode == ProcessingMode::JsonLd1_0 {
								match container_value {
									Nullable::Some(Container::Graph) | Nullable::Some(Container::Id) | Nullable::Some(Container::Type) => {
										return Err(Error::InvalidContainerMapping)
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
							definition.container = container_value.unwrap_or_default();

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
										_ => return Err(Error::InvalidTypeMapping.into()),
									}
								} else {
									// If type mapping in definition is undefined, set it to @id.
									definition.typ = Some(Type::Id)
								}
							}
						}

						// If value contains the entry @index:
						if let Some(Loc(index_value, index_loc)) = value.index {
							// If processing mode is json-ld-1.0 or container mapping does not include
							// `@index`, an invalid term definition has been detected and processing
							// is aborted.
							if !definition.container.contains(ContainerType::Index)
								|| options.processing_mode == ProcessingMode::JsonLd1_0
							{
								return Err(Error::InvalidTermDefinition.into());
							}

							// Initialize `index` to the value associated with the `@index` entry,
							// which MUST be a string expanding to an IRI.
							// Otherwise, an invalid term definition has been detected and processing
							// is aborted.
							match expand_iri_simple(
								active_context,
								Loc(Nullable::Some(ExpandableRef::Key(index_value.into())), index_loc),
								false,
								true,
								warnings,
							) {
								Term::Ref(Reference::Id(_)) => (),
								_ => return Err(Error::InvalidTermDefinition.into()),
							}

							definition.index = Some(index_value.to_owned())
						}

						// If `value` contains the entry `@context`:
						if let Some(Loc(context, _context_loc)) = value.context {
							// If processing mode is json-ld-1.0, an invalid term definition has been
							// detected and processing is aborted.
							if options.processing_mode == ProcessingMode::JsonLd1_0 {
								return Err(Error::InvalidTermDefinition.into());
							}

							// Initialize `context` to the value associated with the @context entry,
							// which is treated as a local context.
							// done.

							// Invoke the Context Processing algorithm using the `active_context`,
							// `context` as local context, `base_url`, and `true` for override
							// protected.
							// If any error is detected, an invalid scoped context error has been
							// detected and processing is aborted.
							super::process_context(
								active_context,
								context,
								remote_contexts.clone(),
								loader,
								base_url,
								options.with_override(),
								warnings,
							)
							.await
							.map_err(|_| Error::from(Error::InvalidScopedContext))?;

							// Set the local context of definition to context, and base URL to base URL.
							definition.context = Some(context.clone());
							definition.base_url = base_url.as_ref().map(|url| url.into());
						}

						// If `value` contains the entry `@language` and does not contain the entry
						// `@type`:
						if value.type_.is_none() {
							if let Some(Loc(language_value, _language_loc)) = value.language {
								// Initialize `language` to the value associated with the `@language`
								// entry, which MUST be either null or a string.
								// If `language` is not well-formed according to section 2.2.9 of
								// [BCP47], processors SHOULD issue a warning.
								// Otherwise, an invalid language mapping error has been detected and
								// processing is aborted.
								// Set the `language` mapping of definition to `language`.
								definition.language = Some(language_value.map(LenientLanguageTag::to_owned));
							}

							// If `value` contains the entry `@direction` and does not contain the
							// entry `@type`:
							if let Some(Loc(direction_value, _direction_loc)) = value.direction {
								// Initialize `direction` to the value associated with the `@direction`
								// entry, which MUST be either null, "ltr", or "rtl".
								definition.direction = Some(direction_value);
							}
						}

						// If value contains the entry @nest:
						if let Some(Loc(nest_value, _nest_loc)) = value.nest {
							// If processing mode is json-ld-1.0, an invalid term definition has been
							// detected and processing is aborted.
							if options.processing_mode == ProcessingMode::JsonLd1_0 {
								return Err(Error::InvalidTermDefinition.into());
							}

							definition.nest = Some(nest_value.to_owned());
						}

						// If value contains the entry @prefix:
						if let Some(Loc(prefix_value, _prefix_loc)) = value.prefix {
							// If processing mode is json-ld-1.0, or if `term` contains a colon (:) or
							// slash (/), an invalid term definition has been detected and processing
							// is aborted.
							if key.as_str().contains(':')
								|| key.as_str().contains('/') || options.processing_mode
								== ProcessingMode::JsonLd1_0
							{
								return Err(Error::InvalidTermDefinition.into());
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
							return Err(Error::InvalidTermDefinition)
						}

						// If override protected is false and previous_definition exists and is protected;
						if !options.override_protected {
							if let Some(previous_definition) = previous_definition {
								if previous_definition.protected {
									// If `definition` is not the same as `previous_definition`
									// (other than the value of protected), a protected term
									// redefinition error has been detected, and processing is aborted.
									if definition.stripped() != previous_definition.stripped() {
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
						active_context.set(key.to_owned(), Some(definition));
					}
					_ => {
						// Otherwise, since keywords cannot be overridden, term MUST NOT be a keyword and
						// a keyword redefinition error has been detected and processing is aborted.
						return Err(Error::KeywordRedefinition);
					}
				}
			}

			defined.end(&term);
		}

		Ok(())
	}
	.boxed()
}
