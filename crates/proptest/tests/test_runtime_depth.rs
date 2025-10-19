use estoa_proptest::strategy::{
    Strategy,
    runtime::{ConstantValueTree, Generation, Generator},
};
use rand::{CryptoRng, RngCore};

struct DepthOne;

impl Strategy for DepthOne {
    type Value = u8;
    type Tree = ConstantValueTree<u8>;

    fn new_tree<R: RngCore + CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        generator.recurse(|inner| inner.accept(ConstantValueTree::new(1)))
    }
}

struct DepthTwo;

impl Strategy for DepthTwo {
    type Value = u8;
    type Tree = ConstantValueTree<u8>;

    fn new_tree<R: RngCore + CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        generator
            .recurse(|g1| g1.recurse(|g2| g2.accept(ConstantValueTree::new(2))))
    }
}

#[test]
fn depth_zero_outside_recurse() {
    let generator = Generator::build(rand::rng());
    match generator.accept(ConstantValueTree::new(0u8)) {
        Generation::Accepted { depth, .. } => assert_eq!(depth, 0),
        Generation::Rejected { .. } => panic!("unexpected rejection"),
    }
}

#[test]
fn depth_one_inside_recurse() {
    let mut generator = Generator::build(rand::rng());
    match DepthOne.new_tree(&mut generator) {
        Generation::Accepted { depth, .. } => assert_eq!(depth, 1),
        Generation::Rejected { .. } => panic!("unexpected rejection"),
    }
}

#[test]
fn depth_two_nested_recurse() {
    let mut generator = Generator::build(rand::rng());
    match DepthTwo.new_tree(&mut generator) {
        Generation::Accepted { depth, .. } => assert_eq!(depth, 2),
        Generation::Rejected { .. } => panic!("unexpected rejection"),
    }
}
