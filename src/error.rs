pub enum Error {
    /**
     * The provided @context is invalid.
     */
    InvalidContext,

    /**
     * The provided @id is not a valid IRI.
     */
    InvalidId,

    /**
     * The provided node @type is not valid.
     */
    InvalidNodeType,

    /**
     * The provided value @type is not valid.
     */
    InvalidValueType,

    /**
     * Node has not the expected type.
     */
    UnexpectedNodeType,

    /**
     * Value has not the expected type.
     */
    UnexpectedValueType
}
