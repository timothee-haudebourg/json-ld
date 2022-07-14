use super::{
	ComponentRef, ContextComponentRef, ContextEntryRef, ContextRef, Count, EntryRef,
	ExpandedTermDefinitionRef, IntoCount, TermDefinitionEntryRef, TermDefinitionRef, ValueRef,
};
use crate::{AnyContextDefinition, AnyContextEntry, ContainerRef, Nullable};
use json_syntax::print::{string_literal, Options, PrecomputeSize, Print, PrintWithSize, Size};
use locspan::Meta;
use std::fmt;

impl<M: Clone + Send + Sync> PrintWithSize for super::ContextEntry<M> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		self.as_entry_ref()
			.fmt_with_size(f, options, indent, sizes, index)
	}
}

impl<M: Clone + Send + Sync> PrecomputeSize for super::ContextEntry<M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		self.as_entry_ref().pre_compute_size(options, sizes)
	}
}

impl<C: AnyContextEntry> Count<C> for C {
	fn count<F>(&self, f: F) -> usize
	where
		F: Clone + Fn(ComponentRef<Self>) -> bool,
	{
		self.as_entry_ref().into_count(f)
	}
}

impl<'a, M, C: AnyContextEntry> IntoCount<C>
	for ContextEntryRef<'a, M, C::Definition, C::Definitions<'a>>
{
	fn into_count<F>(self, f: F) -> usize
	where
		F: Clone + Fn(ComponentRef<C>) -> bool,
	{
		let mut count = 0;

		match self {
			Self::One(Meta(context, _)) => {
				if f(ComponentRef::Context(ContextComponentRef::Context(context))) {
					count += 1;
				}
			}
			Self::Many(contexts) => {
				if f(ComponentRef::Context(ContextComponentRef::ContextArray)) {
					count += 1;
				}

				for Meta(context, _) in contexts {
					count += context.into_count(f.clone())
				}
			}
		}

		count
	}
}

impl<'a, M, D, S: Clone> PrecomputeSize for ContextEntryRef<'a, M, D, S>
where
	S: Iterator<Item = Meta<ContextRef<'a, D>, M>>,
	D: AnyContextDefinition,
	D::ContextEntry: PrecomputeSize,
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

impl<'a, M, D, S: Clone> PrintWithSize for ContextEntryRef<'a, M, D, S>
where
	S: ExactSizeIterator<Item = Meta<ContextRef<'a, D>, M>>,
	D: AnyContextDefinition,
	D::ContextEntry: PrintWithSize,
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

impl<'a, C: AnyContextEntry> IntoCount<C> for ContextRef<'a, C::Definition> {
	fn into_count<F>(self, f: F) -> usize
	where
		F: Clone + Fn(ComponentRef<C>) -> bool,
	{
		let mut count = if f(ComponentRef::Context(ContextComponentRef::Context(self))) {
			1
		} else {
			0
		};

		if let Self::Definition(d) = self {
			for entry in d.entries() {
				match entry {
					EntryRef::Definition(_, b) => match b.definition {
						Meta(Nullable::Some(d), meta) => {
							count += Meta(d, meta).into_count(f.clone())
						}
						definition => {
							if f(ComponentRef::Context(ContextComponentRef::ContextEntry(
								ValueRef::Definition(definition),
							))) {
								count += 1
							}
						}
					},
					entry => {
						if f(ComponentRef::Context(ContextComponentRef::ContextEntry(
							entry.into_value(),
						))) {
							count += 1
						}
					}
				}
			}
		}

		count
	}
}

impl<'a, D> PrecomputeSize for ContextRef<'a, D>
where
	D: AnyContextDefinition,
	D::ContextEntry: PrecomputeSize,
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::IriRef(r) => Size::Width(json_syntax::print::printed_string_size(r.as_str())),
			Self::Definition(d) => json_syntax::print::pre_compute_object_size(
				d.entries().map(|entry| {
					let (key, value) = entry.into_pair();
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
	D: AnyContextDefinition,
	D::ContextEntry: PrintWithSize,
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
					let (key, value) = entry.into_pair();
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

impl<'a, C: AnyContextEntry + PrecomputeSize> PrecomputeSize for ValueRef<'a, C> {
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

impl<'a, C: AnyContextEntry + PrintWithSize> PrintWithSize for ValueRef<'a, C> {
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

impl<'a, M> PrecomputeSize for super::ContextType<M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		json_syntax::print::pre_compute_object_size(
			self.iter().map(|entry| (entry.key().as_str(), entry)),
			options,
			sizes,
		)
	}
}

impl<'a, M> PrintWithSize for super::ContextType<M> {
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

impl<'a, M> PrecomputeSize for super::ContextTypeEntry<'a, M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Container(c) => c.pre_compute_size(options, sizes),
			Self::Protected(p) => p.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, M> PrintWithSize for super::ContextTypeEntry<'a, M> {
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

impl PrecomputeSize for super::TypeContainer {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for super::TypeContainer {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl PrecomputeSize for super::Version {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::V1_1 => Size::Width(3),
		}
	}
}

impl Print for super::Version {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		match self {
			Self::V1_1 => write!(f, "1.1"),
		}
	}
}

impl<'a> PrecomputeSize for super::VocabRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for super::VocabRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for crate::Nullable<super::VocabRef<'a>> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for crate::Nullable<super::VocabRef<'a>> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl<'a, C: AnyContextEntry> IntoCount<C> for Meta<TermDefinitionRef<'a, C>, C::Metadata> {
	fn into_count<F>(self, f: F) -> usize
	where
		F: Clone + Fn(ComponentRef<C>) -> bool,
	{
		let mut count = 0;

		match self {
			Meta(TermDefinitionRef::Expanded(e), _) => count += e.into_count(f),
			Meta(other, meta) => {
				if f(ComponentRef::Context(ContextComponentRef::ContextEntry(
					ValueRef::Definition(Meta(Nullable::Some(other), meta)),
				))) {
					count += 1
				}
			}
		}

		count
	}
}

