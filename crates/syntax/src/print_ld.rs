use crate::{Direction, LenientLangTag, LenientLangTagBuf, Nullable};
use iref::{IriRef, IriRefBuf};
use json_syntax::print::{
	printed_string_size, string_literal, Options, PrecomputeSize, Print, Size,
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

impl PrecomputeSize for LenientLangTagBuf {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(printed_string_size(self.as_str()))
	}
}

impl Print for LenientLangTagBuf {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl PrecomputeSize for LenientLangTag {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(printed_string_size(self.as_str()))
	}
}

impl Print for LenientLangTag {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl PrecomputeSize for Nullable<&IriRef> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => Size::Width(json_syntax::print::printed_string_size(v.as_str())),
		}
	}
}

impl Print for Nullable<&IriRef> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => string_literal(v.as_str(), f),
		}
	}
}

impl PrecomputeSize for Nullable<IriRefBuf> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => Size::Width(printed_string_size(v.as_str())),
		}
	}
}

impl Print for Nullable<IriRefBuf> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => string_literal(v.as_str(), f),
		}
	}
}

impl PrecomputeSize for Nullable<bool> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(b) => b.pre_compute_size(options, sizes),
		}
	}
}

impl Print for Nullable<bool> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(b) => b.fmt_with(f, options, indent),
		}
	}
}

impl PrecomputeSize for Nullable<&LenientLangTagBuf> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => Size::Width(printed_string_size(v.as_str())),
		}
	}
}

impl Print for Nullable<&LenientLangTagBuf> {
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

impl Print for Nullable<crate::Direction> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(d) => string_literal(d.as_str(), f),
		}
	}
}

// impl<M> Print
// 	for crate::Value<M>
// {
// 	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
// 		match self {
// 			Self::Null => f.write_str("null"),
// 			Self::Boolean(b) => b.fmt_with(f, options, indent),
// 			Self::Number(n) => n.fmt_with(f, options, indent),
// 			Self::String(s) => s.fmt_with(f, options, indent),
// 			Self::Array(a) => {
// 				let mut sizes = Vec::with_capacity(self.count(|v| v.is_array() || v.is_object()));
// 				self.pre_compute_size(options, &mut sizes);
// 				let mut index = 0;
// 				a.fmt_with_size(f, options, indent, &sizes, &mut index)
// 			}
// 			Self::Object(o) => {
// 				let mut sizes = Vec::with_capacity(self.count(|v| v.is_array() || v.is_object()));
// 				self.pre_compute_size(options, &mut sizes);
// 				let mut index = 0;
// 				o.fmt_with_size(f, options, indent, &sizes, &mut index)
// 			}
// 		}
// 	}
// }

// impl<M> PrecomputeSize for crate::Value<M> {
// 	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
// 		match self {
// 			crate::Value::Null => Size::Width(4),
// 			crate::Value::Boolean(true) => Size::Width(4),
// 			crate::Value::Boolean(false) => Size::Width(5),
// 			crate::Value::Number(n) => Size::Width(n.as_str().len()),
// 			crate::Value::String(s) => Size::Width(printed_string_size(s)),
// 			crate::Value::Array(a) => pre_compute_array_size(a, options, sizes),
// 			crate::Value::Object(o) => pre_compute_object_size(
// 				o.entries().iter()
// 					.map(|e| (e.key.as_str(), &e.value)),
// 				options,
// 				sizes,
// 			),
// 		}
// 	}
// }

// impl<M> PrintWithSize for crate::Value<M> {
// 	#[inline(always)]
// 	fn fmt_with_size(
// 		&self,
// 		f: &mut fmt::Formatter,
// 		options: &Options,
// 		indent: usize,
// 		sizes: &[Size],
// 		index: &mut usize,
// 	) -> fmt::Result {
// 		match self {
// 			Self::Null => f.write_str("null"),
// 			Self::Boolean(b) => b.fmt_with(f, options, indent),
// 			Self::Number(n) => n.fmt_with(f, options, indent),
// 			Self::String(s) => s.fmt_with(f, options, indent),
// 			Self::Array(a) => a.fmt_with_size(f, options, indent, sizes, index),
// 			Self::Object(o) => o.fmt_with_size(f, options, indent, sizes, index),
// 		}
// 	}
// }

// impl<M> PrintWithSize for crate::Object<M> {
// 	#[inline(always)]
// 	fn fmt_with_size(
// 		&self,
// 		f: &mut fmt::Formatter,
// 		options: &Options,
// 		indent: usize,
// 		sizes: &[Size],
// 		index: &mut usize,
// 	) -> fmt::Result {
// 		json_syntax::print::print_object(
// 			self.entries().iter()
// 				.map(|e| (e.key.as_str(), &e.value)),
// 			f,
// 			options,
// 			indent,
// 			sizes,
// 			index,
// 		)
// 	}
// }
