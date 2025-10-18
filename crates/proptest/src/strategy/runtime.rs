use crate::{
    arbitrary::Arbitrary,
    strategies::{DefaultGenerator, Generation},
    strategy::{Strategy, ValueTree},
};

pub trait StrategyAdapter {
    type Value;

    fn generate(
        &mut self,
        generator: &mut DefaultGenerator,
    ) -> Generation<Self::Value>;
}

pub trait IntoStrategyAdapter {
    type Adapter: StrategyAdapter;

    fn into_strategy_adapter(self) -> Self::Adapter;
}

pub struct IntegratedAdapter<S>
where
    S: Strategy,
{
    strategy: S,
}

impl<S> StrategyAdapter for IntegratedAdapter<S>
where
    S: Strategy,
    S::Value: Clone,
{
    type Value = S::Value;

    fn generate(
        &mut self,
        generator: &mut DefaultGenerator,
    ) -> Generation<Self::Value> {
        match self.strategy.new_tree(generator) {
            Generation::Accepted {
                iteration,
                depth,
                value,
            } => Generation::Accepted {
                iteration,
                depth,
                value: value.current().clone(),
            },
            Generation::Rejected {
                iteration,
                depth,
                value,
            } => Generation::Rejected {
                iteration,
                depth,
                value: value.current().clone(),
            },
        }
    }
}

impl<S> IntoStrategyAdapter for S
where
    S: Strategy,
    S::Value: Clone,
{
    type Adapter = IntegratedAdapter<S>;

    fn into_strategy_adapter(self) -> Self::Adapter {
        IntegratedAdapter { strategy: self }
    }
}

pub fn adapt<S>(strategy: S) -> S::Adapter
where
    S: IntoStrategyAdapter,
{
    strategy.into_strategy_adapter()
}

pub fn execute<S>(
    adapter: &mut S,
    generator: &mut DefaultGenerator,
) -> Generation<S::Value>
where
    S: StrategyAdapter,
{
    adapter.generate(generator)
}

pub fn from_arbitrary<T>(generator: &mut DefaultGenerator) -> Generation<T>
where
    T: Arbitrary,
{
    T::generate(generator)
}
