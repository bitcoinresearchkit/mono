use std::{fmt::Debug, hash::Hash, mem};

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
    K: Debug + Eq + Hash,
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
                let mut pending = None;
                for change in changes {
                    match change {
                        Item::Value {
                            key: change_key,
                            value,
                        } if change_key == key => pending = Some(Some(value)),
                        Item::Tomb(change_key) if change_key == key => {
                            pending = match pending {
                                Some(Some(_)) => None,
                                _ => Some(None),
                            };
                        }
                        _ => {}
                    }
                }
                pending
            }
        }
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        match self {
            Self::Hashed { puts, dels } => {
                let _ = dels.is_empty() || dels.remove(&key);
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
