use iref::{iri, Iri, IriBuf};

pub const PROFILE_EXPANDED_IRI: &Iri = iri!("http://www.w3.org/ns/json-ld#expanded");
pub const PROFILE_COMPACTED_IRI: &Iri = iri!("http://www.w3.org/ns/json-ld#compacted");
pub const PROFILE_CONTEXT_IRI: &Iri = iri!("http://www.w3.org/ns/json-ld#context");
pub const PROFILE_FLATTENED_IRI: &Iri = iri!("http://www.w3.org/ns/json-ld#flattened");
pub const PROFILE_FRAMED_IRI: &Iri = iri!("http://www.w3.org/ns/json-ld#framed");

/// Standard `profile` parameter values defined for the `application/ld+json`.
///
/// See: <https://www.w3.org/TR/json-ld11/#iana-considerations>
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StandardProfile {
	/// To request or specify expanded JSON-LD document form.
	Expanded,

	/// To request or specify compacted JSON-LD document form.
	Compacted,

	/// To request or specify a JSON-LD context document.
	Context,

	/// To request or specify flattened JSON-LD document form.
	Flattened,

	// /// To request or specify a JSON-LD frame document.
	// Frame,
	/// To request or specify a JSON-LD framed document.
	Framed,
}

impl StandardProfile {
	pub fn from_iri(iri: &Iri) -> Option<Self> {
		if iri == PROFILE_EXPANDED_IRI {
			Some(Self::Expanded)
		} else if iri == PROFILE_COMPACTED_IRI {
			Some(Self::Compacted)
		} else if iri == PROFILE_CONTEXT_IRI {
			Some(Self::Context)
		} else if iri == PROFILE_FLATTENED_IRI {
			Some(Self::Flattened)
		} else if iri == PROFILE_FLATTENED_IRI {
			Some(Self::Framed)
		} else {
			None
		}
	}

	pub fn iri(&self) -> &'static Iri {
		match self {
			Self::Expanded => PROFILE_CONTEXT_IRI,
			Self::Compacted => PROFILE_COMPACTED_IRI,
			Self::Context => PROFILE_CONTEXT_IRI,
			Self::Flattened => PROFILE_FLATTENED_IRI,
			Self::Framed => PROFILE_FRAMED_IRI,
		}
	}
}

/// Value for the `profile` parameter defined for the `application/ld+json`.
///
/// Standard values defined by the JSON-LD specification are defined by the
/// [`StandardProfile`] type.
///
/// See: <https://www.w3.org/TR/json-ld11/#iana-considerations>
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Profile(Inner);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Inner {
	Standard(StandardProfile),
	Custom(IriBuf),
}

impl Profile {
	pub const EXPANDED: Self = Self(Inner::Standard(StandardProfile::Expanded));
	pub const COMPACTED: Self = Self(Inner::Standard(StandardProfile::Compacted));
	pub const CONTEXT: Self = Self(Inner::Standard(StandardProfile::Context));
	pub const FLATTENED: Self = Self(Inner::Standard(StandardProfile::Flattened));
	pub const FRAMED: Self = Self(Inner::Standard(StandardProfile::Framed));

	pub fn new(iri: &Iri) -> Self {
		match StandardProfile::from_iri(iri) {
			Some(p) => Self(Inner::Standard(p)),
			None => Self(Inner::Custom(iri.to_owned())),
		}
	}

	pub fn iri(&self) -> &Iri {
		match &self.0 {
			Inner::Standard(s) => s.iri(),
			Inner::Custom(c) => c,
		}
	}

	pub fn as_standard(&self) -> Option<StandardProfile> {
		match &self.0 {
			Inner::Standard(s) => Some(*s),
			Inner::Custom(_) => None,
		}
	}

	pub fn into_standard(self) -> Result<StandardProfile, IriBuf> {
		match self.0 {
			Inner::Standard(s) => Ok(s),
			Inner::Custom(e) => Err(e),
		}
	}
}
