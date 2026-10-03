use std::{sync::Arc, time::Instant};

use brk_mempool::ReadOnlyMempool;
use tokio::{
    sync::Semaphore,
    task::spawn_blocking,
    time::{self, Instant as TimeInstant},
};
use vecdb::ReadOnlyClone;

use crate::{Error, Query, QueryPluginSet, Result};

#[derive(Clone)]
pub struct AsyncQuery(Query);

impl AsyncQuery {
    pub fn read_deadline(&self) -> Instant {
        self.0
            .1
            .unwrap_or_else(|| Instant::now() + Query::UPDATE_WAIT_TIMEOUT)
    }
    /// Read on a blocking worker with deadline-bounded publication waits.
    /// A chain/mempool mismatch returns immediately; actions use `run`.
    pub async fn read<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Query) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        self.read_with_admission(None, f).await
    }

    pub async fn read_with_admission<F, T>(
        &self,
        admission: Option<&Arc<Semaphore>>,
        f: F,
    ) -> Result<T>
    where
        F: FnOnce(&Query) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let deadline = self.read_deadline();
        let read = async {
            let permit = match admission {
                Some(admission) => Some(
                    admission
                        .clone()
                        .acquire_owned()
                        .await
                        .map_err(|_| Error::Internal("query admission closed"))?,
                ),
                None => None,
            };
            let query = self.0.with_deadline(deadline);
            spawn_blocking(move || {
                let _permit = permit;
                query.check_deadline()?;
                f(&query)
            })
            .await?
        };
        time::timeout_at(TimeInstant::from_std(deadline), read)
            .await
            .unwrap_or(Err(Error::ReadTimeout))
    }

    pub fn with_deadline(&self, deadline: Instant) -> Self {
        Self(self.0.with_deadline(deadline))
    }
    pub fn build<P>(plugins: &P, mempool: Option<ReadOnlyMempool>) -> Self
    where
        P: ReadOnlyClone,
        P::ReadOnly: QueryPluginSet + 'static,
    {
        Self(Query::build(plugins, mempool))
    }

    /// Run one blocking operation. Use this for actions and already-prepared
    /// immutable work; use `read` for deadline-bounded publication-aware queries.
    pub async fn run<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Query) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let query = self.0.clone();
        spawn_blocking(move || {
            query.check_deadline()?;
            f(&query)
        })
        .await?
    }

    /// Run a cheap sync operation directly without spawn_blocking.
    /// Use this for simple accessors that don't do I/O.
    ///
    /// # Example
    /// ```ignore
    /// let height = query.sync(|q| q.height());
    /// ```
    pub fn sync<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Query) -> T,
    {
        f(&self.0)
    }
}
