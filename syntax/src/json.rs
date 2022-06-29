use crate::Number;
use indexmap::IndexMap;

pub struct Key(String);

pub enum Json<S, P> {
	Null,
	Boolean(bool),
	Number(Number),
	String(String),
	Array(Json<S, P>),
	Map(Map<S, P>)
}

pub struct Map<S, P>(IndexMap<Key, Json<S, P>>);