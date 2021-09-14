//! Expansion algorithm and types.

mod array;
mod element;
mod expanded;
mod iri;
mod literal;
mod node;
mod value;

use crate::{
	context::{Loader, ProcessingOptions},
	ContextMut, Error, Id, Indexed, Object, ProcessingMode,
};
use futures::Future;
use iref::{Iri, IriBuf};
use json::JsonValue;
use std::cmp::{Ord, Ordering};
use std::collections::HashSet;

pub use array::*;
pub use element::*;
pub use expanded::*;
pub use iri::*;
pub use literal::*;
pub use node::*;
pub use value::*;

#[derive(Clone, Copy, Default)]
pub struct Options {
	/// Sets the processing mode.
	pub processing_mode: ProcessingMode,

	/// Term expansion policy.
	///
	/// Default is `Policy::Standard`.
	pub policy: Policy,

	/// If set to true, input document entries are processed lexicographically.
	/// If false, order is not considered in processing.
	pub ordered: bool,
}

/// Key expansion policy.
///
/// The default behavior of the expansion algorithm
/// is to drop keys that are not defined in the context unless:
///   - there is a vocabulary mapping (`@vocab`) defined in the context; or
///   - the term contains a `:` character.
/// In other words, a key that cannot be expanded into an
/// IRI or a blank node identifier is dropped unless it contains a `:` character.
///
/// Sometimes, it is preferable to keep undefined keys in the
/// expanded document, or to forbid them completely by raising an error.
/// You can define your preferred policy using one of this type variant
/// with the [`Options::policy`] field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Policy {
	/// Relaxed policy.
	///
	/// Undefined keys are always kept in the expanded document
	/// using the [`Reference::Invalid`] variant.
	Relaxed,

	/// Standard policy.
	///
	/// Every key that cannot be expanded into an
	/// IRI or a blank node identifier is dropped unless it contains a `:` character.
	Standard,

	/// Strict policy.
	///
	/// Every key that cannot be expanded into an IRI or a blank node identifier
	/// will raise an error unless the term contains a `:` character.
	Strict,

	/// Strictest policy.
	///
	/// Every key that cannot be expanded into an IRI or a blank node identifier
	/// will raise an error.
	Strictest,
}

impl Policy {
	/// Returns `true` is the policy is `Strict` or `Strictest`.
	pub fn is_strict(&self) -> bool {
		matches!(self, Self::Strict | Self::Strictest)
	}
}

impl Default for Policy {
	fn default() -> Self {
		Self::Standard
	}
}

impl From<Options> for ProcessingOptions {
	fn from(options: Options) -> ProcessingOptions {
		ProcessingOptions {
			processing_mode: options.processing_mode,
			..Default::default()
		}
	}
}

impl From<crate::compaction::Options> for Options {
	fn from(options: crate::compaction::Options) -> Options {
		Options {
			processing_mode: options.processing_mode,
			ordered: options.ordered,
			..Options::default()
		}
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
	!matches!(item.inner(), Object::Value(_))
}

pub fn expand<'a, T: Send + Sync + Id, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(
	active_context: &'a C,
	element: &'a JsonValue,
	base_url: Option<Iri>,
	loader: &'a mut L,
	options: Options,
) -> impl 'a + Send + Future<Output = Result<HashSet<Indexed<Object<T>>>, Error>>
where
	C::LocalContext: Send + Sync + From<L::Output> + From<JsonValue>,
	L::Output: Into<JsonValue>,
{
	let base_url = base_url.map(IriBuf::from);

	async move {
		let base_url = base_url.as_ref().map(|url| url.as_iri());
		let expanded = expand_element(
			active_context,
			None,
			element,
			base_url,
			loader,
			options,
			false,
		)
		.await?;
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
