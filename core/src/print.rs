use json_syntax::print::{
	pre_compute_array_size, pre_compute_object_size, print_array, print_object,
	printed_string_size, string_literal, PrecomputeSize, PrintWithSize, Size,
};
pub use json_syntax::print::{Options, Print, Printed};
use std::collections::HashSet;

use crate::{
	namespace::WithNamespace, object, BorrowWithNamespace, ExpandedDocument, Indexed, IriNamespace,
	Namespace, Object, Reference, StrippedIndexedNode, StrippedIndexedObject,
};

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<&'a ExpandedDocument<T, B, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		pre_compute_array_size(
			self.0.objects().iter().map(|o| o.with_namespace(self.1)),
			options,
			sizes,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> Print
	for WithNamespace<&'a ExpandedDocument<T, B, M>, &'a N>
{
	fn fmt_with(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
	) -> std::fmt::Result {
		let mut sizes = Vec::with_capacity(self.count(|i| i.is_json_array() || i.is_json_object()));
		self.pre_compute_size(options, &mut sizes);
		let mut index = 0;
		self.fmt_with_size(f, options, indent, &sizes, &mut index)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<&'a ExpandedDocument<T, B, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_array(
			self.0.objects().iter().map(|o| o.with_namespace(self.1)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, M, N> PrecomputeSize for WithNamespace<&'a locspan::Meta<T, M>, &'a N>
where
	WithNamespace<&'a T, &'a N>: PrecomputeSize,
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		self.0
			 .0
			.with_namespace(self.1)
			.pre_compute_size(options, sizes)
	}
}

impl<'a, T, M, N> PrintWithSize for WithNamespace<&'a locspan::Meta<T, M>, &'a N>
where
	WithNamespace<&'a T, &'a N>: PrintWithSize,
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		self.0
			 .0
			.with_namespace(self.1)
			.fmt_with_size(f, options, indent, sizes, index)
	}
}

impl<'a, T, N> PrecomputeSize for WithNamespace<&'a locspan::Stripped<T>, &'a N>
where
	WithNamespace<&'a T, &'a N>: PrecomputeSize,
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		self.0
			 .0
			.with_namespace(self.1)
			.pre_compute_size(options, sizes)
	}
}

impl<'a, T, N> PrintWithSize for WithNamespace<&'a locspan::Stripped<T>, &'a N>
where
	WithNamespace<&'a T, &'a N>: PrintWithSize,
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		self.0
			 .0
			.with_namespace(self.1)
			.fmt_with_size(f, options, indent, sizes, index)
	}
}

impl<'a, T, B, N: Namespace<T, B>> PrecomputeSize for WithNamespace<&'a Reference<T, B>, &'a N> {
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		Size::Width(printed_string_size(self.as_str()))
	}
}

