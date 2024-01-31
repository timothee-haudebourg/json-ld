use super::{definition, term_definition, TermDefinition};
use crate::{Container, ContextEntry, Nullable};
use json_syntax::print::{string_literal, Options, PrecomputeSize, Print, PrintWithSize, Size};
use std::fmt;

impl Print for super::Context {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		let mut sizes = Vec::with_capacity(
			self.traverse()
				.filter(|f| f.is_array() || f.is_object())
				.count(),
		);
		self.pre_compute_size(options, &mut sizes);
		let mut index = 0;
		self.fmt_with_size(f, options, indent, &sizes, &mut index)
	}
}

impl PrecomputeSize for super::Context {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::One(context) => context.pre_compute_size(options, sizes),
			Self::Many(contexts) => {
				json_syntax::print::pre_compute_array_size(contexts, options, sizes)
			}
		}
	}
}

impl PrintWithSize for super::Context {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::One(context) => context.fmt_with_size(f, options, indent, sizes, index),
			Self::Many(contexts) => {
				json_syntax::print::print_array(contexts, f, options, indent, sizes, index)
			}
		}
	}
}

// struct ForMeta<T, M>(T, PhantomData);

// trait IntoForMeta: Sized {
// 	fn for_meta(self) -> ForMeta<Self, M> {
// 		ForMeta(self, PhantomData)
// 	}
// }

// impl<T> IntoForMeta for T {}

impl PrecomputeSize for ContextEntry {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			ContextEntry::Null => Size::Width(4),
			ContextEntry::IriRef(r) => {
				Size::Width(json_syntax::print::printed_string_size(r.as_str()))
			}
			ContextEntry::Definition(d) => json_syntax::print::pre_compute_object_size(
				d.iter().map(|entry| {
					let (key, value) = entry.into_key_value();
					(key.as_str(), value)
				}),
				options,
				sizes,
			),
		}
	}
}

impl PrintWithSize for ContextEntry {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			ContextEntry::Null => write!(f, "null"),
			ContextEntry::IriRef(r) => string_literal(r.as_str(), f),
			ContextEntry::Definition(d) => json_syntax::print::print_object(
				d.iter().map(|entry| {
					let (key, value) = entry.into_key_value();
					(key.as_str(), value)
				}),
				f,
				options,
				indent,
				sizes,
				index,
			),
		}
	}
}

impl<'a> PrecomputeSize for definition::EntryValueRef<'a> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Base(v) => v.pre_compute_size(options, sizes),
			Self::Import(v) => Size::Width(json_syntax::print::printed_string_size(v.as_str())),
			Self::Language(v) => v.pre_compute_size(options, sizes),
			Self::Direction(v) => v.pre_compute_size(options, sizes),
			Self::Propagate(v) => v.pre_compute_size(options, sizes),
			Self::Protected(v) => v.pre_compute_size(options, sizes),
			Self::Type(v) => v.pre_compute_size(options, sizes),
			Self::Version(v) => v.pre_compute_size(options, sizes),
			Self::Vocab(v) => v.pre_compute_size(options, sizes),
			Self::Definition(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> PrintWithSize for definition::EntryValueRef<'a> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Base(v) => v.fmt_with(f, options, indent),
			Self::Import(v) => string_literal(v.as_str(), f),
			Self::Language(v) => v.fmt_with(f, options, indent),
			Self::Direction(v) => v.fmt_with(f, options, indent),
			Self::Propagate(v) => v.fmt_with(f, options, indent),
			Self::Protected(v) => v.fmt_with(f, options, indent),
			Self::Type(v) => v.fmt_with_size(f, options, indent, sizes, index),
			Self::Version(v) => v.fmt_with(f, options, indent),
			Self::Vocab(v) => v.fmt_with(f, options, indent),
			Self::Definition(v) => v.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl PrecomputeSize for definition::Type {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		json_syntax::print::pre_compute_object_size(
			self.iter().map(|entry| (entry.key().as_str(), entry)),
			options,
			sizes,
		)
	}
}

impl PrintWithSize for definition::Type {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		json_syntax::print::print_object(
			self.iter().map(|entry| (entry.key().as_str(), entry)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl PrecomputeSize for definition::ContextTypeEntry {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Container(c) => c.pre_compute_size(options, sizes),
			Self::Protected(p) => p.pre_compute_size(options, sizes),
		}
	}
}

impl PrintWithSize for definition::ContextTypeEntry {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		_sizes: &[Size],
		_index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Container(c) => c.fmt_with(f, options, indent),
			Self::Protected(p) => p.fmt_with(f, options, indent),
		}
	}
}

