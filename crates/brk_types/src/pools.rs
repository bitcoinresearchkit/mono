use std::sync::OnceLock;

use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::{AddrBytes, PoolSlug, Version};

use super::Pool;

/// Increment when pool IDs, payout addresses, or coinbase tags change.
pub const POOL_ATTRIBUTION_VERSION: Version = Version::ONE;

const JSON_DATA: &str = include_str!("../pools-v2.json");
const TESTNET_IDS: &[u16] = &[145, 146, 149, 150, 156, 163];

#[derive(Deserialize)]
struct JsonPoolEntry {
    id: u16,
    name: String,
    #[serde(rename = "addresses")]
    addrs: Vec<String>,
    tags: Vec<String>,
    link: String,
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn empty_pool(id: usize) -> Pool {
    Pool {
        slug: PoolSlug::from(id as u8),
        name: "",
        addrs: Box::new([]),
        tags: Box::new([]),
        tags_lowercase: Box::new([]),
        link: "",
    }
}

#[derive(Debug)]
pub struct Pools {
    pools: Vec<Pool>,
    by_addr: FxHashMap<AddrBytes, PoolSlug>,
}

impl Pools {
    pub fn find_from_coinbase_tag(&self, coinbase_tag: &str) -> Option<&Pool> {
        let coinbase_tag = coinbase_tag.to_lowercase();
        self.iter().find(|pool| {
            pool.tags_lowercase
                .iter()
                .any(|pool_tag| coinbase_tag.contains(pool_tag))
        })
    }

    pub fn find_from_addr(&self, addr: &AddrBytes) -> Option<&Pool> {
        self.by_addr.get(addr).map(|&slug| self.get(slug))
    }

    pub fn get_unknown(&self) -> &Pool {
        &self.pools[0]
    }

    pub fn get(&self, slug: PoolSlug) -> &Pool {
        let i: u8 = slug.into();
        &self.pools[i as usize]
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pool> + '_ {
        self.pools.iter().filter(|pool| !pool.name.is_empty())
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

pub fn pools() -> &'static Pools {
    static POOLS: OnceLock<Pools> = OnceLock::new();
    POOLS.get_or_init(|| {
        let entries: Vec<JsonPoolEntry> =
            serde_json::from_str(JSON_DATA).expect("Failed to parse pools-v2.json");

        let max_id = entries.iter().map(|entry| entry.id).max().unwrap_or(0);
        assert!(
            max_id <= u8::MAX as u16,
            "pool ID {max_id} exceeds PoolSlug's u8 range"
        );
        let mut pools: Vec<Pool> = (0..=usize::from(max_id)).map(empty_pool).collect();

        // Position 0: Unknown pool
        pools[0] = Pool {
            slug: PoolSlug::Unknown,
            name: "Unknown",
            addrs: Box::new([]),
            tags: Box::new([]),
            tags_lowercase: Box::new([]),
            link: "",
        };

        for entry in entries {
            if TESTNET_IDS.contains(&entry.id) {
                continue;
            }
            let id = entry.id as usize;
            let slug = PoolSlug::from(id as u8);
            let tags_lowercase = entry
                .tags
                .iter()
                .map(|t| t.to_lowercase())
                .collect::<Vec<_>>()
                .into_boxed_slice();
            pools[id] = Pool {
                slug,
                name: leak_str(entry.name),
                link: leak_str(entry.link),
                addrs: entry
                    .addrs
                    .into_iter()
                    .map(leak_str)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                tags: entry
                    .tags
                    .into_iter()
                    .map(leak_str)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                tags_lowercase,
            };
        }

        let mut by_addr = FxHashMap::default();
        for pool in pools.iter().filter(|pool| !pool.name.is_empty()) {
            for addr in &pool.addrs {
                if let Ok(addr) = addr.parse() {
                    by_addr.entry(addr).or_insert(pool.slug);
                }
            }
        }

        Pools { pools, by_addr }
    })
}

#[cfg(test)]
mod tests {
    use crate::Addr;

    use super::*;

    #[test]
    fn bundled_json_entries_have_named_slugs() {
        let entries: Vec<JsonPoolEntry> =
            serde_json::from_str(JSON_DATA).expect("valid pools-v2.json");

        for entry in entries {
            if TESTNET_IDS.contains(&entry.id) {
                continue;
            }
            let id = u8::try_from(entry.id).expect("pool ID fits PoolSlug");
            let slug = PoolSlug::from(id);
            assert!(
                serde_json::to_string(&slug).is_ok(),
                "pool ID {} ({}) still maps to {slug:?}",
                entry.id,
                entry.name
            );
        }
    }

    #[test]
    fn dmnd_uses_upstream_id_171() {
        let dmnd = pools().get(PoolSlug::Dmnd);
        assert_eq!(dmnd.name, "DMND");
        assert_eq!(dmnd.mempool_unique_id(), 171);
    }

    #[test]
    fn binary_lookup_matches_string_lookup() {
        let pools = pools();

        for pool in pools.iter() {
            for configured in &pool.addrs {
                let Ok(bytes) = configured.parse::<AddrBytes>() else {
                    continue;
                };
                let reconstructed = Addr::try_from(&bytes).unwrap().to_string();
                let expected = pools
                    .iter()
                    .find(|pool| pool.addrs.contains(&reconstructed.as_str()))
                    .map(|pool| pool.slug);

                assert_eq!(
                    pools.find_from_addr(&bytes).map(|pool| pool.slug),
                    expected,
                    "{configured}"
                );
            }
        }
    }
}
