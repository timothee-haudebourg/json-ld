use std::collections::HashSet;
use std::convert::TryFrom;
use std::borrow::Borrow;
use futures::future::{BoxFuture, FutureExt};
use mown::Mown;
// use mapped_mut::MappedMut;
use json::JsonValue;
use crate::{
	Id,
	Context,
	ContextMut,
	Indexed,
	object,
	Object,
	Value,
	Node,
	Reference,
	Lenient,
	Nullable,
	Error,
	ErrorCode,
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
			compact_to_relative: true,
			compact_arrays: true,
			ordered: false
		}
	}
}

pub trait Compact<T: Id> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send;

	fn compact<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, loader: &'a mut L) -> BoxFuture<'a, Result<JsonValue, Error>> where Self: Sync, T: 'a + Sync + Send, C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			let inverse_context = active_context.invert();
			self.compact_with(active_context, active_context, &inverse_context, None, loader, Options::default()).await
		}.boxed()
	}
}

enum TypeLangValue<'a, T: Id> {
	Type(TypeSelection<T>),
	Lang(LangSelection<'a>)
}

pub trait ToLenientTerm<T: Id> {
	type Target: Borrow<Lenient<Term<T>>>;

	fn to_lenient_term(&self) -> Self::Target;
}

impl<'a, T: Id> ToLenientTerm<T> for &'a Lenient<Term<T>> {
	type Target = &'a Lenient<Term<T>>;

	#[inline]
	fn to_lenient_term(&self) -> &'a Lenient<Term<T>> {
		self
	}
}

impl<'a, T: Id> ToLenientTerm<T> for &'a T {
	type Target = Lenient<Term<T>>;

	#[inline]
	fn to_lenient_term(&self) -> Lenient<Term<T>> {
		Lenient::Ok(Term::Ref(Reference::Id((*self).clone())))
	}
}

impl<T: Id> ToLenientTerm<T> for Keyword {
	type Target = Lenient<Term<T>>;

	#[inline]
	fn to_lenient_term(&self) -> Lenient<Term<T>> {
		Lenient::Ok(Term::Keyword(*self))
	}
}

impl<'a, T: Id> ToLenientTerm<T> for &'a Reference<T> {
	type Target = Lenient<Term<T>>;

	#[inline]
	fn to_lenient_term(&self) -> Lenient<Term<T>> {
		Lenient::Ok(Term::Ref((*self).clone()))
	}
}

impl<'a, T: Id> ToLenientTerm<T> for &'a Lenient<Reference<T>> {
	type Target = Lenient<Term<T>>;

	#[inline]
	fn to_lenient_term(&self) -> Lenient<Term<T>> {
		match self {
			Lenient::Ok(r) => Lenient::Ok(Term::Ref((*r).clone())),
			Lenient::Unknown(u) => Lenient::Unknown(u.clone())
		}
	}
}

pub(crate) fn compact_iri<'a, T: 'a + Id, C: Context<T>, V: ToLenientTerm<T>>(active_context: &C, inverse_context: &InverseContext<T>, var: V, vocab: bool, reverse: bool, options: Options) -> Result<JsonValue, Error> {
	compact_iri_full::<T, C, V, Object<T>>(active_context, inverse_context, var, None, vocab, reverse, options)
}

pub(crate) fn compact_iri_with<'a, T: 'a + Id, C: Context<T>, V: ToLenientTerm<T>, N: object::Any<T>>(active_context: &C, inverse_context: &InverseContext<T>, var: V, value: &Indexed<N>, vocab: bool, reverse: bool, options: Options) -> Result<JsonValue, Error> {
	compact_iri_full(active_context, inverse_context, var, Some(value), vocab, reverse, options)
}

// default value for `value` is `None` and `false` for `vocab` and `reverse`.
pub(crate) fn compact_iri_full<'a, T: 'a + Id, C: Context<T>, V: ToLenientTerm<T>, N: object::Any<T>>(active_context: &C, inverse_context: &InverseContext<T>, var: V, value: Option<&Indexed<N>>, vocab: bool, reverse: bool, options: Options) -> Result<JsonValue, Error> {
	let var = var.to_lenient_term();
	let var = var.borrow();

	if var == &Lenient::Ok(Term::Null) {
		return Ok(JsonValue::Null)
	}

	println!("compact iri {}", var.as_str());
	if vocab {
		// println!("vocab");
		if let Lenient::Ok(var) = var {
			// println!("ok");
			if let Some(entry) = inverse_context.get(var) {
				// println!("found reverse entry");
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
					let mut has_index = false;
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
								common_lang_dir = Some(Nullable::Some((active_context.default_language(), active_context.default_base_direction())))
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
													item_lang_dir = Some(Nullable::Some((lang_str.language(), lang_str.direction())))
												},
												Value::Literal(_, Some(ty)) => {
													item_type = Some(Type::Ref(ty.clone()))
												},
												Value::Literal(_, None) => {
													item_lang_dir = Some(Nullable::Null)
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
										common_lang_dir = item_lang_dir
									} else if is_value && common_lang_dir != item_lang_dir {
										common_lang_dir = Some(Nullable::Some((None, None)))
									}

									if common_type.is_none() {
										common_type = Some(item_type)
									} else if *common_type.as_ref().unwrap() != item_type {
										common_type = Some(None)
									}

									if common_lang_dir == Some(Nullable::Some((None, None))) && common_type == Some(None) {
										break
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
									type_lang_value = Some(TypeLangValue::Type(TypeSelection::Type(common_type)))
								} else {
									type_lang_value = Some(TypeLangValue::Lang(LangSelection::Lang(common_lang_dir)))
								}
							}
						},
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
						},
						Some(object::Ref::Value(v)) => {
							// If value is a value object:
							if (v.direction().is_some() || v.language().is_some()) && !has_index {
								type_lang_value = Some(TypeLangValue::Lang(LangSelection::Lang(Nullable::Some((v.language(), v.direction())))));
								containers.push(Container::Language);
								containers.push(Container::LanguageSet)
							} else if let Some(ty) = v.typ() {
								type_lang_value = Some(TypeLangValue::Type(TypeSelection::Type(ty.map(|ty| (*ty).clone()))))
							} else {
								is_simple_value = v.direction().is_none() && v.language().is_none() && !has_index
							}

							containers.push(Container::Set)
						},
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
											let compacted_iri = compact_iri(active_context, inverse_context, id, true, false, options)?;
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
							selection.push(LangSelection::Lang(Nullable::Some((None, None))));

							selection.push(LangSelection::Any);

							if let LangSelection::Lang(Nullable::Some((Some(_), Some(dir)))) = lang_value {
								selection.push(LangSelection::Lang(Nullable::Some((None, Some(dir)))));
							}

							Selection::Lang(selection)
						},
						None => {
							let mut selection = Vec::new();
							selection.push(LangSelection::Lang(Nullable::Null));
							selection.push(LangSelection::Lang(Nullable::Some((None, None))));
							selection.push(LangSelection::Any);
							Selection::Lang(selection)
						}
					}
				};

				// println!("select '{}' {:?} with prefered values {:?}", var.as_str(), containers, selection);
				if let Some(term) = entry.select(&containers, &selection) {
					// println!("selected {}", term);
					return Ok(term.into())
				}

				// println!("no selection.");
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
						if (compact_iri.is_empty() || (candidate.len() <= compact_iri.len() && candidate < compact_iri)) &&
						   (candidate_def.is_none() || (candidate_def.is_some() && candidate_def.map_or(None, |def| def.value.as_ref()).map_or(false, |v| v.as_str() == var.as_str()) && value.is_none())) {
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
			println!("base iri {}", base_iri);
			if let Some(iri) = var.as_iri() {
				return Ok(iri.relative_to(base_iri).as_str().into())
			}
		}
	}

	// Finally, return var as is.
	Ok(var.as_str().into())
}

