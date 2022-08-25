use crate::{Reference, ToReference};
use iref::{AsIri, Iri, IriBuf};
use json_ld_syntax::Nullable;
use rdf_types::{BlankId, BlankIdBuf};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::hash::Hash;

static mut NO_NAMESPACE: () = ();

#[inline(always)]
pub fn no_namespace_mut() -> &'static mut () {
	unsafe { &mut NO_NAMESPACE }
}

pub trait Namespace<I, B>: IriNamespace<I> + BlankIdNamespace<B> {}

pub trait NamespaceMut<I, B>:
	Namespace<I, B> + IriNamespaceMut<I> + BlankIdNamespaceMut<B>
{
}

pub trait IriNamespace<I> {
	fn iri<'i>(&'i self, id: &'i I) -> Option<Iri<'i>>;

	fn get(&self, iri: Iri) -> Option<I>;
}

pub trait IriNamespaceMut<I>: IriNamespace<I> {
	fn insert(&mut self, iri: Iri) -> I;
}

impl IriNamespace<IriBuf> for () {
	fn iri<'i>(&'i self, id: &'i IriBuf) -> Option<Iri<'i>> {
		Some(id.as_iri())
	}

	fn get(&self, iri: Iri) -> Option<IriBuf> {
		Some(iri.into())
	}
}

impl IriNamespaceMut<IriBuf> for () {
	fn insert(&mut self, iri: Iri) -> IriBuf {
		iri.into()
	}
}

pub trait BlankIdNamespace<B> {
	fn blank_id<'b>(&'b self, id: &'b B) -> Option<&'b BlankId>;

	fn get_blank_id(&self, id: &BlankId) -> Option<B>;
}

pub trait BlankIdNamespaceMut<B>: BlankIdNamespace<B> {
	fn insert_blank_id(&mut self, id: &BlankId) -> B;
}

impl BlankIdNamespace<BlankIdBuf> for () {
	fn blank_id<'b>(&'b self, id: &'b BlankIdBuf) -> Option<&'b BlankId> {
		Some(id.as_blank_id_ref())
	}

	fn get_blank_id(&self, id: &BlankId) -> Option<BlankIdBuf> {
		Some(id.to_owned())
	}
}

impl BlankIdNamespaceMut<BlankIdBuf> for () {
	fn insert_blank_id(&mut self, id: &BlankId) -> BlankIdBuf {
		id.to_owned()
	}
}

impl Namespace<IriBuf, BlankIdBuf> for () {}
impl NamespaceMut<IriBuf, BlankIdBuf> for () {}

pub trait BorrowWithNamespace {
	fn with_namespace<'n, N>(&self, namespace: &'n N) -> WithNamespace<&Self, &'n N> {
		WithNamespace(self, namespace)
	}

	fn into_with_namespace<N>(self, namespace: &N) -> WithNamespace<Self, &N>
	where
		Self: Sized,
	{
		WithNamespace(self, namespace)
	}
}

impl<T> BorrowWithNamespace for T {}

#[derive(Clone, Copy)]
pub struct WithNamespace<T, N>(pub(crate) T, pub(crate) N);

impl<T, N> std::ops::Deref for WithNamespace<T, N> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.0
	}
}

impl<T, N> std::ops::DerefMut for WithNamespace<T, N> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.0
	}
}

impl<'t, 'n, T: DisplayWithNamespace<N>, N> fmt::Display for WithNamespace<&'t T, &'n N> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt_with(self.1, f)
	}
}

