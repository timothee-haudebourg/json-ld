use std::{
	iter::Enumerate,
	collections::{HashMap, HashSet}
};

// pub struct HashSubstitutions<T, U, I: Iterator, TF, UF> {
// 	inner: Pairings<HashMap<T, U>, I>
// }

// impl<T, U, I: Iterator> Iterator for HashSubstitutions<T, U, I> {
// 	type Item = HashMap<T, U>;

// 	fn next(&mut self) -> Option<Self::Item> {
// 		match self {
// 			Some(item) => {
// 				// ...
// 			},
// 			None => 
// 		}
// 	}
// }

pub struct Pairings<Q, I: Iterator, F> {
	stack: Vec<Frame<Q, I>>,
	next_state: F
}

impl<Q, I: Iterator, F> Pairings<Q, I, F> {
	pub fn new(a: I, b: I, initial_state: Q, next_state: F) -> Self where I: Clone, F: Fn(&Q, I::Item, I::Item) -> Option<Q> {
		let b = b.enumerate();
		Self {
			stack: vec![Frame {
				state: initial_state,
				a,
				a_candidate: None,
				b: b.clone(),
				b_candidates: b,
				b_selected: HashSet::new()
			}],
			next_state
		}
	}
}

pub struct Frame<Q, I: Iterator> {
	state: Q,
	a: I,
	a_candidate: Option<I::Item>,
	b: Enumerate<I>,
	b_candidates: Enumerate<I>,
	b_selected: HashSet<usize>
}

impl<Q, I, F> Iterator for Pairings<Q, I, F>
where
	I: Clone + Iterator,
	I::Item: Clone + PartialEq,
	F: Fn(&Q, I::Item, I::Item) -> Option<Q>
{
	type Item = Q;

	fn next(&mut self) -> Option<Q> {
		while let Some(mut frame) = self.stack.pop() {
			match frame.a_candidate.clone() {
				Some(item) => {
					while let Some((i, other_item)) = frame.b_candidates.next() {
						if !frame.b_selected.contains(&i) {
							let mut b_selected = frame.b_selected.clone();
							b_selected.insert(i);

							let next_frame = (self.next_state)(&frame.state, item, other_item).map(|next_state| {
								Frame {
									state: next_state,
									a: frame.a.clone(),
									a_candidate: None,
									b: frame.b.clone(),
									b_candidates: frame.b.clone(),
									b_selected
								}
							});

							self.stack.push(frame);

							if let Some(next_frame) = next_frame {
								self.stack.push(next_frame)
							}

							break
						}
					}
				},
				None => {
					match frame.a.next() {
						None => {
							let b_selected = &frame.b_selected;
							if frame.b_candidates.all(move |(i, _)| b_selected.contains(&i)) {
								return Some(frame.state)
							}
						},
						Some(item) => {
							frame.a_candidate = Some(item);
							self.stack.push(frame)
						}
					}
				}
			}
		}

		None
	}
}

#[cfg(test)]
mod tests {
	use std::collections::{BTreeMap, BTreeSet};

	macro_rules! set {
		{ $($value:expr),* } => {
			{
				let mut set = std::collections::BTreeSet::new();
				$(
					set.insert($value);
				)*
				set
			}
		};
	}

	macro_rules! map {
		{ $($key:expr => $value:expr),* } => {
			{
				let mut map = std::collections::BTreeMap::new();
				$(
					map.insert($key, $value);
				)*
				map
			}
		};
	}

	fn test(a: Vec<u32>, b: Vec<u32>, expected_substitutions: BTreeSet<BTreeMap<u32, u32>>) {
		let substitutions: BTreeSet<_> = super::Pairings::new(a.iter(), b.iter(), BTreeMap::<u32, u32>::new(), |substitution, a, b| {
			let mut new_substitution = substitution.clone();

			use std::collections::btree_map::Entry;
			match new_substitution.entry(*a) {
				Entry::Occupied(entry) => {
					if entry.get() != b {
						return None
					}
				}
				Entry::Vacant(entry) => {
					entry.insert(*b);
				}
			}

			Some(new_substitution)
		}).collect();

		assert_eq!(substitutions, expected_substitutions)
	}

	#[test]
	fn substitution1() {
		test(
			vec![0, 1],
			vec![1, 0],
			set![
				map! {0 => 1, 1 => 0},
				map! {0 => 0, 1 => 1}
			]
		)
	}

	#[test]
	fn substitution2() {
		test(
			vec![0, 1, 1, 2],
			vec![2, 0, 0, 1],
			set![
				map! {1 => 0, 0 => 2, 2 => 1},
				map! {1 => 0, 0 => 1, 2 => 2}
			]
		)
	}

	#[test]
	fn substitution3() {
		test(
			vec![0, 1, 1, 2, 2, 2, 3, 3, 4],
			vec![2, 0, 0, 3, 3, 3, 1, 1, 4],
			set![
				map! {0 => 2, 1 => 0, 2 => 3, 3 => 1, 4 => 4},
				map! {0 => 4, 1 => 0, 2 => 3, 3 => 1, 4 => 2},
				map! {0 => 2, 1 => 1, 2 => 3, 3 => 0, 4 => 4},
				map! {0 => 4, 1 => 1, 2 => 3, 3 => 0, 4 => 2}
			]
		)
	}
}