// impl<T: Sync + Send + Id, V: Sync + Send + Compact<T>> Compact<T> for Lenient<V> {
// 	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
// 		async move {
// 			match self {
// 				Lenient::Ok(value) => value.compact_with(active_context, type_scoped_context, inverse_context, active_property, loader, options).await,
// 				Lenient::Unknown(u) => Ok(u.as_str().into())
// 			}
// 		}.boxed()
// 	}
// }

pub trait CompactIndexed<T: Id> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send;
}

impl<T: Sync + Send + Id, V: Sync + Send + CompactIndexed<T>> Compact<T> for Indexed<V> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		self.inner().compact_indexed_with(self.index(), active_context, type_scoped_context, inverse_context, active_property, loader, options)
	}
}

impl<T: Sync + Send + Id, N: object::Any<T> + Sync + Send> CompactIndexed<T> for N {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		match self.as_ref() {
			object::Ref::Value(value) => async move {
				compact_indexed_value_with(value, index, active_context, type_scoped_context, inverse_context, active_property, loader, options).await
			}.boxed(),
			object::Ref::Node(node) => async move {
				compact_indexed_node_with(node, index, active_context, type_scoped_context, inverse_context, active_property, loader, options).await
			}.boxed(),
			object::Ref::List(list) => async move {
				let mut active_context = active_context;
				let mut inverse_context = Mown::Borrowed(inverse_context);
				if let Some(previous_context) = active_context.previous_context() {
					active_context = previous_context;
					inverse_context = Mown::Owned(active_context.invert())
				}

				// If the term definition for active property in active context has a local context:
				let mut active_context = Mown::Borrowed(active_context);
				// let mut inverse_context = Mown::Borrowed(inverse_context);
				if let Some(active_property) = active_property {
					if let Some(active_property_definition) = active_context.get(active_property) {
						if let Some(local_context) = &active_property_definition.context {
							active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?.into_inner());
							inverse_context = Mown::Owned(active_context.invert())
						}
					}
				}

				panic!("WHAT??");
				Ok(self.as_json())
			}.boxed()
		}
	}
}

