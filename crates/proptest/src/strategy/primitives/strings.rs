use std::ops::RangeInclusive;

use rand::Rng;

use super::{AnyChar, IntValueTree};
use crate::{
    arbitrary::STRING_MAX_LEN,
    strategy::{
        SizeHint,
        Strategy,
        ValueTree,
        runtime::{Generation, Generator},
    },
};

fn build_drop_plan(len: usize) -> Vec<usize> {
    let mut plan = Vec::new();
    let mut size = len / 2;

    while size > 0 {
        plan.push(size);
        size /= 2;
    }

    if !plan.contains(&1) && len > 0 {
        plan.push(1);
    }

    plan
}

fn sample_length<R: rand::RngCore + rand::CryptoRng>(
    rng: &mut R,
    range: &RangeInclusive<usize>,
) -> usize {
    let lo = *range.start();
    let hi = *range.end();

    if lo == hi {
        return lo;
    }

    let lo_u = lo as u128;
    let hi_u = hi as u128;
    let span = (hi_u - lo_u) + 1;
    let offset = rng.random_range(0..span);
    (lo_u + offset) as usize
}

#[derive(Clone)]
pub struct AnyString {
    char_strategy: AnyChar,
    len_range: RangeInclusive<usize>,
}

impl AnyString {
    pub fn new<H>(len_hint: H) -> Self
    where
        H: SizeHint,
    {
        Self {
            char_strategy: AnyChar::default(),
            len_range: len_hint.to_inclusive(),
        }
    }
}

impl Default for AnyString {
    fn default() -> Self {
        Self::new(0..=STRING_MAX_LEN)
    }
}

impl Strategy for AnyString {
    type Value = String;
    type Tree = StringValueTree;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let len = sample_length(&mut generator.rng, &self.len_range);
        let min_len = *self.len_range.start();
        let mut char_trees = Vec::with_capacity(len);

        for _ in 0..len {
            match self.char_strategy.new_tree(generator) {
                Generation::Accepted { value, .. } => char_trees.push(value),
                Generation::Rejected {
                    iteration, depth, ..
                } => {
                    return Generation::Rejected {
                        iteration,
                        depth,
                        value: StringValueTree::from_trees(char_trees, min_len),
                    };
                }
            }
        }

        generator.accept(StringValueTree::from_trees(char_trees, min_len))
    }
}

#[derive(Clone, Copy)]
enum Stage {
    Length { chunk_index: usize, offset: usize },
    Elements { index: usize },
}

enum History {
    RemovedChunk {
        index: usize,
        chunk_index: usize,
        chunk: Vec<IntValueTree<char>>,
    },
    Element {
        index: usize,
    },
}

pub struct StringValueTree {
    chars: Vec<IntValueTree<char>>,
    current_chars: Vec<char>,
    current: String,
    min_len: usize,
    drop_plan: Vec<usize>,
    stage: Stage,
    history: Vec<History>,
}

impl StringValueTree {
    pub fn from_trees(chars: Vec<IntValueTree<char>>, min_len: usize) -> Self {
        let drop_plan = build_drop_plan(chars.len());
        let stage = if drop_plan.is_empty() {
            Stage::Elements { index: 0 }
        } else {
            Stage::Length {
                chunk_index: 0,
                offset: 0,
            }
        };

        let mut tree = Self {
            chars,
            current_chars: Vec::new(),
            current: String::new(),
            min_len,
            drop_plan,
            stage,
            history: Vec::new(),
        };

        tree.sync_current();
        tree
    }

    fn sync_current(&mut self) {
        self.current_chars =
            self.chars.iter().map(|tree| *tree.current()).collect();
        self.rebuild_string();
    }

    fn rebuild_string(&mut self) {
        self.current.clear();
        self.current_chars
            .iter()
            .for_each(|ch| self.current.push(*ch));
    }

    fn len(&self) -> usize {
        self.chars.len()
    }

