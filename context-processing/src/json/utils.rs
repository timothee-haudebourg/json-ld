use mown::Mown;
use cc_traits::Get;
use generic_json::{Json, ValueRef};
use json_ld_core::{
	syntax::Term,
	Reference,
	Id
};

/// Local JSON-LD context.
pub struct LocalContextObject<'o, O> {
	objects: Vec<Mown<'o, O>>,
}

impl<'o, O> LocalContextObject<'o, O> {
	pub fn new(object: Mown<'o, O>) -> Self {
		Self {
			objects: vec![object],
		}
	}

	pub fn merge_with(&mut self, object: Mown<'o, O>) {
		self.objects.push(object)
	}

	pub fn get<'q, Q: ?Sized>(
		&self,
		key: &'q Q,
	) -> Option<<O as cc_traits::CollectionRef>::ItemRef<'_>>
	where
		O: cc_traits::Get<&'q Q>,
	{
		for object in self.objects.iter().rev() {
			if let Some(value) = object.get(key) {
				return Some(value);
			}
		}

		None
	}

	pub fn get_key_value<'q, Q: ?Sized>(
		&self,
		key: &'q Q,
	) -> Option<(
		<O as cc_traits::KeyedRef>::KeyRef<'_>,
		<O as cc_traits::CollectionRef>::ItemRef<'_>,
	)>
	where
		O: cc_traits::GetKeyValue<&'q Q>,
	{
		for object in self.objects.iter().rev() {
			if let Some(entry) = object.get_key_value(key) {
				return Some(entry);
			}
		}

		None
	}

	/// Returns an iterator over the entries of the object.
	pub fn iter(&self) -> MergedObjectIter<'_, 'o, O>
	where
		O: cc_traits::MapIter,
	{
		MergedObjectIter {
			objects: &self.objects,
			entries: self.objects.iter().map(|o| o.iter()).rev().collect(),
		}
	}
}

pub struct MergedObjectIter<'a, 'o, O>
where
	O: cc_traits::MapIter,
{
	objects: &'a [Mown<'o, O>],
	entries: Vec<O::Iter<'a>>,
}

impl<'a, 'o, O> Iterator for MergedObjectIter<'a, 'o, O>
where
	O: cc_traits::MapIter + for<'s> Get<&'s str>,
	O::Key: std::ops::Deref<Target = str>,
{
	type Item = (O::KeyRef<'a>, <O as cc_traits::CollectionRef>::ItemRef<'a>);

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.entries.last_mut() {
				Some(entries) => {
					match entries.next() {
						Some((key, value)) => {
							if self.objects.len() > 1 {
								// Checks that the key is not overshadowed by a merged object.
								if self.objects[(self.objects.len() - self.entries.len() + 1)..]
									.iter()
									.any(|object| object.contains(key.as_ref()))
								{
									continue;
								}
							}

							return Some((key, value));
						}
						None => {
							self.entries.pop();
						}
					}
				}
				None => return None,
			}
		}
	}
}

/// JSON value that may be wrapped inside a map `{ "@id": value }`.
pub enum WrappedValue<'a, J: Json> {
	/// Owned `{ "@id": null }` map.
	WrappedNull,

	/// Value wrapped inside a map `{ "@id": value }`.
	Wrapped(&'a J::String, &'a J::MetaData),

	/// Unwrapped value.
	Unwrapped(&'a J::Object),
}

impl<'a, J: Json> WrappedValue<'a, J> {
	pub fn id(&self) -> Option<IdValue<'a, J>> {
		match self {
			Self::WrappedNull => Some(IdValue::Null),
			Self::Wrapped(value, metadata) => Some(IdValue::Unwrapped(*value, *metadata)),
			Self::Unwrapped(object) => object.get("@id").map(IdValue::Ref),
		}
	}

	/// Get the value associated to the given `key`.
	///
	/// It is assumed that `key` is **not** `"@id"`.
	/// Use [`id`](WrappedValue::id) to get the `"@id"` key.
	pub fn get(&self, key: &str) -> Option<<J::Object as cc_traits::CollectionRef>::ItemRef<'a>> {
		debug_assert_ne!(key, "@id");
		match self {
			Self::WrappedNull => None,
			Self::Wrapped(_, _) => None,
			Self::Unwrapped(object) => object.get(key),
		}
	}

	/// Returns an iterator over the entries of the object if it is wrapped,
	/// or an empty iterator.
	pub fn iter(&self) -> WrappedValueIter<'_, J> {
		match self {
			Self::Unwrapped(object) => WrappedValueIter::Iter(object.iter()),
			_ => WrappedValueIter::Empty,
		}
	}
}

pub enum WrappedValueIter<'a, J: Json>
where
	J::Object: 'a,
{
	Iter(<J::Object as cc_traits::MapIter>::Iter<'a>),
	Empty,
}

impl<'a, J: Json> Iterator for WrappedValueIter<'a, J> {
	type Item = (
		<J::Object as cc_traits::KeyedRef>::KeyRef<'a>,
		<J::Object as cc_traits::CollectionRef>::ItemRef<'a>,
	);

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Iter(iter) => iter.next(),
			Self::Empty => None,
		}
	}
}

pub enum IdValue<'a, J: Json>
where
	J::Object: 'a,
{
	Null,
	Unwrapped(&'a J::String, &'a J::MetaData),
	Ref(<J::Object as cc_traits::CollectionRef>::ItemRef<'a>),
}

impl<'a, J: Json> IdValue<'a, J>
where
	J::String: 'a,
	J::Object: 'a,
{
	fn as_value_ref(&self) -> ValueRef<'_, J> {
		match self {
			Self::Null => ValueRef::Null,
			Self::Unwrapped(value, _) => ValueRef::String(*value),
			Self::Ref(value) => value.as_value_ref(),
		}
	}

	fn is_null(&self) -> bool {
		self.as_value_ref().is_null()
	}

	fn as_str(&self) -> Option<&str> {
		self.as_value_ref().into_str()
	}

	fn metadata(&self) -> Option<&J::MetaData> {
		match self {
			Self::Null => None,
			Self::Unwrapped(_, metadata) => Some(*metadata),
			Self::Ref(r) => Some(r.metadata()),
		}
	}
}

fn is_gen_delim(c: char) -> bool {
	matches!(c, ':' | '/' | '?' | '#' | '[' | ']' | '@')
}

// Checks if the input term is an IRI ending with a gen-delim character, or a blank node identifier.
fn is_gen_delim_or_blank<T: Id>(t: &Term<T>) -> bool {
	match t {
		Term::Ref(Reference::Blank(_)) => true,
		Term::Ref(Reference::Id(id)) => {
			if let Some(c) = id.as_iri().as_str().chars().last() {
				is_gen_delim(c)
			} else {
				false
			}
		}
		_ => false,
	}
}

/// Checks if the the given character is included in the given string anywhere but at the first position.
fn contains_after_first(id: &str, c: char) -> bool {
	if let Some(i) = id.find(c) {
		i > 0
	} else {
		false
	}
}

/// Checks if the the given character is included in the given string anywhere but at the first or last position.
fn contains_between_boundaries(id: &str, c: char) -> bool {
	if let Some(i) = id.find(c) {
		let j = id.rfind(c).unwrap();
		i > 0 && j < id.len() - 1
	} else {
		false
	}
}