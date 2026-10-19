use std::{hash::Hash, mem};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{Kind, item::Item};

#[derive(Clone)]
pub enum Pending<K, V> {
    Hashed {
        puts: FxHashMap<K, V>,
        dels: FxHashSet<K>,
    },
    Sequential(Vec<Item<K, V>>),
}

impl<K, V> Pending<K, V>
where
    K: Eq + Hash,
{
    pub fn new(kind: Kind) -> Self {
        match kind {
            Kind::Vec => Self::Sequential(Vec::new()),
            Kind::Random | Kind::Recent => Self::Hashed {
                puts: FxHashMap::default(),
                dels: FxHashSet::default(),
            },
        }
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<Option<&V>> {
        match self {
            Self::Hashed { puts, dels } => {
                if let Some(value) = puts.get(key) {
                    return Some(Some(value));
                }
                if !dels.is_empty() && dels.contains(key) {
                    return Some(None);
                }
                None
            }
            Self::Sequential(changes) => {
                Item::reduce(changes.iter().filter(|change| change.key() == key)).map(|change| {
                    match change {
                        Item::Value { value, .. } => Some(value),
                        Item::Tomb(_) => None,
                    }
                })
            }
        }
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        match self {
            Self::Hashed { puts, dels } => {
                // A key deleted and put again in one batch would be a plain value over the stored one,
                // which a later weak delete would bring back; callers never do it (ingest rejects it).
                debug_assert!(
                    dels.is_empty() || !dels.contains(&key),
                    "put after a deletion in pending store changes"
                );
                puts.insert(key, value);
            }
            Self::Sequential(changes) => changes.push(Item::Value { key, value }),
        }
    }

    #[inline]
    pub fn remove(&mut self, key: K) {
        match self {
            Self::Hashed { puts, dels } => {
                if puts.remove(&key).is_some() {
                    return;
                }
                let inserted = dels.insert(key);
                debug_assert!(inserted, "double deletion in pending store changes");
            }
            Self::Sequential(changes) => changes.push(Item::Tomb(key)),
        }
    }

    pub fn take(&mut self) -> Self {
        match self {
            Self::Hashed { puts, dels } => Self::Hashed {
                puts: mem::take(puts),
                dels: mem::take(dels),
            },
            Self::Sequential(changes) => Self::Sequential(mem::take(changes)),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Hashed { puts, dels } => puts.is_empty() && dels.is_empty(),
            Self::Sequential(changes) => changes.is_empty(),
        }
    }
}
