mod arrays;
mod bools;
mod chars;
mod floats;
mod integers;
mod options;
mod results;
mod strings;
mod tuples;

pub use arrays::*;
pub use bools::*;
pub use chars::*;
pub use floats::*;
pub use integers::*;
pub use options::*;
pub use results::*;
pub use strings::*;
pub use tuples::*;

#[derive(Default)]
pub struct StaticTree<T> {
    value: T,
}

impl<T> StaticTree<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Clone> super::ValueTree for StaticTree<T> {
    type Value = T;

    fn current(&self) -> &Self::Value {
        &self.value
    }

    fn simplify(&mut self) -> bool {
        false
    }

    fn complicate(&mut self) -> bool {
        false
    }
}