pub trait DisplayWithNamespace<N> {
	fn fmt_with(&self, namespace: &N, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<T: DisplayWithNamespace<N>, N> DisplayWithNamespace<N> for Nullable<T> {
	fn fmt_with(&self, namespace: &N, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(namespace, f),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Index(usize);

impl From<usize> for Index {
	fn from(i: usize) -> Self {
		Self(i)
	}
}

impl<'a> TryFrom<Iri<'a>> for Index {
	type Error = ();

	fn try_from(_value: Iri<'a>) -> Result<Self, Self::Error> {
		Err(())
	}
}

impl IndexedIri for Index {
	fn index(&self) -> IriIndex<Iri<'_>> {
		IriIndex::Index(self.0)
	}
}

impl IndexedBlankId for Index {
	fn blank_id_index(&self) -> BlankIdIndex<&'_ BlankId> {
		BlankIdIndex::Index(self.0)
	}
}

impl<'a> TryFrom<&'a BlankId> for Index {
	type Error = ();

	fn try_from(_value: &'a BlankId) -> Result<Self, Self::Error> {
		Err(())
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum IriIndex<I> {
	Index(usize),
	Iri(I),
}

impl<I> From<usize> for IriIndex<I> {
	fn from(i: usize) -> Self {
		Self::Index(i)
	}
}

impl<'a, I: TryFrom<Iri<'a>>> TryFrom<Iri<'a>> for IriIndex<I> {
	type Error = I::Error;

	fn try_from(value: Iri<'a>) -> Result<Self, Self::Error> {
		Ok(Self::Iri(I::try_from(value)?))
	}
}

impl<I, N: IriNamespace<IriIndex<I>>> DisplayWithNamespace<N> for IriIndex<I> {
	fn fmt_with(&self, namespace: &N, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&namespace.iri(self).unwrap(), f)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum BlankIdIndex<B> {
	Index(usize),
	BlankId(B),
}

impl<I> From<usize> for BlankIdIndex<I> {
	fn from(i: usize) -> Self {
		Self::Index(i)
	}
}

impl<'a, I: TryFrom<&'a BlankId>> TryFrom<&'a BlankId> for BlankIdIndex<I> {
	type Error = I::Error;

	fn try_from(value: &'a BlankId) -> Result<Self, Self::Error> {
		Ok(Self::BlankId(I::try_from(value)?))
	}
}

impl<I, N: BlankIdNamespace<BlankIdIndex<I>>> DisplayWithNamespace<N> for BlankIdIndex<I> {
	fn fmt_with(&self, namespace: &N, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&namespace.blank_id(self).unwrap(), f)
	}
}

pub trait IndexedIri: From<usize> + for<'a> TryFrom<Iri<'a>> {
	fn index(&self) -> IriIndex<Iri<'_>>;
}

impl<I> IndexedIri for IriIndex<I>
where
	I: iref::AsIri + for<'a> TryFrom<Iri<'a>>,
{
	fn index(&self) -> IriIndex<Iri<'_>> {
		match self {
			Self::Iri(i) => IriIndex::Iri(i.as_iri()),
			Self::Index(i) => IriIndex::Index(*i),
		}
	}
}

pub trait IndexedBlankId: From<usize> + for<'a> TryFrom<&'a BlankId> {
	fn blank_id_index(&self) -> BlankIdIndex<&'_ BlankId>;
}

impl<B> IndexedBlankId for BlankIdIndex<B>
where
	B: AsRef<BlankId> + for<'a> TryFrom<&'a BlankId>,
{
	fn blank_id_index(&self) -> BlankIdIndex<&'_ BlankId> {
		match self {
			Self::BlankId(i) => BlankIdIndex::BlankId(i.as_ref()),
			Self::Index(i) => BlankIdIndex::Index(*i),
		}
	}
}

#[derive(Default)]
pub struct IndexNamespace {
	allocated: Vec<IriBuf>,
	map: HashMap<IriBuf, usize>,
	blank_allocated: Vec<BlankIdBuf>,
	blank_map: HashMap<BlankIdBuf, usize>,
}

impl IndexNamespace {
	pub fn new() -> Self {
		Self::default()
	}
}

impl<I: IndexedIri> IriNamespace<I> for IndexNamespace {
	fn iri<'i>(&'i self, id: &'i I) -> Option<Iri<'i>> {
		match id.index() {
			IriIndex::Iri(iri) => Some(iri),
			IriIndex::Index(i) => self.allocated.get(i).map(IriBuf::as_iri),
		}
	}

	fn get(&self, iri: Iri) -> Option<I> {
		match I::try_from(iri) {
			Ok(id) => Some(id),
			Err(_) => self.map.get(&iri.to_owned()).cloned().map(I::from),
		}
	}
}

impl<I: IndexedIri> IriNamespaceMut<I> for IndexNamespace {
	fn insert(&mut self, iri: Iri) -> I {
		match I::try_from(iri) {
			Ok(id) => id,
			Err(_) => I::from(*self.map.entry(iri.to_owned()).or_insert_with_key(|key| {
				let index = self.allocated.len();
				self.allocated.push(key.clone());
				index
			})),
		}
	}
}

