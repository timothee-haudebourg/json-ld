use crate::Entry;
use iref::{Iri, IriRef, IriRefBuf};
use locspan::Meta;
use locspan_derive::StrippedPartialEq;
use smallvec::SmallVec;

pub mod definition;
mod print;
pub mod term_definition;
mod try_from_json;

pub use definition::Definition;
pub use term_definition::TermDefinition;
pub use try_from_json::InvalidContext;

/// JSON-LD Context.
///
/// Can represent a single context entry, or a list of context entries.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged, bound(deserialize = "M: Default"))]
#[locspan(ignore(M))]
pub enum Context<M = ()> {
	One(Meta<ContextEntry<M>, M>),
	Many(Vec<Meta<ContextEntry<M>, M>>),
}

impl<M> Default for Context<M> {
	fn default() -> Self {
		Self::Many(Vec::new())
	}
}

impl Context {
	/// Creates a new context with a single entry.
	pub fn one(context: ContextEntry) -> Self {
		Self::One(Meta::none(context))
	}

	/// Creates the `null` context.
	pub fn null() -> Self {
		Self::one(ContextEntry::Null)
	}

	/// Creates a new context with a single IRI-reference entry.
	pub fn iri_ref(iri_ref: IriRefBuf) -> Self {
		Self::one(ContextEntry::IriRef(iri_ref))
	}

	/// Creates a new context with a single context definition entry.
	pub fn definition(def: Definition) -> Self {
		Self::one(ContextEntry::Definition(def))
	}
}

impl<M> Context<M> {
	pub fn len(&self) -> usize {
		match self {
			Self::One(_) => 1,
			Self::Many(l) => l.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::One(_) => false,
			Self::Many(l) => l.is_empty(),
		}
	}

	pub fn as_slice(&self) -> &[Meta<ContextEntry<M>, M>] {
		match self {
			Self::One(c) => std::slice::from_ref(c),
			Self::Many(list) => list,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::One(c) => c.is_object(),
			_ => false,
		}
	}

	pub fn is_array(&self) -> bool {
		matches!(self, Self::Many(_))
	}

	pub fn traverse(&self) -> Traverse<M> {
		match self {
			Self::One(c) => Traverse::new(FragmentRef::Context(c)),
			Self::Many(m) => Traverse::new(FragmentRef::ContextArray(m)),
		}
	}

	pub fn iter(&self) -> std::slice::Iter<Meta<ContextEntry<M>, M>> {
		self.as_slice().iter()
	}
}

impl<'a, M> IntoIterator for &'a Context<M> {
	type IntoIter = std::slice::Iter<'a, Meta<ContextEntry<M>, M>>;
	type Item = &'a Meta<ContextEntry<M>, M>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<M> From<Meta<ContextEntry<M>, M>> for Context<M> {
	fn from(c: Meta<ContextEntry<M>, M>) -> Self {
		Self::One(c)
	}
}

impl<M: Default> From<ContextEntry<M>> for Context<M> {
	fn from(c: ContextEntry<M>) -> Self {
		Self::One(Meta(c, M::default()))
	}
}

impl<M: Default> From<IriRefBuf> for Context<M> {
	fn from(i: IriRefBuf) -> Self {
		Self::One(Meta(ContextEntry::IriRef(i), M::default()))
	}
}

impl<'a, M: Default> From<&'a IriRef> for Context<M> {
	fn from(i: &'a IriRef) -> Self {
		Self::One(Meta(ContextEntry::IriRef(i.to_owned()), M::default()))
	}
}

impl<M: Default> From<iref::IriBuf> for Context<M> {
	fn from(i: iref::IriBuf) -> Self {
		Self::One(Meta(ContextEntry::IriRef(i.into()), M::default()))
	}
}

impl<'a, M: Default> From<&'a Iri> for Context<M> {
	fn from(i: &'a Iri) -> Self {
		Self::One(Meta(
			ContextEntry::IriRef(i.to_owned().into()),
			M::default(),
		))
	}
}

impl<M: Default> From<Definition<M>> for Context<M> {
	fn from(c: Definition<M>) -> Self {
		Self::One(Meta(ContextEntry::Definition(c), M::default()))
	}
}

impl<M> From<Meta<IriRefBuf, M>> for Context<M> {
	fn from(Meta(i, meta): Meta<IriRefBuf, M>) -> Self {
		Self::One(Meta(ContextEntry::IriRef(i), meta))
	}
}