async fn compact_indexed_value_with<T: Sync + Send + Id, C: ContextMut<T>, L: Loader>(value: &Value<T>, index: Option<&str>, active_context: &C, type_scoped_context: &C, inverse_context: &InverseContext<T>, active_property: Option<&str>, loader: &mut L, options: Options) -> Result<JsonValue, Error> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	// If the term definition for active property in active context has a local context:
	let mut active_context = Mown::Borrowed(active_context);
	let mut inverse_context = Mown::Borrowed(inverse_context);
	if let Some(active_property) = active_property {
		if let Some(active_property_definition) = active_context.get(active_property) {
			if let Some(local_context) = &active_property_definition.context {
				active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?.into_inner());
				inverse_context = Mown::Owned(active_context.invert())
			}
		}
	}

	// If element has an @value or @id entry and the result of using the Value Compaction algorithm,
	// passing active context, active property, and element as value is a scalar,
	// or the term definition for active property has a type mapping of @json,
	// return that result.

	// Here starts the Value Compaction Algorithm.

	// Initialize result to a copy of value.
	// NOTE done later.
	let mut result: json::object::Object;

	// If the active context has a null inverse context,
	// set inverse context in active context to the result of calling the
	// Inverse Context Creation algorithm using active context.
	// NOTE never null here (FIXME is that true?)

	// Initialize inverse context to the value of inverse context in active context.
	// DONE

	let active_property_definition = match active_property {
		Some(active_property) => active_context.get(active_property),
		None => None
	};

	// Initialize language to the language mapping for active property in active context,
	// if any, otherwise to the default language of active context.
	let language = match active_property_definition {
		Some(def) => match def.language.as_ref() {
			Some(lang) => lang.as_ref().option(),
			None => active_context.default_language()
		},
		None => active_context.default_language()
	};

	// Initialize direction to the direction mapping for active property in active context,
	// if any, otherwise to the default base direction of active context.
	let direction = match active_property_definition {
		Some(def) => match def.direction {
			Some(dir) => dir.option(),
			None => active_context.default_base_direction()
		},
		None => active_context.default_base_direction()
	};

	// If value has an @id entry and has no other entries other than @index:
	// NOTE not possible here

	// Otherwise, if value has an @type entry whose value matches the type mapping of
	// active property, set result to the value associated with the @value entry of value.
	let type_mapping: Option<Type<&T>> = match active_property_definition {
		Some(def) => def.typ.as_ref().map(|t| t.into()),
		None => None
	};

	let container_mapping = match active_property_definition {
		Some(def) => def.container,
		None => Container::None
	};

	let remove_index = (index.is_some() && container_mapping.contains(ContainerType::Index)) || index.is_none();

	let mut result = json::object::Object::new();

	match value {
		Value::Literal(lit, ty) => {
			use crate::object::value::Literal;
			if ty.as_ref().map(|t| Type::Ref(t)) == type_mapping && remove_index {
				match lit {
					Literal::Null => return Ok(JsonValue::Null),
					Literal::Boolean(b) => return Ok(b.as_json()),
					Literal::Number(n) => return Ok(JsonValue::Number(n.clone())),
					Literal::String(s) => {
						if ty.is_some() || (language.is_none() && direction.is_none()) {
							return Ok(s.as_json())
						} else {
							let compact_key  = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Value, true, false, options)?;
							result.insert(compact_key.as_str().unwrap(), s.as_json())
						}
					}
				}
			} else {
				let compact_key = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Value, true, false, options)?;
				match lit {
					Literal::Null => {
						result.insert(compact_key.as_str().unwrap(), JsonValue::Null)
					},
					Literal::Boolean(b) => {
						result.insert(compact_key.as_str().unwrap(), b.as_json())
					},
					Literal::Number(n) => {
						result.insert(compact_key.as_str().unwrap(), JsonValue::Number(n.clone()))
					},
					Literal::String(s) => {
						result.insert(compact_key.as_str().unwrap(), s.as_json())
					}
				}

				if let Some(ty) = ty {
					let compact_key = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Type, true, false, options)?;
					let compact_ty = compact_iri(active_context.as_ref(), inverse_context.as_ref(), ty, true, false, options)?;
					result.insert(compact_key.as_str().unwrap(), compact_ty)
				}
			}
		},
		Value::LangString(ls) => {
			let ls_language = ls.language();//.map(|l| Nullable::Some(l));
			let ls_direction = ls.direction();//.map(|d| Nullable::Some(d));

			if remove_index
			&& (ls_language.is_none() || language == ls_language) // || (ls.language().is_none() && language.is_none()))
			&& (ls_direction.is_none() || direction == ls_direction) { // || (ls.direction().is_none() && direction.is_none())) {
				return Ok(ls.as_str().as_json())
			} else {
				let compact_key  = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Value, true, false, options)?;
				result.insert(compact_key.as_str().unwrap(), ls.as_str().into());

				if let Some(language) = ls.language() {
					let compact_key  = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Language, true, false, options)?;
					result.insert(compact_key.as_str().unwrap(), language.as_json());
				}

				if let Some(direction) = ls.direction() {
					let compact_key  = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Direction, true, false, options)?;
					result.insert(compact_key.as_str().unwrap(), direction.as_json());
				}
			}
		},
		Value::Json(value) => {
			if type_mapping == Some(Type::Json) && remove_index {
				return Ok(value.clone())
			} else {
				let compact_key  = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Value, true, false, options)?;
				result.insert(compact_key.as_str().unwrap(), value.clone());

				let compact_key = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Type, true, false, options)?;
				let compact_ty = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Json, true, false, options)?;
				result.insert(compact_key.as_str().unwrap(), compact_ty);
			}
		}
	}

	if !remove_index {
		if let Some(index) = index {
			let compact_key = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Index, true, false, options)?;
			result.insert(compact_key.as_str().unwrap(), index.as_json())
		}
	}

	Ok(JsonValue::Object(result))
}

