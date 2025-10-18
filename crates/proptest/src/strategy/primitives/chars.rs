use std::ops::RangeInclusive;

use rand::Rng;

use super::integers::IntValueTree;
use crate::strategy::{
    Strategy,
    runtime::{Generation, Generator},
};

fn preferred_char(range: &RangeInclusive<char>) -> char {
    let preferred = [' ', '0', 'a'];
    for candidate in preferred {
        if range.contains(&candidate) {
            return candidate;
        }
    }
    *range.start()
}

fn halving_sequence(start: u32, target: u32) -> Vec<char> {
    let mut current = start;
    let mut sequence = Vec::new();

    while current != target {
        let diff = current.abs_diff(target);

        let mut step = diff / 2;
        if step == 0 {
            step = 1;
        }

        let next = if current > target {
            current - step
        } else {
            current + step
        };

        if next == current {
            break;
        }

        current = next;

        if let Some(ch) = char::from_u32(current)
            && sequence.last().copied() != Some(ch)
        {
            sequence.push(ch);
        }
    }

    if let Some(ch) = char::from_u32(target)
        && sequence.last().copied() != Some(ch)
    {
        sequence.push(ch);
    }

    sequence
}

fn build_char_candidates(
    value: char,
    range: &RangeInclusive<char>,
) -> Vec<char> {
    let mut candidates = Vec::new();
    let target = preferred_char(range);

    if value != target && range.contains(&target) {
        candidates.push(target);
    }

    for digit in '0'..='9' {
        if digit != value && range.contains(&digit) && digit != target {
            candidates.push(digit);
        }
    }

    for letter in 'a'..='z' {
        if letter != value
            && range.contains(&letter)
            && !candidates.contains(&letter)
        {
            candidates.push(letter);
        }
    }

    let seq = halving_sequence(value as u32, target as u32);
    for ch in seq {
        if ch != value && range.contains(&ch) && !candidates.contains(&ch) {
            candidates.push(ch);
        }
    }

    candidates
}

#[derive(Clone)]
pub struct AnyChar {
    range: RangeInclusive<char>,
}

impl AnyChar {
    pub fn new(range: RangeInclusive<char>) -> Self {
        Self { range }
    }
}

impl Default for AnyChar {
    fn default() -> Self {
        Self::new(char::MIN..=char::MAX)
    }
}

impl Strategy for AnyChar {
    type Value = char;
    type Tree = IntValueTree<char>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let value = generator.rng.random_range(self.range.clone());
        let candidates = build_char_candidates(value, &self.range);
        generator.accept(IntValueTree::new(value, candidates))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::ValueTree;

    #[test]
    fn char_prefers_space() {
        let range = ' '..='z';
        let candidates = build_char_candidates('x', &range);
        assert!(candidates.first().is_some_and(|c| *c == ' '));
    }

    #[test]
    fn char_sequence_approaches_target() {
        let range = 'a'..='z';
        let candidates = build_char_candidates('z', &range);
        assert!(candidates.contains(&'a'));
    }

    #[test]
    fn char_value_tree_shrinks() {
        let mut tree = IntValueTree::new('z', vec!['a', 'm']);
        assert!(tree.simplify());
        assert_eq!(*tree.current(), 'a');
        assert!(tree.complicate());
        assert_eq!(*tree.current(), 'z');
    }
}
