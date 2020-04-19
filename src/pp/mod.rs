use std::fmt;
use std::collections::HashSet;
use crate::{Direction, Id, Term, NodeType, ValueType, Property, Keyword, Object, Value, Literal};

pub struct PrettyPrinter<'a, 'b: 'a> {
	fmt: &'a mut fmt::Formatter<'b>,
	path: Vec<usize>
}

impl<'a, 'b: 'a> PrettyPrinter<'a, 'b> {
	pub fn new(f: &'a mut fmt::Formatter<'b>) -> PrettyPrinter<'a, 'b> {
		PrettyPrinter {
			fmt: f,
			path: Vec::new()
		}
	}

	pub fn begin(&mut self, d: &str) -> fmt::Result {
		write!(self.fmt, "{}\n", d)?;
		self.path.push(0);
		Ok(())
	}

	pub fn end(&mut self, d: &str) -> fmt::Result {
		let i = self.path.pop().unwrap();
		if i > 0 {
			write!(self.fmt, "\n")?;
		}
		self.tab()?;
		write!(self.fmt, "{}", d)
	}

	fn tab(&mut self) -> fmt::Result {
		let depth = self.path.len();
		for _ in 0..depth {
			write!(self.fmt, "\t")?
		}

		Ok(())
	}

	fn next(&mut self) -> usize {
		let count = self.path.last_mut().unwrap();
		let i = *count;
		*count += 1;
		i
	}

	pub fn keyword_entry<T: PrettyPrintable + ?Sized>(&mut self, key: Keyword, value: &T) -> fmt::Result {
		let i = self.next();
		if i > 0 {
			write!(self.fmt, ",\n")?;
		}
		self.tab()?;
		write!(self.fmt, "\"{}\": ", key)?;
		value.pretty_print(self)
	}

	pub fn entry<I: Id, T: PrettyPrintable + ?Sized>(&mut self, key: &Property<I>, value: &T) -> fmt::Result {
		let i = self.next();
		if i > 0 {
			write!(self.fmt, ",\n")?;
		}
		self.tab()?;
		write!(self.fmt, "\"{}\": ", key)?;
		value.pretty_print(self)
	}

	pub fn item<T: PrettyPrintable>(&mut self, value: &T) -> fmt::Result {
		let i = self.next();
		if i > 0 {
			write!(self.fmt, ",\n")?;
		}
		self.tab()?;
		value.pretty_print(self)
	}
}

pub trait PrettyPrintable {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result;
}

pub struct PrettyPrint<'a, T: PrettyPrintable + ?Sized>(pub &'a T);

impl<'a, T: PrettyPrintable + ?Sized> PrettyPrint<'a, T> {
	pub fn new(t: &'a T) -> PrettyPrint<'a, T> {
		PrettyPrint(t)
	}
}

impl<'a, T: PrettyPrintable + ?Sized> fmt::Display for PrettyPrint<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut pp = PrettyPrinter::new(f);
		self.0.pretty_print(&mut pp)
	}
}

impl<T: Id> PrettyPrintable for Term<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		write!(pp.fmt, "\"{}\"", self)
	}
}

impl<T: Id> PrettyPrintable for NodeType<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		write!(pp.fmt, "\"{}\"", self)
	}
}

impl<T: Id> PrettyPrintable for ValueType<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		write!(pp.fmt, "\"{}\"", self)
	}
}

