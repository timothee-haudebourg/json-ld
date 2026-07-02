use rdf_syntax::{Iri, IriBuf, IriRef, IriRefBuf};

mod definition;

pub use definition::*;

/// JSON-LD Context.
///
/// Can represent a single context entry, or a list of context entries.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum JsonLdContext {
	One(JsonLdContextEntry),
	Many(Vec<JsonLdContextEntry>),
}

impl Default for JsonLdContext {
	fn default() -> Self {
		Self::Many(Vec::new())
	}
}

impl JsonLdContext {
	/// Creates a new context with a single entry.
	pub fn one(context: JsonLdContextEntry) -> Self {
		Self::One(context)
	}

	/// Creates the `null` context.
	pub fn null() -> Self {
		Self::one(JsonLdContextEntry::Null)
	}

	/// Creates a new context with a single IRI-reference entry.
	pub fn iri_ref(iri_ref: IriRefBuf) -> Self {
		Self::one(JsonLdContextEntry::IriRef(iri_ref))
	}

	/// Creates a new context with a single context definition entry.
	pub fn definition(def: JsonLdContextDefinition) -> Self {
		Self::one(JsonLdContextEntry::Definition(def))
	}

	/// Deserializes an optional context, preserving `null`.
	///
	/// Maps JSON `null` to `Some(Box(Self::null()))` instead of `None`,
	/// distinguishing absent `@context` from explicit `@context: null`.
	/// Use with `#[serde(default, deserialize_with = "Context::optional")]`.
	#[cfg(feature = "serde")]
	pub fn optional<'de, D>(deserializer: D) -> Result<Option<Box<Self>>, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		<Self as serde::Deserialize>::deserialize(deserializer).map(|c| Some(Box::new(c)))
	}
}

impl JsonLdContext {
	pub fn len(&self) -> usize {
		match self {
			Self::One(_) => 1,
			Self::Many(l) => l.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::One(_) => false,
			Self::Many(l) => l.is_empty(),
		}
	}

	pub fn as_slice(&self) -> &[JsonLdContextEntry] {
		match self {
			Self::One(c) => std::slice::from_ref(c),
			Self::Many(list) => list,
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			Self::One(c) => c.is_object(),
			_ => false,
		}
	}

	pub fn is_array(&self) -> bool {
		matches!(self, Self::Many(_))
	}

	pub fn iter(&self) -> std::slice::Iter<'_, JsonLdContextEntry> {
		self.as_slice().iter()
	}
}

pub enum IntoIter {
	One(Option<JsonLdContextEntry>),
	Many(std::vec::IntoIter<JsonLdContextEntry>),
}

impl Iterator for IntoIter {
	type Item = JsonLdContextEntry;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::One(t) => t.take(),
			Self::Many(t) => t.next(),
		}
	}
}

impl IntoIterator for JsonLdContext {
	type Item = JsonLdContextEntry;
	type IntoIter = IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		match self {
			Self::One(t) => IntoIter::One(Some(t)),
			Self::Many(t) => IntoIter::Many(t.into_iter()),
		}
	}
}

impl<'a> IntoIterator for &'a JsonLdContext {
	type IntoIter = std::slice::Iter<'a, JsonLdContextEntry>;
	type Item = &'a JsonLdContextEntry;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl From<JsonLdContextEntry> for JsonLdContext {
	fn from(c: JsonLdContextEntry) -> Self {
		Self::One(c)
	}
}

impl From<IriRefBuf> for JsonLdContext {
	fn from(i: IriRefBuf) -> Self {
		Self::One(JsonLdContextEntry::IriRef(i))
	}
}

impl<'a> From<&'a IriRef> for JsonLdContext {
	fn from(i: &'a IriRef) -> Self {
		Self::One(JsonLdContextEntry::IriRef(i.to_owned()))
	}
}

impl From<IriBuf> for JsonLdContext {
	fn from(i: IriBuf) -> Self {
		Self::One(JsonLdContextEntry::IriRef(i.into()))
	}
}

impl<'a> From<&'a Iri> for JsonLdContext {
	fn from(i: &'a Iri) -> Self {
		Self::One(JsonLdContextEntry::IriRef(i.to_owned().into()))
	}
}

impl From<JsonLdContextDefinition> for JsonLdContext {
	fn from(c: JsonLdContextDefinition) -> Self {
		Self::One(JsonLdContextEntry::Definition(c))
	}
}

/// Context.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(untagged)
)]
pub enum JsonLdContextEntry {
	Null,
	IriRef(IriRefBuf),
	Definition(JsonLdContextDefinition),
}

impl JsonLdContextEntry {
	pub fn is_object(&self) -> bool {
		matches!(self, Self::Definition(_))
	}
}

impl From<IriRefBuf> for JsonLdContextEntry {
	fn from(i: IriRefBuf) -> Self {
		JsonLdContextEntry::IriRef(i)
	}
}

impl<'a> From<&'a IriRef> for JsonLdContextEntry {
	fn from(i: &'a IriRef) -> Self {
		JsonLdContextEntry::IriRef(i.to_owned())
	}
}

impl From<IriBuf> for JsonLdContextEntry {
	fn from(i: IriBuf) -> Self {
		JsonLdContextEntry::IriRef(i.into())
	}
}

impl<'a> From<&'a Iri> for JsonLdContextEntry {
	fn from(i: &'a Iri) -> Self {
		JsonLdContextEntry::IriRef(i.to_owned().into())
	}
}

impl From<JsonLdContextDefinition> for JsonLdContextEntry {
	fn from(c: JsonLdContextDefinition) -> Self {
		JsonLdContextEntry::Definition(c)
	}
}

/// Context document.
///
/// A context document is a JSON-LD document containing an object with a single
/// `@context` entry.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonLdContextDocumentValue {
	#[cfg_attr(feature = "serde", serde(rename = "@context"))]
	pub context: JsonLdContext,
}

#[cfg(feature = "serde")]
impl TryFrom<json_syntax::JsonValue> for JsonLdContextDocumentValue {
	type Error = crate::syntax::serde::DeserializeError;

	fn try_from(value: json_syntax::JsonValue) -> Result<Self, Self::Error> {
		crate::syntax::serde::from_value(value)
	}
}