async fn compact_indexed_node_with<T: Sync + Send + Id, C: ContextMut<T>, L: Loader>(node: &Node<T>, index: Option<&str>, mut active_context: &C, type_scoped_context: &C, inverse_context: &InverseContext<T>, active_property: Option<&str>, loader: &mut L, options: Options) -> Result<JsonValue, Error> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	// If active context has a previous context, the active context is not propagated.
	// If element does not contain an @value entry, and element does not consist of
	// a single @id entry, set active context to previous context from active context,
	// as the scope of a term-scoped context does not apply when processing new node objects.
	let mut active_context_changed = false;
	let mut inverse_context = Mown::Borrowed(inverse_context);
	if !(node.is_empty() && node.id().is_some()) { // does not consist of a single @id entry
		if let Some(previous_context) = active_context.previous_context() {
			active_context = previous_context;
			active_context_changed = true;
		}
	}

	// If the term definition for active property in active context has a local context:
	let mut active_context = Mown::Borrowed(active_context);
	if let Some(active_property) = active_property {
		println!("looking for term definition for {}", active_property);
		if let Some(active_property_definition) = active_context.get(active_property) {
			println!("found!");
			if let Some(local_context) = &active_property_definition.context {
				active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?.into_inner());
				active_context_changed = true;
			}
		}
	}

	if active_context_changed {
		inverse_context = Mown::Owned(active_context.invert())
	}

	let inside_reverse = active_property == Some("@reverse");
	let mut result = json::object::Object::new();

	if !node.types().is_empty() {
		// If element has an @type entry, create a new array compacted types initialized by
		// transforming each expanded type of that entry into its compacted form by IRI
		// compacting expanded type. Then, for each term in compacted types ordered
		// lexicographically:
		let mut compacted_types = Vec::new();
		for ty in node.types() {
			let compacted_ty = compact_iri(type_scoped_context, inverse_context.as_ref(), ty, true, false, options)?;
			compacted_types.push(compacted_ty)
		}

		compacted_types.sort_by(|a, b| {
			a.as_str().unwrap().cmp(b.as_str().unwrap())
		});

		for term in &compacted_types {
			if let Some(term_definition) = type_scoped_context.get(term.as_str().unwrap()) {
				if let Some(local_context) = &term_definition.context {
					println!("CHANGE ACTIVE CONTEXT FOR {}", term);
					let processing_options = context::ProcessingOptions::from(options).without_propagation();
					active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, term_definition.base_url(), processing_options).await?.into_inner());
				}
			}
		}

		inverse_context = Mown::Owned(active_context.invert());
	}

	// For each key expanded property and value expanded value in element, ordered
	// lexicographically by expanded property if ordered is true:
	let mut expanded_entries: Vec<_> = node.properties.iter().collect();
	if options.ordered {
		expanded_entries.sort_by(|(a, _), (b, _)| {
			a.as_str().cmp(b.as_str())
		})
	}

	// If expanded property is @id:
	if let Some(id) = &node.id {
		if node.is_empty() {
			// This captures step 7:
			// If element has an @value or @id entry and the result of using the
			// Value Compaction algorithm, passing active context, active property,
			// and element as value is a scalar, or the term definition for active property
			// has a type mapping of @json, return that result.
			//
			// in the Value Compaction Algorithm, step 7:
			// If value has an @id entry and has no other entries other than @index:
			//
			// If the type mapping of active property is set to @id,
			// set result to the result of IRI compacting the value associated with the
			// @id entry using false for vocab.
			let type_mapping = match active_property {
				Some(prop) => match active_context.get(prop) {
					Some(def) => def.typ.as_ref(),
					None => None
				},
				None => None
			};

			if type_mapping == Some(&Type::Id) {
				let compacted_value = compact_iri(active_context.as_ref(), inverse_context.as_ref(), id, false, false, options)?;
				return Ok(compacted_value)
			}

			// Otherwise, if the type mapping of active property is set to @vocab,
			// set result to the result of IRI compacting the value associated with the @id entry.
			if type_mapping == Some(&Type::Vocab) {
				let compacted_value = compact_iri(active_context.as_ref(), inverse_context.as_ref(), id, true, false, options)?;
				return Ok(compacted_value)
			}
		}

		// If expanded value is a string, then initialize compacted value by IRI
		// compacting expanded value with vocab set to false.
		let compacted_value = compact_iri(active_context.as_ref(), inverse_context.as_ref(), id, false, false, options)?;

		// Initialize alias by IRI compacting expanded property.
		let alias = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Id, true, false, options)?;

		// Add an entry alias to result whose value is set to compacted value and continue
		// to the next expanded property.
		if let Some(key) = alias.as_str() {
			result.insert(key, compacted_value);
		}
	}

	compact_types(&mut result, &node.types, active_context.as_ref(), type_scoped_context, inverse_context.as_ref(), options)?;

	// If expanded property is @reverse:
	if !node.reverse_properties.is_empty() {
		// Initialize compacted value to the result of using this algorithm recursively,
		// passing active context, @reverse for active property,
		// expanded value for element, and the compactArrays and ordered flags.
		let active_property = "@reverse";
		if let Some(active_property_definition) = active_context.get(active_property) {
			if let Some(local_context) = &active_property_definition.context {
				active_context = Mown::Owned(local_context.process_with(active_context.as_ref(), ProcessingStack::new(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?.into_inner());
				inverse_context = Mown::Owned(active_context.invert());
			}
		}

		let mut reverse_result = json::object::Object::new();
		for (expanded_property, expanded_value) in &node.reverse_properties {
			compact_property(&mut reverse_result, expanded_property.clone().into(), expanded_value, index, active_context.as_ref(), type_scoped_context, inverse_context.as_ref(), loader, true, options).await?;
		}

		// For each property and value in compacted value:
		let mut reverse_map = json::object::Object::new();
		for (property, value) in reverse_result {
			// If the term definition for property in the active context indicates that
			// property is a reverse property
			let mut consumed = false;
			if let Some(term_definition) = active_context.get(&property) {
				if term_definition.reverse_property {
					// Initialize as array to true if the container mapping for property in
					// the active context includes @set, otherwise the negation of compactArrays.
					let as_array = term_definition.container.contains(ContainerType::Set) || !options.compact_arrays;

					// Use add value to add value to the property entry in result using as array.
					add_value(&mut result, &property, value, as_array);
					continue
				}
			}

			reverse_map.insert(&property, value);
		}

		if !reverse_map.is_empty() {
			// Initialize alias by IRI compacting @reverse.
			let alias = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Reverse, true, false, options)?;

			// Set the value of the alias entry of result to compacted value.
			result.insert(alias.as_str().unwrap(), JsonValue::Object(reverse_map));
		}
	}

	// If expanded property is @index and active property has a container mapping in
	// active context that includes @index,
	if let Some(index) = index {
		let mut index_container = false;
		if let Some(active_property) = active_property {
			if let Some(active_property_definition) = active_context.get(active_property) {
				if active_property_definition.container.contains(ContainerType::Index) {
					// then the compacted result will be inside of an @index container,
					// drop the @index entry by continuing to the next expanded property.
					index_container = true;
				}
			}
		}

		if !index_container {
			// Initialize alias by IRI compacting expanded property.
			let alias = compact_iri(active_context.as_ref(), inverse_context.as_ref(), Keyword::Index, true, false, options)?;

			// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
			result.insert(alias.as_str().unwrap(), index.as_json());
		}
	}

	if let Some(graph) = &node.graph {
		compact_property(&mut result, Term::Keyword(Keyword::Graph), graph, index, active_context.as_ref(), type_scoped_context, inverse_context.as_ref(), loader, false, options).await?
	}

	for (expanded_property, expanded_value) in expanded_entries {
		compact_property(&mut result, expanded_property.clone().into(), expanded_value, index, active_context.as_ref(), type_scoped_context, inverse_context.as_ref(), loader, false, options).await?
	}

	Ok(JsonValue::Object(result))
}

