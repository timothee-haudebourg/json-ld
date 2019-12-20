use crate::Keyword;

/// Known key in a JSON-LD context.
pub enum Key<T: grdf::Entity> {
    /// The key is a JSON-LD keyword.
    Keyword(Keyword),

    /// The key is a vocabulary term.
    Term(T)
}

pub struct TermDefinition {
    // ...
}

pub enum ExpandError {
    InvalidLocalContext,

    /// The `@propagate` value is not set to a boolean.
    InvalidPropagateValue,

    InvalidVersionValue,

    InvalidBaseIri,

    InvalidVocabMapping,

    InvalidDefaultLanguage,

    InvalidBaseDirection,

    InvalidContextNullification,

    ContextOverflow,

    LoadingRemoteContextFailed,

    InvalidRemoteContext,
}

/// JSON-LD context.
pub trait Context: Sized + Clone {
    type Term: grdf::Entity;
    type Error;

    /// Retreive the key bound to the given id.
    fn key(&self, id: &str) -> Result<Key<T>, Self::Error>;

    /// Set (or unset) the definition of a term, and return the previous definition if any.
    fn set(&mut self, term: &str, definition: Option<TermDefinition>) -> Result<Option<TermDefinition>, Self::Error>;
}

fn is_keyword(str: &str) -> bool {
    match str {
        "@base" | "@container" | "@context" | "@direction" | "@graph" | "@id" |
        "@import" | "@imported" | "@index" | "@json" | "@language" | "@list" |
        "@nest" | "@none" | "@prefix" | "@propagate" | "@protected" | "@reverse" |
        "@set" | "@value" | "@version" | "@vocab" => true,
        _ => false
    }
}

pub struct Loader<'a, C: 'a + Context> {
    /// Active context. The one beeing modified.
    ctx: &'a mut C,

    /// Terms that are beeing loaded, or are already loaded.
    defined: HashMap<String, bool>,

    /// Local contexts beeing loaded.
    stack: Vec<&'a json::Object>
}

impl Loader {
    /// Resove an iri.
    fn resolve(&self, str: &str) -> Result<(), Self::Error>;

    /// Load remote context.
    fn load_remote(&mut self, iri: &str) -> Result<(), Self::Error>;

    /// Return the current local context.
    /// Panics of there are no current local context.
    fn local_ctx(&self) -> &json::Object {
        self.stack.last()
    }

    /// Expand a given local JSON-LD context into this current context.
    fn load(&mut self, json: &JsonValue, remote: bool) -> Result<(), Self::Error> {
        // This function tries to follow the recommended context proessing algorithm.
        // See `https://www.w3.org/TR/json-ld11-api/#context-processing-algorithm`.

        // Initialize local variables.
        // let mut propagate = true;
        let mut override_protected = false;

        // 2) If local context is an object containing the member @propagate,
        // its value MUST be boolean true or false, set propagate to that value.
        // if let Some(propagate_value) = map.get("@propagate") {
        //     if let JsonValue::Boolean(b) = propagate_value {
        //         propagate = b;
        //     } else {
        //         return Err(ExpandError::InvalidPropagateValue.into())
        //     }
        // }
        // TODO

        // 3) If propagate is false, and result does not have a previous context,
        // set previous context in result to active context.
        // if !propagate {
        //     ctx.set_previous_context(self.clone());
        // }
        // TODO

        // 4) If local context is not an array, set it to an array containing only local context.
        match json {
            JsonValue::Array(items) => {
                // 5) For each item context in local context:
                for context in items {
                    self.load_context_definition(context);
                }
            },
            context => {
                self.load_context_definition(context)
            }
        }
    }

