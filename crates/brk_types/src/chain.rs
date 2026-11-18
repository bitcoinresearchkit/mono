use std::sync::OnceLock;

use bitcoin::{
    Network,
    bech32::{Hrp, hrp},
    constants::{self, SUBSIDY_HALVING_INTERVAL},
};

use brk_error::{Error, Result};

/// Core's built-in chains. rust-bitcoin supplies message starts and address encodings from
/// `network`; the directory, RPC port and intervals come from Core's chainparams.
#[derive(Debug)]
pub struct Chain {
    pub network: Network,
    pub subdir: &'static str,
    pub rpc_port: u16,
    pub base58: (u8, u8),
    pub hrp: Hrp,
    pub halving_interval: u32,
    pub difficulty_interval: u32,
}

impl Chain {
    const fn new(
        network: Network,
        subdir: &'static str,
        rpc_port: u16,
        halving_interval: u32,
        difficulty_interval: u32,
    ) -> Self {
        let base58 = match network {
            Network::Bitcoin => (
                constants::PUBKEY_ADDRESS_PREFIX_MAIN,
                constants::SCRIPT_ADDRESS_PREFIX_MAIN,
            ),
            _ => (
                constants::PUBKEY_ADDRESS_PREFIX_TEST,
                constants::SCRIPT_ADDRESS_PREFIX_TEST,
            ),
        };
        let hrp = match network {
            Network::Bitcoin => hrp::BC,
            Network::Regtest => hrp::BCRT,
            _ => hrp::TB,
        };
        Self {
            base58,
            network,
            subdir,
            rpc_port,
            hrp,
            halving_interval,
            difficulty_interval,
        }
    }

    pub const MAIN: &'static Self = &Self::ALL[0];

    pub const ALL: &'static [Self] = &[
        Self::new(Network::Bitcoin, "", 8332, SUBSIDY_HALVING_INTERVAL, 2016),
        Self::new(
            Network::Testnet,
            "testnet3",
            18332,
            SUBSIDY_HALVING_INTERVAL,
            2016,
        ),
        Self::new(
            Network::Testnet4,
            "testnet4",
            48332,
            SUBSIDY_HALVING_INTERVAL,
            2016,
        ),
        Self::new(
            Network::Signet,
            "signet",
            38332,
            SUBSIDY_HALVING_INTERVAL,
            2016,
        ),
        Self::new(Network::Regtest, "regtest", 18443, 150, 144),
    ];

    pub fn for_network(network: Network) -> &'static Self {
        Self::ALL
            .iter()
            .find(|chain| chain.network == network)
            .expect("all built-in networks")
    }

    pub fn genesis_time(&self) -> crate::Timestamp {
        crate::Timestamp::new(constants::genesis_block(self.network).header.time)
    }

    pub fn is_mainnet(&self) -> bool {
        self.network == Network::Bitcoin
    }

    /// The local template supports chains without testnet's time-dependent work or signet's challenge.
    pub fn supports_block_template(&self) -> bool {
        matches!(self.network, Network::Bitcoin | Network::Regtest)
    }

    /// Serde serializers and index conversions have no context parameter. A daemon serves one
    /// immutable chain for its lifetime.
    pub fn activate(&'static self) -> Result<()> {
        if ACTIVE.get_or_init(|| self).network != self.network {
            return Err(Error::Internal(
                "a different Bitcoin network is already active",
            ));
        }
        Ok(())
    }
}

static ACTIVE: OnceLock<&'static Chain> = OnceLock::new();

pub fn chain() -> &'static Chain {
    ACTIVE
        .get()
        .copied()
        .expect("Bitcoin network has not been activated")
}
