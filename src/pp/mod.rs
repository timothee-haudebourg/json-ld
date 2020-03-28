use std::fmt;
use crate::{Id, Key, Object, Node, Value, Literal};

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
		write!(self.fmt, "{}\n", d)
	}

	fn tab(&mut self) -> fmt::Result {
		let depth = self.path.len();
		for i in 0..depth {
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

	pub fn entry<T: PrettyPrintable>(&mut self, id: &str, value: &T) -> fmt::Result {
		let i = self.next();
		if i > 0 {
			write!(self.fmt, ",\n");
		}
		self.tab()?;
		write!(self.fmt, "\"{}\" = ", id)?;
		value.pretty_print(self)
	}

	pub fn item<T: PrettyPrintable>(&mut self, value: &T) -> fmt::Result {
		let i = self.next();
		if i > 0 {
			write!(self.fmt, ",\n");
		}
		self.tab()?;
		value.pretty_print(self)
	}
}

pub trait PrettyPrintable {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result;
}

pub struct PrettyPrint<'a, T: PrettyPrintable>(&'a T);

impl<'a, T: PrettyPrintable> PrettyPrint<'a, T> {
	pub fn new(t: &'a T) -> PrettyPrint<'a, T> {
		PrettyPrint(t)
	}
}

impl<'a, T: PrettyPrintable> fmt::Display for PrettyPrint<'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut pp = PrettyPrinter::new(f);
		self.0.pretty_print(&mut pp)
	}
}

impl<T: Id> PrettyPrintable for Key<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		write!(pp.fmt, "{}", self)
	}
}

impl<T: Id> PrettyPrintable for Object<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		match self {
			Object::Value(value) => value.pretty_print(pp),
			Object::Node(node) => node.pretty_print(pp)
		}
	}
}

impl<T: Id> PrettyPrintable for Value<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		match self {
			Value::Literal(lit) => lit.pretty_print(pp),
			Value::Ref(id) => id.pretty_print(pp),
			Value::List(items) => {
				pp.begin("{")?;
				// TODO
				pp.end("}")
			}
		}
	}
}

impl<T: Id> PrettyPrintable for Literal<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		Ok(())
	}
}

impl<T: Id> PrettyPrintable for Node<T> {
	fn pretty_print(&self, pp: &mut PrettyPrinter) -> fmt::Result {
		pp.begin("{")?;

		if let Some(id) = &self.id {
			pp.entry("@id", id)?;
		}

		if !self.types.is_empty() {
			pp.entry("@type", &self.types)?;
		}

		pp.end("}")
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
