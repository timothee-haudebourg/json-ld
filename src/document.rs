use std::collections::HashSet;
use std::ops::{
	Deref,
	DerefMut
};
use futures::future::{BoxFuture, FutureExt};
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
	expansion,
	compaction
};

/// Result of the document expansion algorithm.
///
/// It is just an alias for a set of (indexed) objects.
pub type ExpandedDocument<T> = HashSet<Indexed<Object<T>>>;

/// JSON-LD document.
///
/// This trait represent a JSON-LD document that can be expanded into an [`ExpandedDocument`].
/// It is notabily implemented for the [`JsonValue`] type.
pub trait Document<T: Id> {
	/// The type of local contexts that may appear in the document.
	///
	/// This will most likely be [`JsonValue`].
	type LocalContext: context::Local<T>;

	/// Document location, if any.
	fn base_url(&self) -> Option<Iri>;

	/// Expand the document with a custom base URL, initial context, document loader and
	/// expansion options.
	///
	/// If you do not wish to set the base URL and expansion options yourself, the
	/// [`expand`](`Document::expand`) method is more appropriate.
	///
	/// This is an asynchronous method since expanding the context may require loading remote
	/// ressources. It returns a boxed [`Future`](`std::future::Future`) to the result.
	fn expand_with<'a, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(&'a self, base_url: Option<Iri>, context: &'a C, loader: &'a mut L, options: expansion::Options) -> BoxFuture<'a, Result<ExpandedDocument<T>, Error>> where
		C::LocalContext: Send + Sync + From<L::Output> + From<Self::LocalContext>,
		L::Output: Into<Self::LocalContext>,
		T: 'a + Send + Sync;

	/// Expand the document.
	///
	/// Uses the given initial context and the given document loader.
	/// The default implementation is equivalent to [`expand_with`](`Document::expand_with`), but
	/// uses the document [`base_url`](`Document::base_url`), with the default
	/// options.
	///
	/// This is an asynchronous method since expanding the context may require loading remote
	/// ressources. It returns a boxed [`Future`](`std::future::Future`) to the result.
	///
	/// # Example
	/// ```
	/// use std_async::task;
	/// use json_ld::{Document, JsonContext, NoLoader};
	///
	/// // Prepare the initial context.
	/// let context = JsonContext::new();
	///
	/// let doc = json::parse("{
	/// 	\"@context\": {
	/// 		\"name\": \"http://xmlns.com/foaf/0.1/name\",
	/// 		\"knows\": \"http://xmlns.com/foaf/0.1/knows\"
	/// 	},
	/// 	\"@id\": \"http://timothee.haudebourg.net/\",
	/// 	\"name\": \"Timothée Haudebourg\",
	/// 	\"knows\": [
	/// 		{
	/// 			\"name\": \"Amélie Barbe\"
	/// 		}
	/// 	]
	/// }")?;
	/// let expanded_doc = task::block_on(doc.expand(&context, &mut NoLoader))?;
	/// ```
	fn expand<'a, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(&'a self, context: &'a C, loader: &'a mut L) -> BoxFuture<'a, Result<ExpandedDocument<T>, Error>> where
		C::LocalContext: Send + Sync + From<L::Output> + From<Self::LocalContext>,
		L::Output: Into<Self::LocalContext>,
		T: 'a + Send + Sync
	{
		self.expand_with(self.base_url(), context, loader, expansion::Options::default())
	}

	fn compact_with<'a, C: Send + Sync + ContextMut<T>, R: Send + Sync + crate::util::AsJson + Deref<Target=C>, L: Send + Sync + Loader>(&'a self, base_url: Option<Iri<'a>>, context: &'a R, loader: &'a mut L, options: compaction::Options) -> BoxFuture<'a, Result<JsonValue, Error>> where
		C::LocalContext: Send + Sync + From<L::Output> + From<Self::LocalContext>,
		L::Output: Into<Self::LocalContext>,
		T: 'a + Send + Sync,
		Self: Sync
	{
		use compaction::Compact;
		async move {
			let json_context = context.as_json();
			let context = context.deref();
			let expanded = self.expand_with(base_url, context, loader, options.into()).await?;

			use crate::util::AsJson;
			println!("expanded: {}", expanded.as_json().pretty(2));

			let inverse_context = context.invert();
			let compacted = if expanded.len() == 1 {
				expanded.into_iter().next().unwrap().compact_with(context, context, &inverse_context, None, loader, options.into()).await?
			} else {
				expanded.compact_with(context, context, &inverse_context, None, loader, options.into()).await?
			};

			let mut map = match compacted {
				JsonValue::Array(items) => {
					let mut map = json::object::Object::new();
					if !items.is_empty() {
						use crate::{
							Lenient,
							syntax::{
								Term,
								Keyword
							}
						};
						let key = crate::compaction::compact_iri(context, &inverse_context, &Lenient::Ok(Term::Keyword(Keyword::Graph)), None, true, false, options.into())?;
						map.insert(key.as_str().unwrap(), JsonValue::Array(items));
					}

					map
				},
				JsonValue::Object(map) => map,
				_ => panic!("invalid compact document")
			};

			if !map.is_empty() && !json_context.is_null() {
				map.insert("@context", json_context)
			}

			Ok(JsonValue::Object(map))
		}.boxed()
	}

	fn compact<'a, C: Send + Sync + ContextMut<T>, R: Send + Sync + crate::util::AsJson + Deref<Target=C>, L: Send + Sync + Loader>(&'a self, context: &'a R, loader: &'a mut L) -> BoxFuture<'a, Result<JsonValue, Error>> where
		C::LocalContext: Send + Sync + From<L::Output> + From<Self::LocalContext>,
		L::Output: Into<Self::LocalContext>,
		T: 'a + Id + Send + Sync,
		Self: Sync
	{
		self.compact_with(self.base_url(), context, loader, compaction::Options::default())
	}
}

