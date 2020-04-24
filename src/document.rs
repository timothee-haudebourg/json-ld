use std::collections::HashSet;
use futures::future::{LocalBoxFuture, FutureExt};
use iref::{
	Iri,
	IriBuf
};
use json::JsonValue;
use crate::{
	Error,
	Id,
	Indexed,
	Object,
	ContextMut,
	context::{
		self,
		Loader
	},
	expansion
};

pub type ExpandedDocument<T> = HashSet<Indexed<Object<T>>>;

/// JSON-LD document.
///
/// This trait represent a JSON-LD document that can be expanded into an [`ExpandedDocument`].
pub trait Document<T: Id> {
	type LocalContext: context::Local<T>;

	/// Document location, is any.
	fn base_url(&self) -> Option<Iri>;

	/// Expand the document with a custom `base_url`, a custom `loaded` and user defined
	/// expansion options.
	fn expand_with<'a, C: ContextMut<T>, L: Loader>(&'a self, base_url: Option<Iri>, context: &'a C, loader: &'a mut L, options: expansion::Options) -> LocalBoxFuture<'a, Result<ExpandedDocument<T>, Error>> where
		C::LocalContext: From<L::Output> + From<Self::LocalContext>,
		L::Output: Into<Self::LocalContext>,
		T: 'a;

	/// Expand the document with the default loader, and options.
	fn expand<'a, C: ContextMut<T>, L: Loader>(&'a self, context: &'a C, loader: &'a mut L) -> LocalBoxFuture<'a, Result<ExpandedDocument<T>, Error>> where
		C::LocalContext: From<L::Output> + From<Self::LocalContext>,
		L::Output: Into<Self::LocalContext>,
		T: 'a
	{
		self.expand_with(self.base_url(), context, loader, expansion::Options::default())
	}
}

impl<T: Id> Document<T> for JsonValue {
	type LocalContext = JsonValue;

	fn base_url(&self) -> Option<Iri> {
		None
	}

	fn expand_with<'a, C: ContextMut<T>, L: Loader>(&'a self, base_url: Option<Iri>, context: &'a C, loader: &'a mut L, options: expansion::Options) -> LocalBoxFuture<'a, Result<ExpandedDocument<T>, Error>> where
		C::LocalContext: From<L::Output> + From<JsonValue>,
		L::Output: Into<JsonValue>,
		T: 'a
	{
		expansion::expand(context, self, base_url, loader, options).boxed_local()
	}
}

/// Remote JSON-LD document.
///
/// Represent a document located at a given base URL.
pub struct RemoteDocument<D = JsonValue> {
	base_url: IriBuf,
	doc: D,
}

impl<D> RemoteDocument<D> {
	pub fn new(doc: D, base_url: Iri) -> RemoteDocument<D> {
		RemoteDocument {
			base_url: base_url.into(),
			doc: doc
		}
	}
}

impl<T: Id, D: Document<T>> Document<T> for RemoteDocument<D> {
	type LocalContext = D::LocalContext;

	fn base_url(&self) -> Option<Iri> {
		Some(self.base_url.as_iri())
	}

	fn expand_with<'a, C: ContextMut<T>, L: Loader>(&'a self, base_url: Option<Iri>, context: &'a C, loader: &'a mut L, options: expansion::Options) -> LocalBoxFuture<'a, Result<ExpandedDocument<T>, Error>> where
		C::LocalContext: From<L::Output> + From<Self::LocalContext>,
		L::Output: Into<Self::LocalContext>,
		T: 'a
	{
		self.doc.expand_with(base_url, context, loader, options)
	}
}
