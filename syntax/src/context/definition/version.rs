use locspan_derive::StrippedPartialEq;
use std::hash::Hash;

/// Version number.
///
/// The only allowed value is a number with the value `1.1`.
#[derive(Clone, Copy, StrippedPartialEq, PartialOrd, Ord, Debug)]
pub enum Version {
	V1_1,
}

impl Version {
	pub fn into_bytes(self) -> &'static [u8] {
		match self {
			Self::V1_1 => b"1.1",
		}
	}

	pub fn into_str(self) -> &'static str {
		match self {
			Self::V1_1 => "1.1",
		}
	}

	pub fn into_json_number(self) -> &'static json_syntax::Number {
		unsafe { json_syntax::Number::new_unchecked(self.into_bytes()) }
	}

	pub fn into_json_number_buf(self) -> json_syntax::NumberBuf {
		unsafe { json_syntax::NumberBuf::new_unchecked(self.into_bytes().into()) }
	}
}

impl PartialEq for Version {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl Eq for Version {}

impl Hash for Version {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.into_str().hash(state)
	}
}

impl<'a> From<Version> for &'a json_syntax::Number {
	fn from(v: Version) -> Self {
		v.into_json_number()
	}
}

impl From<Version> for json_syntax::NumberBuf {
	fn from(v: Version) -> Self {
		v.into_json_number_buf()
	}
}
