use json_ld::{JsonLdOptions, RemoteDocument};
use json_ld_testing::ManifestEntry;

pub fn build_options(entry: &ManifestEntry) -> JsonLdOptions {
	let mut options = JsonLdOptions::default();
	if let Some(ref opts) = entry.options {
		if let Some(mode) = opts.processing_mode {
			options.processing_mode = mode;
		}
		options.base = opts.base.clone();
		if let Some(ref ctx) = opts.expand_context {
			options.expand_context = Some(RemoteDocument::iri(ctx.clone()));
		}
		if let Some(compact_to_relative) = opts.compact_to_relative {
			options.compact_to_relative = compact_to_relative;
		}
		if let Some(compact_arrays) = opts.compact_arrays {
			options.compact_arrays = compact_arrays;
		}
	}
	options
}
