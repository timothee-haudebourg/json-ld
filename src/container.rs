use std::convert::TryFrom;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContainerType {
	Graph,
	Id,
	Index,
	Language,
	List,
	Set,
	Type
}

impl<'a> TryFrom<&'a str> for ContainerType {
	type Error = &'a str;

	fn try_from(str: &'a str) -> Result<ContainerType, &'a str> {
		use ContainerType::*;
		match str {
			"@graph" => Ok(Graph),
			"@id" => Ok(Id),
			"@index" => Ok(Index),
			"@language" => Ok(Language),
			"@list" => Ok(List),
			"@set" => Ok(Set),
			"@type" => Ok(Type),
			_ => Err(str)
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Container(Vec<ContainerType>);

impl Container {
	pub fn new() -> Container {
		Container(Vec::new())
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn contains(&self, c: ContainerType) -> bool {
		self.0.contains(&c)
	}

	pub fn add(&mut self, c: ContainerType) -> bool {
		if self.is_empty() {
			self.0.push(c);
			true
		} else if self.contains(c) {
			true
		} else {
			use ContainerType::*;
			let valid = if self.len() == 1 {
				match (self.0.first().unwrap(), c) {
					(Set, Index) => true,
					(Set, Graph) => true,
					(Set, Id) => true,
					(Set, Type) => true,
					(Set, Language) => true,
					(Index, Set) => true,
					(Graph, Set) => true,
					(Id, Set) => true,
					(Type, Set) => true,
					(Language, Set) => true,
					//
					(Graph, Id) => true,
					(Id, Graph) => true,
					(Graph, Index) => true,
					(Index, Graph) => true,
					//
					_ => false
				}
			} else if self.len() == 2 {
				match c {
					Set if self.contains(Graph) && (self.contains(Id) || self.contains(Index)) => true,
					Graph if self.contains(Set) && (self.contains(Id) || self.contains(Index)) => true,
					Id if self.contains(Graph) && self.contains(Set) => true,
					Index if self.contains(Graph) && self.contains(Set) => true,
					_ => false
				}
			} else {
				false
			};

			if valid {
				self.0.push(c);
				true
			} else {
				false
			}
		}
	}
}
