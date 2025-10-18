use rand::Rng;

use crate::{
    strategies::{Generation, Generator},
    strategy::{Strategy, ValueTree},
};

pub struct ResultStrategy<OS, ES> {
    ok: OS,
    err: ES,
}

impl<OS, ES> ResultStrategy<OS, ES> {
    pub fn new(ok: OS, err: ES) -> Self {
        Self { ok, err }
    }
}

impl<OS, ES> Strategy for ResultStrategy<OS, ES>
where
    OS: Strategy,
    ES: Strategy,
    OS::Value: Clone,
    ES::Value: Clone,
{
    type Value = Result<OS::Value, ES::Value>;
    type Tree = ResultValueTree<OS::Tree, ES::Tree>;

    fn new_tree<R: rand::RngCore + rand::CryptoRng>(
        &mut self,
        generator: &mut Generator<R>,
    ) -> Generation<Self::Tree> {
        let ok_res = self.ok.new_tree(generator);
        let err_res = self.err.new_tree(generator);

        match (ok_res, err_res) {
            (
                Generation::Accepted { value: ok_tree, .. },
                Generation::Accepted {
                    value: err_tree, ..
                },
            ) => {
                let choose_ok = generator.rng.random::<bool>();
                let current = if choose_ok {
                    Ok(ok_tree.current().clone())
                } else {
                    Err(err_tree.current().clone())
                };

                Generation::Accepted {
                    iteration: generator.iteration(),
                    depth: generator.depth(),
                    value: ResultValueTree::new(
                        ok_tree, err_tree, current, choose_ok,
                    ),
                }
            }
            (
                Generation::Rejected {
                    iteration,
                    depth,
                    value: ok_tree,
                },
                Generation::Accepted {
                    value: err_tree, ..
                },
            ) => Generation::Rejected {
                iteration,
                depth,
                value: {
                    let ok_current = ok_tree.current().clone();
                    ResultValueTree::new(
                        ok_tree,
                        err_tree,
                        Ok(ok_current),
                        true,
                    )
                },
            },
            (
                Generation::Accepted { value: ok_tree, .. },
                Generation::Rejected {
                    iteration,
                    depth,
                    value: err_tree,
                },
            ) => Generation::Rejected {
                iteration,
                depth,
                value: {
                    let err_current = err_tree.current().clone();
                    ResultValueTree::new(
                        ok_tree,
                        err_tree,
                        Err(err_current),
                        false,
                    )
                },
            },
            (
                Generation::Rejected {
                    iteration,
                    depth,
                    value: ok_tree,
                },
                Generation::Rejected {
                    value: err_tree, ..
                },
            ) => Generation::Rejected {
                iteration,
                depth,
                value: {
                    let ok_current = ok_tree.current().clone();
                    ResultValueTree::new(
                        ok_tree,
                        err_tree,
                        Ok(ok_current),
                        true,
                    )
                },
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Variant {
    Ok,
    Err,
}

pub struct ResultValueTree<OT, ET>
where
    OT: ValueTree,
    ET: ValueTree,
    OT::Value: Clone,
    ET::Value: Clone,
{
    ok: OT,
    err: ET,
    current_variant: Variant,
    converted_from_ok: bool,
    current: Result<OT::Value, ET::Value>,
}

impl<OT, ET> ResultValueTree<OT, ET>
where
    OT: ValueTree,
    ET: ValueTree,
    OT::Value: Clone,
    ET::Value: Clone,
{
    fn new(
        ok: OT,
        err: ET,
        current: Result<OT::Value, ET::Value>,
        choose_ok: bool,
    ) -> Self {
        Self {
            ok,
            err,
            current_variant: if choose_ok { Variant::Ok } else { Variant::Err },
            converted_from_ok: false,
            current,
        }
    }
}

impl<OT, ET> ValueTree for ResultValueTree<OT, ET>
where
    OT: ValueTree,
    ET: ValueTree,
    OT::Value: Clone,
    ET::Value: Clone,
{
    type Value = Result<OT::Value, ET::Value>;

    fn current(&self) -> &Self::Value {
        &self.current
    }

    fn simplify(&mut self) -> bool {
        match self.current_variant {
            Variant::Ok => {
                if !self.converted_from_ok {
                    self.converted_from_ok = true;
                    self.current_variant = Variant::Err;
                    self.current = Err(self.err.current().clone());
                    true
                } else if self.ok.simplify() {
                    self.current = Ok(self.ok.current().clone());
                    true
                } else {
                    false
                }
            }
            Variant::Err => {
                if self.err.simplify() {
                    self.current = Err(self.err.current().clone());
                    true
                } else {
                    false
                }
            }
        }
    }

    fn complicate(&mut self) -> bool {
        match self.current_variant {
            Variant::Ok => {
                if self.ok.complicate() {
                    self.current = Ok(self.ok.current().clone());
                    true
                } else {
                    false
                }
            }
            Variant::Err => {
                if self.converted_from_ok {
                    self.current_variant = Variant::Ok;
                    self.converted_from_ok = false;
                    self.current = Ok(self.ok.current().clone());
                    true
                } else if self.err.complicate() {
                    self.current = Err(self.err.current().clone());
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::primitives::IntValueTree;

    #[test]
    fn result_prefers_err() {
        let ok = IntValueTree::new(5, vec![1]);
        let err = IntValueTree::new(7, vec![2]);
        let mut tree = ResultValueTree::new(ok, err, Ok(5), true);
        assert!(tree.simplify());
        assert!(matches!(tree.current(), Err(7)));
    }

    #[test]
    fn result_complicate_reverts_to_ok() {
        let ok = IntValueTree::new(5, vec![1]);
        let err = IntValueTree::new(7, vec![2]);
        let mut tree = ResultValueTree::new(ok, err, Ok(5), true);
        assert!(tree.simplify());
        assert!(tree.complicate());
        assert!(matches!(tree.current(), Ok(5)));
    }
}