fn compact_types<T: Sync + Send + Id, C: ContextMut<T>>(result: &mut json::object::Object, types: &[Lenient<Reference<T>>], active_context: &C, type_scoped_context: &C, inverse_context: &InverseContext<T>, options: Options) -> Result<(), Error> {
	// If expanded property is @type:
	if !types.is_empty() {
		// If expanded value is a string,
		// then initialize compacted value by IRI compacting expanded value using
		// type-scoped context for active context.
		let compacted_value = if types.len() == 1 {
			compact_iri(type_scoped_context, inverse_context, &types[0], true, false, options)?
		} else {
			// Otherwise, expanded value must be a @type array:
			// Initialize compacted value to an empty array.
			let mut compacted_value = Vec::with_capacity(types.len());

			// For each item expanded type in expanded value:
			for ty in types {
				// Set term by IRI compacting expanded type using type-scoped context for active context.
				let compacted_ty = compact_iri(type_scoped_context, inverse_context, ty, true, false, options)?;

				// Append term, to compacted value.
				compacted_value.push(compacted_ty)
			}

			JsonValue::Array(compacted_value)
		};

		// Initialize alias by IRI compacting expanded property.
		let alias = compact_iri(active_context, inverse_context, Keyword::Type, true, false, options)?;

		// Initialize as array to true if processing mode is json-ld-1.1 and the
		// container mapping for alias in the active context includes @set,
		// otherwise to the negation of compactArrays.
		let container_mapping = match active_context.get(alias.as_str().unwrap()) {
			Some(def) => def.container,
			None => Container::None
		};
		let as_array = (options.processing_mode == ProcessingMode::JsonLd1_1 && container_mapping.contains(ContainerType::Set)) || !options.compact_arrays;

		// Use add value to add compacted value to the alias entry in result using as array.
		add_value(result, alias.as_str().unwrap(), compacted_value, as_array)
	}

	Ok(())
}

async fn compact_property_list<T: Sync + Send + Id, C: ContextMut<T>, L: Loader>(list: &[Indexed<Object<T>>], expanded_index: Option<&str>, nest_result: &mut json::object::Object, container: Container, as_array: bool, item_active_property: &str, active_context: &C, inverse_context: &InverseContext<T>, loader: &mut L, options: Options) -> Result<(), Error> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	// If expanded item is a list object:
	let mut compacted_item = compact_collection_with(list.iter(), active_context, active_context, &inverse_context, Some(item_active_property), loader, options).await?;

	// If compacted item is not an array,
	// then set `compacted_item` to an array containing only `compacted_item`.
	if !compacted_item.is_array() {
		compacted_item = JsonValue::Array(vec![compacted_item])
	}

	// If container does not include @list:
	if !container.contains(ContainerType::List) {
		// Convert `compacted_item` to a list object by setting it to
		// a map containing an entry where the key is the result of
		// IRI compacting @list and the value is the original
		// compacted item.
		let key = compact_iri(active_context, inverse_context, Keyword::List, true, false, options)?;
		let mut list_object = json::object::Object::new();
		list_object.insert(key.as_str().unwrap(), compacted_item);
		compacted_item = JsonValue::Object(list_object);

		// If `expanded_item` contains the entry @index-value,
		// then add an entry to compacted item where the key is
		// the result of IRI compacting @index and value is value.
		if let Some(index) = expanded_index {
			let key = compact_iri(active_context, inverse_context, Keyword::Index, true, false, options)?;
			match compacted_item {
				JsonValue::Object(ref mut obj) => obj.insert(key.as_str().unwrap(), index.into()),
				_ => unreachable!()
			}
		}

		// Use add value to add `compacted_item` to
		// the `item_active_property` entry in `nest_result` using `as_array`.
		add_value(nest_result, item_active_property, compacted_item, as_array)
	} else {
		// Otherwise, set the value of the item active property entry in nest result to compacted item.
		nest_result.insert(item_active_property, compacted_item)
	}

	Ok(())
}

