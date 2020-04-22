use std::fmt;
use std::convert::TryFrom;
use json::JsonValue;
use crate::util;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BlankId(String);

impl BlankId {
	pub fn new(name: &str) -> BlankId {
		BlankId("_:".to_string() + name)
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn name(&self) -> &str {
		&self.0[2..self.0.len()]
	}
}

impl<'a> TryFrom<&'a str> for BlankId {
	type Error = ();

	fn try_from(str: &'a str) -> Result<BlankId, ()> {
		if let Some(name) = str.strip_prefix("_:") {
			Ok(BlankId::new(name))
		} else {
			Err(())
		}
	}
}

impl util::AsJson for BlankId {
	fn as_json(&self) -> JsonValue {
		self.0.as_json()
	}
}

impl fmt::Display for BlankId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}
