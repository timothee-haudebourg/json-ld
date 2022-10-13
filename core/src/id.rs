use crate::ValidReference;
use locspan::Meta;

// /// Unique identifier types.
// ///
// /// While JSON-LD uses [Internationalized Resource Identifiers (IRIs)](https://en.wikipedia.org/wiki/Internationalized_resource_identifier)
// /// to uniquely identify each node,
// /// this crate does not imposes the internal representation of identifiers.
// ///
// /// Whatever type you choose, it must implement this trait to ensure that:
// ///  - there is a low cost bijection with IRIs,
// ///  - it can be cloned ([`Clone`]),
// ///  - it can be compared ([`PartialEq`], [`Eq`]),
// ///  - it can be hashed ([`Hash`]).
// ///
// /// # Using `enum` types
// /// If you know in advance which IRIs will be used by your implementation,
// /// one possibility is to use a `enum` type as identifier.
// /// This can be done throught the use of the [`Lexicon`](`crate::Lexicon`) type along with the
// /// [`iref-enum`](https://crates.io/crates/iref-enum) crate:
// /// ```
// /// use iref_enum::*;
// /// use json_ld::Lexicon;
// /// use serde_json::Value;
// ///
// /// /// Vocabulary used in the implementation.
// /// #[derive(IriEnum, Clone, Copy, PartialEq, Eq, Hash)]
// /// #[iri_prefix("rdfs" = "http://www.w3.org/2000/01/rdf-schema#")]
// /// #[iri_prefix("manifest" = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#")]
// /// #[iri_prefix("vocab" = "https://w3c.github.io/json-ld-api/tests/vocab#")]
// /// pub enum Vocab {
// ///   #[iri("rdfs:comment")] Comment,
// ///
// ///   #[iri("manifest:name")] Name,
// ///   #[iri("manifest:entries")] Entries,
// ///   #[iri("manifest:action")] Action,
// ///   #[iri("manifest:result")] Result,
// ///
// ///   #[iri("vocab:PositiveEvaluationTest")] PositiveEvalTest,
// ///   #[iri("vocab:NegativeEvaluationTest")] NegativeEvalTest,
// ///   #[iri("vocab:option")] Option,
// ///   #[iri("vocab:specVersion")] SpecVersion,
// ///   #[iri("vocab:processingMode")] ProcessingMode,
// ///   #[iri("vocab:expandContext")] ExpandContext,
// ///   #[iri("vocab:base")] Base
// /// }
// ///
// /// /// A fully functional identifier type.
// /// pub type Id = Lexicon<Vocab>;
// ///
// /// fn handle_node(node: &json_ld::Node<Value, Id>) {
// ///   for name in node.get(Vocab::Name) { // <- note that we can directly use `Vocab` here.
// ///     println!("node name: {}", name.as_str().unwrap());
// ///   }
// /// }
// /// ```
// pub trait Id: AsIri + Clone + PartialEq + Eq + Hash {
// 	/// Create an identifier from its IRI.
// 	fn from_iri(iri: Iri) -> Self;

// 	fn from_iri_buf(iri_buf: IriBuf) -> Self {
// 		Self::from_iri(iri_buf.as_iri())
// 	}
// }

// impl Id for IriBuf {
// 	#[inline(always)]
// 	fn from_iri(iri: Iri) -> IriBuf {
// 		iri.into()
// 	}
// }

// impl<T: Id> TermLike for T {
// 	#[inline(always)]
// 	fn as_str(&self) -> &str {
// 		self.as_iri().into_str()
// 	}

// 	#[inline(always)]
// 	fn as_iri(&self) -> Option<Iri> {
// 		Some(self.as_iri())
// 	}
// }

/// Node identifier generator.
///
/// When a JSON-LD document is flattened,
/// unidentified blank nodes are assigned a blank node identifier.
/// This trait is used to abstract how
/// fresh identifiers are generated.
pub trait Generator<T, B, M, N> {
	/// Generates a new unique blank node identifier.
	fn next(&mut self, vocabulary: &mut N) -> Meta<ValidReference<T, B>, M>;
}

impl<'a, T, B, M, N, G: Generator<T, B, M, N>> Generator<T, B, M, N> for &'a mut G {
	fn next(&mut self, vocabulary: &mut N) -> Meta<ValidReference<T, B>, M> {
		(*self).next(vocabulary)
	}
}

/// Blank node identifiers built-in generators.
pub mod generator {
	use super::Generator;
	use crate::ValidReference;
	use locspan::Meta;
	use rdf_types::{BlankIdBuf, BlankIdVocabularyMut, IriVocabularyMut};

	/// Generates numbered blank node identifiers,
	/// with an optional prefix.
	///
	/// This generator can create `usize::MAX` unique blank node identifiers.
	/// If [`Generator::next`] is called `usize::MAX + 1` times, it will panic.
	#[derive(Default)]
	pub struct Blank<M> {
		metadata: M,

		/// Prefix string.
		prefix: String,

		/// Number of already generated identifiers.
		count: usize,
	}

	impl<M> Blank<M> {
		/// Creates a new numbered generator with no prefix.
		pub fn new(metadata: M) -> Self {
			Self::new_full(metadata, String::new(), 0)
		}

		/// Creates a new numbered generator with no prefix,
		/// starting with the given `offset` number.
		///
		/// The returned generator can create `usize::MAX - offset` unique blank node identifiers
		/// before panicking.
		pub fn new_with_offset(metadata: M, offset: usize) -> Self {
			Self::new_full(metadata, String::new(), offset)
		}

