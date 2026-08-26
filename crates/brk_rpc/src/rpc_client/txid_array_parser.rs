use std::fmt;

use brk_types::Txid;
use serde::{Deserializer, de, de::DeserializeSeed};
use serde_json::{Deserializer as JsonDeserializer, Result as JsonResult};

/// Parses Core's compact JSON transaction-ID array into one exactly sized
/// allocation. Serde's generic `Vec` visitor has no size hint for JSON arrays.
pub struct TxidArrayParser(usize);

impl TxidArrayParser {
    pub fn parse(json: &str) -> JsonResult<Vec<Txid>> {
        let mut deserializer = JsonDeserializer::from_str(json);
        let txids = Self(json.len() / 67).deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(txids)
    }
}

impl<'de> DeserializeSeed<'de> for TxidArrayParser {
    type Value = Vec<Txid>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TxidArrayVisitor(self.0))
    }
}

struct TxidArrayVisitor(usize);

impl<'de> de::Visitor<'de> for TxidArrayVisitor {
    type Value = Vec<Txid>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an array of transaction IDs")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut txids = Vec::with_capacity(self.0);
        while let Some(txid) = sequence.next_element()? {
            txids.push(txid);
        }
        Ok(txids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compact_and_whitespace_arrays() {
        let first = "0000000000000000000000000000000000000000000000000000000000000001";
        let second = "0000000000000000000000000000000000000000000000000000000000000002";

        for json in [
            format!("[\"{first}\",\"{second}\"]"),
            format!("[ \"{first}\", \n \"{second}\" ]"),
        ] {
            let txids = TxidArrayParser::parse(&json).unwrap();
            assert_eq!(txids.len(), 2);
            assert_eq!(txids[0].to_string(), first);
            assert_eq!(txids[1].to_string(), second);
        }
    }

    #[test]
    fn rejects_trailing_json() {
        assert!(TxidArrayParser::parse("[] null").is_err());
    }
}
