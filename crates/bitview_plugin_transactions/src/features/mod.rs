mod compute;
mod import;
mod vecs;

pub use vecs::Vecs;

/// The features whose per-block counts get cumulative and window sums here, passed to
/// `$callback`: each name is also its indexer feature's, and its count id is `<name>_tx_count`.
macro_rules! with_windowed_features {
    ($callback:ident) => {
        $callback!(
            p2tr,
            annex,
            sighash_all,
            sighash_none,
            sighash_single,
            sighash_default,
            sighash_anyone_can_pay,
            explicitly_rbf,
            dust_output
        )
    };
}
use with_windowed_features;
