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

#[cfg(test)]
mod tests {
    use serde_json::from_str;

    use super::*;

    #[test]
    fn ignores_unused_core_fields() {
        let txid = "0000000000000000000000000000000000000000000000000000000000000001";
        let json = format!(
            r#"{{"vsize":250,"weight":1000,"time":1700000000,"fees":{{"base":0.00001,"modified":0.00001}},"depends":["{txid}"],"spentby":["{txid}"]}}"#
        );
        let entry: MempoolEntry = from_str(&json).unwrap();
        let info = entry.into_info(Txid::COINBASE);

        assert_eq!(u64::from(info.vsize), 250);
        assert_eq!(u64::from(info.weight), 1000);
        assert_eq!(info.depends.len(), 1);
        assert_eq!(info.depends[0].to_string(), txid);
    }

    #[test]
    fn rejects_negative_numeric_fields() {
        let json =
            r#"{"vsize":250,"weight":-1,"time":1700000000,"fees":{"base":0.00001},"depends":[]}"#;
        assert!(from_str::<MempoolEntry>(json).is_err());
    }
}
