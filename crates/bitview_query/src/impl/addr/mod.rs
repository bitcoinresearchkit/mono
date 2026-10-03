pub mod activity;
pub mod hash_prefix;
pub mod mempool;
pub mod resolve;
pub mod stats;
pub mod txs;
pub mod utxos;

use brk_types::{AddrBytes, OutputType};

use crate::{Error, Result};

pub use txs::{ResolvedAddrChainTxs, ResolvedAddrTxs};
pub use utxos::ResolvedAddrUtxos;

/// Parses a client-supplied address: a string that is no address (or one for another
/// network) is invalid input; a valid address of a kind Bitview does not index (a future
/// witness version, say) is an unsupported type.
pub(crate) fn parse_addr(addr: &str) -> Result<AddrBytes> {
    let script = AddrBytes::addr_to_script(addr).map_err(|error| match error {
        brk_error::Error::InvalidNetwork => Error::InvalidNetwork,
        _ => Error::InvalidAddr,
    })?;
    let output_type = OutputType::from(&script);
    AddrBytes::try_from((&script, output_type))
        .map_err(|_| Error::UnsupportedType(output_type.to_string()))
}
