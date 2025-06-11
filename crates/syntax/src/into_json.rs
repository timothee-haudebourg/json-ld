use crate::{
	context, Container, ContainerKind, ContextEntry, Direction, Keyword, LenientLangTagBuf,
	Nullable,
};
use contextual::Contextual;
use indexmap::IndexSet;

impl<T: IntoJsonWithContext<N>, N> IntoJsonWithContext<N> for Vec<T> {
	fn into_json_with(self, context: &N) -> json_syntax::Value {
		json_syntax::Value::Array(
			self.into_iter()
				.map(|item| item.into_json_with(context))
				.collect(),
		)
	}
}

impl<T: IntoJsonWithContext<N>, N> IntoJsonWithContext<N> for IndexSet<T> {
	fn into_json_with(self, context: &N) -> json_syntax::Value {
		json_syntax::Value::Array(
			self.into_iter()
				.map(|item| item.into_json_with(context))
				.collect(),
		)
	}
}

pub trait IntoJsonWithContext<N>: Sized {
	fn into_json_with(self, context: &N) -> json_syntax::Value;
}

pub trait IntoJson: Sized {
	fn into_json(self) -> json_syntax::Value;
}

impl<T: IntoJsonWithContext<N>, N> IntoJson for Contextual<T, &N> {
	fn into_json(self) -> json_syntax::Value {
		T::into_json_with(self.0, self.1)
	}
}

impl<T: IntoJsonWithContext<N>, N> IntoJson for Contextual<T, &mut N> {
	fn into_json(self) -> json_syntax::Value {
		T::into_json_with(self.0, self.1)
	}
}

impl IntoJson for bool {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::Boolean(self)
	}
}

impl<T: IntoJson> IntoJson for Box<T> {
	fn into_json(self) -> json_syntax::Value {
		T::into_json(*self)
	}
}

impl IntoJson for iref::IriRefBuf {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.as_str().into())
	}
}

impl<T: IntoJson> IntoJson for Nullable<T> {
	fn into_json(self) -> json_syntax::Value {
		match self {
			Self::Null => json_syntax::Value::Null,
			Self::Some(other) => T::into_json(other),
		}
	}
}

impl IntoJson for Keyword {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_str().into())
	}
}

impl IntoJson for String {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into())
	}
}

impl IntoJson for LenientLangTagBuf {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_string().into())
	}
}

impl IntoJson for Direction {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.as_str().into())
	}
}

impl IntoJson for context::definition::TypeContainer {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_str().into())
	}
}

impl IntoJson for context::definition::Type {
	fn into_json(self) -> json_syntax::Value {
		let mut object = json_syntax::Object::new();

		object.insert("@container".into(), self.container.into_json());

		if let Some(protected) = self.protected {
			object.insert("@protected".into(), protected.into_json());
		}

		json_syntax::Value::Object(object)
	}
}

impl IntoJson for context::definition::Version {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::Number(self.into())
	}
}

impl IntoJson for context::definition::Vocab {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_string().into())
	}
}

impl IntoJson for context::definition::Key {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_string().into())
	}
}

impl IntoJson for context::term_definition::Index {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_string().into())
	}
}

impl IntoJson for context::term_definition::Nest {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_string().into())
	}
}

impl IntoJson for Container {
	fn into_json(self) -> json_syntax::Value {
		match self {
			Self::One(c) => ContainerKind::into_json(c),
			Self::Many(list) => {
				json_syntax::Value::Array(list.into_iter().map(IntoJson::into_json).collect())
			}
		}
	}
}

impl IntoJson for ContainerKind {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.as_str().into())
	}
}

impl IntoJson for context::term_definition::Id {
	fn into_json(self) -> json_syntax::Value {
		match self {
			Self::Keyword(k) => Keyword::into_json(k),
			Self::Term(t) => String::into_json(t),
		}
	}
}

impl IntoJson for context::Context {
	fn into_json(self) -> json_syntax::Value {
		match self {
			Self::One(c) => c.into_json(),
			Self::Many(list) => {
				json_syntax::Value::Array(list.into_iter().map(IntoJson::into_json).collect())
			}
		}
	}
}

impl IntoJson for ContextEntry {
	fn into_json(self) -> json_syntax::Value {
		match self {
			Self::Null => json_syntax::Value::Null,
			Self::IriRef(iri) => iref::IriRefBuf::into_json(iri),
			Self::Definition(def) => context::Definition::into_json(def),
		}
	}
}

impl IntoJson for context::Definition {
	fn into_json(self) -> json_syntax::Value {
		let mut object = json_syntax::Object::new();

		if let Some(base) = self.base {
			object.insert("@base".into(), base.into_json());
		}

		if let Some(import) = self.import {
			object.insert("@import".into(), import.into_json());
		}

		if let Some(language) = self.language {
			object.insert("@language".into(), language.into_json());
		}

		if let Some(direction) = self.direction {
			object.insert("@direction".into(), direction.into_json());
		}

		if let Some(propagate) = self.propagate {
			object.insert("@propagate".into(), propagate.into_json());
		}

		if let Some(protected) = self.protected {
			object.insert("@protected".into(), protected.into_json());
		}

		if let Some(type_) = self.type_ {
			object.insert("@type".into(), type_.into_json());
		}

		if let Some(version) = self.version {
			object.insert("@version".into(), version.into_json());
		}

		if let Some(vocab) = self.vocab {
			object.insert("@vocab".into(), vocab.into_json());
		}

		for (key, binding) in self.bindings {
			object.insert(key.into_string().into(), binding.into_json());
		}

		json_syntax::Value::Object(object)
	}
}

impl IntoJson for context::TermDefinition {
	fn into_json(self) -> json_syntax::Value {
		match self {
			Self::Simple(s) => context::term_definition::Simple::into_json(s),
			Self::Expanded(e) => context::term_definition::Expanded::into_json(*e),
		}
	}
}

impl IntoJson for context::term_definition::Simple {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_string().into())
	}
}

impl IntoJson for context::term_definition::Type {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_string().into())
	}
}

impl IntoJson for context::term_definition::TypeKeyword {
	fn into_json(self) -> json_syntax::Value {
		json_syntax::Value::String(self.into_str().into())
	}
}

impl IntoJson for context::term_definition::Expanded {
	fn into_json(self) -> json_syntax::Value {
		let mut object = json_syntax::Object::new();

		if let Some(id) = self.id {
			object.insert("@id".into(), id.into_json());
		}

		if let Some(type_) = self.type_ {
			object.insert("@type".into(), type_.into_json());
		}

		if let Some(context) = self.context {
			object.insert("@context".into(), context.into_json());
		}

		if let Some(reverse) = self.reverse {
			object.insert("@reverse".into(), reverse.into_json());
		}

		if let Some(index) = self.index {
			object.insert("@index".into(), index.into_json());
		}

		if let Some(language) = self.language {
			object.insert("@language".into(), language.into_json());
		}

		if let Some(direction) = self.direction {
			object.insert("@direction".into(), direction.into_json());
		}

		if let Some(container) = self.container {
			object.insert("@container".into(), container.into_json());
		}

		if let Some(nest) = self.nest {
			object.insert("@nest".into(), nest.into_json());
		}

		if let Some(prefix) = self.prefix {
			object.insert("@prefix".into(), prefix.into_json());
		}

		if let Some(propagate) = self.propagate {
			object.insert("@propagate".into(), propagate.into_json());
		}

		if let Some(protected) = self.protected {
			object.insert("@protected".into(), protected.into_json());
		}

		json_syntax::Value::Object(object)
	}
}
