use bitview_traversable::Traversable;
use derive_more::{Deref, DerefMut};

/// A fiat value represented in cents and US dollars.
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct Fiat<C, U> {
    /// Reported in US dollars.
    pub usd: U,
    /// Reported in US cents; 100 cents equal one US dollar.
    #[traversable(hidden)]
    pub cents: C,
}

/// A dollar sum over a trailing window, saying when it has a value.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct PricedSumUsd<U>(
    /// Undefined (null) until the window lies at or after the first exchange price (block
    /// 74,641): blocks before it are valued at zero.
    pub U,
);

/// A dollar level's change over a trailing window, saying when it has a value.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct PricedDeltaUsd<U>(
    /// Undefined (null) until the window starts after the first exchange price (block 74,641):
    /// values before it are zero.
    pub U,
);

/// A dollar mean over a trailing window, saying which blocks it's over.
#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct PricedAvgUsd<U>(
    /// Over the window's blocks from the first exchange price (block 74,641) on; undefined
    /// (null) before it.
    pub U,
);
