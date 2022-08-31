use crate::{
	context, Container, ContainerKind, Context, Direction, Entry, Keyword,
	LenientLanguageTagBuf, Nullable
};
use locspan::Meta;

pub trait IntoJson<M>: Sized {
	fn into_json(this: Meta<Self, M>) -> Meta<json_syntax::Value<M>, M>;
}

impl<M> IntoJson<M> for bool {
	fn into_json(Meta(b, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::Boolean(b), meta)
	}
}

impl<T: IntoJson<M>, M> IntoJson<M> for Box<T> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		T::into_json(Meta(*value, meta))
	}
}

impl<M> IntoJson<M> for iref::IriRefBuf {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.as_str().into()), meta)
	}
}

impl<T: IntoJson<M>, M> IntoJson<M> for Nullable<T> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		match value {
			Self::Null => Meta(json_syntax::Value::Null, meta),
			Self::Some(other) => T::into_json(Meta(other, meta)),
		}
	}
}

impl<M> IntoJson<M> for Keyword {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_str().into()), meta)
	}
}

impl<M> IntoJson<M> for String {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into()), meta)
	}
}

// impl<M> IntoJson<M> for Value<M> {
// 	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
// 		let json = match value {
// 			Self::Null => json_syntax::Value::Null,
// 			Self::Boolean(b) => json_syntax::Value::Boolean(b),
// 			Self::Number(n) => json_syntax::Value::Number(n),
// 			Self::String(s) => json_syntax::Value::String(s),
// 			Self::Array(a) => {
// 				json_syntax::Value::Array(a.into_iter().map(Self::into_json).collect())
// 			}
// 			Self::Object(o) => json_syntax::Value::Object(o.into_json_object()),
// 		};

// 		Meta(json, meta)
// 	}
// }

// impl<M> Object<M> {
// 	pub fn into_json_object(self) -> json_syntax::Object<M> {
// 		let mut result = Vec::new();

// 		result.extend(self.into_iter().map(object::Entry::into_json));

// 		json_syntax::Object::from_vec(result)
// 	}
// }

// impl<M> IntoJson<M> for Object<M> {
// 	fn into_json(Meta(object, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
// 		Meta(json_syntax::Value::Object(object.into_json_object()), meta)
// 	}
// }

// impl<M> object::Entry<M> {
// 	pub fn into_json(self) -> json_syntax::object::Entry<M> {
// 		json_syntax::object::Entry {
// 			key: self.key,
// 			value: Value::into_json(self.value),
// 		}
// 	}
// }

impl<T, M> Entry<T, M> {
	pub fn insert_in_json_object(
		self,
		object: &mut json_syntax::Object<M>,
		key: json_syntax::object::Key,
	) -> Option<json_syntax::object::RemovedByInsertion<M>>
	where
		T: IntoJson<M>,
	{
		object.insert(Meta(key, self.key_metadata), T::into_json(self.value))
	}
}

impl<M> IntoJson<M> for LenientLanguageTagBuf {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_string().into()), meta)
	}
}

impl<M> IntoJson<M> for Direction {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.as_str().into()), meta)
	}
}

impl<M> IntoJson<M> for context::definition::TypeContainer {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_str().into()), meta)
	}
}

impl<M> IntoJson<M> for context::definition::Type<M> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		let mut object = json_syntax::Object::new();

		object.insert(
			Meta("@container".into(), value.container.key_metadata),
			context::definition::TypeContainer::into_json(value.container.value),
		);

		if let Some(protected) = value.protected {
			object.insert(
				Meta("@protected".into(), protected.key_metadata),
				bool::into_json(protected.value),
			);
		}

		Meta(json_syntax::Value::Object(object), meta)
	}
}

impl<M> IntoJson<M> for context::definition::Version {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::Number(value.into()), meta)
	}
}

impl<M> IntoJson<M> for context::definition::Vocab {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_string().into()), meta)
	}
}

impl<M> IntoJson<M> for context::definition::Key {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_string().into()), meta)
	}
}

impl<M> IntoJson<M> for context::term_definition::Index {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_string().into()), meta)
	}
}

impl<M> IntoJson<M> for context::term_definition::Nest {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_string().into()), meta)
	}
}

impl<M> IntoJson<M> for Container<M> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		match value {
			Self::One(c) => ContainerKind::into_json(Meta(c, meta)),
			Self::Many(list) => Meta(
				json_syntax::Value::Array(list.into_iter().map(ContainerKind::into_json).collect()),
				meta,
			),
		}
	}
}

impl<M> IntoJson<M> for ContainerKind {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.as_str().into()), meta)
	}
}

