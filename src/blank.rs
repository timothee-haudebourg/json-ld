use crate::util;
use generic_json::Json;
use std::convert::TryFrom;
use std::fmt;

/// Blank node identifier.
///
/// Blank nodes are non-uniquely identified nodes that are local to a JSON-LD document.
/// ```json
/// {
///   "@id": "_:node1",
///   "name": "Local blank node 1",
///   "knows": {
///     "name": "Local blank node 2, that needs to refer to local node 1",
///     "knows": { "@id": "_:node1" }
///   }
/// }
/// ```
/// This type represent a blank node identifier of the form `_:name`.
/// It is used by the `Reference` type to reference blank and non-blank nodes.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct BlankId(String);

impl BlankId {
	/// Create a new blank identifier from a given `name`.
	///
	/// The created blank node will be of the form `_:name`.
	pub fn new(name: &str) -> BlankId {
		BlankId("_:".to_string() + name)
	}

	/// Get the blank identifier as a string.
	///
	/// This includes the `_:` prefix.
	/// Use [`BlankId::name`] to get the suffix part only.
	pub fn as_str(&self) -> &str {
		&self.0
	}

	/// Get the name/suffix part of the identifier.
	///
	/// For a blank identifier `_:name`, this returns a string slice to `name`.
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

impl<J: Json> util::AsJson<J> for BlankId {
	/// Returns a JSON string of the form `_:name`.
	fn as_json_with<M>(&self, meta: M) -> J
	where
		M: Clone + Fn() -> J::MetaData,
	{
		// self.0.as_json_with(meta)
		panic!("TODO BlankID as json")
	}
}

impl fmt::Display for BlankId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}