    fn load_context_definition(&mut self, context: &JsonValue) -> Result<(), Self::Error> {
        match context {
            JsonValue::Null => {
                if !override_protected && ctx.has_protected_items() {
                    return Err(ExpandError::InvalidContextNullification.into())
                } else {
                    // TODO
                }
            },
            // 5.2) If context is a string,
            JsonValue::String(str) => {
                // 5.2.1) Initialize context to the result of resolving context against the
                // base IRI of the document containing the local context.
                let uri = ctx.resolve(str)?;

                // 5.2.2) If the number of entries in the remote contexts array exceeds a
                // processor defined limit, a context overflow error has been detected.
                // TODO

                // 5.2.3 to 5.2.6 are done vie `load_remote`.
                // Load the remote context.
                self.load_remote(ctx, uri)?
            },
            // 5.2 again.
            JsonValue::Short(str) => {
                let uri = ctx.resolve(str)?;
                self.load_remote(ctx, uri)?
            },
            // 5.4) Context definition.
            JsonValue::Object(map) => {
                // 5.5) If context has an @version entry:
                if let Some(version_value) = map.get("@version") {
                    // 5.5.1) If the associated value is not 1.1, an invalid @version value has
                    // been detected.
                    if version_value.as_str() != Some("1.1") && version_value.as_f32() != 1.1 {
                        return Err(ExpandError::InvalidVersionValue.into())
                    }
                }

                // 5.5.2) If processing mode is set to json-ld-1.0, a processing mode conflict
                // error has been detected.
                // TODO

                // 5.6) If context has an @import entry:
                if let Some(import_value) = map.get("@import") {
                    // 5.6.1) If processing mode is json-ld-1.0, an invalid context entry error
                    // has been detected.
                    // TODO

                    if let Some(str) = import_value.as_str() {
                        // 5.6.3) Initialize import to the result of resolving the value of
                        // @import.
                        let import = ctx.resolve(str)?;

                        // 5.6.4) Dereference import.
                        self.load_remote(ctx, uri)?
                    } else {
                        // 5.6.2) If the value of @import is not a string, an invalid
                        // @import value error has been detected.
                        return Err(ExpandError::InvalidImportValue.into())
                    }
                }

                // 5.7) If context has a @base entry and remote contexts is empty, i.e.,
                // the currently being processed context is not a remote context:
                if !remote {
                    // 5.7.1) Initialize value to the value associated with the @base entry.
                    if let Some(value) = map.get("@base") {
                        if value.is_null() {
                            // 5.7.2) If value is null, remove the base IRI of result.
                            self.set_base(None);
                        } else if let Some(iri) = value.as_str() {
                            // 5.7.4) Otherwise, if value is an IRI, the base IRI of result is
                            // set to value.
                            // 5.7.5) Otherwise, if value is a relative IRI reference and the
                            // base IRI of result is not null, set the base IRI of result to
                            // the result of resolving value.
                            self.set_base(Some(self.resolve(iri)?))
                        } else {
                            return Err(ExpandError::InvalidBaseIri.into())
                        }
                    }
                }

                // 5.8) If context has a @vocab entry:
                if let Some(value) = map.get("@vocab") {
                    if value.is_null() {
                        panic!("TODO")
                    } else if let Some(iri) = value.as_str() {
                        panic!("TODO")
                    } else {
                        return Err(ExpandError::InvalidVocabMapping.into())
                    }
                }

                // 5.9) If context has a @language entry:
                if let Some(value) = map.get("@language") {
                    if value.is_null() {
                        panic!("TODO")
                    } else if let Some(iri) = value.as_str() {
                        panic!("TODO")
                    } else {
                        return Err(ExpandError::InvalidDefaultLanguage.into())
                    }
                }

                // 5.10) If context has a @direction entry:
                if let Some(value) = map.get("@direction") {
                    if value.is_null() {
                        panic!("TODO")
                    } else if let Some(iri) = value.as_str() {
                        panic!("TODO")
                    } else {
                        return Err(ExpandError::InvalidBaseDirection.into())
                    }
                }

                // 5.12) Create a map `defined_terms` to keep track of whether or not a term
                // has already been defined or is currently being defined during recursion.
                // done.
                self.stack.push(map);

                // 5.13) For each key-value pair in context where key is not a keyword,
                // invoke the Create Term Definition algorithm.
                for (term, value) in map.iter() {
                    match term {
                        "@base" | "@direction" | "@import" | "@language" | "@propagate" |
                        "@protected" | "@version" | "@vocab" => (),
                        _ => {
                            self.define(term, value)
                        }
                    }
                }

                self.stack.pop();
            },
            // 5.3) An invalid local context error has been detected.
            _ => return Err(ExpandError::InvalidLocalContext.into())
        }
    }

