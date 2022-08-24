use crate::{context, Direction, LenientLanguageTag, Nullable};
use iref::IriRef;
use json_syntax::print::{
	pre_compute_array_size, pre_compute_object_size, printed_string_size, string_literal, Options,
	PrecomputeSize, Print, PrintWithSize, Size,
};
use std::fmt;

impl PrecomputeSize for Direction {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(printed_string_size(self.as_str()))
	}
}

impl Print for Direction {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for LenientLanguageTag<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(printed_string_size(self.as_str()))
	}
}

impl<'a> Print for LenientLanguageTag<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for Nullable<IriRef<'a>> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => Size::Width(printed_string_size(v.as_str())),
		}
	}
}

impl<'a> Print for Nullable<IriRef<'a>> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => string_literal(v.as_str(), f),
		}
	}
}

impl<'a> PrecomputeSize for Nullable<bool> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(b) => b.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for Nullable<bool> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(b) => b.fmt_with(f, options, indent),
		}
	}
}

impl<'a> PrecomputeSize for Nullable<crate::LenientLanguageTag<'a>> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => Size::Width(printed_string_size(v.as_str())),
		}
	}
}

impl<'a> Print for Nullable<crate::LenientLanguageTag<'a>> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(t) => string_literal(t.as_str(), f),
		}
	}
}

impl PrecomputeSize for Nullable<crate::Direction> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => Size::Width(printed_string_size(v.as_str())),
		}
	}
}

impl<'a> Print for Nullable<crate::Direction> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(d) => string_literal(d.as_str(), f),
		}
	}
}

impl<C: context::AnyValue<Metadata = M> + PrecomputeSize + PrintWithSize, M> Print
	for crate::Value<C, M>
{
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => f.write_str("null"),
			Self::Boolean(b) => b.fmt_with(f, options, indent),
			Self::Number(n) => n.fmt_with(f, options, indent),
			Self::String(s) => s.fmt_with(f, options, indent),
			Self::Array(a) => {
				let mut sizes = Vec::with_capacity(self.count(|v| v.is_array() || v.is_object()));
				self.pre_compute_size(options, &mut sizes);
				let mut index = 0;
				a.fmt_with_size(f, options, indent, &sizes, &mut index)
			}
			Self::Object(o) => {
				let mut sizes = Vec::with_capacity(self.count(|v| v.is_array() || v.is_object()));
				self.pre_compute_size(options, &mut sizes);
				let mut index = 0;
				o.fmt_with_size(f, options, indent, &sizes, &mut index)
			}
		}
	}
}

impl<C: PrecomputeSize, M> PrecomputeSize for crate::Value<C, M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			crate::Value::Null => Size::Width(4),
			crate::Value::Boolean(true) => Size::Width(4),
			crate::Value::Boolean(false) => Size::Width(5),
			crate::Value::Number(n) => Size::Width(n.as_str().len()),
			crate::Value::String(s) => Size::Width(printed_string_size(s)),
			crate::Value::Array(a) => pre_compute_array_size(a, options, sizes),
			crate::Value::Object(o) => pre_compute_object_size(
				o.entries_with_context()
					.map(|e| (e.key().as_str(), e.value())),
				options,
				sizes,
			),
		}
	}
}

impl<C: PrintWithSize, M> PrintWithSize for crate::Value<C, M> {
	#[inline(always)]
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Null => f.write_str("null"),
			Self::Boolean(b) => b.fmt_with(f, options, indent),
			Self::Number(n) => n.fmt_with(f, options, indent),
			Self::String(s) => s.fmt_with(f, options, indent),
			Self::Array(a) => a.fmt_with_size(f, options, indent, sizes, index),
			Self::Object(o) => o.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl<C: PrintWithSize, M> PrintWithSize for crate::Object<C, M> {
	#[inline(always)]
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		json_syntax::print::print_object(
			self.entries_with_context()
				.map(|e| (e.key().as_str(), e.value())),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, C: PrecomputeSize, M> PrecomputeSize for crate::object::AnyValueRef<'a, C, M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Context(c) => c.pre_compute_size(options, sizes),
			Self::Value(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, C: PrintWithSize, M> PrintWithSize for crate::object::AnyValueRef<'a, C, M> {
	#[inline(always)]
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Context(c) => c.fmt_with_size(f, options, indent, sizes, index),
			Self::Value(v) => v.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}
