use crate::{
    distribution::{self, AllChainSources},
    frameworks::cointime,
    transactions,
};

pub(crate) struct ImportSources<'a> {
    distribution: &'a distribution::Vecs,
    cointime: &'a cointime::Vecs,
    all_chain: &'a AllChainSources,
    transactions: &'a transactions::Vecs,
}

impl<'a> ImportSources<'a> {
    pub(crate) fn new(
        distribution: &'a distribution::Vecs,
        cointime: &'a cointime::Vecs,
        all_chain: &'a AllChainSources,
        transactions: &'a transactions::Vecs,
    ) -> Self {
        Self {
            distribution,
            cointime,
            all_chain,
            transactions,
        }
    }

    pub(super) fn distribution(&self) -> &distribution::Vecs {
        self.distribution
    }

    pub(super) fn cointime(&self) -> &cointime::Vecs {
        self.cointime
    }

    pub(super) fn all_chain(&self) -> &AllChainSources {
        self.all_chain
    }

    pub(super) fn transactions(&self) -> &transactions::Vecs {
        self.transactions
    }
}
