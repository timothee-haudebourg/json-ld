use crate::{
	compaction,
	context::{self, Loader},
	expansion, flattening, id, loader,
	util::JsonFrom,
	Context, ContextMut, Error, ErrorCode, Id, Loc,
};
use cc_traits::Len;
use derivative::Derivative;
use futures::future::{BoxFuture, FutureExt};
use generic_json::{Json, JsonMut, JsonNew};
use iref::{Iri, IriBuf};
use std::ops::{Deref, DerefMut};

pub mod expanded;
pub mod flattened;

pub use expanded::ExpandedDocument;
pub use flattened::FlattenedDocument;

/// Document expansion error.
pub type ExpansionError<J> = Loc<Error, <J as Json>::MetaData>;

pub type ExpansionResult<T, J> = Result<ExpandedDocument<J, T>, ExpansionError<J>>;

/// Document flattening error.
#[derive(Derivative)]
#[derivative(Debug(bound = "J::MetaData: std::fmt::Debug"))]
pub enum FlatteningError<T: Id, J: Json> {
	Expansion(ExpansionError<J>),
	ConflictingIndexes(flattening::ConflictingIndexes<T>),
}

impl<T: Id, J: Json> FlatteningError<T, J> {
	pub fn code(&self) -> ErrorCode {
		match self {
			Self::Expansion(e) => e.code(),
			Self::ConflictingIndexes(_) => ErrorCode::ConflictingIndexes,
		}
	}
}

impl<T: Id, J: Json> From<ExpansionError<J>> for FlatteningError<T, J> {
	fn from(e: ExpansionError<J>) -> Self {
		Self::Expansion(e)
	}
}

impl<T: Id, J: Json> From<flattening::ConflictingIndexes<T>> for FlatteningError<T, J> {
	fn from(e: flattening::ConflictingIndexes<T>) -> Self {
		Self::ConflictingIndexes(e)
	}
}

pub type FlatteningResult<T, J> = Result<FlattenedDocument<J, T>, FlatteningError<T, J>>;

/// JSON-LD document.
///
/// This trait represent a JSON-LD document that can be expanded into an [`ExpandedDocument`]
/// or compacted. It is the main entry point to the JSON-LD API.
/// It is notably implemented for any type implementing the [generic_json::Json] trait.
pub trait Document<T: Id> {
	type Json: Json;

	/// Document location, if any.
	fn base_url(&self) -> Option<Iri>;

