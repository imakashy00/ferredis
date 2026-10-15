pub mod string;
pub mod set;
pub mod list;
pub mod hash;

pub use string::*;
pub use set::*;
pub use list::*;
pub use hash::*;
// files has no idea what Redis or RESP is

// Method Name	Signature	What it does
// new	fn new() -> Self	Initializes an empty list wrapper around a VecDeque.
// push_left	fn push_left(&mut self, value: Vec<u8>)	Adds an element to the front of the queue.
// pop_left	fn pop_left(&mut self) -> Option<Vec<u8>>	Removes and returns the front element, if it exists.
// len	fn len(&self) -> usize	Returns the total count of elements currently in the list.
