use crate::{syntax::TermLike, Reference};
use generic_json::JsonBuild;
use iref::{AsIri, Iri, IriBuf};
use std::hash::Hash;

/// Unique identifier types.
///
/// While JSON-LD uses [Internationalized Resource Identifiers (IRIs)](https://en.wikipedia.org/wiki/Internationalized_resource_identifier)
/// to uniquely identify each node,
/// this crate does not imposes the internal representation of identifiers.
///
/// Whatever type you choose, it must implement this trait to usure that:
///  - there is a low cost bijection with IRIs,
///  - it can be cloned ([`Clone`]),
///  - it can be compared ([`PartialEq`], [`Eq`]),
///  - it can be hashed ([`Hash`]).
///
/// # Using `enum` types
/// If you know in advance which IRIs will be used by your implementation,
/// one possibility is to use a `enum` type as identifier.
/// This can be done throught the use of the [`Lexicon`](`crate::Lexicon`) type along with the
/// [`iref-enum`](https://crates.io/crates/iref-enum) crate:
/// ```
/// use iref_enum::*;
/// use json_ld::Lexicon;
/// use ijson::IValue;
///
/// /// Vocabulary used in the implementation.
/// #[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
/// #[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
/// #[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
/// #[iri_prefix("vocab" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
/// pub enum Vocab {
///   #[iri("rdfs:comment")] Comment,
///
///   #[iri("manifest:name")] Name,
///   #[iri("manifest:entries")] Entries,
///   #[iri("manifest:action")] Action,
///   #[iri("manifest:result")] Result,
///
///   #[iri("vocab:PositiveEvaluationTest")] PositiveEvalTest,
///   #[iri("vocab:NegativeEvaluationTest")] NegativeEvalTest,
///   #[iri("vocab:option")] Option,
///   #[iri("vocab:specVersion")] SpecVersion,
///   #[iri("vocab:processingMode")] ProcessingMode,
///   #[iri("vocab:expandContext")] ExpandContext,
///   #[iri("vocab:base")] Base
/// }
///
/// /// A fully functional identifier type.
/// pub type Id = Lexicon<Vocab>;
///
/// fn handle_node(node: &json_ld::Node<IValue, Id>) {
///   for name in node.get(Vocab::Name) { // <- note that we can directly use `Vocab` here.
///     println!("node name: {}", name.as_str().unwrap());
///   }
/// }
/// ```
pub trait Id: AsIri + Clone + PartialEq + Eq + Hash {
	/// Create an identifier from its IRI.
	fn from_iri(iri: Iri) -> Self;

	fn from_iri_buf(iri_buf: IriBuf) -> Self {
		Self::from_iri(iri_buf.as_iri())
	}

	#[inline(always)]
	fn as_json<K: JsonBuild>(&self, meta: K::MetaData) -> K {
		K::string(self.as_iri().as_str().into(), meta)
	}
}

impl Id for IriBuf {
	#[inline(always)]
	fn from_iri(iri: Iri) -> IriBuf {
		iri.into()
	}
}

impl<T: Id> TermLike for T {
	#[inline(always)]
	fn as_str(&self) -> &str {
		self.as_iri().into_str()
	}

	#[inline(always)]
	fn as_iri(&self) -> Option<Iri> {
		Some(self.as_iri())
	}
}

/// Node identifier generator.
///
/// When a JSON-LD document is flattened,
/// unidentified blank nodes are assigned a blank node identifier.
/// This trait is used to abstract how
/// fresh identifiers are generated.
pub trait Generator<T: Id> {
	/// Generates a new unique blank node identifier.
	fn next(&mut self) -> Reference<T>;
}

impl<'a, T: Id, G: Generator<T>> Generator<T> for &'a mut G {
	fn next(&mut self) -> Reference<T> {
		(*self).next()
	}
}

/// Blank node identifiers built-in generators.
pub mod generator {
	use super::Generator;
	use crate::{BlankId, Id, Reference};

	/// Generates numbered blank node identifiers,
	/// with an optional prefix.
	///
	/// This generator can create `usize::MAX` unique blank node identifiers.
	/// If [`Generator::next`] is called `usize::MAX + 1` times, it will panic.
	#[derive(Default)]
	pub struct Blank {
		/// Prefix string.
		prefix: String,

		/// Number of already generated identifiers.
		count: usize,
	}

	impl Blank {
		/// Creates a new numbered generator with no prefix.
		pub fn new() -> Self {
			Self::new_full(String::new(), 0)
		}

		/// Creates a new numbered generator with no prefix,
		/// starting with the given `offset` number.
		///
		/// The returned generator can create `usize::MAX - offset` unique blank node identifiers
		/// before panicking.
		pub fn new_with_offset(offset: usize) -> Self {
			Self::new_full(String::new(), offset)
		}

		/// Creates a new numbered generator with the given prefix.
		pub fn new_with_prefix(prefix: String) -> Self {
			Self::new_full(prefix, 0)
		}

