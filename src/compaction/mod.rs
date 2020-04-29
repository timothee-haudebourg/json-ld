use std::collections::HashSet;
use futures::future::{BoxFuture, FutureExt};
use mown::Mown;
use json::JsonValue;
use crate::{
	Id,
	ContextMut,
	Indexed,
	Object,
	Value,
	Node,
	Reference,
	Lenient,
	Error,
	context::{
		self,
		Loader,
		ProcessingStack,
		Local,
		InverseContext
	},
	syntax::{
		Keyword,
		ContainerType,
		Term
	},
	util::AsJson
};

#[derive(Clone, Copy)]
pub struct Options {
	compact_to_relative: bool,
	compact_arrays: bool,
	ordered: bool
}

impl From<Options> for context::ProcessingOptions {
	fn from(_options: Options) -> context::ProcessingOptions {
		context::ProcessingOptions::default()
	}
}

impl Default for Options {
	fn default() -> Options {
		Options {
			compact_to_relative: false,
			compact_arrays: false,
			ordered: false
		}
	}
}

pub trait Compact<T: Id> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send;
}

impl<T: Sync + Send + Id> Compact<T> for Reference<T> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			// TODO compact IRI.
			Ok(JsonValue::Null)
		}.boxed()
	}
}

impl<T: Sync + Send + Id, V: Sync + Send + Compact<T>> Compact<T> for Lenient<V> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			match self {
				Lenient::Ok(value) => value.compact_with(context, type_scoped_context, inverse_context, active_property, loader, options).await,
				Lenient::Unknown(u) => Ok(u.as_str().into())
			}
		}.boxed()
	}
}

pub trait CompactIndexed<T: Id> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send;
}

impl<T: Sync + Send + Id, V: Sync + Send + CompactIndexed<T>> Compact<T> for Indexed<V> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		self.inner().compact_indexed_with(self.index(), context, type_scoped_context, inverse_context, active_property, loader, options)
	}
}

impl<T: Sync + Send + Id> CompactIndexed<T> for Object<T> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		match self {
			Object::Value(value) => value.compact_indexed_with(index, context, type_scoped_context, inverse_context, active_property, loader, options),
			Object::Node(node) => {
				let mut active_context = context;
				if let Some(previous_context) = active_context.previous_context() {
					active_context = previous_context;
				}

				node.compact_indexed_with(index, active_context, type_scoped_context, inverse_context, active_property, loader, options)
			},
			Object::List(list) => async move {
				// TODO
				Ok(self.as_json())
			}.boxed()
		}
	}
}

impl<T: Sync + Send + Id> CompactIndexed<T> for Value<T> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			// TODO
			Ok(self.as_json())
		}.boxed()
	}
}

impl<T: Sync + Send + Id> CompactIndexed<T> for Node<T> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			let inside_reverse = active_property == Some(&Term::Keyword(Keyword::Reverse));
			let mut result = json::object::Object::new();

			let mut inverse_context = Mown::Borrowed(inverse_context);
			if !self.types().is_empty() {
				// If element has an @type entry, create a new array compacted types initialized by
				/// transforming each expanded type of that entry into its compacted form by IRI
				// compacting expanded type. Then, for each term in compacted types ordered
				// lexicographically:
				let mut compacted_types = Vec::new();
				for ty in self.types() {
					compacted_types.push(ty.compact_with(context, type_scoped_context, inverse_context.as_ref(), active_property, loader, options).await?)
				}

				if options.ordered {
					compacted_types.sort_by(|a, b| {
						a.as_str().unwrap().cmp(b.as_str().unwrap())
					});
				}

				let mut context = Mown::Borrowed(context);
				for term in &compacted_types {
					if let Some(term_definition) = type_scoped_context.get(term.as_str().unwrap()) {
						if let Some(local_context) = &term_definition.context {
							context = Mown::Owned(local_context.process_with(context.as_ref(), ProcessingStack::new(), loader, term_definition.base_url(), options.into()).await?);
						}
					}
				}

				inverse_context = Mown::Owned(context.invert());
			}

			// TODO

			Ok(JsonValue::Object(result))
		}.boxed()
	}
}

impl<T: Sync + Send + Id> Compact<T> for HashSet<Indexed<Object<T>>> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, context: &'a C, type_scoped_context: &'a C, inverse_context: &'a InverseContext<T>, active_property: Option<&'a Term<T>>, loader: &'a mut L, options: Options) -> BoxFuture<'a, Result<JsonValue, Error>> where C: Sync + Send, C::LocalContext: Send + Sync + From<L::Output>, L: Sync + Send {
		async move {
			let mut result = Vec::new();

			for item in self {
				match item.compact_with(context, type_scoped_context, inverse_context, active_property, loader, options).await? {
					JsonValue::Null => (),
					compacted_item => result.push(compacted_item)
				}
			}

			let mut list_or_set = false;
			if let Some(active_property) = active_property {
				if let Some(active_property_definition) = context.get(active_property.as_str()) {
					list_or_set = active_property_definition.container.contains(ContainerType::List) || active_property_definition.container.contains(ContainerType::Set);
				}
			}

			if result.is_empty() || result.len() > 1
			|| !options.compact_arrays
			|| active_property == Some(&Term::Keyword(Keyword::Graph)) || active_property == Some(&Term::Keyword(Keyword::Set))
			|| list_or_set {
				return Ok(JsonValue::Array(result))
			}

			return Ok(result.into_iter().next().unwrap())
		}.boxed()
	}
}
