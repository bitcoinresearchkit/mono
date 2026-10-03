/// Typed internal classification; strings exist only at the HTTP boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    NotFound,
    InvalidAddr,
    InvalidNetwork,
    UnsupportedType,
    ParseError,
    NoSeries,
    SeriesUnsupportedIndex,
    WeightExceeded,
    TooManyUtxos,
    UnknownAddr,
    UnknownTxid,
    OutOfRange,
    UnindexableDate,
    NoData,
    SeriesNotFound,
    MempoolNotAvailable,
    StateUpdating,
    InternalError,
    BadRequest,
    #[cfg(any(feature = "chain", test))]
    Overloaded,
    Timeout,
    MethodNotAllowed,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::InvalidAddr => "invalid_addr",
            Self::InvalidNetwork => "invalid_network",
            Self::UnsupportedType => "unsupported_type",
            Self::ParseError => "parse_error",
            Self::NoSeries => "no_series",
            Self::SeriesUnsupportedIndex => "series_unsupported_index",
            Self::WeightExceeded => "weight_exceeded",
            Self::TooManyUtxos => "too_many_utxos",
            Self::UnknownAddr => "unknown_addr",
            Self::UnknownTxid => "unknown_txid",
            Self::OutOfRange => "out_of_range",
            Self::UnindexableDate => "unindexable_date",
            Self::NoData => "no_data",
            Self::SeriesNotFound => "series_not_found",
            Self::MempoolNotAvailable => "mempool_not_available",
            Self::StateUpdating => "state_updating",
            Self::InternalError => "internal_error",
            Self::BadRequest => "bad_request",
            #[cfg(any(feature = "chain", test))]
            Self::Overloaded => "overloaded",
            Self::Timeout => "timeout",
            Self::MethodNotAllowed => "method_not_allowed",
        }
    }

    pub const fn is_transient(self) -> bool {
        match self {
            Self::StateUpdating => true,
            #[cfg(any(feature = "chain", test))]
            Self::Overloaded => true,
            _ => false,
        }
    }
}
