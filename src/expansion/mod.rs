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
use crate::{Error, Id, Object};
use crate::context::{MutableActiveContext, ContextLoader};

pub use expanded::*;
pub use iri::*;
pub use literal::*;
pub use value::*;
pub use node::*;
pub use array::*;
pub use element::*;

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

fn filter_top_level_item<T: Id>(item: &Object<T>) -> bool {
	// Remove dangling values.
	match item {
		Object::Value(_, _) => false,
		_ => true
	}
}

pub fn expand<'a, T: Id, C: MutableActiveContext<T>, L: ContextLoader<C::LocalContext>>(active_context: &'a C, active_property: Option<&'a str>, element: &'a JsonValue, base_url: Option<Iri>, loader: &'a mut L) -> impl 'a + Future<Output=Result<HashSet<Object<T>>, Error>> where C::LocalContext: From<JsonValue> {
	let base_url = base_url.map(|url| IriBuf::from(url));

	async move {
		let base_url = base_url.as_ref().map(|url| url.as_iri());
		let expanded = expand_element(active_context, active_property, element, base_url, loader, false, false).await?;
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
