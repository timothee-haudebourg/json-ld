use derivative::Derivative;
use iref::IriRef;
use locspan::{Meta, StrippedPartialEq};

use super::{
	AnyDefinition, Context, ContextSubFragments, Definition, FragmentRef, Traverse, Value,
};

pub trait AnyValue: Sized + StrippedPartialEq + Clone + Send + Sync {
	type Metadata: Clone + Send + Sync;

	type Definition: AnyDefinition<ContextValue = Self> + Send + Sync;

	type Array<'a>: Iterator<Item = Meta<ContextRef<'a, Self::Definition>, Self::Metadata>>
		+ Clone
		+ Send
		+ Sync
	where
		Self: 'a;

	fn as_value_ref(&self) -> ValueRef<Self::Metadata, Self::Definition, Self::Array<'_>>;

	fn traverse(&self) -> Traverse<Self> {
		match self.as_value_ref() {
			ValueRef::One(Meta(c, _)) => Traverse::new(FragmentRef::Context(c)),
			ValueRef::Many(m) => Traverse::new(FragmentRef::ContextArray(m)),
		}
	}
}

pub trait AnyValueMut {
	fn append(&mut self, context: Self);
}

impl<M: Clone + Send + Sync> AnyValue for Value<M> {
	type Metadata = M;

	type Definition = Definition<M>;
	type Array<'a> = ManyContexts<'a, M> where M: 'a;

	fn as_value_ref(&self) -> ValueRef<M> {
		self.into()
	}
}

impl<M> AnyValueMut for Value<M> {
	fn append(&mut self, context: Self) {
		match self {
			Self::One(a) => {
				let a = unsafe { core::ptr::read(a) };

				let contexts = match context {
					Self::One(b) => vec![a, b],
					Self::Many(b) => {
						let mut contexts = vec![a];
						contexts.extend(b);
						contexts
					}
				};

				unsafe { core::ptr::write(self, Self::Many(contexts)) }
			}
			Self::Many(a) => match context {
				Self::One(b) => a.push(b),
				Self::Many(b) => a.extend(b),
			},
		}
	}
}

/// Reference to a context entry.
#[derive(Derivative)]
#[derivative(Clone(bound = "M: Clone, C: Clone"))]
pub enum ValueRef<'a, M, D = Definition<M>, C = ManyContexts<'a, M>> {
	One(Meta<ContextRef<'a, D>, M>),
	Many(C),
}

impl<'a, M, D, C> ValueRef<'a, M, D, C> {
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Many(_))
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::One(c) => c.is_object(),
			_ => false,
		}
	}
}

impl<'a, M: Clone, D, C: Iterator<Item = Meta<ContextRef<'a, D>, M>>> IntoIterator
	for ValueRef<'a, M, D, C>
{
	type Item = Meta<ContextRef<'a, D>, M>;
	type IntoIter = ContextEntryIter<'a, M, D, C>;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::One(i) => ContextEntryIter::One(Some(i)),
			Self::Many(m) => ContextEntryIter::Many(m),
		}
	}
}

impl<'a, M: Clone> From<&'a Value<M>> for ValueRef<'a, M> {
	fn from(e: &'a Value<M>) -> Self {
		match e {
			Value::One(c) => Self::One(c.borrow_value().cast()),
			Value::Many(m) => Self::Many(ManyContexts(m.iter())),
		}
	}
}

#[derive(Clone)]
pub struct ManyContexts<'a, M>(std::slice::Iter<'a, Meta<Context<M>, M>>);

impl<'a, M: Clone> Iterator for ManyContexts<'a, M> {
	type Item = Meta<ContextRef<'a, Definition<M>>, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|c| c.borrow_value().cast())
	}
}

impl<'a, M: Clone> ExactSizeIterator for ManyContexts<'a, M> {}

impl<'a, M> From<std::slice::Iter<'a, Meta<Context<M>, M>>> for ManyContexts<'a, M> {
	fn from(i: std::slice::Iter<'a, Meta<Context<M>, M>>) -> Self {
		Self(i)
	}
}

pub enum ContextEntryIter<'a, M, D = Definition<M>, C = ManyContexts<'a, M>> {
	One(Option<Meta<ContextRef<'a, D>, M>>),
	Many(C),
}

impl<'a, M, D, C: Iterator<Item = Meta<ContextRef<'a, D>, M>>> Iterator
	for ContextEntryIter<'a, M, D, C>
{
	type Item = Meta<ContextRef<'a, D>, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(i) => i.take(),
			Self::Many(m) => m.next(),
		}
	}
}

/// Reference to context.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum ContextRef<'a, D> {
	Null,
	IriRef(IriRef<'a>),
	Definition(&'a D),
}

impl<'a, D> ContextRef<'a, D> {
	pub fn is_object(&self) -> bool {
		matches!(self, Self::Definition(_))
	}

	pub fn sub_items(&self) -> ContextSubFragments<'a, D>
	where
		D: AnyDefinition,
	{
		match self {
			Self::Definition(d) => ContextSubFragments::Definition(d.entries()),
			_ => ContextSubFragments::None,
		}
	}
}

impl<'a, M> From<&'a Context<M>> for ContextRef<'a, Definition<M>> {
	fn from(c: &'a Context<M>) -> Self {
		match c {
			Context::Null => ContextRef::Null,
			Context::IriRef(i) => ContextRef::IriRef(i.as_iri_ref()),
			Context::Definition(d) => ContextRef::Definition(d),
		}
	}
}
