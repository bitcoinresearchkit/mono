use std::ops::{Deref, DerefMut};

use brk_types::{EmptyAddrData, ExtendedEmptyAddrIndex, FundedAddrData, FundedAddrIndex};

/// Address data and the persistent representation it came from.
#[derive(Debug, Clone)]
pub enum SourcedAddrData<T> {
    New(T),
    FromFunded(FundedAddrIndex, T),
    FromInlineEmpty(T),
    FromExtendedEmpty(ExtendedEmptyAddrIndex, T),
}

impl<T> SourcedAddrData<T> {
    #[inline]
    fn map<U>(self, map: impl FnOnce(T) -> U) -> SourcedAddrData<U> {
        match self {
            Self::New(data) => SourcedAddrData::New(map(data)),
            Self::FromFunded(index, data) => SourcedAddrData::FromFunded(index, map(data)),
            Self::FromInlineEmpty(data) => SourcedAddrData::FromInlineEmpty(map(data)),
            Self::FromExtendedEmpty(index, data) => {
                SourcedAddrData::FromExtendedEmpty(index, map(data))
            }
        }
    }
}

impl<T> Deref for SourcedAddrData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::New(value)
            | Self::FromFunded(_, value)
            | Self::FromInlineEmpty(value)
            | Self::FromExtendedEmpty(_, value) => value,
        }
    }
}

impl<T> DerefMut for SourcedAddrData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::New(value)
            | Self::FromFunded(_, value)
            | Self::FromInlineEmpty(value)
            | Self::FromExtendedEmpty(_, value) => value,
        }
    }
}

impl From<SourcedAddrData<EmptyAddrData>> for SourcedAddrData<FundedAddrData> {
    #[inline]
    fn from(value: SourcedAddrData<EmptyAddrData>) -> Self {
        value.map(Into::into)
    }
}

impl From<SourcedAddrData<FundedAddrData>> for SourcedAddrData<EmptyAddrData> {
    #[inline]
    fn from(value: SourcedAddrData<FundedAddrData>) -> Self {
        value.map(Into::into)
    }
}
