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
