# `estoa-proptest`

`estoa-proptest` is a library to create property tests in Rust, with minimal dependencies and easier expandability. It was created to test Estoa first and foremost.

Property tests exercise invariants with a broad range of automatically generated inputs, effectively testing how your code behaves without the need of creating specific examples to trigger behaviors.

## Usage

We provide a `proptest` macro that can be used to create tests. The test function creates examples for any given arguments, like so:

```rust
use estoa_proptest::proptest;

#[proptest]
fn test_add_commutativity(a: u32, b: u32) {
    assert_eq!(a as u64 + b as u64, b as u64 + a as u64);
}
```

You do not need to specify the input, as long as the property holds true, it will test thousands of different inputs.

```rust
use estoa_proptest::proptest;

#[proptest]
fn test_add_commutativity(a: u32, b: u32) {
    assert_eq!(a as u64 + b as u64 > a as u64);
}
```

To implement it for your own types, you can implement the `Arbitrary` trait, like so:

```rust
use estoa_proptest::{proptest, Arbitrary};

struct CounterExample(u32);

impl Arbitrary for CounterExample {
    fn arbitrary<R: rand::RngCore + rand::CryptoRng + ?Sized>(rng: &mut R) -> Self {
        Self(rng.random())
    }
}

#[proptest]
fn counter_never_underflows(counter: CounterExample, decrement: u16) {
    let mut value = counter.0;
    value = value.saturating_sub(decrement.into());
    assert!(value <= counter.0);
}

```

## Crafting Custom Strategies

You can write ad hoc strategies by defining a small helper type that implements `Strategy`. The generator supplies randomness; the strategy decides whether to accept the candidate it builds (and thus keep it) or reject it by returning `Generation::Accepted` or `Generation::Rejected`.

```rust
use estoa_proptest::{
    proptest,
    strategy::{
        runtime::{Generation, Generator},
        Strategy,
        ValueTree,
    },
};

#[derive(Default)]
struct BoundedPair;

impl Strategy for BoundedPair {
    type Value = (u8, u8);
    type Tree = ConstantValueTree<(u8, u8)>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        loop {
            let first = estoa_proptest::arbitrary::<u8, _>(generator).take();
            let second = estoa_proptest::arbitrary::<u8, _>(generator).take();
            if first <= second {
                return generator.accept(ConstantValueTree::new((first, second)));
            }
        }
    }
}

#[derive(Clone)]
struct ConstantValueTree<T> {
    value: T,
}

impl<T> ConstantValueTree<T> {
    fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> ValueTree for ConstantValueTree<T> {
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

#[proptest]
fn pair_is_increasing(
    #[strategy(BoundedPair::default())] pair: (u8, u8),
) {
    assert!(pair.0 <= pair.1);
}
```

## License

This software is dual-licensed under both the [MIT](./LICENSE) and [Apache 2.0](./LICENSE-APACHE) licenses. This should cover most possible uses, but if you need an exception for any reason, please do get in touch.