impl<'a, T, B, N: Namespace<T, B>> Print for WithNamespace<&'a Reference<T, B>, &'a N> {
	fn fmt_with(
		&self,
		f: &mut std::fmt::Formatter,
		_options: &Options,
		_indent: usize,
	) -> std::fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a, T, B, N: Namespace<T, B>> PrintWithSize for WithNamespace<&'a Reference<T, B>, &'a N> {
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		_options: &Options,
		_indent: usize,
		_sizes: &[Size],
		_index: &mut usize,
	) -> std::fmt::Result {
		string_literal(self.as_str(), f)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<&'a Indexed<Object<T, B, M>, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		pre_compute_object_size(
			self.0.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(
					k.into_with_namespace(self.1).as_str(),
					v.into_with_namespace(self.1),
				)
			}),
			options,
			sizes,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<&'a Indexed<Object<T, B, M>, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_object(
			self.0.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(
					k.into_with_namespace(self.1).as_str(),
					v.into_with_namespace(self.1),
				)
			}),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<&'a HashSet<StrippedIndexedObject<T, B, M>>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		pre_compute_array_size(
			self.0.iter().map(|i| i.with_namespace(self.1)),
			options,
			sizes,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<&'a HashSet<StrippedIndexedObject<T, B, M>>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_array(
			self.0.iter().map(|i| i.with_namespace(self.1)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<&'a HashSet<StrippedIndexedNode<T, B, M>>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		pre_compute_array_size(
			self.0.iter().map(|i| i.with_namespace(self.1)),
			options,
			sizes,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<&'a HashSet<StrippedIndexedNode<T, B, M>>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_array(
			self.0.iter().map(|i| i.with_namespace(self.1)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<object::IndexedEntryValueRef<'a, T, B, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self.0 {
			object::IndexedEntryValueRef::Index(s) => Size::Width(printed_string_size(s)),
			object::IndexedEntryValueRef::Object(e) => e
				.into_with_namespace(self.1)
				.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<object::IndexedEntryValueRef<'a, T, B, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self.0 {
			object::IndexedEntryValueRef::Index(s) => string_literal(s, f),
			object::IndexedEntryValueRef::Object(e) => e
				.into_with_namespace(self.1)
				.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<object::EntryValueRef<'a, T, B, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self.0 {
			object::EntryValueRef::Value(v) => v
				.into_with_namespace(self.1)
				.pre_compute_size(options, sizes),
			object::EntryValueRef::List(l) => {
				pre_compute_array_size(l.iter().map(|i| i.with_namespace(self.1)), options, sizes)
			}
			object::EntryValueRef::Node(n) => n
				.into_with_namespace(self.1)
				.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<object::EntryValueRef<'a, T, B, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self.0 {
			object::EntryValueRef::Value(v) => v
				.into_with_namespace(self.1)
				.fmt_with_size(f, options, indent, sizes, index),
			object::EntryValueRef::List(l) => print_array(
				l.iter().map(|i| i.with_namespace(self.1)),
				f,
				options,
				indent,
				sizes,
				index,
			),
			object::EntryValueRef::Node(n) => n
				.into_with_namespace(self.1)
				.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl<'a, T, M, N: IriNamespace<T>> PrecomputeSize
	for WithNamespace<object::value::EntryRef<'a, T, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self.0 {
			object::value::EntryRef::Value(v) => v.pre_compute_size(options, sizes),
			object::value::EntryRef::Type(t) => t
				.into_with_namespace(self.1)
				.pre_compute_size(options, sizes),
			object::value::EntryRef::Language(l) => l.pre_compute_size(options, sizes),
			object::value::EntryRef::Direction(d) => d.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, T, M, N: IriNamespace<T>> PrintWithSize
	for WithNamespace<object::value::EntryRef<'a, T, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self.0 {
			object::value::EntryRef::Value(v) => v.fmt_with_size(f, options, indent, sizes, index),
			object::value::EntryRef::Type(t) => {
				t.into_with_namespace(self.1).fmt_with(f, options, indent)
			}
			object::value::EntryRef::Language(l) => l.fmt_with(f, options, indent),
			object::value::EntryRef::Direction(d) => d.fmt_with(f, options, indent),
		}
	}
}

impl<'a, T, N: IriNamespace<T>> PrecomputeSize
	for WithNamespace<object::value::TypeRef<'a, T>, &'a N>
{
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		match self.0 {
			object::value::TypeRef::Id(id) => {
				Size::Width(printed_string_size(self.1.iri(id).unwrap().as_str()))
			}
			object::value::TypeRef::Json => Size::Width(printed_string_size("@json")),
		}
	}
}

impl<'a, T, N: IriNamespace<T>> Print for WithNamespace<object::value::TypeRef<'a, T>, &'a N> {
	fn fmt_with(
		&self,
		f: &mut std::fmt::Formatter,
		_options: &Options,
		_indent: usize,
	) -> std::fmt::Result {
		match self.0 {
			object::value::TypeRef::Id(id) => string_literal(self.1.iri(id).unwrap().as_str(), f),
			object::value::TypeRef::Json => string_literal("@json", f),
		}
	}
}

impl<'a, M> PrecomputeSize for object::value::ValueEntryRef<'a, M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Literal(l) => l.pre_compute_size(options, sizes),
			Self::LangString(s) => Size::Width(printed_string_size(s)),
			Self::Json(j) => j.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, M> PrintWithSize for object::value::ValueEntryRef<'a, M> {
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self {
			Self::Literal(l) => l.fmt_with(f, options, indent),
			Self::LangString(s) => string_literal(s, f),
			Self::Json(j) => j.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl PrecomputeSize for object::value::Literal {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Null => Size::Width(4),
			Self::Boolean(b) => b.pre_compute_size(options, sizes),
			Self::Number(n) => Size::Width(n.as_str().len()),
			Self::String(s) => Size::Width(printed_string_size(s)),
		}
	}
}