impl<'a, C: AnyContextEntry + PrecomputeSize> PrecomputeSize
	for crate::Nullable<super::TermDefinitionRef<'a, C>>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(c) => c.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, C: AnyContextEntry + PrecomputeSize> PrecomputeSize for super::TermDefinitionRef<'a, C> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Iri(i) => Size::Width(json_syntax::print::printed_string_size(i.as_str())),
			Self::CompactIri(i) => Size::Width(json_syntax::print::printed_string_size(i.as_str())),
			Self::Blank(i) => Size::Width(json_syntax::print::printed_string_size(i.as_str())),
			Self::Expanded(d) => d.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, C: AnyContextEntry + PrintWithSize> PrintWithSize for super::TermDefinitionRef<'a, C> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Iri(i) => string_literal(i.as_str(), f),
			Self::CompactIri(i) => string_literal(i.as_str(), f),
			Self::Blank(i) => string_literal(i.as_str(), f),
			Self::Expanded(d) => d.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl<'a, C: AnyContextEntry + PrintWithSize> PrintWithSize
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

impl<'a, C: AnyContextEntry> IntoCount<C> for ExpandedTermDefinitionRef<'a, C> {
	fn into_count<F>(self, f: F) -> usize
	where
		F: Clone + Fn(ComponentRef<C>) -> bool,
	{
		let mut count = if f(ComponentRef::Context(
			ContextComponentRef::ExpandedTermDefinition,
		)) {
			1
		} else {
			0
		};

		for entry in self {
			count += entry.into_count(f.clone())
		}

		count
	}
}

impl<'a, C: AnyContextEntry> IntoCount<C> for TermDefinitionEntryRef<'a, C> {
	fn into_count<F>(self, f: F) -> usize
	where
		F: Clone + Fn(ComponentRef<C>) -> bool,
	{
		let mut count = 0;

		if let TermDefinitionEntryRef::Container(Meta(
			Nullable::Some(ContainerRef::Many(containers)),
			_,
		)) = &self
		{
			for c in *containers {
				if f(ComponentRef::Context(
					ContextComponentRef::ExpandedTermDefinitionContainer(c),
				)) {
					count += 1
				}
			}
		}

		if f(ComponentRef::Context(
			ContextComponentRef::ExpandedTermDefinitionEntry(self),
		)) {
			count += 1
		}

		count
	}
}

impl<'a, C: AnyContextEntry + PrecomputeSize> PrecomputeSize
	for super::ExpandedTermDefinitionRef<'a, C>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		json_syntax::print::pre_compute_object_size(
			self.iter().map(|entry| (entry.key().as_str(), entry)),
			options,
			sizes,
		)
	}
}

impl<'a, C: AnyContextEntry + PrintWithSize> PrintWithSize
	for super::ExpandedTermDefinitionRef<'a, C>
{
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

impl<'a, C: AnyContextEntry + PrecomputeSize> PrecomputeSize
	for super::TermDefinitionEntryRef<'a, C>
{
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

impl<'a, C: AnyContextEntry + PrintWithSize> PrintWithSize
	for super::TermDefinitionEntryRef<'a, C>
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

impl<'a> PrecomputeSize for super::IdRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for super::IdRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for crate::Nullable<super::IdRef<'a>> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for crate::Nullable<super::IdRef<'a>> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl<'a> PrecomputeSize for super::TermDefinitionTypeRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for super::TermDefinitionTypeRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for crate::Nullable<super::TermDefinitionTypeRef<'a>> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Some(v) => v.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> Print for crate::Nullable<super::TermDefinitionTypeRef<'a>> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Some(v) => v.fmt_with(f, options, indent),
		}
	}
}

impl<'a> PrecomputeSize for super::KeyRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for super::KeyRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for super::IndexRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for super::IndexRef<'a> {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a> PrecomputeSize for super::NestRef<'a> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl<'a> Print for super::NestRef<'a> {
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

impl PrecomputeSize for crate::ContainerType {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(json_syntax::print::printed_string_size(self.as_str()))
	}
}

impl Print for crate::ContainerType {
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl PrintWithSize for crate::ContainerType {
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
