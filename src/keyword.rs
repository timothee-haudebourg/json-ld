/// JSON-LD keywords.
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

    /// `@imported`.
    /// Used in a top-level node object to define an included block, for including secondary node
    /// objects within another node object.
    Imported,

    /// `@index`.
    /// Used to specify that a container is used to index information and that processing should
    /// continue deeper into a JSON data structure.
    Index,

    /// `@json`.
    /// Used as the @type value of a JSON literal.
    JSON,

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
    Vocab
}