impl<B: IndexedBlankId> BlankIdNamespace<B> for IndexNamespace {
	fn blank_id<'b>(&'b self, id: &'b B) -> Option<&'b BlankId> {
		match id.blank_id_index() {
			BlankIdIndex::BlankId(id) => Some(id),
			BlankIdIndex::Index(i) => self.blank_allocated.get(i).map(BlankIdBuf::as_blank_id_ref),
		}
	}

	fn get_blank_id(&self, blank_id: &BlankId) -> Option<B> {
		match B::try_from(blank_id) {
			Ok(id) => Some(id),
			Err(_) => self.blank_map.get(blank_id).cloned().map(B::from),
		}
	}
}

impl<B: IndexedBlankId> BlankIdNamespaceMut<B> for IndexNamespace {
	fn insert_blank_id(&mut self, blank_id: &BlankId) -> B {
		match B::try_from(blank_id) {
			Ok(id) => id,
			Err(_) => B::from(
				*self
					.blank_map
					.entry(blank_id.to_owned())
					.or_insert_with_key(|key| {
						let index = self.blank_allocated.len();
						self.blank_allocated.push(key.clone());
						index
					}),
			),
		}
	}
}

impl<I: IndexedIri, B: IndexedBlankId> Namespace<I, B> for IndexNamespace {}

impl<I: IndexedIri, B: IndexedBlankId> NamespaceMut<I, B> for IndexNamespace {}

/// Lexicon identifier.
///
/// # Example
/// The following example builds a lexicon from a statically known vocabulary, defined as an
/// `enum` type. It uses the [`iref-enum`](https://crates.io/crates/iref-enum)
/// crate to automatically derive the conversion of the from/into IRIs.
/// ```
/// # use json_ld_core as json_ld;
/// use iref_enum::*;
/// use json_ld::namespace::Lexicon;
///
/// /// Vocabulary used in the implementation.
/// #[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
/// #[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
/// #[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
/// #[iri_prefix("vocab" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
/// pub enum Vocab {
///     #[iri("rdfs:comment")] Comment,
///
///     #[iri("manifest:name")] Name,
///     #[iri("manifest:entries")] Entries,
///     #[iri("manifest:action")] Action,
///     #[iri("manifest:result")] Result,
///
///     #[iri("vocab:PositiveEvaluationTest")] PositiveEvalTest,
///     #[iri("vocab:NegativeEvaluationTest")] NegativeEvalTest,
///     #[iri("vocab:option")] Option,
///     #[iri("vocab:specVersion")] SpecVersion,
///     #[iri("vocab:processingMode")] ProcessingMode,
///     #[iri("vocab:expandContext")] ExpandContext,
///     #[iri("vocab:base")] Base
/// }
///
/// /// A fully functional identifier type.
/// pub type Id = Lexicon<Vocab>;
///
/// fn handle_node(node: &json_ld::Node<Id>) {
///   for name in node.get(Vocab::Name) { // <- note that we can directly use `Vocab` here.
///     println!("node name: {}", name.as_value().unwrap().as_str().unwrap());
///   }
/// }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub enum Lexicon<V> {
	/// Identifier from the known vocabulary.
	Id(V),

	/// Any other IRI outside of the vocabulary.
	Iri(IriBuf),
}

impl<V> Lexicon<V> {
	pub fn as_str(&self) -> &str
	where
		V: AsRef<str>,
	{
		match self {
			Self::Id(i) => i.as_ref(),
			Self::Iri(i) => i.as_str(),
		}
	}

	pub fn as_iri(&self) -> Iri
	where
		V: AsIri,
	{
		match self {
			Self::Id(i) => i.as_iri(),
			Self::Iri(i) => i.as_iri(),
		}
	}
}

impl<V: AsIri> AsIri for Lexicon<V> {
	fn as_iri(&self) -> Iri {
		self.as_iri()
	}
}

impl<V: Hash> Hash for Lexicon<V> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Id(i) => i.hash(state),
			Self::Iri(i) => i.hash(state),
		}
	}
}

impl<T, B> ToReference<Lexicon<T>, B> for T {
	type Reference = Reference<Lexicon<T>, B>;

	fn to_ref(self) -> Self::Reference {
		Reference::Id(Lexicon::Id(self))
	}
}

impl<V: fmt::Display> fmt::Display for Lexicon<V> {
	#[inline]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Lexicon::Id(id) => id.fmt(f),
			Lexicon::Iri(iri) => iri.fmt(f),
		}
	}
}
