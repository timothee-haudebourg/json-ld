use std::borrow::Cow;

use rdf_syntax::{InvalidIri, Iri, IriBuf, IriRef, IriRefBuf};

#[derive(Debug, thiserror::Error)]
#[error("invalid compact IRI")]
pub struct InvalidCompactIri<T>(pub T);

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct CompactIri(str);

impl CompactIri {
	/// Checks that `s` has the syntactic shape of a compact IRI, namely
	/// `prefix:suffix`, where `prefix` is not `_` (which denotes a blank
	/// node identifier instead) and `suffix` does not start with `//`
	/// (which would make it ambiguous with an absolute IRI with an
	/// authority).
	///
	/// A compact IRI's `prefix` is a term name, resolved against the active
	/// context, so it is **not** required to also be a syntactically valid
	/// IRI on its own: term names may use characters, such as `_`, that are
	/// not permitted in an IRI scheme. For instance `my_term:value` is a
	/// valid compact IRI but not a valid absolute IRI. See
	/// [§ 4.1.5 Compact IRIs](https://www.w3.org/TR/json-ld11/#compact-iris)
	/// of the JSON-LD 1.1 specification.
	pub fn new(s: &str) -> Result<&Self, InvalidCompactIri<&str>> {
		match s.split_once(':') {
			Some((prefix, suffix)) if prefix != "_" && !suffix.starts_with("//") => {
				match IriRef::new(s) {
					Ok(_) => Ok(unsafe { Self::new_unchecked(s) }),
					Err(_) => Err(InvalidCompactIri(s)),
				}
			}
			_ => Err(InvalidCompactIri(s)),
		}
	}

	/// Creates a new compact IRI without parsing it.
	///
	/// # Safety
	///
	/// The input string must be a compact IRI.
	pub unsafe fn new_unchecked(s: &str) -> &Self {
		unsafe { std::mem::transmute(s) }
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn to_owned(&self) -> CompactIriBuf {
		CompactIriBuf(self.0.to_owned())
	}

	pub fn prefix(&self) -> &str {
		let i = self.find(':').unwrap();
		&self[0..i]
	}

	pub fn suffix(&self) -> &str {
		let i = self.find(':').unwrap();
		&self[i + 1..]
	}

	/// Returns this compact IRI as an absolute IRI, if it happens to also be
	/// a syntactically valid one.
	///
	/// This is `None` whenever `prefix` is not itself a valid IRI scheme
	/// (e.g. because it contains a `_`). To properly turn a compact IRI into
	/// an absolute IRI, resolve its prefix against the active context and
	/// use [`Self::resolve`] instead.
	pub fn as_iri(&self) -> Option<&Iri> {
		Iri::new(self.as_str()).ok()
	}

	/// Returns this compact IRI as an IRI reference, if it happens to also
	/// be a syntactically valid one. See [`Self::as_iri`] for why this may
	/// be `None`.
	pub fn as_iri_ref(&self) -> Option<&IriRef> {
		IriRef::new(self.as_str()).ok()
	}

	pub fn resolve<'a>(
		&self,
		resolver: impl FnOnce(&str) -> Option<Cow<'a, Iri>>,
	) -> Result<IriBuf, InvalidIri<String>> {
		match resolver(self.prefix()) {
			Some(prefix) => IriBuf::from_string(format!("{}{}", prefix, self.suffix())),
			None => IriBuf::from_string(self.as_str().to_owned()),
		}
	}
}

impl std::ops::Deref for CompactIri {
	type Target = str;

	fn deref(&self) -> &str {
		&self.0
	}
}

impl std::borrow::Borrow<str> for CompactIri {
	fn borrow(&self) -> &str {
		&self.0
	}
}

impl AsRef<str> for CompactIri {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct CompactIriBuf(String);

impl CompactIriBuf {
	pub fn from_string(s: String) -> Result<Self, InvalidCompactIri<String>> {
		match CompactIri::new(&s) {
			Ok(_) => Ok(unsafe { Self::new_unchecked(s) }),
			Err(_) => Err(InvalidCompactIri(s)),
		}
	}

	/// Creates a new compact IRI without parsing it.
	///
	/// # Safety
	///
	/// The input string must be a compact IRI.
	pub unsafe fn new_unchecked(s: String) -> Self {
		Self(s)
	}

	pub fn as_compact_iri(&self) -> &CompactIri {
		unsafe { CompactIri::new_unchecked(&self.0) }
	}

	pub fn into_iri_ref(self) -> IriRefBuf {
		IriRefBuf::new(self.0).unwrap()
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

impl std::borrow::Borrow<CompactIri> for CompactIriBuf {
	fn borrow(&self) -> &CompactIri {
		self.as_compact_iri()
	}
}

impl std::ops::Deref for CompactIriBuf {
	type Target = CompactIri;

	fn deref(&self) -> &CompactIri {
		self.as_compact_iri()
	}
}