async fn compact_property_graph<T: Sync + Send + Id, C: ContextMut<T>, L: Loader>(node: &Node<T>, expanded_index: Option<&str>, nest_result: &mut json::object::Object, container: Container, as_array: bool, item_active_property: &str, active_context: &C, type_scoped_context: &C, inverse_context: &InverseContext<T>, loader: &mut L, options: Options) -> Result<(), Error> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	// If expanded item is a graph object
	let mut compacted_item = node.graph.as_ref().unwrap().compact_with(active_context, active_context, &inverse_context, Some(item_active_property), loader, options).await?;

	// If `container` includes @graph and @id:
	if container.contains(ContainerType::Graph) && container.contains(ContainerType::Id) {
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if !nest_result.get(item_active_property).is_some() {
			nest_result.insert(item_active_property, JsonValue::new_object())
		}

		let map_object = match nest_result.get_mut(item_active_property) {
			Some(JsonValue::Object(map)) => map,
			_ => unreachable!()
		};

		// Initialize `map_key` by IRI compacting the value of @id in
		// `expanded_item` or @none if no such value exists
		// with `vocab` set to false if there is an @id entry in
		// `expanded_item`.
		let (id_value, vocab): (Lenient<Term<T>>, bool) = match node.id() {
			Some(term) => (term.clone().cast(), false),
			None => (Lenient::Ok(Term::Keyword(Keyword::None)), true)
		};

		let map_key = compact_iri(active_context, inverse_context, &id_value, vocab, false, options)?;

		// Use `add_value` to add `compacted_item` to
		// the `map_key` entry in `map_object` using `as_array`.
		add_value(map_object, map_key.as_str().unwrap(), compacted_item, as_array)
	} else if container.contains(ContainerType::Graph) && container.contains(ContainerType::Index) && node.is_simple_graph() {
		// Initialize `map_object` to the value of `item_active_property`
		// in `nest_result`, initializing it to a new empty map,
		// if necessary.
		if !nest_result.get(item_active_property).is_some() {
			nest_result.insert(item_active_property, JsonValue::new_object())
		}

		let map_object = match nest_result.get_mut(item_active_property) {
			Some(JsonValue::Object(map)) => map,
			_ => unreachable!()
		};

		// Initialize `map_key` the value of @index in `expanded_item`
		// or @none, if no such value exists.
		let map_key = match expanded_index {
			Some(index) => index,
			None => "@none"
		};

		// Use `add_value` to add `compacted_item` to
		// the `map_key` entry in `map_object` using `as_array`.
		add_value(map_object, map_key, compacted_item, as_array)
	} else if container.contains(ContainerType::Graph) && node.is_simple_graph() {
		// Otherwise, if `container` includes @graph and
		// `expanded_item` is a simple graph object
		// the value cannot be represented as a map object.

		// If `compacted_item` is an array with more than one value,
		// it cannot be directly represented,
		// as multiple objects would be interpreted as different named graphs.
		// Set `compacted_item` to a new map,
		// containing the key from IRI compacting @included and
		// the original `compacted_item` as the value.
		compacted_item = match compacted_item {
			JsonValue::Array(items) if items.len() > 1 => {
				let key = compact_iri(active_context, inverse_context, Keyword::Included, true, false, options)?;
				let mut map = json::object::Object::new();
				map.insert(key.as_str().unwrap(), JsonValue::Array(items));
				JsonValue::Object(map)
			},
			item => item
		};

		// Use `add_value` to add `compacted_item` to the
		// `item_active_property` entry in `nest_result` using `as_array`.
		add_value(nest_result, item_active_property, compacted_item, as_array)
	} else {
		// Otherwise, `container` does not include @graph or
		// otherwise does not match one of the previous cases.

		// Set `compacted_item` to a new map containing the key from
		// IRI compacting @graph using the original `compacted_item` as a value.
		let key = compact_iri(active_context, inverse_context, Keyword::Graph, true, false, options)?;
		let mut map = json::object::Object::new();
		map.insert(key.as_str().unwrap(), compacted_item);

		// If `expanded_item` contains an @id entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @id using the value of
		// IRI compacting the value of @id in `expanded_item` using
		// false for vocab.
		if let Some(id) = node.id() {
			let key = compact_iri(active_context, inverse_context, Keyword::Id, false, false, options)?;
			let value = compact_iri(active_context, inverse_context, id, false, false, options)?;
			map.insert(key.as_str().unwrap(), value);
		}

		// If `expanded_item` contains an @index entry,
		// add an entry in `compacted_item` using the key from
		// IRI compacting @index and the value of @index in `expanded_item`.
		if let Some(index) = expanded_index {
			let key = compact_iri(active_context, inverse_context, Keyword::Index, true, false, options)?;
			map.insert(key.as_str().unwrap(), index.into());
		}

		// Use `add_value` to add `compacted_item` to the
		// `item_active_property` entry in `nest_result` using `as_array`.
		let compacted_item = JsonValue::Object(map);
		add_value(nest_result, item_active_property, compacted_item, as_array)
	}

	Ok(())
}

fn select_nest_result<'a, T: Id, C: ContextMut<T>>(result: &'a mut json::object::Object, active_context: &C, item_active_property: &str, compact_arrays: bool) -> Result<(&'a mut json::object::Object, Container, bool), Error> {
	let (nest_result, container) = match active_context.get(item_active_property) {
		Some(term_definition) => {
			let nest_result = match &term_definition.nest {
				Some(nest_term) => {
					// If nest term is not @nest,
					// or a term in the active context that expands to @nest,
					// an invalid @nest value error has been detected,
					// and processing is aborted.
					if nest_term != "@nest" {
						match active_context.get(nest_term.as_ref()) {
							Some(term_def) if term_def.value == Some(Term::Keyword(Keyword::Nest)) => (),
							_ => return Err(ErrorCode::InvalidNestValue.into())
						}
					}

					// If result does not have a nest_term entry,
					// initialize it to an empty map.
					if result.get(nest_term).is_none() {
						result.insert(nest_term, JsonValue::new_object())
					}

					// Initialize `nest_result` to the value of `nest_term` in result.
					match result.get_mut(nest_term) {
						Some(JsonValue::Object(map)) => map,
						_ => unreachable!()
					}
				},
				None => {
					// Otherwise, initialize `nest_result` to result.
					result
				}
			};

			(nest_result, term_definition.container)
		},
		None => {
			(result, Container::None)
		}
	};

	// Initialize container to container mapping for item active property
	// in active context, or to a new empty array,
	// if there is no such container mapping.
	// DONE.

	// Initialize `as_array` to true if `container` includes @set,
	// or if `item_active_property` is @graph or @list,
	// otherwise the negation of `options.compact_arrays`.
	let as_array = if container.contains(ContainerType::Set) || item_active_property == "@graph" || item_active_property == "@list" {
		true
	} else {
		!compact_arrays
	};

	Ok((nest_result, container, as_array))
}

