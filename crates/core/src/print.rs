use json_syntax::print::{
	pre_compute_array_size, pre_compute_object_size, print_array, print_object,
	printed_string_size, string_literal, PrecomputeSize, PrecomputeSizeWithContext,
	PrintWithContext, PrintWithSize, PrintWithSizeAndContext, Size,
};
pub use json_syntax::print::{Options, Print, Printed};

use crate::{object, ExpandedDocument, Id, Indexed, Object};
use contextual::WithContext;
use rdf_types::vocabulary::{IriVocabulary, Vocabulary};

pub trait PrintWithSizeAndVocabulary<V> {
	fn fmt_with_size_and(
		&self,
		vocabulary: &V,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result;
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for ExpandedDocument<T, B>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		pre_compute_array_size(
			self.objects().iter().map(|o| o.with(vocabulary)),
			options,
			sizes,
		)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithContext<N> for ExpandedDocument<T, B> {
	fn contextual_fmt_with(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
	) -> std::fmt::Result {
		let mut sizes = Vec::with_capacity(self.count(|i| i.is_json_array() || i.is_json_object()));
		self.contextual_pre_compute_size(vocabulary, options, &mut sizes);
		let mut index = 0;
		self.contextual_fmt_with_size(vocabulary, f, options, indent, &sizes, &mut index)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithContext<N> for object::Node<T, B> {
	fn contextual_fmt_with(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
	) -> std::fmt::Result {
		let mut sizes = Vec::with_capacity(self.count(|i| i.is_json_array() || i.is_json_object()));
		self.contextual_pre_compute_size(vocabulary, options, &mut sizes);
		let mut index = 0;
		self.contextual_fmt_with_size(vocabulary, f, options, indent, &sizes, &mut index)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N>
	for ExpandedDocument<T, B>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_array(
			self.objects().iter().map(|o| o.with(vocabulary)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N> for Id<T, B> {
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		_options: &Options,
		_sizes: &mut Vec<Size>,
	) -> Size {
		Size::Width(printed_string_size(self.with(vocabulary).as_str()))
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithContext<N> for Id<T, B> {
	fn contextual_fmt_with(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		_options: &Options,
		_indent: usize,
	) -> std::fmt::Result {
		string_literal(self.with(vocabulary).as_str(), f)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N> for Id<T, B> {
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		_options: &Options,
		_indent: usize,
		_sizes: &[Size],
		_index: &mut usize,
	) -> std::fmt::Result {
		string_literal(self.with(vocabulary).as_str(), f)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for Indexed<Object<T, B>>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		pre_compute_object_size(
			self.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(k.into_with(vocabulary).into_str(), v.into_with(vocabulary))
			}),
			options,
			sizes,
		)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N>
	for Indexed<Object<T, B>>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_object(
			self.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(k.into_with(vocabulary).into_str(), v.into_with(vocabulary))
			}),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for object::IndexedEntryValueRef<'a, T, B>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		match self {
			object::IndexedEntryValueRef::Index(s) => Size::Width(printed_string_size(s)),
			object::IndexedEntryValueRef::Object(e) => {
				e.into_with(vocabulary).pre_compute_size(options, sizes)
			}
		}
	}
}

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N>
	for object::IndexedEntryValueRef<'a, T, B>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self {
			object::IndexedEntryValueRef::Index(s) => string_literal(s, f),
			object::IndexedEntryValueRef::Object(e) => e
				.into_with(vocabulary)
				.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for object::EntryValueRef<'a, T, B>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		match self {
			object::EntryValueRef::Value(v) => {
				v.into_with(vocabulary).pre_compute_size(options, sizes)
			}
			object::EntryValueRef::List(l) => {
				pre_compute_array_size(l.iter().map(|i| i.with(vocabulary)), options, sizes)
			}
			object::EntryValueRef::Node(n) => {
				n.into_with(vocabulary).pre_compute_size(options, sizes)
			}
		}
	}
}

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N>
	for object::EntryValueRef<'a, T, B>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self {
			object::EntryValueRef::Value(v) => v
				.into_with(vocabulary)
				.fmt_with_size(f, options, indent, sizes, index),
			object::EntryValueRef::List(l) => print_array(
				l.iter().map(|i| i.with(vocabulary)),
				f,
				options,
				indent,
				sizes,
				index,
			),
			object::EntryValueRef::Node(n) => n
				.into_with(vocabulary)
				.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl<'a, T, N: IriVocabulary<Iri = T>> PrecomputeSizeWithContext<N>
	for object::value::EntryRef<'a, T>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		match self {
			object::value::EntryRef::Value(v) => v.pre_compute_size(options, sizes),
			object::value::EntryRef::Type(t) => {
				t.into_with(vocabulary).pre_compute_size(options, sizes)
			}
			object::value::EntryRef::Language(l) => l.pre_compute_size(options, sizes),
			object::value::EntryRef::Direction(d) => d.pre_compute_size(options, sizes),
		}
	}
}

