use iref::Iri;

use crate::{
	algorithms::expansion::ExpansionOptions,
	context::{RawProcessedContext, TermDefinitionRef},
};

/// Expander.
pub struct Expander<'a> {
	pub base_url: Option<&'a Iri>,
	pub options: ExpansionOptions,
	pub active_context: &'a RawProcessedContext,
	pub active_property: Option<&'a str>,
}

impl<'a> Expander<'a> {
	pub fn active_property_definition(&self) -> Option<TermDefinitionRef> {
		self.active_property
			.and_then(|t| self.active_context.get(t))
	}

	// pub fn warn(&mut self, w: Warning) {
	// 	(self.on_warning)(w)
	// }

	pub fn with_active_context<'b>(
		&'b self,
		active_context: &'b RawProcessedContext,
	) -> Expander<'b> {
		Expander {
			// loader: &mut *self.loader,
			// on_warning: &mut *self.on_warning,
			base_url: self.base_url,
			options: self.options,
			active_context,
			active_property: self.active_property,
		}
	}

	pub fn with_active_property<'b>(&'b self, active_property: Option<&'b str>) -> Expander<'b> {
		Expander {
			// loader: &mut *self.loader,
			// on_warning: &mut *self.on_warning,
			base_url: self.base_url,
			options: self.options,
			active_context: self.active_context,
			active_property,
		}
	}

	pub fn with_active_context_and_property<'b>(
		&'b self,
		active_context: &'b RawProcessedContext,
		active_property: Option<&'b str>,
	) -> Expander<'b> {
		Expander {
			// loader: &mut *self.loader,
			// on_warning: &mut *self.on_warning,
			base_url: self.base_url,
			options: self.options,
			active_context,
			active_property,
		}
	}
}
