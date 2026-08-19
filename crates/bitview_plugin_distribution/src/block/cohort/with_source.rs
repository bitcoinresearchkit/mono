use std::ops::{Deref, DerefMut};

use brk_types::{EmptyAddrData, EmptyAddrIndex, FundedAddrData, FundedAddrIndex};

/// Address data wrapped with its source location for flush operations.
///
/// This enum tracks where the data came from so it can be correctly
/// updated or created during the flush phase.
#[derive(Debug, Clone)]
pub enum WithAddrDataSource<T> {
    /// Brand new address (never seen before)
    New(T),
    /// Funded from funded address storage (with original index)
    FromFunded(FundedAddrIndex, T),
    /// Funded from empty address storage (with original index)
    FromEmpty(EmptyAddrIndex, T),
}

impl<T> Deref for WithAddrDataSource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::New(v) | Self::FromFunded(_, v) | Self::FromEmpty(_, v) => v,
        }
    }
}

impl<T> DerefMut for WithAddrDataSource<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::New(v) | Self::FromFunded(_, v) | Self::FromEmpty(_, v) => v,
        }
    }
}

impl From<WithAddrDataSource<EmptyAddrData>> for WithAddrDataSource<FundedAddrData> {
    #[inline]
    fn from(value: WithAddrDataSource<EmptyAddrData>) -> Self {
        match value {
            WithAddrDataSource::New(v) => Self::New(v.into()),
            WithAddrDataSource::FromFunded(i, v) => Self::FromFunded(i, v.into()),
            WithAddrDataSource::FromEmpty(i, v) => Self::FromEmpty(i, v.into()),
        }
    }
}

impl From<WithAddrDataSource<FundedAddrData>> for WithAddrDataSource<EmptyAddrData> {
    #[inline]
    fn from(value: WithAddrDataSource<FundedAddrData>) -> Self {
        match value {
            WithAddrDataSource::New(v) => Self::New(v.into()),
            WithAddrDataSource::FromFunded(i, v) => Self::FromFunded(i, v.into()),
            WithAddrDataSource::FromEmpty(i, v) => Self::FromEmpty(i, v.into()),
        }
    }
}