impl<'a, M> From<Meta<&'a IriRef, M>> for Context<M> {
	fn from(Meta(i, meta): Meta<&'a IriRef, M>) -> Self {
		Self::One(Meta(ContextEntry::IriRef(i.to_owned()), meta))
	}
}

impl<M> From<Meta<iref::IriBuf, M>> for Context<M> {
	fn from(Meta(i, meta): Meta<iref::IriBuf, M>) -> Self {
		Self::One(Meta(ContextEntry::IriRef(i.into()), meta))
	}
}

impl<'a, M> From<Meta<&'a Iri, M>> for Context<M> {
	fn from(Meta(i, meta): Meta<&'a Iri, M>) -> Self {
		Self::One(Meta(ContextEntry::IriRef(i.to_owned().into()), meta))
	}
}

/// Context.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged, bound(deserialize = "M: Default"))]
#[locspan(ignore(M))]
pub enum ContextEntry<M = ()> {
	Null,
	IriRef(#[locspan(stripped)] IriRefBuf),
	Definition(Definition<M>),
}

impl<M> ContextEntry<M> {
	fn sub_items(&self) -> ContextSubFragments<M> {
		match self {
			Self::Definition(d) => ContextSubFragments::Definition(Box::new(d.iter())),
			_ => ContextSubFragments::None,
		}
	}

	pub fn is_object(&self) -> bool {
		matches!(self, Self::Definition(_))
	}
}

impl<D> From<IriRefBuf> for ContextEntry<D> {
	fn from(i: IriRefBuf) -> Self {
		ContextEntry::IriRef(i)
	}
}

impl<'a, D> From<&'a IriRef> for ContextEntry<D> {
	fn from(i: &'a IriRef) -> Self {
		ContextEntry::IriRef(i.to_owned())
	}
}

impl<D> From<iref::IriBuf> for ContextEntry<D> {
	fn from(i: iref::IriBuf) -> Self {
		ContextEntry::IriRef(i.into())
	}
}

impl<'a, D> From<&'a Iri> for ContextEntry<D> {
	fn from(i: &'a Iri) -> Self {
		ContextEntry::IriRef(i.to_owned().into())
	}
}

impl<M> From<Definition<M>> for ContextEntry<M> {
	fn from(c: Definition<M>) -> Self {
		ContextEntry::Definition(c)
	}
}

/// Context value fragment.
pub enum FragmentRef<'a, M> {
	/// Context array.
	ContextArray(&'a [Meta<ContextEntry<M>, M>]),

	/// Context.
	Context(&'a Meta<ContextEntry<M>, M>),

	/// Context definition fragment.
	DefinitionFragment(definition::FragmentRef<'a, M>),
}

impl<'a, M> FragmentRef<'a, M> {
	pub fn is_array(&self) -> bool {
		match self {
			Self::ContextArray(_) => true,
			Self::DefinitionFragment(i) => i.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::Context(c) => c.is_object(),
			Self::DefinitionFragment(i) => i.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> SubFragments<'a, M> {
		match self {
			Self::ContextArray(a) => SubFragments::ContextArray(a.iter()),
			Self::Context(c) => SubFragments::Context(c.sub_items()),
			Self::DefinitionFragment(d) => SubFragments::Definition(Box::new(d.sub_items())),
		}
	}
}

pub enum ContextSubFragments<'a, M> {
	None,
	Definition(Box<definition::Entries<'a, M>>),
}

impl<'a, M> Iterator for ContextSubFragments<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Definition(e) => e
				.next()
				.map(|e| FragmentRef::DefinitionFragment(definition::FragmentRef::Entry(e))),
		}
	}
}

pub enum SubFragments<'a, M> {
	ContextArray(std::slice::Iter<'a, Meta<ContextEntry<M>, M>>),
	Context(ContextSubFragments<'a, M>),
	Definition(Box<definition::SubItems<'a, M>>),
}

impl<'a, M> Iterator for SubFragments<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::ContextArray(a) => a.next().map(FragmentRef::Context),
			Self::Context(i) => i.next(),
			Self::Definition(i) => i.next().map(FragmentRef::DefinitionFragment),
		}
	}
}

pub struct Traverse<'a, M> {
	stack: SmallVec<[FragmentRef<'a, M>; 8]>,
}

impl<'a, M> Traverse<'a, M> {
	pub(crate) fn new(item: FragmentRef<'a, M>) -> Self {
		let mut stack = SmallVec::new();
		stack.push(item);
		Self { stack }
	}
}

impl<'a, M> Iterator for Traverse<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.stack.pop() {
			Some(item) => {
				self.stack.extend(item.sub_items());
				Some(item)
			}
			None => None,
		}
	}
}