impl<T: Id> PrettyPrintable for Object<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		match self {
			Object::Value(value, data) => {
				pp.begin("{")?;

				match value {
					Value::Literal(ref lit, ref types) => {
						match lit {
							Literal::Null => {
								pp.tab()?;
								write!(pp.fmt, "@value: null")?;
							},
							Literal::Boolean(b) => {
								pp.keyword_entry(Keyword::Value, b)?;
							},
							Literal::Number(n) => {
								pp.keyword_entry(Keyword::Value, n)?;
							},
							Literal::String{ data, language, direction } => {
								pp.keyword_entry(Keyword::Value, data)?;
								if let Some(language) = language {
									pp.keyword_entry(Keyword::Language, language)?;
								}
								if let Some(direction) = direction {
									pp.keyword_entry(Keyword::Direction, direction)?;
								}
							},
							Literal::Json(json) => {
								pp.keyword_entry(Keyword::Value, json)?;
							},
							Literal::Ref(id) => {
								//id.pretty_print(pp)
								pp.keyword_entry(Keyword::Value, id)?;
							}
						}

						if !types.is_empty() {
							pp.keyword_entry(Keyword::Type, types)?;
						}
					},
					Value::List(items) => {
						pp.keyword_entry(Keyword::List, items)?;
					},
				};

				if let Some(index) = &data.index {
					pp.keyword_entry(Keyword::Index, index)?;
				}

				pp.end("}")
			},
			Object::Node(node, data) => {
				pp.begin("{")?;

				if let Some(id) = &node.id {
					pp.keyword_entry(Keyword::Id, id)?;
				}

				if !node.types.is_empty() {
					pp.keyword_entry(Keyword::Type, &node.types)?;
				}

				if let Some(index) = &data.index {
					pp.keyword_entry(Keyword::Index, index)?;
				}

				if !node.reverse_properties.is_empty() {
					pp.begin("\"@reverse\": {")?;
					for (key, value) in &node.reverse_properties {
						pp.entry(key, value)?;
					}
					pp.end("}")?;
				}

				for (key, value) in &node.properties {
					pp.entry(key, value)?;
				}

				pp.end("}")
			}
		}
	}
}

// impl<T: Id> PrettyPrintable for Value<T> {
// 	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
//
// 	}
// }

// impl<T: Id> PrettyPrintable for Literal<T> {
// 	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
// 		write!(pp.fmt, "{}", self.value)
// 	}
// }

// impl<T: Id> PrettyPrintable for Node<T> {
// 	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
//
// 	}
// }

impl PrettyPrintable for Direction {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		use fmt::Display;
		self.fmt(pp.fmt)
	}
}

impl<T: PrettyPrintable> PrettyPrintable for Vec<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		pp.begin("[")?;
		for item in self {
			pp.item(item)?;
		}
		pp.end("]")
	}
}

impl<T: PrettyPrintable> PrettyPrintable for HashSet<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		// pp.begin("[")?;
		// for item in self {
		// 	pp.item(item)?;
		// }
		// pp.end("]")

		match self.len() {
			0 => write!(pp.fmt, "null"),
			1 => {
				self.iter().next().unwrap().pretty_print(pp)
			},
			_ => {
				pp.begin("[")?;
				for item in self {
					pp.item(item)?;
				}
				pp.end("]")
			}
		}
	}
}

impl<T: PrettyPrintable> PrettyPrintable for Option<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		match self {
			Some(v) => v.pretty_print(pp),
			None => write!(pp.fmt, "null")
		}
	}
}

impl PrettyPrintable for bool {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		if *self {
			write!(pp.fmt, "true")
		} else {
			write!(pp.fmt, "false")
		}
	}
}

impl PrettyPrintable for json::number::Number {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		use fmt::Display;
		self.fmt(pp.fmt)
	}
}

impl PrettyPrintable for json::JsonValue {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		use fmt::Display;
		self.fmt(pp.fmt)
	}
}

impl PrettyPrintable for String {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		write!(pp.fmt, "\"{}\"", self)
	}
}

// impl<T: PrettyPrintable> PrettyPrintable for [T] {
// 	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
// 		match self.len() {
// 			0 => write!(pp.fmt, "null"),
// 			1 => {
// 				self.first().unwrap().pretty_print(pp)
// 			},
// 			_ => {
// 				pp.begin("[")?;
// 				for item in self {
// 					pp.item(item)?;
// 				}
// 				pp.end("]")
// 			}
// 		}
// 	}
// }
