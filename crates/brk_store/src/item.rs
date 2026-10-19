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

    /// What one key's changes in a batch leave, taken in arrival order: a put replaces an earlier
    /// put, and a weak delete cancels a pending put, else stays as a tombstone. Nothing may follow
    /// a tombstone.
    #[inline]
    pub fn reduce<'a>(changes: impl IntoIterator<Item = &'a Self>) -> Option<&'a Self>
    where
        K: 'a,
        V: 'a,
    {
        changes
            .into_iter()
            .fold(None, |pending, change| match change {
                Self::Value { .. } => {
                    // See `Pending::insert`: a put after a deletion in one batch would hide the stored
                    // value only.
                    debug_assert!(
                        !matches!(pending, Some(Self::Tomb(_))),
                        "put after a deletion in pending store changes"
                    );
                    Some(change)
                }
                Self::Tomb(_) => {
                    debug_assert!(
                        !matches!(pending, Some(Self::Tomb(_))),
                        "double deletion in pending store changes"
                    );
                    match pending {
                        Some(Self::Value { .. }) => None,
                        _ => Some(change),
                    }
                }
            })
    }
}
