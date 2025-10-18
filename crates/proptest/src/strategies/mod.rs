use rand::rngs::ThreadRng;

use crate::Arbitrary;

mod collections;
mod generator;
mod size_hint;

pub use collections::*;
pub use generator::*;
pub use size_hint::*;

pub use crate::strategy::{Strategy, Strategy as NewStrategy, ValueTree};

pub type DefaultGenerator = Generator<ThreadRng>;

pub fn any<T: Arbitrary>() -> impl FnMut(&mut DefaultGenerator) -> Generation<T>
{
    move |generator: &mut DefaultGenerator| T::generate(generator)
}

pub fn different<T: Arbitrary + PartialEq>()
-> impl FnMut(&mut DefaultGenerator) -> Generation<(T, T)> {
    let mut pair_strategy = any::<(T, T)>();
    move |generator: &mut DefaultGenerator| match pair_strategy(generator) {
        Generation::Accepted { value, .. } => {
            if value.0 != value.1 {
                generator.accept(value)
            } else {
                generator.reject(value)
            }
        }
        Generation::Rejected {
            iteration,
            depth,
            value,
        } => Generation::Rejected {
            iteration,
            depth,
            value,
        },
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::ThreadRng;

    use super::*;

    #[test]
    fn test_recurse_tracks_depth() {
        let mut generator =
            Generator::build_with_limit(ThreadRng::default(), usize::MAX);
        assert_eq!(generator.depth(), 0);

        let result: usize = generator.recurse(|outer| {
            assert_eq!(outer.depth(), 1);
            outer.recurse(|inner| {
                assert_eq!(inner.depth(), 2);
                42
            })
        });

        assert_eq!(result, 42);
        assert_eq!(generator.depth(), 0);
    }
}
