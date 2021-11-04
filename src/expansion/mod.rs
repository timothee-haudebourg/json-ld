//! Expansion algorithm and related types.
use crate::{
	context::{Loader, ProcessingOptions},
	ContextMut, Error, Id, Indexed, Object, ProcessingMode,
};
use cc_traits::{CollectionRef, KeyedRef};
use derivative::Derivative;
use generic_json::{Json, JsonClone, JsonHash, JsonLft, JsonSendSync};
use iref::IriBuf;
use std::cmp::{Ord, Ordering};
use std::collections::HashSet;

mod array;
mod element;
mod expanded;
mod iri;
mod literal;
mod node;
mod value;

use array::*;
use element::*;
use expanded::*;
pub(crate) use iri::*;
use literal::*;
use node::*;
use value::*;

/// JSON document that can be expanded.
pub trait JsonExpand = JsonSendSync + JsonHash + JsonClone + JsonLft<'static>;

/// Expansion options.
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
	/// using the [`Reference::Invalid`](crate::Reference::Invalid) variant.
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

/// JSON object entry reference.
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub(crate) struct Entry<'a, J: Json>(
	<J::Object as KeyedRef>::KeyRef<'a>,
	<J::Object as CollectionRef>::ItemRef<'a>,
)
where
	J::Object: 'a;

impl<'a, J: Json> PartialEq for Entry<'a, J>
where
	J::Object: 'a,
{
	fn eq(&self, other: &Self) -> bool {
		*self.0 == *other.0
	}
}

impl<'a, J: Json> Eq for Entry<'a, J> where J::Object: 'a {}

impl<'a, J: Json> PartialOrd for Entry<'a, J>
where
	J::Object: 'a,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		(*self.0).partial_cmp(&**other.0)
	}
}

impl<'a, J: Json> Ord for Entry<'a, J>
where
	J::Object: 'a,
{
	fn cmp(&self, other: &Self) -> Ordering {
		(*self.0).cmp(&*other.0)
	}
}

/// JSON object entry, with the expanded key.
pub(crate) struct ExpandedEntry<'a, J: Json, T>(
	<J::Object as KeyedRef>::KeyRef<'a>,
	T,
	<J::Object as CollectionRef>::ItemRef<'a>,
)
where
	J::Object: 'a;

impl<'a, J: Json, T> PartialEq for ExpandedEntry<'a, J, T>
where
	J::Object: 'a,
{
	fn eq(&self, other: &Self) -> bool {
		*self.0 == *other.0
	}
}

impl<'a, J: Json, T> Eq for ExpandedEntry<'a, J, T> where J::Object: 'a {}

impl<'a, J: Json, T> PartialOrd for ExpandedEntry<'a, J, T>
where
	J::Object: 'a,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		(*self.0).partial_cmp(&**other.0)
	}
}

impl<'a, J: Json, T> Ord for ExpandedEntry<'a, J, T>
where
	J::Object: 'a,
{
	fn cmp(&self, other: &Self) -> Ordering {
		(*self.0).cmp(&*other.0)
	}
}

fn filter_top_level_item<J: JsonHash, T: Id>(item: &Indexed<Object<J, T>>) -> bool {
	// Remove dangling values.
	!matches!(item.inner(), Object::Value(_))
}

/// Expand the given JSON-LD document.
///
/// Note that you probably do not want to use this function directly,
/// but instead use the [`Document::expand`](crate::Document::expand) method, implemented for
/// every JSON type implementing the [`generic_json::Json`] trait.
pub async fn expand<'a, J: JsonExpand, T: Id, C: ContextMut<T>, L: Loader>(
	active_context: &'a C,
	document: &'a J,
	base_url: Option<IriBuf>,
	loader: &'a mut L,
	options: Options,
// ) -> impl 'a + Send + Future<Output = Result<HashSet<Indexed<Object<J, T>>>, Error>>
) -> Result<HashSet<Indexed<Object<J, T>>>, Error>
where
	T: Send + Sync,
	C: Send + Sync,
	C::LocalContext: From<L::Output> + From<J>,
	L: Send + Sync,
	L::Output: Into<J>,
{
	// let base_url = base_url.map(IriBuf::from);

	// async move {
		let base_url = base_url.as_ref().map(|url| url.as_iri());
		let expanded = expand_element(
			active_context,
			None,
			document,
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
	// }
}
