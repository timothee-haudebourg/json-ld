use crate::{
	expand_array, expand_iri, expand_literal, expand_node, expand_value, Error, Expanded,
	GivenLiteralValue, LiteralValue, Loader, Options, Warning,
};
use futures::future::{BoxFuture, FutureExt};
use iref::Iri;
use json_ld_context_processing::{ContextLoader, Process, ProcessingOptions};
use json_ld_core::{Context, Id, Indexed, Object, Reference, Term};
use json_ld_syntax::{object::Entry, Keyword, Nullable, Value};
use locspan::{At, Meta};
use mown::Mown;
use std::borrow::Cow;

pub(crate) struct ExpandedEntry<'a, T, C, M>(
	pub Meta<&'a str, &'a M>,
	pub Term<T>,
	pub &'a Meta<Value<C, M>, M>,
);

pub(crate) enum ActiveProperty<'a, M> {
	Some(Meta<&'a str, &'a M>),
	None,
}

impl<'a, M> ActiveProperty<'a, M> {
	pub fn is_some(&self) -> bool {
		matches!(self, Self::Some(_))
	}

	pub fn is_none(&self) -> bool {
		matches!(self, Self::None)
	}

	pub fn get_from<'c, T, C>(
		&self,
		context: &'c Context<T, C>,
	) -> Option<&'c json_ld_core::context::TermDefinition<T, C>> {
		match self {
			Self::Some(Meta(s, _)) => context.get(*s),
			Self::None => None,
		}
	}
}

impl<'a, M> Clone for ActiveProperty<'a, M> {
	fn clone(&self) -> Self {
		match self {
			Self::Some(m) => Self::Some(*m),
			Self::None => Self::None,
		}
	}
}

impl<'a, M> Copy for ActiveProperty<'a, M> {}

impl<'a, M> PartialEq<Keyword> for ActiveProperty<'a, M> {
	fn eq(&self, other: &Keyword) -> bool {
		match self {
			Self::Some(Meta(s, _)) => *s == other.into_str(),
			_ => false,
		}
	}
}

/// Result of the expansion of a single element in a JSON-LD document.
pub(crate) type ElementExpansionResult<T, M, L, W> = Result<(Expanded<T, M>, W), Meta<Error<L>, M>>;

