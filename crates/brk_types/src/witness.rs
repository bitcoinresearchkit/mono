use bitcoin::{Script, Witness as BitcoinWitness};
use derive_more::{From, Into};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Transaction witness: a stack of byte arrays, one per witness item.
///
/// Wraps `bitcoin::Witness` (single-buffer layout with offsets, much
/// more compact than `Vec<Vec<u8>>`). Serializes as a JSON array of
/// hex strings - the format used by Bitcoin Core REST and mempool.space
/// and matching brk's `script_sig: ScriptBuf` (bytes internally, hex
/// on the wire).
#[derive(Debug, Clone, Default, Serialize, Deserialize, From, Into)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(transparent)]
#[cfg_attr(feature = "schemars", schemars(with = "Vec<String>"))]
pub struct Witness(BitcoinWitness);

impl Witness {
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// P2WSH witness script (BIP141): the last item.
    #[inline]
    pub(crate) fn witness_script(&self) -> Option<&Script> {
        self.0.witness_script()
    }

    /// Taproot script-path leaf script (BIP341, annex-aware).
    #[inline]
    pub(crate) fn taproot_leaf_script(&self) -> Option<&Script> {
        self.0.taproot_leaf_script().map(|leaf| leaf.script)
    }
}

impl From<&Witness> for BitcoinWitness {
    #[inline]
    fn from(w: &Witness) -> Self {
        w.0.clone()
    }
}
