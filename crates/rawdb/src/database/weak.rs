use std::sync::{Arc, Weak};

use super::{Database, owner::DatabaseOwner};

/// Regions borrow both ownership roles without keeping the database alive.
#[derive(Debug)]
pub(crate) struct WeakDatabase(Weak<DatabaseOwner>, Weak<DatabaseOwner>);

impl WeakDatabase {
    pub(crate) fn new(db: &Database) -> Self {
        Self(
            db.inner.foreground.clone(),
            Arc::downgrade(db.inner.background.as_ref().unwrap_or(&db.inner)),
        )
    }

    #[inline]
    pub(crate) fn upgrade(&self) -> Database {
        match self.0.upgrade() {
            Some(inner) => Database { inner },
            None => self.upgrade_background(),
        }
    }

    #[cold]
    fn upgrade_background(&self) -> Database {
        Database {
            inner: self
                .1
                .upgrade()
                .expect("Database was dropped while Region still exists"),
        }
    }
}
