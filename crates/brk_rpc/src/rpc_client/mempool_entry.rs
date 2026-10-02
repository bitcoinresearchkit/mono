use brk_types::{Bitcoin, MempoolEntryInfo, Sats, Timestamp, Txid, VSize, Weight};
use serde::Deserialize;

/// The subset of a `getmempoolentry` response that BRK consumes.
#[derive(Deserialize)]
pub struct MempoolEntry {
    vsize: VSize,
    weight: Weight,
    time: Timestamp,
    fees: MempoolFees,
    depends: Vec<Txid>,
}

impl MempoolEntry {
    pub fn into_info(self, txid: Txid) -> MempoolEntryInfo {
        MempoolEntryInfo {
            txid,
            vsize: self.vsize,
            weight: self.weight,
            fee: Sats::from(self.fees.base),
            first_seen: self.time,
            depends: self.depends,
        }
    }
}

#[derive(Deserialize)]
struct MempoolFees {
    base: Bitcoin,
}
