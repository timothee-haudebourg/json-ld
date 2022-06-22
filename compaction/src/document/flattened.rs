impl<J: JsonHash, T: Id> FlattenedDocument<J, T> {
	pub async fn compact<'a, K: JsonFrom<J>, C: ContextMut<T>, L: Loader, M>(
		&'a self,
		context: &'a context::ProcessedOwned<K, context::Inversible<T, C>>,
		loader: &'a mut L,
		options: compaction::Options,
		meta: M,
	) -> Result<K, Error>
	where
		K: Clone + JsonFrom<C::LocalContext>,
		J: compaction::JsonSrc,
		T: 'a + Sync + Send,
		C: Sync + Send,
		C::LocalContext: From<L::Output>,
		L: Sync + Send,
		M: 'a + Clone + Send + Sync + Fn(Option<&J::MetaData>) -> K::MetaData,
	{
		use compaction::Compact;
		let mut compacted: K = self
			.nodes
			.compact_full(
				context.as_ref(),
				context.as_ref(),
				None,
				loader,
				options,
				meta.clone(),
			)
			.await?;

		use crate::Document;
		compacted.embed_context(context, options, || meta(None))?;

		Ok(compacted)
	}
}