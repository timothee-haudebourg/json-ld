use derivative::Derivative;
use iref::IriRef;
use locspan::{Meta, StrippedPartialEq};

use super::{
	AnyDefinition, Context, ContextSubFragments, Definition, FragmentRef, Traverse, Value,
};

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub enum ArrayIter<'a, M, D> {
	Owned(core::slice::Iter<'a, Meta<Context<D>, M>>),
	Borrowed(core::slice::Iter<'a, Meta<ContextRef<'a, D>, M>>)
}

impl<'a, D, M: Clone> Iterator for ArrayIter<'a, M, D> {
	type Item = Meta<ContextRef<'a, D>, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Self::Owned(m) => m.size_hint(),
			Self::Borrowed(m) => m.size_hint()
		}
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Owned(m) => m.next().map(|Meta(d, m)| Meta(d.as_context_ref(), m.clone())),
			Self::Borrowed(m) => m.next().cloned()
		}
	}
}

impl<'a, D, M: Clone> ExactSizeIterator for ArrayIter<'a, M, D> {}

pub trait AnyValue<M>: Sized + StrippedPartialEq + Clone + Send + Sync {
	type Definition: AnyDefinition<M, ContextValue = Self> + Send + Sync;

	fn as_value_ref(&self) -> ValueRef<M, Self::Definition>;

	fn traverse(&self) -> Traverse<M, Self> {
		match self.as_value_ref() {
			ValueRef::One(Meta(c, _)) => Traverse::new(FragmentRef::Context(c)),
			ValueRef::Many(m) => Traverse::new(FragmentRef::ContextArray(m)),
		}
	}
}

pub trait AnyValueMut {
	fn append(&mut self, context: Self);
}

impl<M: Clone + Send + Sync> AnyValue<M> for Value<M> {
	type Definition = Definition<M, Self>;

	fn as_value_ref(&self) -> ValueRef<M, Self::Definition> {
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
#[derivative(Clone(bound = "M: Clone"))]
pub enum ValueRef<'a, M, D = Definition<M>> {
	One(Meta<ContextRef<'a, D>, M>),
	Many(ArrayIter<'a, M, D>),
}

impl<'a, M, D> ValueRef<'a, M, D> {
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

impl<'a, M: Clone, D> IntoIterator
	for ValueRef<'a, M, D>
{
	type Item = Meta<ContextRef<'a, D>, M>;
	type IntoIter = ContextEntryIter<'a, M, D>;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::One(i) => ContextEntryIter::One(Some(i)),
			Self::Many(m) => ContextEntryIter::Many(m),
		}
	}
}

impl<'a, M: Clone> From<&'a Value<M>> for ValueRef<'a, M, Definition<M>> {
	fn from(e: &'a Value<M>) -> Self {
		match e {
			Value::One(c) => Self::One(c.borrow_value().cast()),
			Value::Many(m) => Self::Many(ArrayIter::Owned(m.iter())),
		}
	}
}

// #[derive(Clone)]
// pub struct ManyContexts<'a, M>(std::slice::Iter<'a, Meta<Context<Definition<M>>, M>>);

// impl<'a, M: Clone> Iterator for ManyContexts<'a, M> {
// 	type Item = Meta<ContextRef<'a, Definition<M>>, M>;

// 	fn size_hint(&self) -> (usize, Option<usize>) {
// 		self.0.size_hint()
// 	}

// 	fn next(&mut self) -> Option<Self::Item> {
// 		self.0.next().map(|c| c.borrow_value().cast())
// 	}
// }

// impl<'a, M: Clone> ExactSizeIterator for ManyContexts<'a, M> {}

// impl<'a, M, D> From<std::slice::Iter<'a, Meta<Context<D>, M>>> for ManyContexts<'a, M> {
// 	fn from(i: std::slice::Iter<'a, Meta<Context<D>, M>>) -> Self {
// 		Self(i)
// 	}
// }

pub enum ContextEntryIter<'a, M, D = Definition<M>> {
	One(Option<Meta<ContextRef<'a, D>, M>>),
	Many(ArrayIter<'a, M, D>),
}

impl<'a, M: Clone, D> Iterator
	for ContextEntryIter<'a, M, D>
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

	pub fn sub_items<M>(&self) -> ContextSubFragments<'a, M, D> where D: AnyDefinition<M> {
		match self {
			Self::Definition(d) => ContextSubFragments::Definition(d.entries()),
			_ => ContextSubFragments::None,
		}
	}
}

impl<'a, D> From<&'a Context<D>> for ContextRef<'a, D> {
	fn from(c: &'a Context<D>) -> Self {
		match c {
			Context::Null => ContextRef::Null,
			Context::IriRef(i) => ContextRef::IriRef(i.as_iri_ref()),
			Context::Definition(d) => ContextRef::Definition(d),
		}
	}
}
