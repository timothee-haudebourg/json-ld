use crate::context::Entry;
use locspan_derive::StrippedPartialEq;
use std::hash::Hash;

#[derive(Clone, Copy, PartialEq, StrippedPartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[stripped_ignore(M)]
pub struct Type<M> {
	pub container: Entry<TypeContainer, M>,
	pub protected: Option<Entry<bool, M>>,
}

impl<M> Type<M> {
	pub fn iter(&self) -> ContextTypeEntries<M> {
		ContextTypeEntries {
			container: Some(&self.container),
			protected: self.protected.as_ref(),
		}
	}
}

pub struct ContextTypeEntries<'a, M> {
	container: Option<&'a Entry<TypeContainer, M>>,
	protected: Option<&'a Entry<bool, M>>,
}

impl<'a, M> Iterator for ContextTypeEntries<'a, M> {
	type Item = ContextTypeEntry<'a, M>;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let mut len = 0;

		if self.container.is_some() {
			len += 1;
		}

		if self.protected.is_some() {
			len += 1;
		}

		(len, Some(len))
	}

	fn next(&mut self) -> Option<Self::Item> {
		match self.container.take() {
			Some(c) => Some(ContextTypeEntry::Container(c)),
			None => self.protected.take().map(ContextTypeEntry::Protected),
		}
	}
}

impl<'a, M> ExactSizeIterator for ContextTypeEntries<'a, M> {}

pub enum ContextTypeEntry<'a, M> {
	Container(&'a Entry<TypeContainer, M>),
	Protected(&'a Entry<bool, M>),
}

impl<'a, M> ContextTypeEntry<'a, M> {
	pub fn key(&self) -> ContextTypeKey {
		match self {
			Self::Container(_) => ContextTypeKey::Container,
			Self::Protected(_) => ContextTypeKey::Protected,
		}
	}
}

pub enum ContextTypeKey {
	Container,
	Protected,
}

impl ContextTypeKey {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Container => "@container",
			Self::Protected => "@protected",
		}
	}
}

#[derive(Clone, Copy, StrippedPartialEq, PartialOrd, Ord, Debug)]
pub enum TypeContainer {
	Set,
}

impl TypeContainer {
	pub fn into_str(self) -> &'static str {
		match self {
			Self::Set => "@set",
		}
	}
}

impl PartialEq for TypeContainer {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl Eq for TypeContainer {}

impl Hash for TypeContainer {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}
