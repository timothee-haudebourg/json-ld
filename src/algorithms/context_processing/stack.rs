use std::sync::Arc;

use iref::{Iri, IriBuf};

/// Context processing stack.
///
/// Contains the list of the loaded contexts to detect loops.
#[derive(Clone)]
pub struct ProcessingStack {
	head: Option<Arc<StackNode>>,
}

impl ProcessingStack {
	/// Creates a new empty processing stack.
	pub fn new() -> Self {
		Self { head: None }
	}

	/// Checks if the stack is empty.
	pub fn is_empty(&self) -> bool {
		self.head.is_none()
	}

	/// Checks if the given URL is already in the stack.
	///
	/// This is used for loop detection.
	pub fn cycle(&self, url: &Iri) -> bool {
		match &self.head {
			Some(head) => head.contains(url),
			None => false,
		}
	}

	/// Push a new URL to the stack, unless it is already in the stack.
	///
	/// Returns `true` if the URL was successfully added or
	/// `false` if a loop has been detected.
	pub fn push(&mut self, url: IriBuf) -> bool {
		if self.cycle(&url) {
			false
		} else {
			let mut head = None;
			std::mem::swap(&mut head, &mut self.head);
			self.head = Some(Arc::new(StackNode::new(head, url)));
			true
		}
	}
}

impl Default for ProcessingStack {
	fn default() -> Self {
		Self::new()
	}
}

/// Single frame of the context processing stack.
struct StackNode {
	/// Previous frame.
	previous: Option<Arc<StackNode>>,

	/// URL of the last loaded context.
	url: IriBuf,
}

impl StackNode {
	/// Create a new stack frame registering the load of the given context URL.
	fn new(previous: Option<Arc<StackNode>>, url: IriBuf) -> StackNode {
		StackNode { previous, url }
	}

	/// Checks if this frame or any parent holds the given URL.
	fn contains(&self, url: &Iri) -> bool {
		if self.url == *url {
			true
		} else {
			match &self.previous {
				Some(prev) => prev.contains(url),
				None => false,
			}
		}
	}
}