impl Print for object::value::Literal {
	fn fmt_with(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
	) -> std::fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Boolean(b) => b.fmt_with(f, options, indent),
			Self::Number(n) => std::fmt::Display::fmt(n, f),
			Self::String(s) => string_literal(s, f),
		}
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<&'a object::Node<T, B, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		pre_compute_object_size(
			self.0.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(
					k.into_with_namespace(self.1).as_str(),
					v.into_with_namespace(self.1),
				)
			}),
			options,
			sizes,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<&'a object::Node<T, B, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_object(
			self.0.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(
					k.into_with_namespace(self.1).as_str(),
					v.into_with_namespace(self.1),
				)
			}),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<&'a Indexed<object::Node<T, B, M>, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		pre_compute_object_size(
			self.0.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(
					k.into_with_namespace(self.1).as_str(),
					v.into_with_namespace(self.1),
				)
			}),
			options,
			sizes,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<&'a Indexed<object::Node<T, B, M>, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_object(
			self.0.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(
					k.into_with_namespace(self.1).as_str(),
					v.into_with_namespace(self.1),
				)
			}),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<object::node::IndexedEntryValueRef<'a, T, B, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self.0 {
			object::node::IndexedEntryValueRef::Index(s) => Size::Width(printed_string_size(s)),
			object::node::IndexedEntryValueRef::Node(e) => e
				.into_with_namespace(self.1)
				.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<object::node::IndexedEntryValueRef<'a, T, B, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self.0 {
			object::node::IndexedEntryValueRef::Index(s) => string_literal(s, f),
			object::node::IndexedEntryValueRef::Node(e) => e
				.into_with_namespace(self.1)
				.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<object::node::EntryValueRef<'a, T, B, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self.0 {
			object::node::EntryValueRef::Id(v) => {
				v.with_namespace(self.1).pre_compute_size(options, sizes)
			}
			object::node::EntryValueRef::Type(v) => {
				pre_compute_array_size(v.iter().map(|i| i.with_namespace(self.1)), options, sizes)
			}
			object::node::EntryValueRef::Graph(v) => {
				v.with_namespace(self.1).pre_compute_size(options, sizes)
			}
			object::node::EntryValueRef::Included(v) => {
				v.with_namespace(self.1).pre_compute_size(options, sizes)
			}
			object::node::EntryValueRef::Reverse(v) => {
				v.with_namespace(self.1).pre_compute_size(options, sizes)
			}
			object::node::EntryValueRef::Property(v) => {
				pre_compute_array_size(v.iter().map(|i| i.with_namespace(self.1)), options, sizes)
			}
		}
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<object::node::EntryValueRef<'a, T, B, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self.0 {
			object::node::EntryValueRef::Id(v) => {
				v.with_namespace(self.1).fmt_with(f, options, indent)
			}
			object::node::EntryValueRef::Type(v) => print_array(
				v.iter().map(|i| i.with_namespace(self.1)),
				f,
				options,
				indent,
				sizes,
				index,
			),
			object::node::EntryValueRef::Graph(v) => v
				.with_namespace(self.1)
				.fmt_with_size(f, options, indent, sizes, index),
			object::node::EntryValueRef::Included(v) => v
				.with_namespace(self.1)
				.fmt_with_size(f, options, indent, sizes, index),
			object::node::EntryValueRef::Reverse(v) => v
				.with_namespace(self.1)
				.fmt_with_size(f, options, indent, sizes, index),
			object::node::EntryValueRef::Property(v) => print_array(
				v.iter().map(|i| i.with_namespace(self.1)),
				f,
				options,
				indent,
				sizes,
				index,
			),
		}
	}
}

struct ListRef<'a, T>(&'a [T]);

impl<'a, T, N> PrecomputeSize for WithNamespace<ListRef<'a, T>, &'a N>
where
	WithNamespace<&'a T, &'a N>: PrecomputeSize,
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		pre_compute_array_size(
			self.0 .0.iter().map(|i| i.with_namespace(self.1)),
			options,
			sizes,
		)
	}
}

impl<'a, T, N> PrintWithSize for WithNamespace<ListRef<'a, T>, &'a N>
where
	WithNamespace<&'a T, &'a N>: PrintWithSize,
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_array(
			self.0 .0.iter().map(|i| i.with_namespace(self.1)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrecomputeSize
	for WithNamespace<&'a object::node::ReverseProperties<T, B, M>, &'a N>
{
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		pre_compute_object_size(
			self.0.iter().map(|(k, v)| {
				(
					k.into_with_namespace(self.1).as_str(),
					ListRef(v).into_with_namespace(self.1),
				)
			}),
			options,
			sizes,
		)
	}
}

impl<'a, T, B, M, N: Namespace<T, B>> PrintWithSize
	for WithNamespace<&'a object::node::ReverseProperties<T, B, M>, &'a N>
{
	fn fmt_with_size(
		&self,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_object(
			self.0.iter().map(|(k, v)| {
				(
					k.into_with_namespace(self.1).as_str(),
					ListRef(v).into_with_namespace(self.1),
				)
			}),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}
