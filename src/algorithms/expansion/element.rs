use std::borrow::Cow;

use json_syntax::{object::Entry, Value};
use mown::Mown;

use crate::{
	algorithms::{
		context_processing::ContextProcessingOptions, Error, ProcessingEnvironment,
		ProcessingEnvironmentRefMut, Warning,
	},
	object::ListObject,
	syntax::{Context, Keyword},
	Id, Indexed, Nullable, Object, Term, ValidId,
};

use super::{ExpandableLiteralValue, Expanded, Expander};

pub struct ExpandedEntry<'a>(pub &'a str, pub Term, pub &'a Value);

impl<'a> Expander<'a> {
	/// Expand an element.
	///
	/// See <https://www.w3.org/TR/json-ld11-api/#expansion-algorithm>.
	/// The default specified value for `ordered` and `from_map` is `false`.
	#[allow(clippy::too_many_arguments)]
	pub async fn expand_element(
		&self,
		env: &mut impl ProcessingEnvironment,
		element: &Value,
		from_map: bool,
	) -> Result<Expanded, Error> {
		// If `element` is null, return null.
		if element.is_null() {
			return Ok(Expanded::Null);
		}

		let active_property_definition = self.active_property_definition();

		// If `active_property` has a term definition in `active_context` with a local context,
		// initialize property-scoped context to that local context.
		let mut property_scoped_base_url = None;
		let property_scoped_context = if let Some(definition) = active_property_definition {
			if let Some(base_url) = definition.base_url() {
				property_scoped_base_url = Some(base_url);
			}

			definition.context()
		} else {
			None
		};

		match element {
			Value::Null => unreachable!(),
			Value::Array(element) => {
				self.expand_array(
					env,
					// active_context,
					// active_property,
					active_property_definition,
					element,
					// base_url,
					// options,
					from_map,
				)
				.await
			}

			Value::Object(element) => {
				// let entries: Cow<[Entry<_, C>]> = if options.ordered {
				// 	Cow::Owned(element.entries().iter().cloned().collect())
				// } else {
				// 	Cow::Borrowed(element.entries().as_slice())
				// };

				// Preliminary key expansions.
				let mut preliminary_value_entry = None;
				let mut preliminary_id_entry = None;
				for (key, value) in element.entries() {
					match self.active_context.expand_iri(
						// &mut env,
						// active_context,
						Nullable::Some(key.as_str().into()),
						false,
						true,
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
				let mut active_context = Mown::Borrowed(self.active_context);
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
					let options: ContextProcessingOptions = self.options.into();
					active_context = Mown::Owned(
						property_scoped_context
							.process_with(
								ProcessingEnvironmentRefMut(&mut *env),
								property_scoped_base_url.as_deref(),
								active_context.as_ref(),
								options.with_override(),
							)
							.await?,
					);
				}

				// If `element` contains the entry `@context`, set `active_context` to the result
				// of the Context Processing algorithm, passing `active_context`, the value of the
				// `@context` entry as `local_context` and `base_url`.
				if let Some(local_context) = element
					.get_unique("@context")
					.map_err(Error::duplicate_key_ref)?
				{
					// use json_ld_syntax::TryFromJson;
					// let local_context =
					// 	json_ld_syntax::context::Context::try_from_json(local_context.clone())?;
					let local_context: Context = json_syntax::from_value(local_context.clone())
						.map_err(Error::ContextSyntax)?;

					active_context = Mown::Owned(
						local_context
							.process_with(
								ProcessingEnvironmentRefMut(&mut *env),
								self.base_url,
								&active_context,
								self.options.into(),
							)
							.await?,
					);
				}

				let entries: Cow<[Entry]> = if self.options.ordered {
					Cow::Owned(element.entries().to_vec())
				} else {
					Cow::Borrowed(element.entries())
				};

				let mut type_entries: Vec<&Entry> = Vec::new();
				for entry @ (key, _) in entries.iter() {
					let expanded_key = active_context.expand_iri(
						// &mut env,
						// active_context.as_ref(),
						Nullable::Some(key.as_str().into()),
						false,
						true,
					);

					if let Term::Keyword(Keyword::Type) = expanded_key {
						type_entries.push(entry);
					}
				}

				type_entries.sort_unstable_by_key(|(_, key)| key);

				// Initialize `type_scoped_context` to `active_context`.
				// This is used for expanding values that may be relevant to any previous
				// type-scoped context.
				let type_scoped_context = active_context.as_ref();
				let mut active_context = Mown::Borrowed(active_context.as_ref());

				// For each `key` and `value` in `element` ordered lexicographically by key where
				// key IRI expands to @type:
				for (_, value) in &type_entries {
					// Convert `value` into an array, if necessary.
					let value = Value::force_as_array(value);

					// For each `term` which is a value of `value` ordered lexicographically,
					let mut sorted_value = Vec::with_capacity(value.len());
					for term in value {
						if let Some(s) = term.as_string() {
							sorted_value.push(s);
						}
					}

					sorted_value.sort_unstable();

					// if `term` is a string, and `term`'s term definition in `type_scoped_context`
					// has a `local_context`,
					for term in sorted_value {
						if let Some(term_definition) = type_scoped_context.get(term) {
							if let Some(local_context) = term_definition.context() {
								// set `active_context` to the result of
								// Context Processing algorithm, passing `active_context`, the value of the
								// `term`'s local context as `local_context`, `base_url` from the term
								// definition for value in `active_context`, and `false` for `propagate`.
								let options: ContextProcessingOptions = self.options.into();
								active_context = Mown::Owned(
									local_context
										.process_with(
											ProcessingEnvironmentRefMut(&mut *env),
											term_definition.base_url(),
											active_context.as_ref(),
											options.without_propagation(),
										)
										.await?,
								);
							}
						}
					}
				}

				// Initialize `input_type` to expansion of the last value of the first entry in
				// `element` expanding to `@type` (if any), ordering entries lexicographically by
				// key.
				// Both the key and value of the matched entry are IRI expanded.
				let input_type = if let Some((_, value)) = type_entries.first() {
					let value = Value::force_as_array(value);
					if let Some(input_type) = value.last() {
						input_type.as_string().map(|input_type_str| {
							active_context.expand_iri(
								// &mut env,
								// active_context.as_ref(),
								Nullable::Some(input_type_str.into()),
								false,
								true,
							)
						})
					} else {
						None
					}
				} else {
					None
				};

				let mut expanded_entries: Vec<ExpandedEntry> = Vec::with_capacity(element.len());
				let mut list_entry = None;
				let mut set_entry = None;
				let mut value_entry = None;
				for (key, value) in entries.iter() {
					if key.is_empty() {
						env.warn(Warning::EmptyTerm);
					}

					let expanded_key = active_context.expand_iri(
						// &mut env,
						// active_context.as_ref(),
						Nullable::Some(key.as_str().into()),
						false,
						true,
					);

					match &expanded_key {
						Term::Keyword(Keyword::Value) => value_entry = Some(value.clone()),
						Term::Keyword(Keyword::List) => {
							if self.active_property.is_some_and(|p| p != Keyword::Graph) {
								list_entry = Some(value.clone())
							}
						}
						Term::Keyword(Keyword::Set) => set_entry = Some(value.clone()),
						Term::Id(Id::Valid(ValidId::BlankId(id))) => {
							env.warn(Warning::BlankNodeIdProperty(id.clone()));
						}
						_ => (),
					}

					expanded_entries.push(ExpandedEntry(key, expanded_key, value))
				}

				if let Some(list_entry) = list_entry {
					// List objects.
					let mut index = None;
					for ExpandedEntry(_, expanded_key, value) in expanded_entries {
						match expanded_key {
							Term::Keyword(Keyword::Index) => match value.as_string() {
								Some(value) => index = Some(value.to_string()),
								None => return Err(Error::InvalidIndexValue),
							},
							Term::Keyword(Keyword::List) => (),
							_ => return Err(Error::InvalidSetOrListObject),
						}
					}

					// Initialize expanded value to the result of using this algorithm
					// recursively passing active context, active property, value for element,
					// base URL, and the ordered flags, ensuring that the
					// result is an array..
					let mut result = Vec::new();
					let list_entry = Value::force_as_array(&list_entry);
					for item in list_entry {
						let e = Box::pin(self.with_active_context(&active_context).expand_element(
							env,
							// Environment {
							// 	vocabulary: env.vocabulary,
							// 	loader: env.loader,
							// 	warnings: env.warnings,
							// },
							// active_context.as_ref(),
							// active_property,
							item, false,
						))
						.await?;
						result.extend(e)
					}

					Ok(Expanded::Object(Indexed::new(
						Object::List(ListObject::new(result)),
						index,
					)))
				} else if let Some(set_entry) = set_entry {
					// Set objects.
					for ExpandedEntry(_, expanded_key, _) in expanded_entries {
						match expanded_key {
							Term::Keyword(Keyword::Index) => {
								// having an `@index` here is tolerated,
								// but is ignored.
							}
							Term::Keyword(Keyword::Set) => (),
							_ => return Err(Error::InvalidSetOrListObject),
						}
					}

					// set expanded value to the result of using this algorithm recursively,
					// passing active context, active property, value for element, base URL,
					// and ordered flags.
					Box::pin(self.with_active_context(&active_context).expand_element(
						env,
						// env,
						// active_context.as_ref(),
						// active_property,
						&set_entry, // base_url,
						// options,
						false,
					))
					.await
				} else if let Some(value_entry) = value_entry {
					// Value objects.
					let expanded_value = self.expand_value(
						|w| env.warn(w),
						input_type,
						type_scoped_context,
						expanded_entries,
						&value_entry,
					)?;

					if let Some(value) = expanded_value {
						Ok(Expanded::Object(value))
					} else {
						Ok(Expanded::Null)
					}
				} else {
					// Node objects.
					let e = self
						.with_active_context(&active_context)
						.expand_node(
							env,
							// env,
							// active_context.as_ref(),
							type_scoped_context,
							// active_property,
							expanded_entries,
							// base_url,
							// options,
						)
						.await?;
					if let Some(result) = e {
						Ok(Expanded::Object(result.cast::<Object>()))
					} else {
						Ok(Expanded::Null)
					}
				}
			}

			_ => {
				// Literals.

				// If element is a scalar (bool, int, string, null),
				// If `active_property` is `null` or `@graph`, drop the free-floating scalar by
				// returning null.
				if self.active_property.is_none_or(|p| p == Keyword::Graph) {
					return Ok(Expanded::Null);
				}

				// If `property_scoped_context` is defined, set `active_context` to the result of the
				// Context Processing algorithm, passing `active_context`, `property_scoped_context` as
				// local context, and `base_url` from the term definition for `active_property` in
				// `active context`.
				let active_context = if let Some(property_scoped_context) = property_scoped_context
				{
					// FIXME it is unclear what we should use as `base_url` if there is no term definition for `active_context`.
					let base_url = self
						.active_property_definition()
						.and_then(|definition| definition.base_url().map(ToOwned::to_owned));

					let result = property_scoped_context
						.process_with(
							ProcessingEnvironmentRefMut(env),
							base_url.as_deref(),
							self.active_context,
							self.options.into(),
						)
						.await?;
					Mown::Owned(result)
				} else {
					Mown::Borrowed(self.active_context)
				};

				// Return the result of the Value Expansion algorithm, passing the `active_context`,
				// `active_property`, and `element` as value.
				Ok(Expanded::Object(
					self.with_active_context(&active_context).expand_literal(
						|w| env.warn(w),
						// env,
						// active_context.as_ref(),
						// active_property,
						ExpandableLiteralValue::new(element),
					)?,
				))
			}
		}
	}
}