impl<M> IntoJson<M> for context::term_definition::Id {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		match value {
			Self::Keyword(k) => Keyword::into_json(Meta(k, meta)),
			Self::Term(t) => String::into_json(Meta(t, meta)),
		}
	}
}

impl<M> IntoJson<M> for context::Value<M> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		match value {
			Self::One(c) => Context::into_json(c),
			Self::Many(list) => Meta(
				json_syntax::Value::Array(list.into_iter().map(Context::into_json).collect()),
				meta,
			),
		}
	}
}

impl<M> IntoJson<M> for Context<context::Definition<M>> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		match value {
			Self::Null => Meta(json_syntax::Value::Null, meta),
			Self::IriRef(iri) => iref::IriRefBuf::into_json(Meta(iri, meta)),
			Self::Definition(def) => context::Definition::into_json(Meta(def, meta)),
		}
	}
}

impl<M> IntoJson<M> for context::Definition<M> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		let mut object = json_syntax::Object::new();

		if let Some(base) = value.base {
			base.insert_in_json_object(&mut object, "@base".into());
		}

		if let Some(import) = value.import {
			import.insert_in_json_object(&mut object, "@import".into());
		}

		if let Some(language) = value.language {
			language.insert_in_json_object(&mut object, "@language".into());
		}

		if let Some(direction) = value.direction {
			direction.insert_in_json_object(&mut object, "@direction".into());
		}

		if let Some(propagate) = value.propagate {
			propagate.insert_in_json_object(&mut object, "@propagate".into());
		}

		if let Some(protected) = value.protected {
			protected.insert_in_json_object(&mut object, "@protected".into());
		}

		if let Some(type_) = value.type_ {
			type_.insert_in_json_object(&mut object, "@type".into());
		}

		if let Some(version) = value.version {
			version.insert_in_json_object(&mut object, "@version".into());
		}

		if let Some(vocab) = value.vocab {
			vocab.insert_in_json_object(&mut object, "@vocab".into());
		}

		for (key, binding) in value.bindings {
			object.insert(
				Meta(key.into_string().into(), binding.key_metadata),
				Nullable::into_json(binding.definition),
			);
		}

		Meta(json_syntax::Value::Object(object), meta)
	}
}

impl<M> IntoJson<M> for context::TermDefinition<M> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		match value {
			Self::Simple(s) => context::term_definition::Simple::into_json(Meta(s, meta)),
			Self::Expanded(e) => context::term_definition::Expanded::into_json(Meta(e, meta)),
		}
	}
}

impl<M> IntoJson<M> for context::term_definition::Simple {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_string().into()), meta)
	}
}

impl<M> IntoJson<M> for context::term_definition::Type {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_string().into()), meta)
	}
}

impl<M> IntoJson<M> for context::term_definition::TypeKeyword {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		Meta(json_syntax::Value::String(value.into_str().into()), meta)
	}
}

impl<M> IntoJson<M> for context::term_definition::Expanded<M> {
	fn into_json(Meta(value, meta): Meta<Self, M>) -> Meta<json_syntax::Value<M>, M> {
		let mut object = json_syntax::Object::new();

		if let Some(id) = value.id {
			id.insert_in_json_object(&mut object, "@id".into());
		}

		if let Some(type_) = value.type_ {
			type_.insert_in_json_object(&mut object, "@type".into());
		}

		if let Some(context) = value.context {
			context.insert_in_json_object(&mut object, "@context".into());
		}

		if let Some(reverse) = value.reverse {
			reverse.insert_in_json_object(&mut object, "@reverse".into());
		}

		if let Some(index) = value.index {
			index.insert_in_json_object(&mut object, "@index".into());
		}

		if let Some(language) = value.language {
			language.insert_in_json_object(&mut object, "@language".into());
		}

		if let Some(direction) = value.direction {
			direction.insert_in_json_object(&mut object, "@direction".into());
		}

		if let Some(container) = value.container {
			container.insert_in_json_object(&mut object, "@container".into());
		}

		if let Some(nest) = value.nest {
			nest.insert_in_json_object(&mut object, "@nest".into());
		}

		if let Some(prefix) = value.prefix {
			prefix.insert_in_json_object(&mut object, "@prefix".into());
		}

		if let Some(propagate) = value.propagate {
			propagate.insert_in_json_object(&mut object, "@propagate".into());
		}

		if let Some(protected) = value.protected {
			protected.insert_in_json_object(&mut object, "@protected".into());
		}

		Meta(json_syntax::Value::Object(object), meta)
	}
}
