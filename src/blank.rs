use crate::util;
use generic_json::JsonBuild;
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
	#[inline(always)]
	pub fn new(name: &str) -> BlankId {
		BlankId("_:".to_string() + name)
	}

	/// Get the blank identifier as a string.
	///
	/// This includes the `_:` prefix.
	/// Use [`BlankId::name`] to get the suffix part only.
	#[inline(always)]
	pub fn as_str(&self) -> &str {
		&self.0
	}

	/// Get the name/suffix part of the identifier.
	///
	/// For a blank identifier `_:name`, this returns a string slice to `name`.
	#[inline(always)]
	pub fn name(&self) -> &str {
		&self.0[2..self.0.len()]
	}
}

impl<'a> TryFrom<&'a str> for BlankId {
	type Error = ();

	#[inline(always)]
	fn try_from(str: &'a str) -> Result<BlankId, ()> {
		if let Some(name) = str.strip_prefix("_:") {
			Ok(BlankId::new(name))
		} else {
			Err(())
		}
	}
}

impl<K: JsonBuild> util::AsAnyJson<K> for BlankId {
	/// Returns a JSON string of the form `_:name`.
	#[inline(always)]
	fn as_json_with(&self, meta: K::MetaData) -> K {
		self.0.as_json_with(meta)
	}
}

impl fmt::Display for BlankId {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt(f)
	}
}