		/// Creates a new numbered generator with the given prefix,
		/// starting with the given `offset` number.
		///
		/// The returned generator can create `usize::MAX - offset` unique blank node identifiers
		/// before panicking.
		pub fn new_full(prefix: String, offset: usize) -> Self {
			Self {
				prefix,
				count: offset,
			}
		}

		/// Returns the prefix of this generator.
		pub fn prefix(&self) -> &str {
			&self.prefix
		}

		/// Returns the number of already generated identifiers.
		pub fn count(&self) -> usize {
			self.count
		}

		pub fn next_blank_id(&mut self) -> BlankId {
			unsafe { BlankId::from_raw(format!("_:{}{}", self.prefix, self.count)) }
		}
	}

	impl<T: Id> Generator<T> for Blank {
		fn next(&mut self) -> Reference<T> {
			Reference::Blank(self.next_blank_id())
		}
	}

	/// Generates UUID blank node identifiers based on the [`uuid`](https://crates.io/crates/uuid) crate.
	///
	/// This is an enum type with different UUID versions supported
	/// by the `uuid` library, so you can choose which kind of UUID
	/// better fits your application.
	/// Version 1 is not supported.
	///
	/// You need to enable the `uuid-generator` feature to
	/// use this type.
	/// You also need to enable the features of each version you need
	/// in the `uuid` crate.
	pub enum Uuid {
		/// UUIDv3.
		///
		/// You must provide a namespace UUID and a name.
		/// See [uuid::Uuid::new_v3] for more information.
		#[cfg(feature = "uuid-generator-v3")]
		V3(uuid::Uuid, String),

		/// UUIDv4.
		///
		/// See [uuid::Uuid::new_v4] for more information.
		#[cfg(feature = "uuid-generator-v4")]
		V4,

		/// UUIDv5.
		///
		/// You must provide a namespace UUID and a name.
		/// See [uuid::Uuid::new_v5] for more information.
		#[cfg(feature = "uuid-generator-v5")]
		V5(uuid::Uuid, String),
	}

	#[cfg(any(
		feature = "uuid-generator-v3",
		feature = "uuid-generator-v4",
		feature = "uuid-generator-v5"
	))]
	impl Uuid {
		pub fn next_uuid(&self) -> uuid::Uuid {
			match self {
				#[cfg(feature = "uuid-generator-v3")]
				Self::V3(namespace, name) => uuid::Uuid::new_v3(namespace, name.as_bytes()),
				#[cfg(feature = "uuid-generator-v4")]
				Self::V4 => uuid::Uuid::new_v4(),
				#[cfg(feature = "uuid-generator-v5")]
				Self::V5(namespace, name) => uuid::Uuid::new_v5(namespace, name.as_bytes()),
			}
		}
	}

	#[cfg(any(
		feature = "uuid-generator-v3",
		feature = "uuid-generator-v4",
		feature = "uuid-generator-v5"
	))]
	impl<T: Id> Generator<T> for Uuid {
		fn next(&mut self) -> Reference<T> {
			unsafe {
				let mut buffer = Vec::with_capacity(uuid::adapter::Urn::LENGTH);
				let ptr = buffer.as_mut_ptr();
				let capacity = buffer.capacity();
				std::mem::forget(buffer);
				let len = self
					.next_uuid()
					.to_urn()
					.encode_lower(std::slice::from_raw_parts_mut(
						ptr,
						uuid::adapter::Urn::LENGTH,
					))
					.len();
				let buffer = Vec::from_raw_parts(ptr, len, capacity);
				let p = iref::parsing::ParsedIriRef::new(&buffer).unwrap();
				let iri = iref::IriBuf::from_raw_parts(buffer, p);
				Reference::Id(T::from_iri_buf(iri))
			}
		}
	}

	#[cfg(any(
		feature = "uuid-generator-v3",
		feature = "uuid-generator-v4",
		feature = "uuid-generator-v5"
	))]
	#[cfg(test)]
	mod tests {
		use super::*;

		#[cfg(feature = "uuid-generator-v3")]
		#[test]
		fn uuidv3_iri() {
			let mut uuid_gen = Uuid::V3(
				uuid::Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
				"test".to_string(),
			);
			for _ in 0..100 {
				let reference: Reference = uuid_gen.next();
				assert!(iref::IriBuf::new(reference.as_str()).is_ok())
			}
		}

		#[cfg(feature = "uuid-generator-v4")]
		#[test]
		fn uuidv4_iri() {
			let mut uuid_gen = Uuid::V4;
			for _ in 0..100 {
				let reference: Reference = uuid_gen.next();
				assert!(iref::IriBuf::new(reference.as_str()).is_ok())
			}
		}

		#[cfg(feature = "uuid-generator-v5")]
		#[test]
		fn uuidv5_iri() {
			let mut uuid_gen = Uuid::V5(
				uuid::Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
				"test".to_string(),
			);
			for _ in 0..100 {
				let reference: Reference = uuid_gen.next();
				assert!(iref::IriBuf::new(reference.as_str()).is_ok())
			}
		}
	}
}
