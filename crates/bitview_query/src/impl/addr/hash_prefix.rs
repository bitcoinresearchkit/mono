use std::ops::Bound;

use bitview_primitives::AddrHash;
use bitview_types::AddressHashPrefixMatches;
use brk_types::{Address, OutputType};

use crate::{Error, Query, Result};

const ADDR_HASH_PREFIX_MATCH_LIMIT: usize = 100;

impl Query {
    pub fn addr_hash_prefix_matches(
        &self,
        addr_type: OutputType,
        prefix: &str,
    ) -> Result<AddressHashPrefixMatches> {
        if !addr_type.is_addr() {
            return Err(Error::UnsupportedType(addr_type.to_string()));
        }

        let prefix = AddrHashPrefix::parse(prefix)?;
        let pin = self.pin_safe_lengths()?;
        let stores = self.indexer().stores();
        let safe_type_index = pin.lengths().to_type_index(addr_type);
        let addr_readers = self.indexer().vecs().addrs.addr_readers();
        let mut addresses = Vec::new();

        for record in stores.addr_hash_range(addr_type, prefix.range)? {
            let (_, type_index) = record?;
            if type_index >= safe_type_index {
                continue;
            }

            let script = addr_readers.script_pubkey(addr_type, type_index);
            addresses.push(
                Address::try_from((&script, addr_type))
                    .map_err(|_| Error::Internal("stored script is not an address"))?,
            );

            if addresses.len() > ADDR_HASH_PREFIX_MATCH_LIMIT {
                break;
            }
        }

        let truncated = addresses.len() > ADDR_HASH_PREFIX_MATCH_LIMIT;
        addresses.truncate(ADDR_HASH_PREFIX_MATCH_LIMIT);

        Ok(AddressHashPrefixMatches {
            address_type: addr_type,
            prefix: prefix.text,
            truncated,
            addresses,
        })
    }
}

struct AddrHashPrefix {
    text: String,
    /// The hashes starting with the prefix; an all-`f` prefix runs to the last hash.
    range: (Bound<AddrHash>, Bound<AddrHash>),
}

impl AddrHashPrefix {
    const MAX_NIBBLES: usize = u64::BITS as usize / 4;

    fn parse(prefix: &str) -> Result<Self> {
        let nibbles = prefix.len();
        if !(1..=Self::MAX_NIBBLES).contains(&nibbles)
            || !prefix.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(Self::parse_error());
        }

        let value = u64::from_str_radix(prefix, 16).map_err(|_| Self::parse_error())?;
        let shift = (Self::MAX_NIBBLES - nibbles) * 4;
        let factor = 1_u64 << shift;
        let lower = Bound::Included(AddrHash::new(value * factor));
        let upper = value
            .checked_add(1)
            .and_then(|value| value.checked_mul(factor))
            .map_or(Bound::Unbounded, |end| Bound::Excluded(AddrHash::new(end)));

        Ok(Self {
            text: prefix.to_ascii_lowercase(),
            range: (lower, upper),
        })
    }

    fn parse_error() -> Error {
        Error::InvalidParam(format!(
            "hash prefix must be 1 to {} hexadecimal characters",
            Self::MAX_NIBBLES
        ))
    }
}
