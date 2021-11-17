use iref::{
	IriRef,
	IriRefBuf
};
use crate::Meta;

/// Value located behind an IRI reference.
pub struct Loc<T> {
	/// The value.
	value: T,
	
	/// Path to the value.
	path: IriRefBuf
}

impl<T> Loc<T> {
	pub fn new(value: T, path: IriRefBuf) -> Self {
		Self {
			value, path
		}
	}

	pub fn value(&self) -> &T {
		&self.value
	}

	pub fn path(&self) -> IriRef<'_> {
		self.path.as_iri_ref()
	}
}

impl<T, M> Loc<Meta<T, M>> {
	pub fn cast_metadata<N>(self) -> Loc<Meta<T, N>> where N: From<M> {
		Loc::new(self.value.cast_metadata(), self.path)
	}

	pub fn with_metadata<N>(self, metadata: N) -> Loc<Meta<T, N>> {
		Loc::new(self.value.with_metadata(metadata), self.path)
	}
}