impl<'a, T, N: IriVocabulary<Iri = T>> PrintWithSizeAndContext<N>
	for object::value::EntryRef<'a, T>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self {
			object::value::EntryRef::Value(v) => v.fmt_with_size(f, options, indent, sizes, index),
			object::value::EntryRef::Type(t) => {
				t.into_with(vocabulary).fmt_with(f, options, indent)
			}
			object::value::EntryRef::Language(l) => l.fmt_with(f, options, indent),
			object::value::EntryRef::Direction(d) => d.fmt_with(f, options, indent),
		}
	}
}

impl<'a, T, N: IriVocabulary<Iri = T>> PrecomputeSizeWithContext<N>
	for object::value::TypeRef<'a, T>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		_options: &Options,
		_sizes: &mut Vec<Size>,
	) -> Size {
		match self {
			object::value::TypeRef::Id(id) => {
				Size::Width(printed_string_size(vocabulary.iri(id).unwrap().as_str()))
			}
			object::value::TypeRef::Json => Size::Width(printed_string_size("@json")),
		}
	}
}

impl<'a, T, N: IriVocabulary<Iri = T>> PrintWithContext<N> for object::value::TypeRef<'a, T> {
	fn contextual_fmt_with(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		_options: &Options,
		_indent: usize,
	) -> std::fmt::Result {
		match self {
			object::value::TypeRef::Id(id) => {
				string_literal(vocabulary.iri(id).unwrap().as_str(), f)
			}
			object::value::TypeRef::Json => string_literal("@json", f),
		}
	}
}

impl<'a> PrecomputeSize for object::value::ValueEntryRef<'a> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			Self::Literal(l) => l.pre_compute_size(options, sizes),
			Self::LangString(s) => Size::Width(printed_string_size(s)),
			Self::Json(j) => j.pre_compute_size(options, sizes),
		}
	}
}

