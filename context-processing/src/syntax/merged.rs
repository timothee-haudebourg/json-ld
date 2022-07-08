use json_ld_syntax::{
	self as syntax,
	context::{KeyRef, TermBindingRef}
};
use locspan::Meta;
use iref::IriRef;
use syntax::AnyContextDefinition;

pub struct Merged<'a, C: syntax::AnyContextEntry> {
	base: &'a C::Definition,
	imported: Option<C>
}

impl<'a, C: syntax::AnyContextEntry> Merged<'a, C> {
	pub fn new(
		base: &'a C::Definition,
		imported: Option<C>
	) -> Self {
		Self {
			base,
			imported
		}
	}

	pub fn imported(&self) -> Option<&C::Definition> {
		self.imported.as_ref().and_then(|imported| match imported.as_entry_ref() {
			syntax::ContextEntryRef::One(Meta(syntax::ContextRef::Definition(import_context), _)) => Some(import_context),
			_ => None
		})
	}

	pub fn base(&self) -> Option<Meta<syntax::Nullable<IriRef>, C::Metadata>> {
		self.imported().and_then(|i| i.base()).or_else(|| self.base.base())
	}

	pub fn vocab(&self) -> Option<Meta<syntax::Nullable<syntax::context::VocabRef>, C::Metadata>> {
		self.imported().and_then(|i| i.vocab()).or_else(|| self.base.vocab())
	}

	pub fn language(&self) -> Option<Meta<syntax::Nullable<syntax::LenientLanguageTag>, C::Metadata>> {
		self.imported().and_then(|i| i.language()).or_else(|| self.base.language())
	}

	pub fn direction(&self) -> Option<Meta<syntax::Nullable<syntax::Direction>, C::Metadata>> {
		self.imported().and_then(|i| i.direction()).or_else(|| self.base.direction())
	}

	pub fn protected(&self) -> Option<Meta<bool, C::Metadata>> {
		self.imported().and_then(|i| i.protected()).or_else(|| self.base.protected())
	}

	pub fn bindings(&self) -> MergedBindings<C> {
		todo!()
	}

	pub fn get(&self, key: &syntax::context::KeyOrKeyword) -> Option<syntax::context::EntryRef<C>> {
		self.imported().and_then(|i| i.get(key)).or_else(|| self.base.get(key))
	}
}

impl<'a, C: syntax::AnyContextEntry> From<&'a C::Definition> for Merged<'a, C> {
	fn from(base: &'a C::Definition) -> Self {
		Self {
			base,
			imported: None
		}
	}
} 

pub struct MergedBindings<'a, C: 'a + syntax::AnyContextEntry> {
	base_bindings: <C::Definition as syntax::AnyContextDefinition<C>>::Bindings<'a>,
	imported: Option<MergedImportedBindings<'a, C>>
}

pub struct MergedImportedBindings<'a, C: 'a + syntax::AnyContextEntry> {
	bindings: <C::Definition as syntax::AnyContextDefinition<C>>::Bindings<'a>,
	context: &'a C::Definition
}

impl<'a, C: 'a + syntax::AnyContextEntry> Iterator for MergedBindings<'a, C> {
	type Item = (KeyRef<'a>, TermBindingRef<'a, C>);

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.imported {
			Some(imported) => {
				for (key_ref, def) in self.base_bindings.by_ref() {
					let key = key_ref.to_owned();
					if imported.context.get_binding(&key).is_none() {
						return Some((key_ref, def))
					}
				}

				imported.bindings.next()
			}
			None => self.base_bindings.next()
		}
	}
}