async fn compact_property<'a, T: 'a + Sync + Send + Id, N: 'a + object::Any<T> + Sync + Send, O: IntoIterator<Item=&'a Indexed<N>>, C: ContextMut<T>, L: Loader>(mut result: &mut json::object::Object, expanded_property: Term<T>, expanded_value: O, index: Option<&str>, active_context: &C, type_scoped_context: &C, inverse_context: &InverseContext<T>, loader: &mut L, inside_reverse: bool, options: Options)
-> Result<(), Error> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	println!("compact prop {}", expanded_property.as_str());
	let lenient_expanded_property: Lenient<Term<T>> = expanded_property.into();
	let mut is_empty = true;

	// For each item `expanded_item` in `expanded value`
	for expanded_item in expanded_value {
		is_empty = false;
		// Initialize `item_active_property` by IRI compacting `expanded_property`
		// using `expanded_item` for value and `inside_reverse` for `reverse`.
		let item_active_property = compact_iri_with(active_context, inverse_context, &lenient_expanded_property, expanded_item, true, inside_reverse, options)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term)
		if let Some(item_active_property) = item_active_property.as_str() {
			let (nest_result, container, as_array) = select_nest_result(result, active_context, item_active_property, options.compact_arrays)?;

			// Initialize `compacted_item` to the result of using this algorithm
			// recursively, passing `active_context`, `item_active_property` for
			// `active_property`, `expanded_item` for `element`, along with the
			// `compact_arrays` and `ordered_flags`.
			// If `expanded_item` is a list object or a graph object,
			// use the value of the @list or @graph entries, respectively,
			// for `element` instead of `expanded_item`.
			match expanded_item.inner().as_ref() {
				object::Ref::List(list) => {
					compact_property_list(list, expanded_item.index(), nest_result, container, as_array, item_active_property, active_context, inverse_context, loader, options).await?
				},
				object::Ref::Node(node) if node.is_graph() => {
					compact_property_graph(node, expanded_item.index(), nest_result, container, as_array, item_active_property, active_context, type_scoped_context, inverse_context, loader, options).await?
				},
				_ => {
					let mut compacted_item = expanded_item.compact_with(active_context, active_context, &inverse_context, Some(item_active_property), loader, options).await?;

					// if container includes @language, @index, @id,
					// or @type and container does not include @graph:
					if !container.contains(ContainerType::Graph) && (container.contains(ContainerType::Language) || container.contains(ContainerType::Index) || container.contains(ContainerType::Id) || container.contains(ContainerType::Type)) {
						// Initialize `map_object` to the value of
						// `item_active_property` in `nest_result`,
						// initializing it to a new empty map, if necessary.
						if !nest_result.get(item_active_property).is_some() {
							nest_result.insert(item_active_property, JsonValue::new_object())
						}

						let map_object = match nest_result.get_mut(item_active_property) {
							Some(JsonValue::Object(map)) => map,
							_ => unreachable!()
						};

						// Initialize container key by IRI compacting either
						// @language, @index, @id, or @type based on the contents of container.
						let container_type = if container.contains(ContainerType::Language) {
							ContainerType::Language
						} else if container.contains(ContainerType::Index) {
							ContainerType::Index
						} else if container.contains(ContainerType::Id) {
							ContainerType::Id
						} else {
							ContainerType::Type
						};

						let mut container_key = compact_iri(active_context, inverse_context, &Lenient::Ok(Term::Keyword(container_type.into())), true, false, options)?;

						// Initialize `index_key` to the value of index mapping in
						// the term definition associated with `item_active_property`
						// in active context, or @index, if no such value exists.
						let index_key = match active_context.get(item_active_property) {
							Some(def) if def.index.is_some() => def.index.as_ref().unwrap(),
							_ => "@index"
						};

						// If `container` includes @language and `expanded_item`
						// contains a @value entry, then set `compacted_item` to
						// the value associated with its @value entry.
						// Set `map_key` to the value of @language in `expanded_item`,
						// if any.
						let map_key = if container_type == ContainerType::Language && expanded_item.is_value() {
							if let object::Ref::Value(value) = expanded_item.inner().as_ref() {
								compacted_item = value_value(value)
							}

							match expanded_item.language() {
								Some(lang) => Some(lang.clone()),
								None => None
							}
						} else if container_type == ContainerType::Index {
							if index_key == "@index" {
								// Otherwise, if `container` includes @index and
								// `index_key` is @index, set `map_key` to the value of
								// @index in `expanded_item`, if any.
								match expanded_item.index() {
									Some(index) => Some(index.to_string()),
									None => None
								}
							} else {
								// Otherwise, if `container` includes @index and
								// `index_key` is not @index:

								// Reinitialize `container_key` by
								// IRI compacting `index_key`.
								let lenient_index : Lenient<Term<T>> = Lenient::Unknown(index_key.to_string());
								container_key = compact_iri(active_context, inverse_context, &lenient_index, true, false, options)?;

								// Set `map_key` to the first value of
								// `container_key` in `compacted_item`, if any.
								let map_key = match &compacted_item {
									JsonValue::Object(map) => match map.get(container_key.as_str().unwrap()) {
										Some(value) => match value.as_str() {
											Some(str) => Some(str.to_string()),
											None => None
										},
										None => None
									},
									_ => None
								};

								// If there are remaining values in `compacted_item`
								// for container key, use `add_value` to add
								// those remaining values to the `container_key`
								// in `compacted_item`.
								// Otherwise, remove that entry from compacted item.
								// TODO 12.8.9.6.3

								map_key
							}
						} else if container_type == ContainerType::Id {
							// Otherwise, if `container` includes @id,
							// set `map_key` to the value of `container_key` in
							// `compacted_item` and remove `container_key` from
							// `compacted_item`.
							match &mut compacted_item {
								JsonValue::Object(map) => match map.remove(container_key.as_str().unwrap()) {
									Some(JsonValue::String(str)) => Some(str.to_string()),
									Some(JsonValue::Short(str)) => Some(str.to_string()),
									_ => None
								},
								_ => None
							}
						} else {
							// Otherwise, if container includes @type:

							// Set `map_key` to the first value of `container_key` in
							// `compacted_item`, if any.
							let map_key = match &compacted_item {
								JsonValue::Object(map) => match map.get(container_key.as_str().unwrap()) {
									Some(value) => match value.as_str() {
										Some(str) => Some(str.to_string()),
										None => None
									},
									None => None
								},
								_ => None
							};

							// If there are remaining values in `compacted_item` for
							// `container_key`, use `add_value` to add those
							// remaining values to the `container_key` in
							// `compacted_item`.
							// Otherwise, remove that entry from compacted item.
							// TODO 12.8.9.8.2

							// If `compacted_item` contains a single entry with a key
							// expanding to @id, set `compacted_item` to the result of
							// using this algorithm recursively,
							// passing `active_context`, `item_active_property` for
							// `active_property`, and a map composed of the single
							// entry for @id from `expanded_item` for `element`.
							if let JsonValue::Object(map) = &compacted_item {
								if map.len() == 1 {
									if let Some(_) = map.get("@id") {
										let obj = Object::Node(Node::with_id(expanded_item.id().unwrap().clone()));
										compacted_item = obj.compact_indexed_with(None, active_context, active_context, &inverse_context, Some(item_active_property), loader, options).await?
									}
								}
							}

							None
						};

						// If `map_key` is null, set it to the result of
						// IRI compacting @none.
						let map_key = match map_key {
							Some(key) => key,
							None => {
								let key = compact_iri(active_context, inverse_context, Keyword::None, true, false, options)?;
								key.as_str().unwrap().to_string()
							}
						};

						// Use `add_value` to add `compacted_item` to
						// the `map_key` entry in `map_object` using `as_array`.
						add_value(map_object, &map_key, compacted_item, as_array)
					} else {
						// Otherwise, use `add_value` to add `compacted_item` to the
						// `item_active_property` entry in `nest_result` using `as_array`.
						add_value(nest_result, item_active_property, compacted_item, as_array)
					}
				}
			};
		}
	}

	// If expanded value is an empty array:
	if is_empty {
		// Initialize `item_active_property` by IRI compacting
		// `expanded_property` using `expanded_value` for `value` and
		// `inside_reverse` for `reverse`.
		let item_active_property = compact_iri_with(active_context, inverse_context, &lenient_expanded_property, &Indexed::new(Object::Node(Node::new()), None), true, inside_reverse, options)?;

		// If the term definition for `item_active_property` in the active context
		// has a nest value entry (nest term):
		if let Some(item_active_property) = item_active_property.as_str() {
			let (nest_result, _, _) = select_nest_result(result, active_context, item_active_property, options.compact_arrays)?;

			// Use `add_value` to add an empty array to the `item_active_property` entry in
			// `nest_result` using true for `as_array`.
			add_value(nest_result, item_active_property, JsonValue::Array(Vec::new()), true)
		}
	}

	Ok(())
}

