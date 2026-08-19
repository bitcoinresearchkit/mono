use brk_error::Error;
use brk_types::{Addr, AddrHash, AddrHashPrefixMatches, OutputType};

use crate::Query;

const ADDR_HASH_PREFIX_MATCH_LIMIT: usize = 100;

impl Query {
    pub fn addr_hash_prefix_matches(
        &self,
        addr_type: OutputType,
        prefix: &str,
    ) -> brk_error::Result<AddrHashPrefixMatches> {
        if !addr_type.is_addr() {
            return Err(Error::UnsupportedType(addr_type.to_string()));
        }

        let prefix = AddrHashPrefix::parse(prefix)?;
        let stores = self.indexer().stores();
        let safe_type_index = self.safe_lengths().to_type_index(addr_type);
        let addr_readers = self.indexer().vecs().addrs.addr_readers();
        let mut addresses = Vec::new();
        let max_hash = AddrHash::new(u64::MAX);

        if let Some(upper) = prefix.upper {
            for (_, type_index) in stores.addr_hash_range(addr_type, prefix.lower..upper)? {
                if type_index >= safe_type_index {
                    continue;
                }

                let script = addr_readers.script_pubkey(addr_type, type_index);
                addresses.push(Addr::try_from((&script, addr_type))?);

                if addresses.len() > ADDR_HASH_PREFIX_MATCH_LIMIT {
                    break;
                }
            }
        } else {
            for (_, type_index) in stores.addr_hash_range(addr_type, prefix.lower..max_hash)? {
                if type_index >= safe_type_index {
                    continue;
                }

                let script = addr_readers.script_pubkey(addr_type, type_index);
                addresses.push(Addr::try_from((&script, addr_type))?);

                if addresses.len() > ADDR_HASH_PREFIX_MATCH_LIMIT {
                    break;
                }
            }

            if addresses.len() <= ADDR_HASH_PREFIX_MATCH_LIMIT
                && let Some(type_index) = stores.addr_index(addr_type, &max_hash)?
                && type_index < safe_type_index
            {
                let script = addr_readers.script_pubkey(addr_type, type_index);
                addresses.push(Addr::try_from((&script, addr_type))?);
            }
        }

        let truncated = addresses.len() > ADDR_HASH_PREFIX_MATCH_LIMIT;
        addresses.truncate(ADDR_HASH_PREFIX_MATCH_LIMIT);

        Ok(AddrHashPrefixMatches {
            addr_type,
            prefix: prefix.text,
            truncated,
            addresses,
        })
    }
}

struct AddrHashPrefix {
    text: String,
    lower: AddrHash,
    upper: Option<AddrHash>,
}

impl AddrHashPrefix {
    const MAX_NIBBLES: usize = u64::BITS as usize / 4;

    fn parse(prefix: &str) -> brk_error::Result<Self> {
        let nibbles = prefix.len();
        if !(1..=Self::MAX_NIBBLES).contains(&nibbles) {
            return Err(Self::parse_error());
        }

        let value = u64::from_str_radix(prefix, 16).map_err(|_| Self::parse_error())?;
        let shift = (Self::MAX_NIBBLES - nibbles) * 4;
        let factor = 1_u64 << shift;
        let lower = value * factor;
        let upper = value
            .checked_add(1)
            .and_then(|value| value.checked_mul(factor))
            .map(AddrHash::new);

        Ok(Self {
            text: prefix.to_ascii_lowercase(),
            lower: AddrHash::new(lower),
            upper,
        })
    }

    fn parse_error() -> Error {
        Error::Parse(format!(
            "hash prefix must be 1 to {} hexadecimal characters",
            Self::MAX_NIBBLES
        ))
    }
}
