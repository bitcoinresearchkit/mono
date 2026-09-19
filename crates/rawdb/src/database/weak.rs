use std::sync::{Arc, Weak};

use super::{Database, inner::DatabaseInner, owner::DatabaseOwner};

/// Weak reference to a [`Database`], held by regions to avoid reference cycles.
#[derive(Debug)]
pub(crate) struct WeakDatabase(Weak<DatabaseInner>, Weak<DatabaseOwner>);

impl WeakDatabase {
    pub(crate) fn new(db: &Database) -> Self {
        Self(
            Arc::downgrade(&db.inner),
            db.owner.as_ref().map(Arc::downgrade).unwrap_or_default(),
        )
    }

    pub(crate) fn upgrade(&self) -> Database {
        Database {
            inner: self
                .0
                .upgrade()
                .expect("Database was dropped while Region still exists"),
            owner: self.1.upgrade(),
        }
    }
}
