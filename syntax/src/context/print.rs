use super::{
	definition, term_definition, AnyDefinition, AnyValue, ContextRef, TermDefinitionRef, ValueRef,
};
use crate::Nullable;
use json_syntax::print::{string_literal, Options, PrecomputeSize, Print, PrintWithSize, Size};
use locspan::Meta;
use std::fmt;

impl<M: Clone + Send + Sync> PrintWithSize for super::Value<M> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		self.as_value_ref()
			.fmt_with_size(f, options, indent, sizes, index)
	}
}

impl<M: Clone + Send + Sync> PrecomputeSize for super::Value<M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		self.as_value_ref().pre_compute_size(options, sizes)
	}
}

impl<'a, M, D, S: Clone> PrecomputeSize for ValueRef<'a, M, D, S>
where
	S: Iterator<Item = Meta<ContextRef<'a, D>, M>>,
	D: AnyDefinition,
	D::ContextValue: PrecomputeSize,
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::One(context) => context.pre_compute_size(options, sizes),
			Self::Many(contexts) => {
				json_syntax::print::pre_compute_array_size(contexts.clone(), options, sizes)
			}
		}
	}
}

impl<'a, M, D, S: Clone> PrintWithSize for ValueRef<'a, M, D, S>
where
	S: ExactSizeIterator<Item = Meta<ContextRef<'a, D>, M>>,
	D: AnyDefinition,
	D::ContextValue: PrintWithSize,
{
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
				json_syntax::print::print_array(contexts.clone(), f, options, indent, sizes, index)
			}
		}
	}
}

impl<'a, D> PrecomputeSize for ContextRef<'a, D>
where
	D: AnyDefinition,
	D::ContextValue: PrecomputeSize,
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::IriRef(r) => Size::Width(json_syntax::print::printed_string_size(r.as_str())),
			Self::Definition(d) => json_syntax::print::pre_compute_object_size(
				d.entries().map(|entry| {
					let (key, value) = entry.into_key_value();
					(key.as_str(), value)
				}),
				options,
				sizes,
			),
		}
	}
}

impl<'a, D> PrintWithSize for ContextRef<'a, D>
where
	D: AnyDefinition,
	D::ContextValue: PrintWithSize,
{
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
			Self::IriRef(r) => string_literal(r.as_str(), f),
			Self::Definition(d) => json_syntax::print::print_object(
				d.entries().map(|entry| {
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

impl<'a, C: AnyValue + PrecomputeSize> PrecomputeSize for definition::EntryValueRef<'a, C> {
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

impl<'a, C: AnyValue + PrintWithSize> PrintWithSize for definition::EntryValueRef<'a, C> {
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

impl<M> PrecomputeSize for definition::Type<M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		json_syntax::print::pre_compute_object_size(
			self.iter().map(|entry| (entry.key().as_str(), entry)),
			options,
			sizes,
		)
	}
}

impl<M> PrintWithSize for definition::Type<M> {
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

impl<'a, M> PrecomputeSize for definition::ContextTypeEntry<'a, M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Container(c) => c.pre_compute_size(options, sizes),
			Self::Protected(p) => p.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, M> PrintWithSize for definition::ContextTypeEntry<'a, M> {
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

impl<'a> PrecomputeSize for definition::VocabRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for definition::VocabRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for Nullable<definition::VocabRef<'a>> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for crate::Nullable<definition::VocabRef<'a>> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl<'a, C: AnyValue + PrecomputeSize> PrecomputeSize
	for crate::Nullable<super::TermDefinitionRef<'a, C>>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(c) => c.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, C: AnyValue + PrecomputeSize> PrecomputeSize for TermDefinitionRef<'a, C> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Simple(s) => s.pre_compute_size(options, sizes),
			Self::Expanded(d) => d.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> PrecomputeSize for term_definition::SimpleRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a, C: AnyValue + PrintWithSize> PrintWithSize for TermDefinitionRef<'a, C> {
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

impl<'a> Print for term_definition::SimpleRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a, C: AnyValue + PrintWithSize> PrintWithSize
	for crate::Nullable<super::TermDefinitionRef<'a, C>>
{
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

impl<'a, C: AnyValue + PrecomputeSize> PrecomputeSize for term_definition::ExpandedRef<'a, C> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		json_syntax::print::pre_compute_object_size(
			self.iter().map(|entry| (entry.key().as_str(), entry)),
			options,
			sizes,
		)
	}
}

impl<'a, C: AnyValue + PrintWithSize> PrintWithSize for term_definition::ExpandedRef<'a, C> {
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

impl<'a, C: AnyValue + PrecomputeSize> PrecomputeSize for term_definition::EntryRef<'a, C> {
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

impl<'a, C: AnyValue + PrintWithSize> PrintWithSize for term_definition::EntryRef<'a, C> {
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

impl<'a> PrecomputeSize for term_definition::IdRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for term_definition::IdRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for Nullable<term_definition::IdRef<'a>> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for Nullable<term_definition::IdRef<'a>> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl<'a> PrecomputeSize for term_definition::TypeRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for term_definition::TypeRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for Nullable<term_definition::TypeRef<'a>> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for Nullable<term_definition::TypeRef<'a>> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl<'a> PrecomputeSize for definition::KeyRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for definition::KeyRef<'a> {
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

impl<'a> PrecomputeSize for term_definition::IndexRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for term_definition::IndexRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for term_definition::NestRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for term_definition::NestRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a, M> PrecomputeSize for crate::Nullable<crate::ContainerRef<'a, M>> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, M> PrintWithSize for crate::Nullable<crate::ContainerRef<'a, M>> {
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

impl<'a, M> PrecomputeSize for crate::ContainerRef<'a, M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::One(c) => c.pre_compute_size(options, sizes),
			Self::Many(m) => json_syntax::print::pre_compute_array_size(*m, options, sizes),
		}
	}
}

impl<'a, M> PrintWithSize for crate::ContainerRef<'a, M> {
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
			Self::Many(m) => json_syntax::print::print_array(*m, f, options, indent, sizes, index),
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
