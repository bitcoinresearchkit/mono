mod compute;
mod import;
mod vecs;

pub use vecs::{Scripts, Signatures, Vecs};

/// The features with a flag, by group, passed to `$callback`: each name is also its indexer
/// feature's, and its count id is `<name>_tx_count`.
macro_rules! with_flagged_features {
    ($callback:ident) => {
        $callback!(
            scripts: [
                p2pk,
                p2ms,
                p2pkh,
                p2sh,
                p2wpkh,
                p2wsh,
                p2tr,
                p2a,
                empty_script,
                unknown_script,
                fake_pubkey,
                fake_scripthash
            ],
            signatures: [
                annex,
                sighash_all,
                sighash_none,
                sighash_single,
                sighash_default,
                sighash_anyone_can_pay
            ],
            others: [explicitly_rbf, dust_output]
        )
    };
}
use with_flagged_features;