/// Expand an element.
///
/// See <https://www.w3.org/TR/json-ld11-api/#expansion-algorithm>.
/// The default specified value for `ordered` and `from_map` is `false`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn expand_element<'a, T: Id, C: Process<T>, L: Loader + ContextLoader, W>(
	active_context: &'a Context<T, C>,
	active_property: ActiveProperty<'a, C::Metadata>,
	Meta(element, meta): &'a Meta<Value<C, C::Metadata>, C::Metadata>,
	base_url: Option<Iri<'a>>,
	loader: &'a mut L,
	options: Options,
	from_map: bool,
	mut warnings: W,
) -> BoxFuture<'a, ElementExpansionResult<T, C::Metadata, L, W>>
where
	T: Sync + Send,
	L: Sync + Send,
	<L as Loader>::Output: Into<Value<C, C::Metadata>>,
	<L as ContextLoader>::Output: Into<C>,
	W: 'a + Send + FnMut(Meta<Warning, C::Metadata>),
{
	async move {
		// If `element` is null, return null.
		if element.is_null() {
			return Ok((Expanded::Null, warnings));
		}

		let active_property_definition = active_property.get_from(active_context);

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

		match element {
			Value::Null => unreachable!(),
			Value::Array(element) => {
				expand_array(
					active_context,
					active_property,
					active_property_definition,
					element,
					base_url,
					loader,
					options,
					from_map,
					warnings,
				)
				.await
			}

			Value::Object(element) => {
				// let entries: Cow<[Entry<C, _>]> = if options.ordered {
				// 	Cow::Owned(element.entries().iter().cloned().collect())
				// } else {
				// 	Cow::Borrowed(element.entries().as_slice())
				// };

				// Preliminary key expansions.
				let mut preliminary_value_entry = None;
				let mut preliminary_id_entry = None;
				for Entry {
					key: Meta(key, key_metadata),
					value,
				} in element.entries()
				{
					match expand_iri(
						active_context,
						Meta(Nullable::Some(key.as_str().into()), key_metadata.clone()),
						false,
						true,
						&crate::ignore_warning,
					) {
						Term::Keyword(Keyword::Value) => {
							preliminary_value_entry = Some(value.clone())
						}
						Term::Keyword(Keyword::Id) => preliminary_id_entry = Some(value.clone()),
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
						&& preliminary_value_entry.is_none()
						&& !(element.len() == 1 && preliminary_id_entry.is_some())
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
							.await
							.map_err(Meta::cast)?, // .err_at(|| active_property.as_ref().map(Meta::metadata).cloned().unwrap_or_default())?
						                        // .into_inner(),
					);
				}

				// If `element` contains the entry `@context`, set `active_context` to the result
				// of the Context Processing algorithm, passing `active_context`, the value of the
				// `@context` entry as `local_context` and `base_url`.
				if let Some(Meta(local_context, _)) = element.context() {
					active_context = Mown::Owned(
						local_context
							.process_with(active_context.as_ref(), loader, base_url, options.into())
							.await
							.map_err(Meta::cast)?,
					);
				}

				let entries: Cow<[Entry<C, _>]> = if options.ordered {
					Cow::Owned(element.entries().iter().cloned().collect())
				} else {
					Cow::Borrowed(element.entries().as_slice())
				};

				let mut type_entries: Vec<&Entry<C, _>> = Vec::new();
				for entry @ Entry {
					key: Meta(key, key_metadata),
					..
				} in entries.iter()
				{
					let expanded_key = expand_iri(
						active_context.as_ref(),
						Meta(Nullable::Some(key.as_str().into()), key_metadata.clone()),
						false,
						true,
						&mut warnings,
					);

					if let Term::Keyword(Keyword::Type) = expanded_key {
						type_entries.push(entry);
					}
				}

				type_entries.sort_unstable_by_key(|entry| entry.key.value());

				// Initialize `type_scoped_context` to `active_context`.
				// This is used for expanding values that may be relevant to any previous
				// type-scoped context.
				let type_scoped_context = active_context.as_ref();
				let mut active_context = Mown::Borrowed(active_context.as_ref());

				// For each `key` and `value` in `element` ordered lexicographically by key where
				// key IRI expands to @type:
				for Entry { value, .. } in &type_entries {
					// Convert `value` into an array, if necessary.
					let value = Value::force_as_array(value);

					// For each `term` which is a value of `value` ordered lexicographically,
					let mut sorted_value = Vec::with_capacity(value.len());
					for Meta(term, meta) in value {
						if let Some(s) = term.as_string() {
							sorted_value.push(Meta(s, meta));
						}
					}

					sorted_value.sort_unstable_by_key(|s| *s.value());

					// if `term` is a string, and `term`'s term definition in `type_scoped_context`
					// has a `local_context`,
					for Meta(term, _) in sorted_value {
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
										.await
										.map_err(Meta::cast)?,
								);
							}
						}
					}
				}

				// Initialize `input_type` to expansion of the last value of the first entry in
				// `element` expanding to `@type` (if any), ordering entries lexicographically by
				// key.
				// Both the key and value of the matched entry are IRI expanded.
				let input_type = if let Some(Entry { value, .. }) = type_entries.first() {
					let value = Value::force_as_array(value);
					if let Some(Meta(input_type, input_metadata)) = value.last() {
						input_type.as_string().map(|input_type_str| {
							expand_iri(
								active_context.as_ref(),
								Meta(
									Nullable::Some(input_type_str.into()),
									input_metadata.clone(),
								),
								false,
								true,
								&mut warnings,
							)
						})
					} else {
						None
					}
				} else {
					None
				};

				let mut expanded_entries: Vec<ExpandedEntry<T, C, C::Metadata>> =
					Vec::with_capacity(element.len());
				let mut list_entry = None;
				let mut set_entry = None;
				let mut value_entry = None;
				for Entry {
					key: Meta(key, key_metadata),
					value,
				} in entries.iter()
				{
					if key.is_empty() {
						warnings(Meta::new(Warning::EmptyTerm, key_metadata.clone()));
					}

					let expanded_key = expand_iri(
						active_context.as_ref(),
						Meta(Nullable::Some(key.as_str().into()), key_metadata.clone()),
						false,
						true,
						&mut warnings,
					);

					match &expanded_key {
						Term::Keyword(Keyword::Value) => value_entry = Some(value.clone()),
						Term::Keyword(Keyword::List) => {
							if active_property.is_some() && active_property != Keyword::Graph {
								list_entry = Some(value.clone())
							}
						}
						Term::Keyword(Keyword::Set) => set_entry = Some(value.clone()),
						Term::Ref(Reference::Blank(id)) => {
							warnings(Meta::new(
								Warning::BlankNodeIdProperty(id.clone()),
								key_metadata.clone(),
							));
						}
						_ => (),
					}

					expanded_entries.push(ExpandedEntry(
						Meta(key, key_metadata),
						expanded_key,
						value,
					))
				}

				if let Some(list_entry) = list_entry {
					// List objects.
					let mut index = None;
					for ExpandedEntry(Meta(_, key_metadata), expanded_key, value) in
						expanded_entries
					{
						match expanded_key {
							Term::Keyword(Keyword::Index) => match value.as_string() {
								Some(value) => index = Some(value.to_string()),
								None => {
									return Err(
										Error::InvalidIndexValue.at(value.metadata().clone())
									)
								}
							},
							Term::Keyword(Keyword::List) => (),
							_ => return Err(Error::InvalidSetOrListObject.at(key_metadata.clone())),
						}
					}

					// Initialize expanded value to the result of using this algorithm
					// recursively passing active context, active property, value for element,
					// base URL, and the ordered flags, ensuring that the
					// result is an array..
					let mut result = Vec::new();
					let list_entry = Value::force_as_array(&list_entry);
					for item in list_entry {
						let (e, w) = expand_element(
							active_context.as_ref(),
							active_property,
							item,
							base_url,
							loader,
							options,
							false,
							warnings,
						)
						.await?;
						warnings = w;
						result.extend(e)
					}

					Ok((
						Expanded::Object(Indexed::new(Object::List(result), index)),
						warnings,
					))
				} else if let Some(set_entry) = set_entry {
					// Set objects.
					for ExpandedEntry(Meta(_, key_metadata), expanded_key, _) in expanded_entries {
						match expanded_key {
							Term::Keyword(Keyword::Index) => {
								// having an `@index` here is tolerated,
								// but is ignored.
							}
							Term::Keyword(Keyword::Set) => (),
							_ => return Err(Error::InvalidSetOrListObject.at(key_metadata.clone())),
						}
					}

					// set expanded value to the result of using this algorithm recursively,
					// passing active context, active property, value for element, base URL,
					// and ordered flags.
					expand_element(
						active_context.as_ref(),
						active_property,
						&set_entry,
						base_url,
						loader,
						options,
						false,
						warnings,
					)
					.await
				} else if let Some(value_entry) = value_entry {
					// Value objects.
					let (expanded_value, warnings) = expand_value(
						input_type,
						type_scoped_context,
						expanded_entries,
						&value_entry,
						warnings,
					)
					.map_err(Meta::cast)?;

					if let Some(value) = expanded_value {
						Ok((Expanded::Object(value), warnings))
					} else {
						Ok((Expanded::Null, warnings))
					}
				} else {
					// Node objects.
					let (e, warnings) = expand_node(
						active_context.as_ref(),
						type_scoped_context,
						active_property,
						expanded_entries,
						base_url,
						loader,
						options,
						warnings,
					)
					.await?;
					if let Some(result) = e {
						Ok((result.cast::<Object<T, C::Metadata>>().into(), warnings))
					} else {
						Ok((Expanded::Null, warnings))
					}
				}
			}

			_ => {
				// Literals.

				// If element is a scalar (bool, int, string, null),
				// If `active_property` is `null` or `@graph`, drop the free-floating scalar by
				// returning null.
				if active_property.is_none() || active_property == Keyword::Graph {
					return Ok((Expanded::Null, warnings));
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of the
				// Context Processing algorithm, passing `active_context`, `property_scoped_context` as
				// local context, and `base_url` from the term definition for `active_property` in
				// `active context`.
				let active_context = if let Some(property_scoped_context) = property_scoped_context
				{
					// FIXME it is unclear what we should use as `base_url` if there is no term definition for `active_context`.
					let base_url =
						active_property
							.get_from(active_context)
							.and_then(|definition| {
								definition
									.base_url
									.as_ref()
									.map(|base_url| base_url.as_iri())
							});

					let result = property_scoped_context
						.process_with(active_context, loader, base_url, options.into())
						.await
						.map_err(Meta::cast)?;
					Mown::Owned(result)
				} else {
					Mown::Borrowed(active_context)
				};

				// Return the result of the Value Expansion algorithm, passing the `active_context`,
				// `active_property`, and `element` as value.
				Ok((
					Expanded::Object(
						expand_literal(
							active_context.as_ref(),
							active_property,
							Meta(LiteralValue::Given(GivenLiteralValue::new(element)), meta),
							&mut warnings,
						)
						.map_err(Meta::cast)?,
					),
					warnings,
				))
			}
		}
	}
	.boxed()
}
