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
#[stripped_ignore(M)]
pub enum Value<M> {
	One(Meta<Context<M>, M>),
	Many(Vec<Meta<Context<M>, M>>),
}

impl<M> Value<M> {
	pub fn as_slice(&self) -> &[Meta<Context<M>, M>] {
		match self {
			Self::One(c) => std::slice::from_ref(c),
			Self::Many(list) => list,
		}
	}

	pub fn traverse(&self) -> Traverse<Self>
	where
		M: Clone + Send + Sync,
	{
		match self {
			Self::One(c) => Traverse::new(FragmentRef::Context(c.as_context_ref())),
			Self::Many(m) => Traverse::new(FragmentRef::ContextArray(m.iter().into())),
		}
	}
}

impl<M> From<Meta<Context<M>, M>> for Value<M> {
	fn from(c: Meta<Context<M>, M>) -> Self {
		Self::One(c)
	}
}

impl<M: Default> From<Context<M>> for Value<M> {
	fn from(c: Context<M>) -> Self {
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

impl<M: Default> From<Definition<M>> for Value<M> {
	fn from(c: Definition<M>) -> Self {
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

impl<M> From<Meta<Definition<M>, M>> for Value<M> {
	fn from(Meta(c, meta): Meta<Definition<M>, M>) -> Self {
		Self::One(Meta(Context::Definition(c), meta))
	}
}

/// Context.
#[derive(PartialEq, StrippedPartialEq, Eq, Clone, Debug)]
#[stripped_ignore(M)]
pub enum Context<M> {
	Null,
	IriRef(#[stripped] IriRefBuf),
	Definition(Definition<M>),
}

impl<M> Context<M> {
	pub fn as_context_ref(&self) -> ContextRef<Definition<M>> {
		match self {
			Self::Null => ContextRef::Null,
			Self::IriRef(i) => ContextRef::IriRef(i.as_iri_ref()),
			Self::Definition(d) => ContextRef::Definition(d.into()),
		}
	}
}

impl<M> From<IriRefBuf> for Context<M> {
	fn from(i: IriRefBuf) -> Self {
		Context::IriRef(i)
	}
}

impl<'a, M> From<iref::IriRef<'a>> for Context<M> {
	fn from(i: iref::IriRef<'a>) -> Self {
		Context::IriRef(i.into())
	}
}

impl<M> From<iref::IriBuf> for Context<M> {
	fn from(i: iref::IriBuf) -> Self {
		Context::IriRef(i.into())
	}
}

impl<'a, M> From<iref::Iri<'a>> for Context<M> {
	fn from(i: iref::Iri<'a>) -> Self {
		Context::IriRef(i.into())
	}
}

impl<M> From<Definition<M>> for Context<M> {
	fn from(c: Definition<M>) -> Self {
		Context::Definition(c)
	}
}

/// Context value fragment.
pub enum FragmentRef<'a, C: AnyValue> {
	/// Context array.
	ContextArray(C::Array<'a>),

	/// Context.
	Context(ContextRef<'a, C::Definition>),

	/// Context definition fragment.
	DefinitionFragment(definition::FragmentRef<'a, C>),
}

impl<'a, C: AnyValue> FragmentRef<'a, C> {
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

	pub fn sub_items(&self) -> SubFragments<'a, C> {
		match self {
			Self::ContextArray(a) => SubFragments::ContextArray(a.clone()),
			Self::Context(c) => SubFragments::Context(c.sub_items()),
			Self::DefinitionFragment(d) => SubFragments::Definition(d.sub_items()),
		}
	}
}

pub enum ContextSubFragments<'a, D: 'a + AnyDefinition> {
	None,
	Definition(definition::Entries<'a, D::ContextValue, D::Bindings<'a>>),
}

impl<'a, D: 'a + AnyDefinition> Iterator for ContextSubFragments<'a, D> {
	type Item = FragmentRef<'a, D::ContextValue>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Definition(e) => e
				.next()
				.map(|e| FragmentRef::DefinitionFragment(definition::FragmentRef::Entry(e))),
		}
	}
}

pub enum SubFragments<'a, C: AnyValue> {
	ContextArray(C::Array<'a>),
	Context(ContextSubFragments<'a, C::Definition>),
	Definition(definition::SubItems<'a, C>),
}

impl<'a, C: AnyValue> Iterator for SubFragments<'a, C> {
	type Item = FragmentRef<'a, C>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::ContextArray(a) => a.next().map(|c| FragmentRef::Context(c.into_value())),
			Self::Context(i) => i.next(),
			Self::Definition(i) => i.next().map(FragmentRef::DefinitionFragment),
		}
	}
}

pub struct Traverse<'a, C: AnyValue> {
	stack: SmallVec<[FragmentRef<'a, C>; 8]>,
}

impl<'a, C: AnyValue> Traverse<'a, C> {
	pub(crate) fn new(item: FragmentRef<'a, C>) -> Self {
		let mut stack = SmallVec::new();
		stack.push(item);
		Self { stack }
	}
}

impl<'a, C: AnyValue> Iterator for Traverse<'a, C> {
	type Item = FragmentRef<'a, C>;

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
