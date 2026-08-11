mod block_metrics;
mod data_bytes_series;
mod fees_series;
mod vecs;

use brk_traversable::Traversable;
use brk_types::{OpReturnKind, OpReturnPolicyId};
use vecdb::ColumnId;

use super::{by_kind::ByKind, policy::Policy};

pub(crate) use block_metrics::BlockMetrics;
pub use data_bytes_series::DataBytesSeries;
pub use fees_series::FeesSeries;
pub use vecs::BreakdownVecs;

pub trait BreakdownAxis: ColumnId {
    type Series<T>: Clone + Traversable
    where
        T: Clone + Traversable + Send + Sync + 'static;

    fn series<T>(create: impl FnMut(Self, &'static str) -> T) -> Self::Series<T>
    where
        T: Clone + Traversable + Send + Sync + 'static;
}

impl BreakdownAxis for OpReturnKind {
    type Series<T>
        = ByKind<T>
    where
        T: Clone + Traversable + Send + Sync + 'static;

    fn series<T>(create: impl FnMut(Self, &'static str) -> T) -> Self::Series<T>
    where
        T: Clone + Traversable + Send + Sync + 'static,
    {
        ByKind::new(create)
    }
}

impl BreakdownAxis for OpReturnPolicyId {
    type Series<T>
        = Policy<T>
    where
        T: Clone + Traversable + Send + Sync + 'static;

    fn series<T>(create: impl FnMut(Self, &'static str) -> T) -> Self::Series<T>
    where
        T: Clone + Traversable + Send + Sync + 'static,
    {
        Policy::new(create)
    }
}