/// Get the `@value` field of a value object.
fn value_value<T: Id>(value: &Value<T>) -> JsonValue {
	use crate::object::value::Literal;
	match value {
		Value::Literal(lit, ty) => {
			match lit {
				Literal::Null => JsonValue::Null,
				Literal::Boolean(b) => b.as_json(),
				Literal::Number(n) => JsonValue::Number(n.clone()),
				Literal::String(s) => s.as_json()
			}
		},
		Value::LangString(str) => str.as_str().into(),
		Value::Json(json) => json.clone()
	}
}

/// Default value of `as_array` is false.
fn add_value(map: &mut json::object::Object, key: &str, value: JsonValue, as_array: bool) {
	match map.get(key) {
		Some(JsonValue::Array(_)) => (),
		Some(original_value) => {
			let value = original_value.clone();
			map.insert(key, JsonValue::Array(vec![value]))
		},
		None if as_array => map.insert(key, JsonValue::Array(Vec::new())),
		None => ()
	}

	match value {
		JsonValue::Array(values) => {
			for value in values {
				add_value(map, key, value, false)
			}
		},
		value => {
			match map.get_mut(key) {
				Some(JsonValue::Array(values)) => values.push(value),
				Some(_) => unreachable!(),
				None => map.insert(key, value)
			}
		}
	}
}

fn compact_collection_with<'a, T: Sync + Send + Id, O: 'a + Send + Iterator<Item=&'a Indexed<Object<T>>>, C: ContextMut<T>, L: Loader>(items: O, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
	async move {
		let mut result = Vec::new();

		for item in items {
			match item.compact_with(active_context, type_scoped_context, inverse_context, active_property, loader, options).await? {
				JsonValue::Null => (),
				compacted_item => {
					result.push(compacted_item)
				}
			}
		}

		let mut list_or_set = false;
		if let Some(active_property) = active_property {
			if let Some(active_property_definition) = active_context.get(active_property) {
				list_or_set = active_property_definition.container.contains(ContainerType::List) || active_property_definition.container.contains(ContainerType::Set);
			}
		}

		if result.is_empty()
		|| result.len() > 1
		|| !options.compact_arrays
		|| active_property == Some("@graph") || active_property == Some("@set")
		|| list_or_set {
			return Ok(JsonValue::Array(result))
		}

		return Ok(result.into_iter().next().unwrap())
	}.boxed()
}

impl<T: Sync + Send + Id> Compact<T> for HashSet<Indexed<Object<T>>> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		compact_collection_with(self.iter(), active_context, type_scoped_context, inverse_context, active_property, loader, options)
	}
}