	fn compare_with<'a, C: 'a + ContextMut<T>, L: 'a + Loader>(
		&'a self,
		other: &'a Self,
		base_url: Option<Iri<'a>>,
		context: &'a C,
		loader: &'a mut L,
		options: expansion::Options,
	) -> BoxFuture<'a, Result<bool, ExpansionError<Self::Json>>>
	where
		Self: Sync,
		Self::Json: expansion::JsonExpand,
		T: 'a + Send + Sync,
		C: Send + Sync,
		C::LocalContext: From<L::Output> + From<Self::Json>,
		L: Send + Sync,
		L::Output: Into<Self::Json>
	{
		async move {
			let expanded_self = self.expand_with(base_url, context, loader, options).await?;
			let expanded_other = other.expand_with(base_url, context, loader, options).await?;

			Ok(expanded_self == expanded_other)
		}.boxed()
	}

	/// Expand the document with a custom base URL, initial context, document loader and
	/// expansion options.
	///
	/// If you do not wish to set the base URL and expansion options yourself, the
	/// [`expand`](`Document::expand`) method is more appropriate.
	///
	/// This is an asynchronous method since expanding the context may require loading remote
	/// ressources. It returns a boxed [`Future`](`std::future::Future`) to the result.
	fn expand_with<'a, C: 'a + ContextMut<T>, L: 'a + Loader>(
		&'a self,
		base_url: Option<Iri>,
		context: &'a C,
		loader: &'a mut L,
		options: expansion::Options,
	) -> BoxFuture<'a, ExpansionResult<T, Self::Json>>
	where
		Self::Json: expansion::JsonExpand,
		T: 'a + Send + Sync,
		C: Send + Sync,
		C::LocalContext: From<L::Output> + From<Self::Json>,
		L: Send + Sync,
		L::Output: Into<Self::Json>; // TODO get rid of this bound?

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
	/// # fn main() -> Result<(), json_ld::Loc<json_ld::Error, ()>> {
	/// use async_std::task;
	/// use json_ld::{Document, context, NoLoader};
	/// use ijson::IValue;
	///
	/// let doc: IValue = serde_json::from_str("{
	///   \"@context\": {
	///     \"name\": \"http://xmlns.com/foaf/0.1/name\",
	///     \"knows\": \"http://xmlns.com/foaf/0.1/knows\"
	///   },
	///   \"@id\": \"http://timothee.haudebourg.net/\",
	///   \"name\": \"Timothée Haudebourg\",
	///   \"knows\": [
	///     {
	///       \"name\": \"Amélie Barbe\"
	///     }
	///   ]
	/// }").unwrap();
	/// let mut loader = NoLoader::<IValue>::new();
	/// let expanded_doc = task::block_on(doc.expand::<context::Json<IValue>, _>(&mut loader))?;
	/// # Ok(())
	/// # }
	/// ```
	#[inline(always)]
	fn expand<'a, C: 'a + ContextMut<T>, L: Loader>(
		&'a self,
		loader: &'a mut L,
	) -> BoxFuture<'a, ExpansionResult<T, Self::Json>>
	where
		Self: Send + Sync,
		Self::Json: expansion::JsonExpand,
		C: Send + Sync,
		C::LocalContext: From<L::Output> + From<Self::Json>,
		L: Send + Sync,
		L::Output: Into<Self::Json>,
		T: 'a + Send + Sync,
	{
		async move {
			let context = C::new(self.base_url());
			self.expand_with(
				self.base_url(),
				&context,
				loader,
				expansion::Options::default(),
			)
			.await
		}
		.boxed()
	}

	/// Compact the document with a custom base URL, context, document loader and options.
	///
	/// The `meta_context` parameter is a function to convert the metadata
	/// associated to the input context (JSON representation) to `K::MetaData`.
	/// The `meta_document` parameter is another conversion function for the
	/// metadata attached to the document.
	fn compact_with<'a, K: JsonFrom<Self::Json>, E: ContextMut<T>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		base_url: Option<Iri<'a>>,
		expansion_context: &'a E,
		compaction_context: &'a context::ProcessedOwned<K, context::Inversible<T, C>>,
		loader: &'a mut L,
		options: compaction::Options,
		meta: M,
	) -> BoxFuture<'a, Result<K, Error>>
	where
		Self: Sync,
		Self::Json: expansion::JsonExpand + compaction::JsonSrc,
		T: 'a + Send + Sync,
		K: Clone + JsonFrom<C::LocalContext>,
		E: Send + Sync,
		E::LocalContext: From<L::Output> + From<Self::Json>,
		C: Send + Sync,
		C::LocalContext:
			compaction::JsonSrc + From<L::Output>,
		L: 'a + Send + Sync,
		M: 'a + Clone + Send + Sync + Fn(Option<&<Self::Json as Json>::MetaData>) -> K::MetaData,
		L::Output: Into<Self::Json>,
	{
		async move {
			// let json_compaction_context: K = compaction_context.as_json_with(meta_context);
			// let compaction_context = context::Inversible::new(compaction_context.deref());

			let expanded: ExpandedDocument<Self::Json, T> = self
				.expand_with(base_url, expansion_context, loader, options.into())
				.await
				.map_err(Loc::unwrap)?;

			expanded.compact(
				compaction_context,
				loader,
				options,
				meta,
			)
			.await
		}
		.boxed()
	}

	/// Compact the document.
	#[inline(always)]
	fn compact<'a, C: ContextMut<T>, L: Loader>(
		&'a self,
		context: &'a context::ProcessedOwned<Self::Json, context::Inversible<T, C>>,
		loader: &'a mut L,
	) -> BoxFuture<'a, Result<Self::Json, Error>>
	where
		Self: Sync,
		Self::Json: Clone + JsonFrom<C::LocalContext> + expansion::JsonExpand + compaction::JsonSrc,
		<Self::Json as Json>::MetaData: Default,
		T: 'a + Send + Sync,
		C: Send + Sync,
		C::LocalContext:
			compaction::JsonSrc + From<L::Output> + From<Self::Json>,
		L: 'a + Send + Sync,
		L::Output: Into<Self::Json>,
	{
		self.compact_with(
			self.base_url(),
			context.as_ref().as_ref(),
			context,
			loader,
			compaction::Options::default(),
			|m| m.cloned().unwrap_or_default()
		)
	}

	fn flatten_with<'a, G: 'a + id::Generator<T>, C: 'a + ContextMut<T>, L: 'a + Loader>(
		&'a self,
		generator: G,
		base_url: Option<Iri>,
		context: &'a C,
		loader: &'a mut L,
		options: expansion::Options,
	) -> BoxFuture<'a, FlatteningResult<T, Self::Json>>
	where
		Self::Json: expansion::JsonExpand,
		G: Send,
		T: 'a + Send + Sync,
		C: Send + Sync,
		C::LocalContext: From<L::Output> + From<Self::Json>,
		L: Send + Sync,
		L::Output: Into<Self::Json>, // TODO get rid of this bound?
	{
		let expand = self.expand_with(base_url, context, loader, options.unordered());
		async move {
			let expanded = expand.await?;
			Ok(expanded.flatten(generator, options.ordered)?)
		}
		.boxed()
	}

	fn flatten<'a, G: 'a + id::Generator<T>, C: 'a + ContextMut<T>, L: 'a + Loader>(
		&'a self,
		generator: G,
		context: &'a C,
		loader: &'a mut L,
	) -> BoxFuture<'a, FlatteningResult<T, Self::Json>>
	where
		Self::Json: expansion::JsonExpand,
		G: Send,
		T: 'a + Send + Sync,
		C: Send + Sync,
		C::LocalContext: From<L::Output> + From<Self::Json>,
		L: Send + Sync,
		L::Output: Into<Self::Json>, // TODO get rid of this bound?
	{
		self.flatten_with(
			generator,
			self.base_url(),
			context,
			loader,
			expansion::Options::default(),
		)
	}

	fn embed_context<C: Context<T>, M>(
		&mut self,
		context: &context::ProcessedOwned<Self::Json, context::Inversible<T, C>>,
		options: compaction::Options,
		meta: M
	) -> Result<(), Error> where
		Self::Json: Clone + JsonMut + JsonNew,
		<Self::Json as Json>::Object: Default,
		M: Fn() -> <Self::Json as Json>::MetaData;
}

