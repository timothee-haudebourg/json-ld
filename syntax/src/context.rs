use crate::Entry;
use iref::IriRefBuf;
use locspan::Meta;
use locspan_derive::StrippedPartialEq;
use smallvec::SmallVec;

pub mod definition;
mod print;
mod reference;
pub mod term_definition;
mod try_from_json;

pub use definition::{AnyDefinition, Definition};
pub use reference::*;
pub use term_definition::{TermDefinition, TermDefinitionRef};
pub use try_from_json::InvalidContext;

/// Context entry.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[locspan(ignore(M))]
pub enum Value<M> {
	One(Meta<Context<Definition<M, Self>>, M>),
	Many(Vec<Meta<Context<Definition<M, Self>>, M>>),
}

impl<M> Value<M> {
	pub fn as_slice(&self) -> &[Meta<Context<Definition<M, Self>>, M>] {
		match self {
			Self::One(c) => std::slice::from_ref(c),
			Self::Many(list) => list,
		}
	}

	pub fn traverse(&self) -> Traverse<M, Self>
	where
		M: Clone + Send + Sync,
	{
		match self {
			Self::One(c) => Traverse::new(FragmentRef::Context(c.as_context_ref())),
			Self::Many(m) => Traverse::new(FragmentRef::ContextArray(ArrayIter::Owned(m.iter()))),
		}
	}
}

impl<M> From<Meta<Context<Definition<M, Self>>, M>> for Value<M> {
	fn from(c: Meta<Context<Definition<M, Self>>, M>) -> Self {
		Self::One(c)
	}
}

impl<M: Default> From<Context<Definition<M, Self>>> for Value<M> {
	fn from(c: Context<Definition<M, Self>>) -> Self {
		Self::One(Meta(c, M::default()))
	}
}

impl<M: Default> From<IriRefBuf> for Value<M> {
	fn from(i: IriRefBuf) -> Self {
		Self::One(Meta(Context::IriRef(i), M::default()))
	}
}

impl<'a, M: Default> From<iref::IriRef<'a>> for Value<M> {
	fn from(i: iref::IriRef<'a>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), M::default()))
	}
}

impl<M: Default> From<iref::IriBuf> for Value<M> {
	fn from(i: iref::IriBuf) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), M::default()))
	}
}

impl<'a, M: Default> From<iref::Iri<'a>> for Value<M> {
	fn from(i: iref::Iri<'a>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), M::default()))
	}
}

impl<M: Default> From<Definition<M, Self>> for Value<M> {
	fn from(c: Definition<M, Self>) -> Self {
		Self::One(Meta(Context::Definition(c), M::default()))
	}
}

impl<M> From<Meta<IriRefBuf, M>> for Value<M> {
	fn from(Meta(i, meta): Meta<IriRefBuf, M>) -> Self {
		Self::One(Meta(Context::IriRef(i), meta))
	}
}

impl<'a, M> From<Meta<iref::IriRef<'a>, M>> for Value<M> {
	fn from(Meta(i, meta): Meta<iref::IriRef<'a>, M>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), meta))
	}
}

impl<M> From<Meta<iref::IriBuf, M>> for Value<M> {
	fn from(Meta(i, meta): Meta<iref::IriBuf, M>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), meta))
	}
}

impl<'a, M> From<Meta<iref::Iri<'a>, M>> for Value<M> {
	fn from(Meta(i, meta): Meta<iref::Iri<'a>, M>) -> Self {
		Self::One(Meta(Context::IriRef(i.into()), meta))
	}
}

// impl<M, D> From<Meta<D, M>> for Value<M, D> {
// 	fn from(Meta(c, meta): Meta<D, M>) -> Self {
// 		Self::One(Meta(Context::Definition(c), meta))
// 	}
// }

