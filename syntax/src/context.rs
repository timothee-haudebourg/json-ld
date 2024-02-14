use iref::{Iri, IriRef, IriRefBuf};
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
	pub fn definition(def: Definition) -> Self {
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

	pub fn traverse(&self) -> Traverse {
		match self {
			Self::One(c) => Traverse::new(FragmentRef::Context(c)),
			Self::Many(m) => Traverse::new(FragmentRef::ContextArray(m)),
		}
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

impl From<Definition> for Context {
	fn from(c: Definition) -> Self {
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
	Definition(Definition),
}

impl ContextEntry {
	fn sub_items(&self) -> ContextSubFragments {
		match self {
			Self::Definition(d) => ContextSubFragments::Definition(Box::new(d.iter())),
			_ => ContextSubFragments::None,
		}
	}

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

impl From<Definition> for ContextEntry {
	fn from(c: Definition) -> Self {
		ContextEntry::Definition(c)
	}
}

/// Context value fragment.
pub enum FragmentRef<'a> {
	/// Context array.
	ContextArray(&'a [ContextEntry]),

	/// Context.
	Context(&'a ContextEntry),

	/// Context definition fragment.
	DefinitionFragment(definition::FragmentRef<'a>),
}

impl<'a> FragmentRef<'a> {
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

	pub fn sub_items(&self) -> SubFragments<'a> {
		match self {
			Self::ContextArray(a) => SubFragments::ContextArray(a.iter()),
			Self::Context(c) => SubFragments::Context(c.sub_items()),
			Self::DefinitionFragment(d) => SubFragments::Definition(Box::new(d.sub_items())),
		}
	}
}

pub enum ContextSubFragments<'a> {
	None,
	Definition(Box<definition::Entries<'a>>),
}

impl<'a> Iterator for ContextSubFragments<'a> {
	type Item = FragmentRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Definition(e) => e
				.next()
				.map(|e| FragmentRef::DefinitionFragment(definition::FragmentRef::Entry(e))),
		}
	}
}

pub enum SubFragments<'a> {
	ContextArray(std::slice::Iter<'a, ContextEntry>),
	Context(ContextSubFragments<'a>),
	Definition(Box<definition::SubItems<'a>>),
}

impl<'a> Iterator for SubFragments<'a> {
	type Item = FragmentRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::ContextArray(a) => a.next().map(FragmentRef::Context),
			Self::Context(i) => i.next(),
			Self::Definition(i) => i.next().map(FragmentRef::DefinitionFragment),
		}
	}
}

pub struct Traverse<'a> {
	stack: SmallVec<[FragmentRef<'a>; 8]>,
}

impl<'a> Traverse<'a> {
	pub(crate) fn new(item: FragmentRef<'a>) -> Self {
		let mut stack = SmallVec::new();
		stack.push(item);
		Self { stack }
	}
}

impl<'a> Iterator for Traverse<'a> {
	type Item = FragmentRef<'a>;

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

/// Context document.
///
/// A context document is a JSON-LD document containing an object with a single
/// `@context` entry.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContextDocument {
	#[cfg_attr(feature = "serde", serde(rename = "@context"))]
	pub context: Context,
}