/// Default JSON document implementation.
impl<J: Json, T: Id> Document<T> for J {
	type Json = Self;

	/// Returns `None`.
	///
	/// Use [`RemoteDocument`] to attach a base URL to a `JsonValue` document.
	#[inline(always)]
	fn base_url(&self) -> Option<Iri> {
		None
	}

	#[inline(always)]
	fn expand_with<'a, C: ContextMut<T>, L: Loader>(
		&'a self,
		base_url: Option<Iri>,
		context: &'a C,
		loader: &'a mut L,
		options: expansion::Options,
	) -> BoxFuture<'a, ExpansionResult<T, Self>>
	where
		Self: expansion::JsonExpand,
		C: Send + Sync,
		C::LocalContext: From<L::Output> + From<Self>,
		L: Send + Sync,
		L::Output: Into<Self>,
		T: 'a + Send + Sync,
	{
		let base_url = base_url.map(IriBuf::from);

		async move {
			let mut warnings = Vec::new();
			let objects =
				expansion::expand(context, self, base_url, loader, options, &mut warnings).await?;
			Ok(ExpandedDocument::new(objects, warnings))
		}
		.boxed()
	}

	fn embed_context<C: Context<T>, M>(
		&mut self,
		context: &context::ProcessedOwned<Self::Json, context::Inversible<T, C>>,
		options: compaction::Options,
		meta: M,
	) -> Result<(), Error> where
		Self::Json: Clone + JsonMut + JsonNew,
		<Self::Json as Json>::Object: Default,
		M: Fn() -> <Self::Json as Json>::MetaData
	{
		if !self.is_object() {
			let mut value = Self::empty_object(meta());
			std::mem::swap(self, &mut value);

			if let (generic_json::Value::Array(items), metadata) = value.into_parts() {
				if !items.is_empty() {
					use crate::syntax::{Keyword, Term};
					let key = crate::compaction::compact_iri::<generic_json::Null, _, _>(
						context,
						&Term::Keyword(Keyword::Graph),
						true,
						false,
						options,
					);

					let mut value = Self::array(items, metadata);

					match key {
						Ok(key) => {
							self.as_object_mut().unwrap().insert(
								Self::new_key(&key.unwrap(), meta()),
								value,
							);
						},
						Err(e) => {
							std::mem::swap(self, &mut value);
							return Err(e)
						}
					}
				}
			}
		}

		let map = self.as_object_mut().unwrap();
		let json_context = context.json().clone();

		if !map.is_empty()
			&& !json_context.is_null()
			&& !json_context.is_empty_array_or_object()
		{
			map.insert(Self::new_key("@context", meta()), json_context);
		}

		Ok(())
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
/// use static_iref::*;
///
/// use async_std::task;
/// use ijson::IValue;
/// use json_ld::{
///   Loader,
///   FsLoader,
///   RemoteDocument
/// };
///
/// // Prepare the loader.
/// let mut loader = FsLoader::<IValue>::new(|s| serde_json::from_str(s));
/// loader.mount(iri!("https://w3c.github.io/json-ld-api"), "json-ld-api");
///
/// // Load the remote document.
/// let url = iri!("https://w3c.github.io/json-ld-api/tests/expand-manifest.jsonld");
/// let doc: RemoteDocument<IValue> = task::block_on(loader.load(url)).unwrap();
/// ```
#[derive(Clone)]
pub struct RemoteDocument<D> {
	/// Base URL of the document.
	base_url: IriBuf,

	/// Document id.
	source: loader::Id,

	/// Document contents.
	doc: D,
}

impl<D> RemoteDocument<D> {
	/// Create a new remote document from the document contents and base URL.
	#[inline(always)]
	pub fn new(doc: D, base_url: IriBuf, source: loader::Id) -> RemoteDocument<D> {
		RemoteDocument {
			base_url,
			source,
			doc,
		}
	}

	pub fn source(&self) -> loader::Id {
		self.source
	}

	/// Consume the remote document and return the inner document.
	#[inline(always)]
	pub fn into_document(self) -> D {
		self.doc
	}

	/// Consume the remote document and return the inner document along with its base URL.
	#[inline(always)]
	pub fn into_parts(self) -> (D, loader::Id, IriBuf) {
		(self.doc, self.source, self.base_url)
	}
}

/// A Remote document is a document.
impl<T: Id, D: Document<T>> Document<T> for RemoteDocument<D> {
	type Json = D::Json;

	#[inline(always)]
	fn base_url(&self) -> Option<Iri> {
		Some(self.base_url.as_iri())
	}

	#[inline(always)]
	fn expand_with<'a, C: 'a + ContextMut<T> + Send + Sync, L: 'a + Loader + Send + Sync>(
		&'a self,
		base_url: Option<Iri>,
		context: &'a C,
		loader: &'a mut L,
		options: expansion::Options,
	) -> BoxFuture<'a, ExpansionResult<T, Self::Json>>
	where
		D::Json: expansion::JsonExpand,
		C::LocalContext: From<L::Output> + From<Self::Json>,
		L::Output: Into<Self::Json>,
		T: 'a + Send + Sync,
	{
		self.doc.expand_with(base_url, context, loader, options)
	}

	fn embed_context<C: Context<T>, M>(
		&mut self,
		context: &context::ProcessedOwned<Self::Json, context::Inversible<T, C>>,
		options: compaction::Options,
		meta: M,
	) -> Result<(), Error> where
		Self::Json: Clone + JsonMut + JsonNew,
		<Self::Json as Json>::Object: Default,
		M: Fn() -> <Self::Json as Json>::MetaData
	{
		self.doc.embed_context(context, options, meta)
	}
}

impl<D> AsRef<D> for RemoteDocument<D> {
	fn as_ref(&self) -> &D {
		&self.doc
	}
}

impl<D> AsMut<D> for RemoteDocument<D> {
	fn as_mut(&mut self) -> &mut D {
		&mut self.doc
	}
}

impl<D> Deref for RemoteDocument<D> {
	type Target = D;

	#[inline(always)]
	fn deref(&self) -> &D {
		&self.doc
	}
}

impl<D> DerefMut for RemoteDocument<D> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut D {
		&mut self.doc
	}
}
