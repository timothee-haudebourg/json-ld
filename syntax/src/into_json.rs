use crate::{
	context, Container, ContainerKind, Context, Direction, Entry, Keyword, LenientLanguageTagBuf,
	Nullable,
};
use contextual::Contextual;
use locspan::Meta;

pub trait IntoJsonWithContextMeta<M, N>: Sized {
	fn into_json_meta_with(self, meta: M, context: &N) -> Meta<json_syntax::Value<M>, M>;
}

impl<T: IntoJsonWithContext<M, N>, M, N> IntoJsonWithContextMeta<M, N> for Vec<T> {
	fn into_json_meta_with(self, meta: M, context: &N) -> Meta<json_syntax::Value<M>, M> {
		Meta(
			json_syntax::Value::Array(
				self.into_iter()
					.map(|item| item.into_json_with(context))
					.collect(),
			),
			meta,
		)
	}
}

impl<T: IntoJsonWithContext<M, N>, M, N> IntoJsonWithContextMeta<M, N>
	for std::collections::HashSet<T>
{
	fn into_json_meta_with(self, meta: M, context: &N) -> Meta<json_syntax::Value<M>, M> {
		Meta(
			json_syntax::Value::Array(
				self.into_iter()
					.map(|item| item.into_json_with(context))
					.collect(),
			),
			meta,
		)
	}
}

pub trait IntoJsonWithContext<M, N>: Sized {
	fn into_json_with(self, context: &N) -> Meta<json_syntax::Value<M>, M>;
}

impl<T: IntoJsonWithContextMeta<M, N>, M, N> IntoJsonWithContext<M, N> for Meta<T, M> {
	fn into_json_with(self, context: &N) -> Meta<json_syntax::Value<M>, M> {
		T::into_json_meta_with(self.0, self.1, context)
	}
}

impl<T: IntoJsonWithContext<M, N>, M, N> IntoJsonWithContext<M, N> for locspan::Stripped<T> {
	fn into_json_with(self, context: &N) -> Meta<json_syntax::Value<M>, M> {
		T::into_json_with(self.0, context)
	}
}

pub trait IntoJsonMeta<M>: Sized {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M>;
}

pub trait IntoJson<M>: Sized {
	fn into_json(self) -> Meta<json_syntax::Value<M>, M>;
}

impl<T: IntoJsonMeta<M>, M> IntoJson<M> for Meta<T, M> {
	fn into_json(self) -> Meta<json_syntax::Value<M>, M> {
		T::into_json_meta(self.0, self.1)
	}
}

impl<'n, T: IntoJsonWithContext<M, N>, M, N> IntoJson<M> for Contextual<T, &'n N> {
	fn into_json(self) -> Meta<json_syntax::Value<M>, M> {
		T::into_json_with(self.0, self.1)
	}
}

impl<'n, T: IntoJsonWithContext<M, N>, M, N> IntoJson<M> for Contextual<T, &'n mut N> {
	fn into_json(self) -> Meta<json_syntax::Value<M>, M> {
		T::into_json_with(self.0, self.1)
	}
}

impl<M> IntoJsonMeta<M> for bool {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::Boolean(self), meta)
	}
}

impl<T: IntoJsonMeta<M>, M> IntoJsonMeta<M> for Box<T> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		T::into_json_meta(*self, meta)
	}
}

impl<M> IntoJsonMeta<M> for iref::IriRefBuf {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.as_str().into()), meta)
	}
}

impl<T: IntoJsonMeta<M>, M> IntoJsonMeta<M> for Nullable<T> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		match self {
			Self::Null => Meta(json_syntax::Value::Null, meta),
			Self::Some(other) => T::into_json_meta(other, meta),
		}
	}
}

impl<M> IntoJsonMeta<M> for Keyword {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_str().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for String {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into()), meta)
	}
}

impl<T, M> Entry<T, M> {
	pub fn insert_in_json_object(
		self,
		object: &mut json_syntax::Object<M>,
		key: json_syntax::object::Key,
	) -> Option<json_syntax::object::RemovedByInsertion<M>>
	where
		T: IntoJsonMeta<M>,
	{
		object.insert(Meta(key, self.key_metadata), self.value.into_json())
	}
}

impl<M> IntoJsonMeta<M> for LenientLanguageTagBuf {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_string().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for Direction {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.as_str().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::definition::TypeContainer {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_str().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::definition::Type<M> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		let mut object = json_syntax::Object::new();

		object.insert(
			Meta("@container".into(), self.container.key_metadata),
			self.container.value.into_json(),
		);

		if let Some(protected) = self.protected {
			object.insert(
				Meta("@protected".into(), protected.key_metadata),
				protected.value.into_json(),
			);
		}