		/// Creates a new numbered generator with the given prefix.
		pub fn new_with_prefix(metadata: M, prefix: String) -> Self {
			Self::new_full(metadata, prefix, 0)
		}

		/// Creates a new numbered generator with the given prefix,
		/// starting with the given `offset` number.
		///
		/// The returned generator can create `usize::MAX - offset` unique blank node identifiers
		/// before panicking.
		pub fn new_full(metadata: M, prefix: String, offset: usize) -> Self {
			Self {
				metadata,
				prefix,
				count: offset,
			}
		}

		pub fn metadata(&self) -> &M {
			&self.metadata
		}

		/// Returns the prefix of this generator.
		pub fn prefix(&self) -> &str {
			&self.prefix
		}

		/// Returns the number of already generated identifiers.
		pub fn count(&self) -> usize {
			self.count
		}

		pub fn next_blank_id(&mut self) -> BlankIdBuf {
			let id =
				unsafe { BlankIdBuf::new_unchecked(format!("_:{}{}", self.prefix, self.count)) };
			self.count += 1;
			id
		}
	}

	impl<T, B, M: Clone, N: BlankIdVocabularyMut<BlankId=B>> Generator<T, B, M, N> for Blank<M> {
		fn next(&mut self, vocabulary: &mut N) -> Meta<ValidReference<T, B>, M> {
			Meta(
				ValidReference::Blank(vocabulary.insert_blank_id(&self.next_blank_id())),
				self.metadata.clone(),
			)
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
	pub enum Uuid<M> {
		/// UUIDv3.
		///
		/// You must provide a vocabulary UUID and a name.
		/// See [uuid::Uuid::new_v3] for more information.
		#[cfg(feature = "uuid-generator-v3")]
		V3(M, uuid::Uuid, String),

		/// UUIDv4.
		///
		/// See [uuid::Uuid::new_v4] for more information.
		#[cfg(feature = "uuid-generator-v4")]
		V4(M),

		/// UUIDv5.
		///
		/// You must provide a vocabulary UUID and a name.
		/// See [uuid::Uuid::new_v5] for more information.
		#[cfg(feature = "uuid-generator-v5")]
		V5(M, uuid::Uuid, String),
	}

	#[cfg(any(
		feature = "uuid-generator-v3",
		feature = "uuid-generator-v4",
		feature = "uuid-generator-v5"
	))]
	impl<M: Clone> Uuid<M> {
		pub fn next_uuid(&self) -> Meta<uuid::Uuid, M> {
			match self {
				#[cfg(feature = "uuid-generator-v3")]
				Self::V3(meta, vocabulary, name) => Meta(
					uuid::Uuid::new_v3(vocabulary, name.as_bytes()),
					meta.clone(),
				),
				#[cfg(feature = "uuid-generator-v4")]
				Self::V4(meta) => Meta(uuid::Uuid::new_v4(), meta.clone()),
				#[cfg(feature = "uuid-generator-v5")]
				Self::V5(meta, vocabulary, name) => Meta(
					uuid::Uuid::new_v5(vocabulary, name.as_bytes()),
					meta.clone(),
				),
			}
		}
	}

	#[cfg(any(
		feature = "uuid-generator-v3",
		feature = "uuid-generator-v4",
		feature = "uuid-generator-v5"
	))]
	impl<T, B, M: Clone, N: IriVocabularyMut<Iri=T>> Generator<T, B, M, N> for Uuid<M> {
		fn next(&mut self, vocabulary: &mut N) -> Meta<ValidReference<T, B>, M> {
			unsafe {
				let mut buffer = Vec::with_capacity(uuid::adapter::Urn::LENGTH);
				let ptr = buffer.as_mut_ptr();
				let capacity = buffer.capacity();
				std::mem::forget(buffer);
				let Meta(uuid, meta) = self.next_uuid();
				let len = uuid
					.to_urn()
					.encode_lower(std::slice::from_raw_parts_mut(
						ptr,
						uuid::adapter::Urn::LENGTH,
					))
					.len();
				let buffer = Vec::from_raw_parts(ptr, len, capacity);
				let p = iref::parsing::ParsedIriRef::new(&buffer).unwrap();
				let iri = iref::IriBuf::from_raw_parts(buffer, p);
				Meta(ValidReference::Id(vocabulary.insert(iri.as_iri())), meta)
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
				(),
				uuid::Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
				"test".to_string(),
			);
			for _ in 0..100 {
				let reference: ValidReference = uuid_gen.next(&mut ()).into_value();
				assert!(iref::IriBuf::new(reference.as_str()).is_ok())
			}
		}

		#[cfg(feature = "uuid-generator-v4")]
		#[test]
		fn uuidv4_iri() {
			let mut uuid_gen = Uuid::V4(());
			for _ in 0..100 {
				let reference: ValidReference = uuid_gen.next(&mut ()).into_value();
				assert!(iref::IriBuf::new(reference.as_str()).is_ok())
			}
		}

		#[cfg(feature = "uuid-generator-v5")]
		#[test]
		fn uuidv5_iri() {
			let mut uuid_gen = Uuid::V5(
				(),
				uuid::Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap(),
				"test".to_string(),
			);
			for _ in 0..100 {
				let reference: ValidReference = uuid_gen.next(&mut ()).into_value();
				assert!(iref::IriBuf::new(reference.as_str()).is_ok())
			}
		}
	}
}

pub trait IdentifyAll<T, B, M> {
	fn identify_all_in<N, G: Generator<T, B, M, N>>(&mut self, vocabulary: &mut N, generator: G)
	where
		M: Clone;

	fn identify_all<G: Generator<T, B, M, ()>>(&mut self, generator: G)
	where
		M: Clone;
}
