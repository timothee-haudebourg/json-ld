use iref::{IriRef, IriRefBuf};

pub struct InvalidCompactIri<T>(pub T);

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct CompactIri(str);

impl CompactIri {
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
		std::mem::transmute(s)
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

	pub fn as_iri_ref(&self) -> &IriRef {
		IriRef::new(self.as_str()).unwrap()
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
	pub fn new(s: String) -> Result<Self, InvalidCompactIri<String>> {
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