impl PrecomputeSize for definition::TypeContainer {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.into_str()))
	}
}

impl Print for definition::TypeContainer {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.into_str(), f)
	}
}

impl PrecomputeSize for definition::Version {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::V1_1 => Size::Width(3),
		}
	}
}

impl Print for definition::Version {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		match self {
			Self::V1_1 => write!(f, "1.1"),
		}
	}
}

impl PrecomputeSize for definition::Vocab {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for definition::Vocab {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for Nullable<&'a definition::Vocab> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for Nullable<&'a definition::Vocab> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl<'a> PrecomputeSize for Nullable<&'a TermDefinition> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(c) => c.pre_compute_size(options, sizes),
		}
	}
}

impl PrecomputeSize for TermDefinition {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Simple(s) => s.pre_compute_size(options, sizes),
			Self::Expanded(d) => d.pre_compute_size(options, sizes),
		}
	}
}

impl PrecomputeSize for term_definition::Simple {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl PrintWithSize for TermDefinition {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Simple(i) => i.fmt_with(f, options, indent),
			Self::Expanded(d) => d.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl Print for term_definition::Simple {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrintWithSize for Nullable<&'a TermDefinition> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl PrecomputeSize for term_definition::Expanded {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		json_syntax::print::pre_compute_object_size(
			self.iter().map(|entry| (entry.key().as_str(), entry)),
			options,
			sizes,
		)
	}
}

impl PrintWithSize for term_definition::Expanded {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		json_syntax::print::print_object(
			self.iter().map(|entry| (entry.key().as_str(), entry)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a> PrecomputeSize for term_definition::EntryRef<'a> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Id(v) => v.pre_compute_size(options, sizes),
			Self::Type(v) => v.pre_compute_size(options, sizes),
			Self::Context(v) => v.pre_compute_size(options, sizes),
			Self::Reverse(v) => v.pre_compute_size(options, sizes),
			Self::Index(v) => v.pre_compute_size(options, sizes),
			Self::Language(v) => v.pre_compute_size(options, sizes),
			Self::Direction(v) => v.pre_compute_size(options, sizes),
			Self::Container(v) => v.pre_compute_size(options, sizes),
			Self::Nest(v) => v.pre_compute_size(options, sizes),
			Self::Prefix(v) => v.pre_compute_size(options, sizes),
			Self::Propagate(v) => v.pre_compute_size(options, sizes),
			Self::Protected(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> PrintWithSize for term_definition::EntryRef<'a> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Id(v) => v.fmt_with(f, options, indent),
			Self::Type(v) => v.fmt_with(f, options, indent),
			Self::Context(v) => v.fmt_with_size(f, options, indent, sizes, index),
			Self::Reverse(v) => v.fmt_with(f, options, indent),
			Self::Index(v) => v.fmt_with(f, options, indent),
			Self::Language(v) => v.fmt_with(f, options, indent),
			Self::Direction(v) => v.fmt_with(f, options, indent),
			Self::Container(v) => v.fmt_with_size(f, options, indent, sizes, index),
			Self::Nest(v) => v.fmt_with(f, options, indent),
			Self::Prefix(v) => v.fmt_with(f, options, indent),
			Self::Propagate(v) => v.fmt_with(f, options, indent),
			Self::Protected(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl PrecomputeSize for term_definition::Id {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for term_definition::Id {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for Nullable<&'a term_definition::Id> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for Nullable<&'a term_definition::Id> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl PrecomputeSize for term_definition::Type {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for term_definition::Type {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for Nullable<&'a term_definition::Type> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for Nullable<&'a term_definition::Type> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl PrecomputeSize for definition::Key {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for definition::Key {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for definition::EntryKeyRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for definition::EntryKeyRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl PrecomputeSize for term_definition::Index {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for term_definition::Index {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl PrecomputeSize for term_definition::Nest {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for term_definition::Nest {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for Nullable<&'a Container> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> PrintWithSize for Nullable<&'a Container> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl PrecomputeSize for Container {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::One(c) => c.pre_compute_size(options, sizes),
			Self::Many(m) => json_syntax::print::pre_compute_array_size(m, options, sizes),
		}
	}
}

impl PrintWithSize for Container {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::One(c) => c.fmt_with(f, options, indent),
			Self::Many(m) => json_syntax::print::print_array(m, f, options, indent, sizes, index),
		}
	}
}

impl PrecomputeSize for crate::ContainerKind {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for crate::ContainerKind {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl PrintWithSize for crate::ContainerKind {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		_sizes: &[Size],
		_index: &mut usize,
	) -> fmt::Result {
		self.fmt_with(f, options, indent)
	}
}
