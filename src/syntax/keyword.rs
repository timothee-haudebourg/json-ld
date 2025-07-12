use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub struct NotAKeyword<T = String>(pub T);

impl<T: ?Sized + ToOwned> NotAKeyword<&T> {
	pub fn into_owned(self) -> NotAKeyword<T::Owned> {
		NotAKeyword(self.0.to_owned())
	}
}

macro_rules! keyword {
	{
		$(
			$(#[$meta:meta])*
			$ident:ident : $lit:literal
		),*
	} => {
		/// JSON-LD keywords.
		#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
		#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
		pub enum Keyword {
			$(
				$(#[$meta])*
				#[cfg_attr(feature = "serde", serde(rename = $lit))]
				$ident
			),*
		}

		impl Keyword {
			pub fn into_str(self) -> &'static str {
				match self {
					$(
						Self::$ident => $lit
					),*
				}
			}
		}

		impl<'a> TryFrom<&'a str> for Keyword {
			type Error = NotAKeyword<&'a str>;

			fn try_from(input: &'a str) -> Result<Keyword, NotAKeyword<&'a str>> {
				match input {
					$(
						$lit => Ok(Self::$ident),
					)*
					_ => Err(NotAKeyword(input))
				}
			}
		}
    };
}

keyword! {
	/// `@base`.
	/// Used to set the base IRI against which to resolve those relative IRI references
	/// which are otherwise interpreted relative to the document.
	Base : "@base",

	/// `@container`.
	/// Used to set the default container type for a term.
	Container : "@container",

	/// `@context`.
	/// Used to define the short-hand names that are used throughout a JSON-LD document.
	Context : "@context",

	/// `@direction`.
	/// Used to set the base direction of a JSON-LD value, which are not typed values.
	/// (e.g. strings, or language-tagged strings).
	Direction : "@direction",

	/// `@graph`.
	/// Used to express a graph.
	Graph : "@graph",

	/// `@id`.
	/// Used to uniquely identify node objects that are being described in the document with IRIs
	/// or blank node identifiers.
	Id : "@id",

	/// `@import`.
	/// Used in a context definition to load an external context within which the containing
	/// context definition is merged.
	Import : "@import",

	/// `@included`.
	/// Used in a top-level node object to define an included block, for including secondary node
	/// objects within another node object.
	Included : "@included",

	/// `@index`.
	/// Used to specify that a container is used to index information and that processing should
	/// continue deeper into a JSON data structure.
	Index : "@index",

	/// `@json`.
	/// Used as the @type value of a JSON literal.
	Json : "@json",

	/// `@language`.
	/// Used to specify the language for a particular string value or the default language of a
	/// JSON-LD document.
	Language : "@language",

	/// `@list`.
	/// Used to express an ordered set of data.
	List : "@list",

	/// `@nest`.
	/// Used to define a property of a node object that groups together properties of that node,
	/// but is not an edge in the graph.
	Nest : "@nest",

	/// `@none`.
	/// Used as an index value in an index map, id map, language map, type map, or elsewhere where
	/// a map is used to index into other values, when the indexed node does not have the feature
	/// being indexed.
	None : "@none",

	/// `@prefix`.
	/// With the value true, allows this term to be used to construct a compact IRI when
	/// compacting.
	Prefix : "@prefix",

	/// `@propagate`.
	/// Used in a context definition to change the scope of that context.
	///
	/// By default, it is true, meaning that contexts propagate across node objects
	/// (other than for type-scoped contexts, which default to false).
	/// Setting this to false causes term definitions created within that context to be removed
	/// when entering a new node object.
	Propagate : "@propagate",

	/// `@protected`.
	/// Used to prevent term definitions of a context to be overridden by other contexts.
	Protected : "@protected",

	/// `@reverse`.
	/// Used to express reverse properties.
	Reverse : "@reverse",

	/// `@set`.
	/// Used to express an unordered set of data and to ensure that values are always represented
	/// as arrays.
	Set : "@set",

	/// `@type`.
	/// Used to set the type of a node or the datatype of a typed value.
	Type : "@type",

	/// `@value`.
	/// Used to specify the data that is associated with a particular property in the graph.
	Value : "@value",

	/// `@version`.
	/// Used in a context definition to set the processing mode.
	Version : "@version",

	/// `@vocab`.
	/// Used to expand properties and values in @type with a common prefix IRI.
	Vocab : "@vocab"
}

impl Keyword {
	pub fn as_str(&self) -> &'static str {
		self.into_str()
	}
}

impl FromStr for Keyword {
	type Err = NotAKeyword;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::try_from(s).map_err(NotAKeyword::into_owned)
	}
}

impl From<Keyword> for &'static str {
	fn from(k: Keyword) -> &'static str {
		k.into_str()
	}
}

impl fmt::Display for Keyword {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.into_str().fmt(f)
	}
}

impl PartialEq<str> for Keyword {
	fn eq(&self, other: &str) -> bool {
		self.as_str() == other
	}
}

impl PartialEq<&str> for Keyword {
	fn eq(&self, other: &&str) -> bool {
		self.as_str() == *other
	}
}

impl PartialEq<Keyword> for str {
	fn eq(&self, other: &Keyword) -> bool {
		self == other.as_str()
	}
}

impl PartialEq<Keyword> for &str {
	fn eq(&self, other: &Keyword) -> bool {
		*self == other.as_str()
	}
}

pub fn is_keyword(str: &str) -> bool {
	Keyword::try_from(str).is_ok()
}

fn is_alpha(c: char) -> bool {
	let c = c as u32;
	(0x41..=0x5a).contains(&c) || (0x61..=0x7a).contains(&c)
}

pub fn is_keyword_like(s: &str) -> bool {
	if s.len() > 1 {
		for (i, c) in s.chars().enumerate() {
			if (i == 0 && c != '@') || (i > 0 && !is_alpha(c)) {
				return false;
			}
		}

		true
	} else {
		false
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct KeywordType;

impl KeywordType {
	pub fn as_str(&self) -> &'static str {
		"@type"
	}
}

impl Borrow<str> for KeywordType {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}
