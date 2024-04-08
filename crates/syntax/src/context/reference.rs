use educe::Educe;
use iref::IriRef;
use locspan::{Meta, StrippedPartialEq};

use super::{
	AnyDefinition, Context, ContextSubFragments, Definition, FragmentRef, Traverse, Value,
};

#[derive(Educe)]
#[educe(Clone)]
pub enum ArrayIter<'a, M> {
	Owned(core::slice::Iter<'a, Meta<Context<M>, M>>),
	Borrowed(core::slice::Iter<'a, Meta<ContextRef<'a, M>, M>>),
}

impl<'a, M: Clone> Iterator for ArrayIter<'a, M> {
	type Item = Meta<ContextRef<'a, M>, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		match self {
			Self::Owned(m) => m.size_hint(),
			Self::Borrowed(m) => m.size_hint(),
		}
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Owned(m) => m
				.next()
				.map(|Meta(d, m)| Meta(d.as_context_ref(), m.clone())),
			Self::Borrowed(m) => m.next().cloned(),
		}
	}
}

impl<'a, M: Clone> ExactSizeIterator for ArrayIter<'a, M> {}

impl<M> Value<M> {
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
#[derive(Educe)]
#[educe(Clone(bound = "M: Clone"))]
pub enum ValueRef<'a, M> {
	One(Meta<ContextRef<'a, M>, M>),
	Many(ArrayIter<'a, M>),
}

impl<'a, M> ValueRef<'a, M> {
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

impl<'a, M: Clone> IntoIterator for ValueRef<'a, M> {
	type Item = Meta<ContextRef<'a, M>, M>;
	type IntoIter = ContextEntryIter<'a, M>;

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
			Value::Many(m) => Self::Many(ArrayIter::Owned(m.iter())),
		}
	}
}

pub enum ContextEntryIter<'a, M> {
	One(Option<Meta<ContextRef<'a, M>, M>>),
	Many(ArrayIter<'a, M>),
}

impl<'a, M: Clone> Iterator for ContextEntryIter<'a, M> {
	type Item = Meta<ContextRef<'a, M>, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(i) => i.take(),
			Self::Many(m) => m.next(),
		}
	}
}

/// Reference to context.
#[derive(Educe)]
#[educe(Clone, Copy)]
pub enum ContextRef<'a, M = ()> {
	Null,
	IriRef(IriRef<'a>),
	Definition(&'a Definition<M>),
}

impl<'a, M> ContextRef<'a, M> {
	pub fn is_object(&self) -> bool {
		matches!(self, Self::Definition(_))
	}

	pub fn sub_items(&self) -> ContextSubFragments<'a, M> where M: Clone {
		match self {
			Self::Definition(d) => ContextSubFragments::Definition(Box::new(d.entries())),
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
