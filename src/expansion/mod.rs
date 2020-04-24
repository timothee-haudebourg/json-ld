mod expanded;
mod iri;
mod literal;
mod value;
mod node;
mod array;
mod element;

use std::cmp::{Ord, Ordering};
use std::collections::HashSet;
use futures::Future;
use iref::{Iri, IriBuf};
use json::JsonValue;
use crate::{
	ProcessingMode,
	Error,
	Id,
	Indexed,
	Object,
	ContextMut,
	context::{
		ProcessingOptions,
		Loader
	}
};

pub use expanded::*;
pub use iri::*;
pub use literal::*;
pub use value::*;
pub use node::*;
pub use array::*;
pub use element::*;

#[derive(Clone, Copy, Default)]
pub struct Options {
	/// Sets the processing mode.
	pub processing_mode: ProcessingMode,

	/// If set to true, input document entries are processed lexicographically.
	/// If false, order is not considered in processing.
	pub ordered: bool
}

impl From<Options> for ProcessingOptions {
	fn from(options: Options) -> ProcessingOptions {
		let mut copt = ProcessingOptions::default();
		copt.processing_mode = options.processing_mode;
		copt
	}
}

#[derive(PartialEq, Eq)]
pub struct Entry<'a, T>(T, &'a JsonValue);

impl<'a, T: PartialOrd> PartialOrd for Entry<'a, T> {
	fn partial_cmp(&self, other: &Entry<'a, T>) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, T: Ord> Ord for Entry<'a, T> {
	fn cmp(&self, other: &Entry<'a, T>) -> Ordering {
		self.0.cmp(&other.0)
	}
}

fn filter_top_level_item<T: Id>(item: &Indexed<Object<T>>) -> bool {
	// Remove dangling values.
	match item.inner() {
		Object::Value(_) => false,
		_ => true
	}
}

pub fn expand<'a, T: Id, C: ContextMut<T>, L: Loader>(active_context: &'a C, element: &'a JsonValue, base_url: Option<Iri>, loader: &'a mut L, options: Options) -> impl 'a + Future<Output=Result<HashSet<Indexed<Object<T>>>, Error>> where C::LocalContext: From<L::Output> + From<JsonValue>, L::Output: Into<JsonValue> {
	let base_url = base_url.map(|url| IriBuf::from(url));

	async move {
		let base_url = base_url.as_ref().map(|url| url.as_iri());
		let expanded = expand_element(active_context, None, element, base_url, loader, options).await?;
		if expanded.len() == 1 {
			match expanded.into_iter().next().unwrap().into_unnamed_graph() {
				Ok(graph) => Ok(graph),
				Err(obj) => {
					let mut set = HashSet::new();
					if filter_top_level_item(&obj) {
						set.insert(obj);
					}
					Ok(set)
				}
			}
		} else {
			Ok(expanded.into_iter().filter(filter_top_level_item).collect())
		}
	}
}