/// Context.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[locspan(ignore(M))]
pub enum Context<D> {
	Null,
	IriRef(#[locspan(stripped)] IriRefBuf),
	Definition(D),
}

impl<D> Context<D> {
	pub fn as_context_ref(&self) -> ContextRef<D> {
		match self {
			Self::Null => ContextRef::Null,
			Self::IriRef(i) => ContextRef::IriRef(i.as_iri_ref()),
			Self::Definition(d) => ContextRef::Definition(d),
		}
	}
}

impl<D> From<IriRefBuf> for Context<D> {
	fn from(i: IriRefBuf) -> Self {
		Context::IriRef(i)
	}
}

impl<'a, D> From<iref::IriRef<'a>> for Context<D> {
	fn from(i: iref::IriRef<'a>) -> Self {
		Context::IriRef(i.into())
	}
}

impl<D> From<iref::IriBuf> for Context<D> {
	fn from(i: iref::IriBuf) -> Self {
		Context::IriRef(i.into())
	}
}

impl<'a, D> From<iref::Iri<'a>> for Context<D> {
	fn from(i: iref::Iri<'a>) -> Self {
		Context::IriRef(i.into())
	}
}

impl<M, C> From<Definition<M, C>> for Context<Definition<M, C>> {
	fn from(c: Definition<M, C>) -> Self {
		Context::Definition(c)
	}
}

/// Context value fragment.
pub enum FragmentRef<'a, M, C: AnyValue<M>> {
	/// Context array.
	ContextArray(ArrayIter<'a, M, C::Definition>),

	/// Context.
	Context(ContextRef<'a, C::Definition>),

	/// Context definition fragment.
	DefinitionFragment(definition::FragmentRef<'a, M, C>),
}

impl<'a, M, C: AnyValue<M>> FragmentRef<'a, M, C> {
	pub fn is_array(&self) -> bool {
		match self {
			Self::ContextArray(_) => true,
			Self::DefinitionFragment(i) => i.is_array(),
			_ => false,
		}
	}

	pub fn is_object(&self) -> bool
	where
		M: Clone,
	{
		match self {
			Self::Context(c) => c.is_object(),
			Self::DefinitionFragment(i) => i.is_object(),
			_ => false,
		}
	}

	pub fn sub_items(&self) -> SubFragments<'a, M, C>
	where
		M: Clone,
	{
		match self {
			Self::ContextArray(a) => SubFragments::ContextArray(a.clone()),
			Self::Context(c) => SubFragments::Context(c.sub_items()),
			Self::DefinitionFragment(d) => SubFragments::Definition(Box::new(d.sub_items())),
		}
	}
}

pub enum ContextSubFragments<'a, M, D: AnyDefinition<M>> {
	None,
	Definition(Box<definition::Entries<'a, M, D::ContextValue>>),
}

impl<'a, M: 'a + Clone, D: AnyDefinition<M>> Iterator for ContextSubFragments<'a, M, D> {
	type Item = FragmentRef<'a, M, D::ContextValue>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Definition(e) => e
				.next()
				.map(|e| FragmentRef::DefinitionFragment(definition::FragmentRef::Entry(e))),
		}
	}
}

pub enum SubFragments<'a, M, C: AnyValue<M>> {
	ContextArray(ArrayIter<'a, M, C::Definition>),
	Context(ContextSubFragments<'a, M, C::Definition>),
	Definition(Box<definition::SubItems<'a, M, C>>),
}

impl<'a, M: Clone, C: AnyValue<M>> Iterator for SubFragments<'a, M, C> {
	type Item = FragmentRef<'a, M, C>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::ContextArray(a) => a.next().map(|c| FragmentRef::Context(c.into_value())),
			Self::Context(i) => i.next(),
			Self::Definition(i) => i.next().map(FragmentRef::DefinitionFragment),
		}
	}
}

pub struct Traverse<'a, M, C: AnyValue<M>> {
	stack: SmallVec<[FragmentRef<'a, M, C>; 8]>,
}

impl<'a, M, C: AnyValue<M>> Traverse<'a, M, C> {
	pub(crate) fn new(item: FragmentRef<'a, M, C>) -> Self {
		let mut stack = SmallVec::new();
		stack.push(item);
		Self { stack }
	}
}

impl<'a, M: Clone, C: AnyValue<M>> Iterator for Traverse<'a, M, C> {
	type Item = FragmentRef<'a, M, C>;

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