/// Default JSON document implementation.
impl<T: Id> Document<T> for JsonValue {
	type LocalContext = JsonValue;

	/// Returns `None`.
	///
	/// Use [`RemoteDocument`] to attach a base URL to a `JsonValue` document.
	fn base_url(&self) -> Option<Iri> {
		None
	}

	fn expand_with<'a, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(&'a self, base_url: Option<Iri>, context: &'a C, loader: &'a mut L, options: expansion::Options) -> BoxFuture<'a, Result<ExpandedDocument<T>, Error>> where
		C::LocalContext: Send + Sync + From<L::Output> + From<JsonValue>,
		L::Output: Into<JsonValue>,
		T: 'a + Send + Sync
	{
		expansion::expand(context, self, base_url, loader, options).boxed()
	}
}

/// Remote JSON-LD document.
///
/// Represent a document located at a given base URL.
/// This is the result of loading a document with [`Loader::load`](`crate::Loader::load`).
/// It is a simple wrapper that [`Deref`] to the underlying document while remembering its
/// base URL.
///
/// # Example
/// ```
/// #[macro_use]
/// extern crate static_iref;
///
/// use std_async::task;
/// use json_ld::FsLoader;
///
/// # Prepare the loader.
/// let mut loader = FsLoader::new();
/// loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");
///
/// # Load the remote document.
/// let url = iri!("https://w3c.github.io/json-ld-api/tests/expand-manifest.jsonld");
/// let doc: RemoteDocument<JsonValue> = task::block_on(loader.load(url)).unwrap();
/// ```
#[derive(Clone)]
pub struct RemoteDocument<D = JsonValue> {
	/// The base URL of the document.
	base_url: IriBuf,

	/// The document contents.
	doc: D,
}

impl<D> RemoteDocument<D> {
	/// Create a new remote document from the document contents and base URL.
	pub fn new(doc: D, base_url: Iri) -> RemoteDocument<D> {
		RemoteDocument {
			base_url: base_url.into(),
			doc: doc
		}
	}

	/// Consume the remote document and return the inner document.
	pub fn into_document(self) -> D {
		self.doc
	}

	/// Consume the remote document and return the inner document along with its base URL.
	pub fn into_parts(self) -> (D, IriBuf) {
		(self.doc, self.base_url)
	}
}

/// A Remote document is a document.
impl<T: Id, D: Document<T>> Document<T> for RemoteDocument<D> {
	type LocalContext = D::LocalContext;

	fn base_url(&self) -> Option<Iri> {
		Some(self.base_url.as_iri())
	}

	fn expand_with<'a, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(&'a self, base_url: Option<Iri>, context: &'a C, loader: &'a mut L, options: expansion::Options) -> BoxFuture<'a, Result<ExpandedDocument<T>, Error>> where
		C::LocalContext: Send + Sync + From<L::Output> + From<Self::LocalContext>,
		L::Output: Into<Self::LocalContext>,
		T: 'a + Send + Sync
	{
		self.doc.expand_with(base_url, context, loader, options)
	}
}

impl<D> Deref for RemoteDocument<D> {
	type Target = D;

	fn deref(&self) -> &D {
		&self.doc
	}
}

impl<D> DerefMut for RemoteDocument<D> {
	fn deref_mut(&mut self) -> &mut D {
		&mut self.doc
	}
}
