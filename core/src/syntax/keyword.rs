use crate::utils;
use generic_json::JsonBuild;
use std::convert::TryFrom;
use std::fmt;

/// JSON-LD keywords.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Keyword {
	/// `@base`.
	/// Used to set the base IRI against which to resolve those relative IRI references
	/// which are otherwise interpreted relative to the document.
	Base,

	/// `@container`.
	/// Used to set the default container type for a term.
	Container,

	/// `@context`.
	/// Used to define the short-hand names that are used throughout a JSON-LD document.
	Context,

	/// `@direction`.
	/// Used to set the base direction of a JSON-LD value, which are not typed values.
	/// (e.g. strings, or language-tagged strings).
	Direction,

	/// `@graph`.
	/// Used to express a graph.
	Graph,

	/// `@id`.
	/// Used to uniquely identify node objects that are being described in the document with IRIs
	/// or blank node identifiers.
	Id,

	/// `@import`.
	/// Used in a context definition to load an external context within which the containing
	/// context definition is merged.
	Import,

	/// `@included`.
	/// Used in a top-level node object to define an included block, for including secondary node
	/// objects within another node object.
	Included,

	/// `@index`.
	/// Used to specify that a container is used to index information and that processing should
	/// continue deeper into a JSON data structure.
	Index,

	/// `@json`.
	/// Used as the @type value of a JSON literal.
	Json,

	/// `@language`.
	/// Used to specify the language for a particular string value or the default language of a
	/// JSON-LD document.
	Language,

	/// `@list`.
	/// Used to express an ordered set of data.
	List,

	/// `@nest`.
	/// Used to define a property of a node object that groups together properties of that node,
	/// but is not an edge in the graph.
	Nest,

	/// `@none`.
	/// Used as an index value in an index map, id map, language map, type map, or elsewhere where
	/// a map is used to index into other values, when the indexed node does not have the feature
	/// being indexed.
	None,

	/// `@prefix`.
	/// With the value true, allows this term to be used to construct a compact IRI when
	/// compacting.
	Prefix,

	/// `@propagate`.
	/// Used in a context definition to change the scope of that context.
	///
	/// By default, it is true, meaning that contexts propagate across node objects
	/// (other than for type-scoped contexts, which default to false).
	/// Setting this to false causes term definitions created within that context to be removed
	/// when entering a new node object.
	Propagate,

	/// `@protected`.
	/// Used to prevent term definitions of a context to be overridden by other contexts.
	Protected,

	/// `@reverse`.
	/// Used to express reverse properties.
	Reverse,

	/// `@set`.
	/// Used to express an unordered set of data and to ensure that values are always represented
	/// as arrays.
	Set,

	/// `@type`.
	/// Used to set the type of a node or the datatype of a typed value.
	Type,

	/// `@value`.
	/// Used to specify the data that is associated with a particular property in the graph.
	Value,

	/// `@version`.
	/// Used in a context definition to set the processing mode.
	Version,

	/// `@vocab`.
	/// Used to expand properties and values in @type with a common prefix IRI.
	Vocab,
}

impl Keyword {
	pub fn into_str(self) -> &'static str {
		use Keyword::*;
		match self {
			Base => "@base",
			Container => "@container",
			Context => "@context",
			Direction => "@direction",
			Graph => "@graph",
			Id => "@id",
			Import => "@import",
			Included => "@included",
			Index => "@index",
			Json => "@json",
			Language => "@language",
			List => "@list",
			Nest => "@nest",
			None => "@none",
			Prefix => "@prefix",
			Propagate => "@propagate",
			Protected => "@protected",
			Reverse => "@reverse",
			Set => "@set",
			Type => "@type",
			Value => "@value",
			Version => "@version",
			Vocab => "@vocab",
		}
	}
}

impl<'a> TryFrom<&'a str> for Keyword {
	type Error = &'a str;

	fn try_from(str: &'a str) -> Result<Keyword, &'a str> {
		use Keyword::*;
		match str {
			"@base" => Ok(Base),
			"@container" => Ok(Container),
			"@context" => Ok(Context),
			"@direction" => Ok(Direction),
			"@graph" => Ok(Graph),
			"@id" => Ok(Id),
			"@import" => Ok(Import),
			"@included" => Ok(Included),
			"@index" => Ok(Index),
			"@json" => Ok(Json),
			"@language" => Ok(Language),
			"@list" => Ok(List),
			"@nest" => Ok(Nest),
			"@none" => Ok(None),
			"@prefix" => Ok(Prefix),
			"@propagate" => Ok(Propagate),
			"@protected" => Ok(Protected),
			"@reverse" => Ok(Reverse),
			"@set" => Ok(Set),
			"@type" => Ok(Type),
			"@value" => Ok(Value),
			"@version" => Ok(Version),
			"@vocab" => Ok(Vocab),
			_ => Err(str),
		}
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

impl<K: JsonBuild> utils::AsAnyJson<K> for Keyword {
	fn as_json_with(&self, meta: K::MetaData) -> K {
		self.into_str().as_json_with(meta)
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
