use iref::{Iri, IriRef, IriRefBuf};

mod definition;

pub use definition::*;

/// JSON-LD Context.
///
/// Can represent a single context entry, or a list of context entries.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Context {
	One(ContextEntry),
	Many(Vec<ContextEntry>),
}

impl Default for Context {
	fn default() -> Self {
		Self::Many(Vec::new())
	}
}

impl Context {
	/// Creates a new context with a single entry.
	pub fn one(context: ContextEntry) -> Self {
		Self::One(context)
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
	pub fn definition(def: ContextDefinition) -> Self {
		Self::one(ContextEntry::Definition(def))
	}
}

impl Context {
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

	pub fn as_slice(&self) -> &[ContextEntry] {
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

	pub fn iter(&self) -> std::slice::Iter<ContextEntry> {
		self.as_slice().iter()
	}
}

pub enum IntoIter {
	One(Option<ContextEntry>),
	Many(std::vec::IntoIter<ContextEntry>),
}

impl Iterator for IntoIter {
	type Item = ContextEntry;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(t) => t.take(),
			Self::Many(t) => t.next(),
		}
	}
}

impl IntoIterator for Context {
	type Item = ContextEntry;
	type IntoIter = IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::One(t) => IntoIter::One(Some(t)),
			Self::Many(t) => IntoIter::Many(t.into_iter()),
		}
	}
}

impl<'a> IntoIterator for &'a Context {
	type IntoIter = std::slice::Iter<'a, ContextEntry>;
	type Item = &'a ContextEntry;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl From<ContextEntry> for Context {
	fn from(c: ContextEntry) -> Self {
		Self::One(c)
	}
}

impl From<IriRefBuf> for Context {
	fn from(i: IriRefBuf) -> Self {
		Self::One(ContextEntry::IriRef(i))
	}
}

impl<'a> From<&'a IriRef> for Context {
	fn from(i: &'a IriRef) -> Self {
		Self::One(ContextEntry::IriRef(i.to_owned()))
	}
}

impl From<iref::IriBuf> for Context {
	fn from(i: iref::IriBuf) -> Self {
		Self::One(ContextEntry::IriRef(i.into()))
	}
}

impl<'a> From<&'a Iri> for Context {
	fn from(i: &'a Iri) -> Self {
		Self::One(ContextEntry::IriRef(i.to_owned().into()))
	}
}

impl From<ContextDefinition> for Context {
	fn from(c: ContextDefinition) -> Self {
		Self::One(ContextEntry::Definition(c))
	}
}

/// Context.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(untagged)
)]
pub enum ContextEntry {
	Null,
	IriRef(IriRefBuf),
	Definition(ContextDefinition),
}

impl ContextEntry {
	pub fn is_object(&self) -> bool {
		matches!(self, Self::Definition(_))
	}
}

impl From<IriRefBuf> for ContextEntry {
	fn from(i: IriRefBuf) -> Self {
		ContextEntry::IriRef(i)
	}
}

impl<'a> From<&'a IriRef> for ContextEntry {
	fn from(i: &'a IriRef) -> Self {
		ContextEntry::IriRef(i.to_owned())
	}
}

impl From<iref::IriBuf> for ContextEntry {
	fn from(i: iref::IriBuf) -> Self {
		ContextEntry::IriRef(i.into())
	}
}

impl<'a> From<&'a Iri> for ContextEntry {
	fn from(i: &'a Iri) -> Self {
		ContextEntry::IriRef(i.to_owned().into())
	}
}

impl From<ContextDefinition> for ContextEntry {
	fn from(c: ContextDefinition) -> Self {
		ContextEntry::Definition(c)
	}
}

/// Context document.
///
/// A context document is a JSON-LD document containing an object with a single
/// `@context` entry.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContextDocumentValue {
	#[cfg_attr(feature = "serde", serde(rename = "@context"))]
	pub context: Context,
}