impl<'a> PrintWithSize for object::value::ValueEntryRef<'a> {
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

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for object::Node<T, B>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		pre_compute_object_size(
			self.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(k.into_with(vocabulary).into_str(), v.into_with(vocabulary))
			}),
			options,
			sizes,
		)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N> for object::Node<T, B> {
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_object(
			self.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(k.into_with(vocabulary).into_str(), v.into_with(vocabulary))
			}),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for Indexed<object::Node<T, B>>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		pre_compute_object_size(
			self.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(k.into_with(vocabulary).into_str(), v.into_with(vocabulary))
			}),
			options,
			sizes,
		)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N>
	for Indexed<object::Node<T, B>>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_object(
			self.entries().map(|e| {
				let (k, v) = e.into_key_value();
				(k.into_with(vocabulary).into_str(), v.into_with(vocabulary))
			}),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for object::node::IndexedEntryValueRef<'a, T, B>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		match self {
			object::node::IndexedEntryValueRef::Index(s) => Size::Width(printed_string_size(s)),
			object::node::IndexedEntryValueRef::Node(e) => {
				e.into_with(vocabulary).pre_compute_size(options, sizes)
			}
		}
	}
}

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N>
	for object::node::IndexedEntryValueRef<'a, T, B>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self {
			object::node::IndexedEntryValueRef::Index(s) => string_literal(s, f),
			object::node::IndexedEntryValueRef::Node(e) => e
				.into_with(vocabulary)
				.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for object::node::EntryValueRef<'a, T, B>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		match *self {
			object::node::EntryValueRef::Id(v) => {
				v.contextual_pre_compute_size(vocabulary, options, sizes)
			}
			object::node::EntryValueRef::Type(v) => {
				pre_compute_array_size(v.iter().map(|i| i.with(vocabulary)), options, sizes)
			}
			object::node::EntryValueRef::Graph(v) => {
				pre_compute_array_size(v.iter().map(|i| i.with(vocabulary)), options, sizes)
				// v.contextual_pre_compute_size(vocabulary, options, sizes)
			}
			object::node::EntryValueRef::Included(v) => {
				pre_compute_array_size(v.iter().map(|i| i.with(vocabulary)), options, sizes)
				// v.contextual_pre_compute_size(vocabulary, options, sizes)
			}
			object::node::EntryValueRef::Reverse(v) => {
				v.contextual_pre_compute_size(vocabulary, options, sizes)
			}
			object::node::EntryValueRef::Property(v) => {
				pre_compute_array_size(v.iter().map(|i| i.with(vocabulary)), options, sizes)
			}
		}
	}
}

impl<'a, T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N>
	for object::node::EntryValueRef<'a, T, B>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		match self {
			object::node::EntryValueRef::Id(v) => v.with(vocabulary).fmt_with(f, options, indent),
			object::node::EntryValueRef::Type(v) => print_array(
				v.iter().map(|i| i.with(vocabulary)),
				f,
				options,
				indent,
				sizes,
				index,
			),
			object::node::EntryValueRef::Graph(v) => {
				// v.contextual_fmt_with_size(vocabulary, f, options, indent, sizes, index)
				print_array(
					v.iter().map(|i| i.with(vocabulary)),
					f,
					options,
					indent,
					sizes,
					index,
				)
			}
			object::node::EntryValueRef::Included(v) => {
				// v.contextual_fmt_with_size(vocabulary, f, options, indent, sizes, index)
				print_array(
					v.iter().map(|i| i.with(vocabulary)),
					f,
					options,
					indent,
					sizes,
					index,
				)
			}
			object::node::EntryValueRef::Reverse(v) => {
				v.contextual_fmt_with_size(vocabulary, f, options, indent, sizes, index)
			}
			object::node::EntryValueRef::Property(v) => print_array(
				v.iter().map(|i| i.with(vocabulary)),
				f,
				options,
				indent,
				sizes,
				index,
			),
		}
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrecomputeSizeWithContext<N>
	for object::node::ReverseProperties<T, B>
{
	fn contextual_pre_compute_size(
		&self,
		vocabulary: &N,
		options: &Options,
		sizes: &mut Vec<Size>,
	) -> Size {
		pre_compute_object_size(
			self.iter()
				.map(|(k, v)| (k.into_with(vocabulary).as_str(), v.into_with(vocabulary))),
			options,
			sizes,
		)
	}
}

impl<T, B, N: Vocabulary<Iri = T, BlankId = B>> PrintWithSizeAndContext<N>
	for object::node::ReverseProperties<T, B>
{
	fn contextual_fmt_with_size(
		&self,
		vocabulary: &N,
		f: &mut std::fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> std::fmt::Result {
		print_object(
			self.iter()
				.map(|(k, v)| (k.into_with(vocabulary).as_str(), v.into_with(vocabulary))),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}
