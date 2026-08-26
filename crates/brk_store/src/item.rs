use std::cmp::Ordering;

#[derive(Clone)]
pub enum Item<K, V> {
    Value { key: K, value: V },
    Tomb(K),
}

impl<K, V> Item<K, V> {
    #[inline]
    pub fn key(&self) -> &K {
        match self {
            Self::Value { key, .. } | Self::Tomb(key) => key,
        }
    }

    #[inline]
    pub fn apply_to(self, pending: &mut Option<Self>) {
        match self {
            Self::Value { .. } => *pending = Some(self),
            Self::Tomb(key) => match pending.take() {
                Some(Self::Value { .. }) => {}
                Some(Self::Tomb(_)) => {
                    debug_assert!(false, "double deletion in pending store changes");
                    *pending = Some(Self::Tomb(key));
                }
                None => *pending = Some(Self::Tomb(key)),
            },
        }
    }
}

impl<K: Ord, V> Ord for Item<K, V> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(other.key())
    }
}

impl<K: Ord, V> PartialOrd for Item<K, V> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Eq, V> PartialEq for Item<K, V> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl<K: Eq, V> Eq for Item<K, V> {}