    fn seek_length_from(
        &mut self,
        mut chunk_index: usize,
        mut offset: usize,
    ) -> Option<(usize, usize, usize)> {
        while chunk_index < self.drop_plan.len() {
            let chunk_size = self.drop_plan[chunk_index];

            if chunk_size == 0
                || self.len() <= self.min_len
                || chunk_size > self.len()
                || self.len().saturating_sub(chunk_size) < self.min_len
            {
                chunk_index += 1;
                offset = 0;
                continue;
            }

            if offset + chunk_size > self.len() {
                chunk_index += 1;
                offset = 0;
                continue;
            }

            self.stage = Stage::Length {
                chunk_index,
                offset,
            };
            return Some((chunk_index, offset, chunk_size));
        }

        self.stage = Stage::Elements { index: 0 };
        None
    }
}

impl ValueTree for StringValueTree {
    type Value = String;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        loop {
            match self.stage {
                Stage::Length {
                    chunk_index,
                    offset,
                } => {
                    let Some((ci, off, chunk_size)) =
                        self.seek_length_from(chunk_index, offset)
                    else {
                        continue;
                    };

                    let removed: Vec<IntValueTree<char>> =
                        self.chars.drain(off..off + chunk_size).collect();
                    self.current_chars.drain(off..off + chunk_size).count();
                    self.rebuild_string();
                    self.history.push(History::RemovedChunk {
                        index: off,
                        chunk_index: ci,
                        chunk: removed,
                    });
                    return true;
                }
                Stage::Elements { index } => {
                    if index >= self.len() {
                        return false;
                    }

                    if self.chars[index].simplify() {
                        self.current_chars[index] =
                            *self.chars[index].current();
                        self.rebuild_string();
                        self.history.push(History::Element { index });
                        return true;
                    } else {
                        self.stage = Stage::Elements { index: index + 1 };
                    }
                }
            }
        }
    }

    fn complicate(&mut self) -> bool {
        let Some(entry) = self.history.pop() else {
            return false;
        };

        match entry {
            History::RemovedChunk {
                index,
                chunk_index,
                chunk,
            } => {
                let values: Vec<char> =
                    chunk.iter().map(|tree| *tree.current()).collect();
                self.chars.splice(index..index, chunk);
                self.current_chars.splice(index..index, values);
                self.rebuild_string();

                match self.seek_length_from(chunk_index, index + 1) {
                    Some(_) => true,
                    None => !self.current_chars.is_empty(),
                }
            }
            History::Element { index } => {
                if self.chars[index].complicate() {
                    self.current_chars[index] = *self.chars[index].current();
                    self.rebuild_string();
                    self.history.push(History::Element { index });
                    true
                } else {
                    self.current_chars[index] = *self.chars[index].current();
                    self.rebuild_string();
                    if index + 1 < self.len() {
                        self.stage = Stage::Elements { index: index + 1 };
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::ValueTree;

    fn make_char_tree(c: char) -> IntValueTree<char> {
        IntValueTree::new(c, Vec::new())
    }

    #[test]
    fn string_drop_plan_halves() {
        let plan = build_drop_plan(8);
        assert_eq!(plan, vec![4, 2, 1]);
    }

    #[test]
    fn string_shrinks_length_first() {
        let mut tree = StringValueTree::from_trees(
            vec![
                make_char_tree('a'),
                make_char_tree('b'),
                make_char_tree('c'),
            ],
            0,
        );
        assert_eq!(tree.current(), "abc");
        assert!(tree.simplify());
        assert!(tree.current().len() < 3);
    }

    #[test]
    fn string_respects_min_len() {
        let mut tree = StringValueTree::from_trees(
            vec![make_char_tree('x'), make_char_tree('y')],
            2,
        );
        assert!(!tree.simplify());
    }

    #[test]
    fn string_eventually_shrinks_characters() {
        let mut tree = StringValueTree::from_trees(
            vec![make_char_tree('x'), IntValueTree::new('y', vec!['a'])],
            1,
        );

        assert!(tree.simplify());
        assert_eq!(tree.current(), "y");

        assert!(tree.simplify());
        assert_eq!(tree.current(), "a");
    }
}
