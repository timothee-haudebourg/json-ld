use iref::IriRef;

use crate::{
	syntax::{
		context::{
			BindingsIter, ContextDefinition, ContextTerm, ContextType, EntryValueRef, KeyOrKeyword,
			TermDefinition, Vocab,
		},
		Context, ContextEntry,
	},
	Direction, LenientLangTagBuf, Nullable,
};

pub struct Merged<'a> {
	base: &'a ContextDefinition,
	imported: Option<Context>,
}

impl<'a> Merged<'a> {
	pub fn new(base: &'a ContextDefinition, imported: Option<Context>) -> Self {
		Self { base, imported }
	}

	pub fn imported(&self) -> Option<&ContextDefinition> {
		self.imported.as_ref().and_then(|imported| match imported {
			Context::One(ContextEntry::Definition(import_context)) => Some(import_context),
			_ => None,
		})
	}

	pub fn base(&self) -> Option<Nullable<&IriRef>> {
		self.base
			.base
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.base.as_ref()))
			.map(Nullable::as_deref)
	}

	pub fn vocab(&self) -> Option<Nullable<&Vocab>> {
		self.base
			.vocab
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.vocab.as_ref()))
			.map(Nullable::as_ref)
	}

	pub fn language(&self) -> Option<Nullable<&LenientLangTagBuf>> {
		self.base
			.language
			.as_ref()
			.or_else(|| self.imported().and_then(|i| i.language.as_ref()))
			.map(Nullable::as_ref)
	}

	pub fn direction(&self) -> Option<Nullable<Direction>> {
		self.base
			.direction
			.or_else(|| self.imported().and_then(|i| i.direction))
	}

	pub fn protected(&self) -> Option<bool> {
		self.base
			.protected
			.or_else(|| self.imported().and_then(|i| i.protected))
	}

	pub fn type_(&self) -> Option<ContextType> {
		self.base
			.type_
			.or_else(|| self.imported().and_then(|i| i.type_))
	}

	pub fn bindings(&self) -> MergedBindings {
		MergedBindings {
			base: self.base,
			base_bindings: self.base.bindings.iter(),
			imported_bindings: self.imported().map(|i| i.bindings.iter()),
		}
	}

	pub fn get(&self, key: &KeyOrKeyword) -> Option<EntryValueRef> {
		self.base
			.get(key)
			.or_else(|| self.imported().and_then(|i| i.get(key)))
		// self.imported()
		// 	.and_then(|i| i.get(key))
		// 	.or_else(|| self.base.get(key))
	}
}

impl<'a> From<&'a ContextDefinition> for Merged<'a> {
	fn from(base: &'a ContextDefinition) -> Self {
		Self {
			base,
			imported: None,
		}
	}
}

// #[derive(Default)]
// pub struct StaticMergedBindings {
// 	base_offset: usize,
// 	imported_offset: usize
// }

// impl StaticMergedBindings {
// 	pub fn next<'a>(
// 		&mut self,
// 		context: &Merged<'a>
// 	) -> Option<BindingRef<'a>> {
// 		match context.base.bindings.get_entry(self.base_offset) {
// 			Some(entry) => {
// 				self.base_offset += 1;
// 				Some(entry)
// 			},
// 			None => {
// 				match context.imported() {
// 					Some(imported) => {
// 						while let Some(entry) = imported.bindings.get_entry(self.imported_offset) {
// 							self.imported_offset += 1;
// 							if context.base.get_binding(entry.0).is_none() {
// 								return Some(entry)
// 							}
// 						}

// 						None
// 					},
// 					None => None
// 				}
// 			}
// 		}
// 	}
// }

type BindingRef<'a> = (&'a ContextTerm, Nullable<&'a TermDefinition>);

pub struct MergedBindings<'a> {
	base: &'a ContextDefinition,
	base_bindings: BindingsIter<'a>,
	imported_bindings: Option<BindingsIter<'a>>,
}

impl<'a> Iterator for MergedBindings<'a> {
	type Item = BindingRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.imported_bindings {
			Some(imported_bindings) => {
				for (key_ref, def) in imported_bindings {
					let key = key_ref.to_owned();
					if self.base.get_binding(&key).is_none() {
						return Some((key_ref, def));
					}
				}

				self.base_bindings.next()
			}
			None => self.base_bindings.next(),
		}
	}
}
