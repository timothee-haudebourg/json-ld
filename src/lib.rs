extern crate json;
extern crate linked_data;

use JsonValue;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    Context,
    Id,
    Type
}

#[derive(PartialEq, Eq, Clone)]
pub enum Key<'a> {
    Keyword(Keyword),
    Relation(&'a str)
}

pub trait Document<C: Context> {
    fn as_dataset(&self, context: C) -> Result<linked_data::Dataset, Error>;
}

impl Document for JsonValue {
    fn as_dataset(&self, mut context: C) -> Result<linked_data::Dataset, Error> {
        let mut ds = linked_data::Dataset::new();
        let _root = self.as_node(&mut context, ds, None)?;

        Ok(ds)
    }
}

impl Value for JsonValue {
    fn as_value(&self, context: &mut C, ds: &mut linked_data::Dataset, expected_ty: Option<ValueType>) -> Result<linked_data::Value, Error> {
        // ...
    }
}

impl Node for JsonValue {
    fn as_node(&self, context: &mut C, ds: &mut linked_data::Dataset, expected_ty: Option<NodeType>) -> Result<linked_data::NodeRef, Error> {
        use Error::*;
        use JsonValue::*;

        match self {
            Object(obj) => {
                let mut types = Vec::new();
                let mut id = None;

                // Process keywords.
                for (key, value) in obj.iter() {
                    match context.parse_key(key) {
                        Key::Keyword(Keyword::Context) => {
                            match value {
                                Null => (),
                                Short(s) => {
                                    match s.as_str().parse::<Iri>() {
                                        Some(iri) => {
                                            // ...
                                        },
                                        None => return Err(InvalidContext)
                                    }
                                },
                                String(s) => {
                                    match s.as_str().parse::<Iri>() {
                                        Some(iri) => {
                                            // ...
                                        },
                                        None => return Err(InvalidContext)
                                    }
                                },
                                Object(obj) => {
                                    // ...
                                },
                                Array(contexts) => {
                                    // ...
                                },
                                _ => return Err(InvalidContext)
                            }
                        },
                        Key::Keyword(Keyword::Type) => {
                            match value {
                                Short(id) => {
                                    if let Some(ty) = context.node_type(id.as_str()) {
                                        types.push(ty)
                                    }
                                },
                                String(id) => {
                                    if let Some(ty) = context.node_type(id.as_str()) {
                                        types.push(ty)
                                    }
                                },
                                Array(ids) => {
                                    for value in ids {
                                        match value {
                                            Short(id) => {
                                                if let Some(ty) = context.node_type(id.as_str()) {
                                                    types.push(ty)
                                                }
                                            },
                                            String(id) => {
                                                if let Some(ty) = context.node_type(id.as_str()) {
                                                    types.push(ty)
                                                }
                                            },
                                            _ => return Err(InvalidNodeType)
                                        }
                                    }
                                },
                                _ => return Err(InvalidNodeType)
                            }
                        },
                        Key::Keyword(Keyword::Id) => {
                            match value.parse::<Iri>() {
                                Some(iri) => {
                                    ds.identify(node, iri)
                                },
                                None => return Err(InvalidId)
                            }
                        },
                        _ => ()
                    }
                }

                // Create the node.
                let mut node = ds.new_node(id, types);

                // Process properties.
                for (key, value) in obj.iter() {
                    match context.parse_key(key) {
                        _ => (),
                        Key::Relation(id) => {
                            if let Some(rel) = context.relation(id) {
                                node.define(rel, value.as_value(context, ds, ref.expected_ty())?)
                            }
                        }
                    }
                }
            }
        }
    }
}