    fn define(&mut self, term: &str, value: &JsonValue) -> Result<(), Self::Error> {
        match self.defined.get(term) {
            // 1) If defined contains the entry term and the associated value is true
            // (indicating that the term definition has already been created), return
            Some(true) => Ok(()),
            // Otherwise, if the value is false, a cyclic IRI mapping error has been detected.
            Some(false) => Err(ExpandError::CyclicIriMapping.into()),
            None => {
                // 2) Set the value associated with defined's term entry to false.
                defined.insert(term.to_string(), false);

                match term {
                    // 4) If term is @type...
                    "@type" => {
                        // ...and processing mode is json-ld-1.0, a keyword redefinition error has been detected.
                        // TODO

                        // At this point, value MUST be a map with only the entry @container and value
                        // @set and optional entry @protected.
                        if let JsonValue::Object(map) = value {
                            for (key, value) in map.iter() {
                                "@container" | "@set" | "@protected" => (),
                                _ => return Err(ExpandError::KeywordRedefinition.into());
                            }
                        } else {
                            return Err(ExpandError::KeywordRedefinition.into());
                        }
                    },

                    // 5) Otherwise, since keywords cannot be overridden, term MUST NOT be a keyword
                    // and a keyword redefinition error has been detected.
                    _ if is_keyword(term) => {
                        return Err(ExpandError::KeywordRedefinition.into())
                    },

                    // If term has the form of a keyword (i.e., it matches the ABNF rule "@"1*ALPHA
                    // from [RFC5234]), return; processors SHOULD generate a warning.
                    // TODO
                    _ => ()
                }

                // 6) Initialize previous definition to any existing term definition for term in
                // active context, removing that term definition from active context.
                let previous = ctx.set(term, None)?;
                let simple_term = false;

                // 7) If value is null, convert it to a map consisting of a single entry whose key
                // is @id and whose value is null.
                if value.is_null() {
                    value = object![ "@id" => json::Null ];
                }

                // 8) Otherwise, if value is a string, convert it to a map consisting of a single
                // entry whose key is @id and whose value is value.
                if value.is_string() {
                    value = object![ "@id" => value ];
                    simple_term = true;
                }

                // 9) Otherwise, value MUST be a map...
                if let JsonValue::Object(map) = value {
                    // 10) Create a new term definition, definition.
                    let definition = TermDefinition::new();

                    // 11) If the @protected entry in value is true set the protected flag in
                    // definition to true.
                    if let Some(protected_value) = map["@protected"] {
                        if let JsonValue::Boolean(b) = protected_value {
                            definition.protected = b;
                        } else {
                            // If the value of @protected is not a boolean, an invalid @protected
                            // value error has been detected.
                            return Err(ExpandError::InvalidProtectedValue.into())

                            // If processing mode is json-ld-1.0, an invalid term definition has
                            // been detected.
                            // TODO
                        }
                    } else {
                        // 12) Otherwise, if there is no @protected entry in value and the
                        // protected parameter is true, set the protected in definition to true.
                        definition.protected = protected;
                    }

                    // 13) If value contains the entry @type:
                    if let Some(ty_value) = map["@type"] {
                        // 13.1) Initialize type to the value associated with the @type entry,
                        // which MUST be a string.
                        if let Some(ty) = ty_value.as_str() {
                            // TODO
                        } else {
                            // Otherwise, an invalid type mapping error has been detected.
                            return Err(ExpandError::InvalidTypeMapping.into())
                        }
                    }

                    // 14) If value contains the entry @reverse:
                    if let Some(reverse_value) = map["@reverse"] {
                        panic!("TODO")
                    }

                    // 15) Set the reverse property flag of definition to false.
                    definition.reverse_property = false;

                    // 16) If value contains the entry @id and its value does not equal term:
                    let id_value = map["@id"];
                    if id_value.is_some() && id_value.unwrap().as_str() != Some(term) {
                        panic!("TODO")
                    } else {
                        // 17) Otherwise if the term contains a colon (:) anywhere after the first
                        // character:
                        if is_curie_or_blank_id(term) {
                            panic!("TODO");
                        } else {
                            // 18) Otherwise if the term contains a slash (/):
                            if is_relative_iri_ref(term) {
                                panic!("TODO");
                            } else {
                                // 19) Otherwise, if term is @type...
                                if term == "@type" {
                                    // ...set the IRI mapping of definition to @type.
                                    panic!("TODO");
                                } else {
                                    // 20) Otherwise, if active context has a vocabulary mapping...
                                    if let Some(vocab) = ctx.vocabulary_mapping() {
                                        // ...the IRI mapping of definition is set to the result of
                                        // concatenating the value associated with the vocabulary
                                        // mapping and term.
                                        panic!("TODO");
                                    } else {
                                        // If it does not have a vocabulary mapping,
                                        // an invalid IRI mapping error been detected.
                                        return Err(ExpandError::InvalidIriMapping.into())
                                    }
                                }
                            }
                        }
                    }

                    // 21) If value contains the entry @container:
                    if let Some(container_value) = map["@container"] {
                        panic!("TODO")
                    }

                    // 22) If value contains the entry @index:
                    if let Some(container_value) = map["@index"] {
                        panic!("TODO")
                    }

                    // 23) If value contains the entry @context:
                    if let Some(context_value) = map["@context"] {
                        panic!("TODO")
                    }

                    let has_type = map["@type"].is_some();

                    if !has_type {
                        // 24) If value contains the entry @language and does not contain the entry @type:
                        if let Some(language_value) = map["@language"] {
                            panic!("TODO")
                        }

                        // 25) If value contains the entry @direction and does not contain the entry @type:
                        if let Some(direction_value) = map["@direction"] {
                            panic!("TODO")
                        }
                    }

                    // 26) If value contains the entry @nest:
                    if let Some(nest_value) = map["@nest"] {
                        panic!("TODO")
                    }

                    // 27) If value contains the entry @prefix:
                    if let Some(prefix_value) = map["@prefix"] {
                        panic!("TODO")
                    }

                    for (key, v) in map.iter() {
                        match key {
                            // 28) If the value contains any entry other than @id, @reverse,
                            // @container, @context, @language, @nest, @prefix, or @type, an
                            // invalid term definition error has been detected.
                            _ => return Err(ExpandError::InvalidTermDefinition.into())
                        }
                    }

                    // 29) If override protected is false...
                    if !override_protected {
                        // ...and previous definition exists...
                        if let Some(previous) = previous {
                            // ...and is protected;
                            if previous.protected && previous != definition {
                                // 29.1) If definition is not the same as previous definition
                                // (other than the value of protected), a protected term
                                // redefinition error has been detected.
                                return Err(ExpandError::ProtectedTermRedefinition.into())
                            } else {
                                // 29.2) Set definition to previous definition to retain the value
                                // of protected.
                                // Note: in our case we change the value of protected in the new
                                // definition.
                                definition.protected = previous.protected;
                            }
                        }
                    }

                    ctx.set(term, Some(definition))?;
                    defined.insert(term.to_string(), true);
                } else {
                    // ...if not, an invalid term definition error has been detected.
                    return Err(ExpandError::InvalidTermDefinition.into())
                }
            }
        }
    }

