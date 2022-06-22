use cc_traits::{Get, Iter, Len, MapIter};
use generic_json::{Json, ValueRef};

// pub async fn json_ld_eq<J: JsonContext + JsonExpand>(
// 	a: &J,
// 	b: &J,
// ) -> Result<bool, ExpansionError<J>>
// where
// 	J::Number: PartialEq,
// {
// 	Ok(simple_json_ld_eq(a, b) || full_json_ld_eq(a, b).await?)
// }

// pub async fn full_json_ld_eq<J: JsonContext + JsonExpand>(
// 	a: &J,
// 	b: &J,
// ) -> Result<bool, ExpansionError<J>> {
// 	let context_a: crate::context::Json<J> = context::Json::new(None);
// 	let context_b: crate::context::Json<J> = context::Json::new(None);

// 	let expanded_a = a
// 		.expand_with(
// 			None,
// 			&context_a,
// 			&mut crate::NoLoader::<J>::new(),
// 			crate::expansion::Options::default(),
// 		)
// 		.await?;

// 	let expanded_b = b
// 		.expand_with(
// 			None,
// 			&context_b,
// 			&mut crate::NoLoader::<J>::new(),
// 			crate::expansion::Options::default(),
// 		)
// 		.await?;

// 	let blank_ids_a = blank_ids(&expanded_a);
// 	let blank_ids_b = blank_ids(&expanded_b);

// 	if blank_ids_a.len() == blank_ids_b.len() {
// 		use crate::object::MappedEq;
// 		use permutohedron::LexicalPermutation;
// 		let source: HashMap<_, _> = blank_ids_a
// 			.into_iter()
// 			.enumerate()
// 			.map(|(i, id)| (id, i))
// 			.collect();
// 		let mut target: Vec<_> = blank_ids_b.into_iter().collect();

// 		loop {
// 			if expanded_a
// 				.objects()
// 				.mapped_eq(expanded_b.objects(), |id| &target[source[id]])
// 			{
// 				// eprintln!("found equality after substituting blank identifiers");
// 				// for (a, i) in source {
// 				// 	eprintln!("{} => {}", a, &target[i])
// 				// }
// 				break Ok(true);
// 			}

// 			if !target.next_permutation() {
// 				break Ok(false);
// 			}
// 		}
// 	} else {
// 		Ok(false)
// 	}
// }

pub fn simple_json_ld_eq<J: Json, K: Json>(a: &J, b: &K) -> bool
where
	J::Number: PartialEq<K::Number>,
{
	match (a.as_value_ref(), b.as_value_ref()) {
		(ValueRef::Array(a), ValueRef::Array(b)) if a.len() == b.len() => {
			let mut selected = Vec::with_capacity(a.len());
			selected.resize(a.len(), false);

			'a_items: for item in a.iter() {
				for (i, sel) in selected.iter_mut().enumerate() {
					if !*sel && simple_json_ld_eq(&*item, &*b.get(i).unwrap()) {
						*sel = true;
						continue 'a_items;
					}
				}

				return false;
			}

			true
		}
		(ValueRef::Object(a), ValueRef::Object(b)) if a.len() == b.len() => {
			for (key, value_a) in a.iter() {
				let key = key.as_ref();
				if let Some(value_b) = b.get(key) {
					if key == "@list" {
						match (value_a.as_value_ref(), value_b.as_value_ref()) {
							(ValueRef::Array(item_a), ValueRef::Array(item_b))
								if item_a.len() == item_b.len() =>
							{
								for i in 0..item_a.len() {
									if !simple_json_ld_eq(
										&*item_a.get(i).unwrap(),
										&*item_b.get(i).unwrap(),
									) {
										return false;
									}
								}
							}
							_ => {
								if !simple_json_ld_eq(&*value_a, &*value_b) {
									return false;
								}
							}
						}
					} else if !simple_json_ld_eq(&*value_a, &*value_b) {
						return false;
					}
				} else {
					return false;
				}
			}

			true
		}
		(ValueRef::Null, ValueRef::Null) => true,
		(ValueRef::Boolean(a), ValueRef::Boolean(b)) => a == b,
		(ValueRef::Number(a), ValueRef::Number(b)) => a == b,
		(ValueRef::String(a), ValueRef::String(b)) => (**a) == (**b),
		_ => false,
	}
}
