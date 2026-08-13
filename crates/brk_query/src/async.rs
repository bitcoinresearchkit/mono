use brk_computer::Computer;
use brk_error::Result;
use brk_indexer::Indexer;
use brk_mempool::Mempool;

use tokio::task::spawn_blocking;

use crate::Query;

#[derive(Clone)]
pub struct AsyncQuery(Query);

impl AsyncQuery {
    pub fn build(indexer: &Indexer, computer: &Computer, mempool: Option<Mempool>) -> Self {
        Self(Query::build(indexer, computer, mempool))
    }

    /// Run a blocking query operation on a spawn_blocking thread.
    /// Use this for I/O-heavy or CPU-intensive operations.
    ///
    /// # Example
    /// ```ignore
    /// let addr_stats = query.run(move |q| q.addr(addr)).await?;
    /// ```
    pub async fn run<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Query) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let query = self.0.clone();
        spawn_blocking(move || f(&query)).await?
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

    pub fn generation(&self) -> Option<usize> {
        self.0.indexer().generation()
    }

    #[inline]
    pub fn inner(&self) -> &Query {
        &self.0
    }
}