		Meta(json_syntax::Value::Object(object), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::definition::Version {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::Number(self.into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::definition::Vocab {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_string().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::definition::Key {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_string().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::term_definition::Index {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_string().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::term_definition::Nest {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_string().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for Container<M> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		match self {
			Self::One(c) => ContainerKind::into_json_meta(c, meta),
			Self::Many(list) => Meta(
				json_syntax::Value::Array(list.into_iter().map(IntoJson::into_json).collect()),
				meta,
			),
		}
	}
}

impl<M> IntoJsonMeta<M> for ContainerKind {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.as_str().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::term_definition::Id {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		match self {
			Self::Keyword(k) => Keyword::into_json_meta(k, meta),
			Self::Term(t) => String::into_json_meta(t, meta),
		}
	}
}

impl<M> IntoJsonMeta<M> for context::Value<M> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		match self {
			Self::One(c) => c.into_json(),
			Self::Many(list) => Meta(
				json_syntax::Value::Array(list.into_iter().map(IntoJson::into_json).collect()),
				meta,
			),
		}
	}
}

impl<M> IntoJsonMeta<M> for Context<M> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		match self {
			Self::Null => Meta(json_syntax::Value::Null, meta),
			Self::IriRef(iri) => iref::IriRefBuf::into_json_meta(iri, meta),
			Self::Definition(def) => context::Definition::into_json_meta(def, meta),
		}
	}
}

impl<M> IntoJsonMeta<M> for context::Definition<M> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		let mut object = json_syntax::Object::new();

		if let Some(base) = self.base {
			base.insert_in_json_object(&mut object, "@base".into());
		}

		if let Some(import) = self.import {
			import.insert_in_json_object(&mut object, "@import".into());
		}

		if let Some(language) = self.language {
			language.insert_in_json_object(&mut object, "@language".into());
		}

		if let Some(direction) = self.direction {
			direction.insert_in_json_object(&mut object, "@direction".into());
		}

		if let Some(propagate) = self.propagate {
			propagate.insert_in_json_object(&mut object, "@propagate".into());
		}

		if let Some(protected) = self.protected {
			protected.insert_in_json_object(&mut object, "@protected".into());
		}

		if let Some(type_) = self.type_ {
			type_.insert_in_json_object(&mut object, "@type".into());
		}

		if let Some(version) = self.version {
			version.insert_in_json_object(&mut object, "@version".into());
		}

		if let Some(vocab) = self.vocab {
			vocab.insert_in_json_object(&mut object, "@vocab".into());
		}

		for (key, binding) in self.bindings {
			object.insert(
				Meta(key.into_string().into(), binding.key_metadata),
				binding.definition.into_json(),
			);
		}

		Meta(json_syntax::Value::Object(object), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::TermDefinition<M> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		match self {
			Self::Simple(s) => context::term_definition::Simple::into_json_meta(s, meta),
			Self::Expanded(e) => context::term_definition::Expanded::into_json_meta(*e, meta),
		}
	}
}

impl<M> IntoJsonMeta<M> for context::term_definition::Simple {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_string().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::term_definition::Type {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_string().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::term_definition::TypeKeyword {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(self.into_str().into()), meta)
	}
}

impl<M> IntoJsonMeta<M> for context::term_definition::Expanded<M> {
	fn into_json_meta(self, meta: M) -> Meta<json_syntax::Value<M>, M> {
		let mut object = json_syntax::Object::new();

		if let Some(id) = self.id {
			id.insert_in_json_object(&mut object, "@id".into());
		}

		if let Some(type_) = self.type_ {
			type_.insert_in_json_object(&mut object, "@type".into());
		}

		if let Some(context) = self.context {
			context.insert_in_json_object(&mut object, "@context".into());
		}

		if let Some(reverse) = self.reverse {
			reverse.insert_in_json_object(&mut object, "@reverse".into());
		}

		if let Some(index) = self.index {
			index.insert_in_json_object(&mut object, "@index".into());
		}

		if let Some(language) = self.language {
			language.insert_in_json_object(&mut object, "@language".into());
		}

		if let Some(direction) = self.direction {
			direction.insert_in_json_object(&mut object, "@direction".into());
		}

		if let Some(container) = self.container {
			container.insert_in_json_object(&mut object, "@container".into());
		}

		if let Some(nest) = self.nest {
			nest.insert_in_json_object(&mut object, "@nest".into());
		}

		if let Some(prefix) = self.prefix {
			prefix.insert_in_json_object(&mut object, "@prefix".into());
		}

		if let Some(propagate) = self.propagate {
			propagate.insert_in_json_object(&mut object, "@propagate".into());
		}

		if let Some(protected) = self.protected {
			protected.insert_in_json_object(&mut object, "@protected".into());
		}

		Meta(json_syntax::Value::Object(object), meta)
	}
}