    fn ensure_defined(&self, term: &str) -> Result<(), Self::Error> {
        if let Some(value) = self.local_ctx().get(term) {
            self.define(term, value)
        }
    }

    fn expand_iri(&self, value: &str) -> String {
        // 1) If value is a keyword or null, return value as is.
        if is_keyword(value) {
            return value.to_string()
        } else {
            // 2) If value has the form of a keyword (i.e., it matches the ABNF rule "@"1*ALPHA
            // from [RFC5234]), a processor SHOULD generate a warning and return null.
            // TODO

            // 3)
            // 4)
            // 5)
            // TODO

            // 6) If value contains a colon (:) anywhere after the first character,
            // it is either an IRI, a compact IRI, or a blank node identifier:
            // 6.1) Split value into a prefix and suffix at the first occurrence of a colon (:).
            if let Some((prefix, suffix)) = split_on(':', value) {
                // 6.2) If prefix is underscore (_) or suffix begins with double-forward-slash
                // (//), return value as it is already an IRI or a blank node identifier.
                if prefix == '_' {
                    // blank node id
                } else if starts_with("//", suffix) {
                    // IRI
                } else {
                    // 6.3) If local context is not null, it contains a prefix entry, and the value
                    // of the prefix entry in defined is not true, invoke the Create Term
                    // Definition algorithm. This will ensure that a term definition is created for
                    // prefix in active context during Context Processing.
                    // Note: this is to make sure that the prefix term has been processed even if
                    // it is defined after.
                    self.ensure_defined(prefix);

                    // 6.4) If active context contains a term definition for prefix having a
                    // non-null IRI mapping and the prefix flag of the term definition is true,
                    // return the result of concatenating the IRI mapping associated with prefix
                    // and suffix.
                    if let Some(def) = self.ctx.get(prefix) {
                        if def.is_prefix() {
                            if let Some(iri) = def.iri() {
                                return iri.to_string() + suffix;
                            }
                        }
                    }

                    // 6.5) If value has the form of an IRI, return value.
                    if is_iri(value) {
                        return value.to_string()
                    } else {
                        panic!("undefined"); // TODO
                    }
                }
            }

            // 7) If vocab is true, and active context has a vocabulary mapping, return the result
            // of concatenating the vocabulary mapping with value.
            if self.vocab {
                if let Some(v) = self.vocab_mapping {
                    return Ok(v + value)
                }
            }

            // 8) Otherwise, if document relative is true set value to the result of resolving
            // value against the base IRI from active context.

            // Only the basic algorithm in section
            // 5.2 of [RFC3986] is used; neither Syntax-Based Normalization nor
            // Scheme-Based Normalization are performed. Characters additionally allowed in IRI references are treated in the same way that unreserved characters are treated in URI references, per section 6.5 of [RFC3987].
            return Ok(self.resolve(value)?);

            // 9) Return value as is.
            Ok(value.to_string())
        }
    }
}
