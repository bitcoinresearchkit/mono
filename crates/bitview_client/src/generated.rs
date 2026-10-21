// Auto-generated Bitview Rust client
// Do not edit manually

#![allow(non_snake_case)]
#![allow(clippy::useless_format)]

use crate::{
    BitviewClientBase, BitviewClientOptions, BitviewError, DateSeriesEndpoint, FormatResponse,
    Node, Result, SeriesEndpoint, SeriesValue,
    address::validate_address_payload_for_type,
    address_hash_prefix, address_payload_hash_prefix,
    client_base::{parse_text, path_segment, query_pair},
};
pub use bitview_catalog::*;
pub use bitview_cohort::*;
pub use bitview_primitives::*;
pub use bitview_types::*;
pub use brk_types::*;
use std::sync::{Arc, OnceLock};

const _I1: &[Index] = &[
    Index::Minute10,
    Index::Minute30,
    Index::Hour1,
    Index::Hour4,
    Index::Hour12,
    Index::Day1,
    Index::Day3,
    Index::Week1,
    Index::Month1,
    Index::Month3,
    Index::Month6,
    Index::Year1,
    Index::Year10,
    Index::Halving,
    Index::Epoch,
    Index::Height,
];
const _I2: &[Index] = &[
    Index::Minute10,
    Index::Minute30,
    Index::Hour1,
    Index::Hour4,
    Index::Hour12,
    Index::Day1,
    Index::Day3,
    Index::Week1,
    Index::Month1,
    Index::Month3,
    Index::Month6,
    Index::Year1,
    Index::Year10,
    Index::Halving,
    Index::Epoch,
];
const _I3: &[Index] = &[Index::Minute10];
const _I4: &[Index] = &[Index::Minute30];
const _I5: &[Index] = &[Index::Hour1];
const _I6: &[Index] = &[Index::Hour4];
const _I7: &[Index] = &[Index::Hour12];
const _I8: &[Index] = &[Index::Day1];
const _I9: &[Index] = &[Index::Day3];
const _I10: &[Index] = &[Index::Week1];
const _I11: &[Index] = &[Index::Month1];
const _I12: &[Index] = &[Index::Month3];
const _I13: &[Index] = &[Index::Month6];
const _I14: &[Index] = &[Index::Year1];
const _I15: &[Index] = &[Index::Year10];
const _I16: &[Index] = &[Index::Halving];
const _I17: &[Index] = &[Index::Epoch];
const _I18: &[Index] = &[Index::Height];
const _I19: &[Index] = &[Index::TxIndex];
const _I20: &[Index] = &[Index::TxInIndex];
const _I21: &[Index] = &[Index::TxInIndex];
const _I22: &[Index] = &[Index::TxOutIndex];
const _I23: &[Index] = &[Index::TxOutIndex];
const _I24: &[Index] = &[Index::EmptyOutputIndex];
const _I25: &[Index] = &[Index::OpReturnIndex];
const _I26: &[Index] = &[Index::P2AAddressIndex];
const _I27: &[Index] = &[Index::P2MSOutputIndex];
const _I28: &[Index] = &[Index::P2PK33AddressIndex];
const _I29: &[Index] = &[Index::P2PK65AddressIndex];
const _I30: &[Index] = &[Index::P2PKHAddressIndex];
const _I31: &[Index] = &[Index::P2SHAddressIndex];
const _I32: &[Index] = &[Index::P2TRAddressIndex];
const _I33: &[Index] = &[Index::P2WPKHAddressIndex];
const _I34: &[Index] = &[Index::P2WSHAddressIndex];
const _I35: &[Index] = &[Index::UnknownOutputIndex];

accessor! { SeriesPattern1, SeriesPattern1By, _I1, T::Nullable {
    date minute10: Minute10 -> T::Nullable,
    date minute30: Minute30 -> T::Nullable,
    date hour1: Hour1 -> T::Nullable,
    date hour4: Hour4 -> T::Nullable,
    date hour12: Hour12 -> T::Nullable,
    date day1: Day1 -> T::Nullable,
    date day3: Day3 -> T::Nullable,
    date week1: Week1 -> T::Nullable,
    date month1: Month1 -> T::Nullable,
    date month3: Month3 -> T::Nullable,
    date month6: Month6 -> T::Nullable,
    date year1: Year1 -> T::Nullable,
    date year10: Year10 -> T::Nullable,
    plain halving: Halving -> T,
    plain epoch: Epoch -> T,
    plain height: Height -> T,
} }
accessor! { SeriesPattern2, SeriesPattern2By, _I2, T {
    date minute10: Minute10 -> T,
    date minute30: Minute30 -> T,
    date hour1: Hour1 -> T,
    date hour4: Hour4 -> T,
    date hour12: Hour12 -> T,
    date day1: Day1 -> T,
    date day3: Day3 -> T,
    date week1: Week1 -> T,
    date month1: Month1 -> T,
    date month3: Month3 -> T,
    date month6: Month6 -> T,
    date year1: Year1 -> T,
    date year10: Year10 -> T,
    plain halving: Halving -> T,
    plain epoch: Epoch -> T,
} }
accessor! { SeriesPattern3, SeriesPattern3By, _I3, T {
    date minute10: Minute10 -> T,
} }
accessor! { SeriesPattern4, SeriesPattern4By, _I4, T {
    date minute30: Minute30 -> T,
} }
accessor! { SeriesPattern5, SeriesPattern5By, _I5, T {
    date hour1: Hour1 -> T,
} }
accessor! { SeriesPattern6, SeriesPattern6By, _I6, T {
    date hour4: Hour4 -> T,
} }
accessor! { SeriesPattern7, SeriesPattern7By, _I7, T {
    date hour12: Hour12 -> T,
} }
accessor! { SeriesPattern8, SeriesPattern8By, _I8, T {
    date day1: Day1 -> T,
} }
accessor! { SeriesPattern9, SeriesPattern9By, _I9, T {
    date day3: Day3 -> T,
} }
accessor! { SeriesPattern10, SeriesPattern10By, _I10, T {
    date week1: Week1 -> T,
} }
accessor! { SeriesPattern11, SeriesPattern11By, _I11, T {
    date month1: Month1 -> T,
} }
accessor! { SeriesPattern12, SeriesPattern12By, _I12, T {
    date month3: Month3 -> T,
} }
accessor! { SeriesPattern13, SeriesPattern13By, _I13, T {
    date month6: Month6 -> T,
} }
accessor! { SeriesPattern14, SeriesPattern14By, _I14, T {
    date year1: Year1 -> T,
} }
accessor! { SeriesPattern15, SeriesPattern15By, _I15, T {
    date year10: Year10 -> T,
} }
accessor! { SeriesPattern16, SeriesPattern16By, _I16, T {
    plain halving: Halving -> T,
} }
accessor! { SeriesPattern17, SeriesPattern17By, _I17, T {
    plain epoch: Epoch -> T,
} }
accessor! { SeriesPattern18, SeriesPattern18By, _I18, T {
    plain height: Height -> T,
} }
accessor! { SeriesPattern19, SeriesPattern19By, _I19, T {
    plain tx_index: TxIndex -> T,
} }
accessor! { SeriesPattern20, SeriesPattern20By, _I20, T {
    plain txin_index: TxInIndex -> T,
} }
accessor! { SeriesPattern21, SeriesPattern21By, _I21, T::Nullable {
    plain txin_index: TxInIndex -> T::Nullable,
} }
accessor! { SeriesPattern22, SeriesPattern22By, _I22, T {
    plain txout_index: TxOutIndex -> T,
} }
accessor! { SeriesPattern23, SeriesPattern23By, _I23, T::Nullable {
    plain txout_index: TxOutIndex -> T::Nullable,
} }
accessor! { SeriesPattern24, SeriesPattern24By, _I24, T {
    plain empty_output_index: EmptyOutputIndex -> T,
} }
accessor! { SeriesPattern25, SeriesPattern25By, _I25, T {
    plain op_return_index: OpReturnIndex -> T,
} }
accessor! { SeriesPattern26, SeriesPattern26By, _I26, T {
    plain p2a_address_index: P2AAddressIndex -> T,
} }
accessor! { SeriesPattern27, SeriesPattern27By, _I27, T {
    plain p2ms_output_index: P2MSOutputIndex -> T,
} }
accessor! { SeriesPattern28, SeriesPattern28By, _I28, T {
    plain p2pk33_address_index: P2PK33AddressIndex -> T,
} }
accessor! { SeriesPattern29, SeriesPattern29By, _I29, T {
    plain p2pk65_address_index: P2PK65AddressIndex -> T,
} }
accessor! { SeriesPattern30, SeriesPattern30By, _I30, T {
    plain p2pkh_address_index: P2PKHAddressIndex -> T,
} }
accessor! { SeriesPattern31, SeriesPattern31By, _I31, T {
    plain p2sh_address_index: P2SHAddressIndex -> T,
} }
accessor! { SeriesPattern32, SeriesPattern32By, _I32, T {
    plain p2tr_address_index: P2TRAddressIndex -> T,
} }
accessor! { SeriesPattern33, SeriesPattern33By, _I33, T {
    plain p2wpkh_address_index: P2WPKHAddressIndex -> T,
} }
accessor! { SeriesPattern34, SeriesPattern34By, _I34, T {
    plain p2wsh_address_index: P2WSHAddressIndex -> T,
} }
accessor! { SeriesPattern35, SeriesPattern35By, _I35, T {
    plain unknown_output_index: UnknownOutputIndex -> T,
} }
impl SeriesValue for Address {
    type Nullable = Option<Address>;
}
impl SeriesValue for BlockHash {
    type Nullable = Option<BlockHash>;
}
impl SeriesValue for Boolean {
    type Nullable = Option<Boolean>;
}
impl SeriesValue for Bytes {
    type Nullable = Option<Bytes>;
}
impl SeriesValue for CoinbaseTag {
    type Nullable = Option<CoinbaseTag>;
}
impl SeriesValue for Count {
    type Nullable = Option<Count>;
}
impl SeriesValue for CountSigned {
    type Nullable = Option<CountSigned>;
}
impl SeriesValue for Date {
    type Nullable = Option<Date>;
}
impl SeriesValue for Day1 {
    type Nullable = Option<Day1>;
}
impl SeriesValue for Day3 {
    type Nullable = Option<Day3>;
}
impl SeriesValue for EmptyOutputIndex {
    type Nullable = Option<EmptyOutputIndex>;
}
impl SeriesValue for Epoch {
    type Nullable = Option<Epoch>;
}
impl SeriesValue for Halving {
    type Nullable = Option<Halving>;
}
impl SeriesValue for Height {
    type Nullable = Option<Height>;
}
impl SeriesValue for Hour1 {
    type Nullable = Option<Hour1>;
}
impl SeriesValue for Hour12 {
    type Nullable = Option<Hour12>;
}
impl SeriesValue for Hour4 {
    type Nullable = Option<Hour4>;
}
impl SeriesValue for Minute10 {
    type Nullable = Option<Minute10>;
}
impl SeriesValue for Minute30 {
    type Nullable = Option<Minute30>;
}
impl SeriesValue for Month1 {
    type Nullable = Option<Month1>;
}
impl SeriesValue for Month3 {
    type Nullable = Option<Month3>;
}
impl SeriesValue for Month6 {
    type Nullable = Option<Month6>;
}
impl SeriesValue for OHLCDollars {
    type Nullable = Option<OHLCDollars>;
}
impl SeriesValue for OpReturnIndex {
    type Nullable = Option<OpReturnIndex>;
}
impl SeriesValue for OpReturnKind {
    type Nullable = Option<OpReturnKind>;
}
impl SeriesValue for OutPoint {
    type Nullable = Option<OutPoint>;
}
impl SeriesValue for OutputType {
    type Nullable = Option<OutputType>;
}
impl SeriesValue for P2AAddressIndex {
    type Nullable = Option<P2AAddressIndex>;
}
impl SeriesValue for P2ABytes {
    type Nullable = Option<P2ABytes>;
}
impl SeriesValue for P2MSOutputIndex {
    type Nullable = Option<P2MSOutputIndex>;
}
impl SeriesValue for P2PK33AddressIndex {
    type Nullable = Option<P2PK33AddressIndex>;
}
impl SeriesValue for P2PK33Bytes {
    type Nullable = Option<P2PK33Bytes>;
}
impl SeriesValue for P2PK65AddressIndex {
    type Nullable = Option<P2PK65AddressIndex>;
}
impl SeriesValue for P2PK65Bytes {
    type Nullable = Option<P2PK65Bytes>;
}
impl SeriesValue for P2PKHAddressIndex {
    type Nullable = Option<P2PKHAddressIndex>;
}
impl SeriesValue for P2PKHBytes {
    type Nullable = Option<P2PKHBytes>;
}
impl SeriesValue for P2SHAddressIndex {
    type Nullable = Option<P2SHAddressIndex>;
}
impl SeriesValue for P2SHBytes {
    type Nullable = Option<P2SHBytes>;
}
impl SeriesValue for P2TRAddressIndex {
    type Nullable = Option<P2TRAddressIndex>;
}
impl SeriesValue for P2TRBytes {
    type Nullable = Option<P2TRBytes>;
}
impl SeriesValue for P2WPKHAddressIndex {
    type Nullable = Option<P2WPKHAddressIndex>;
}
impl SeriesValue for P2WPKHBytes {
    type Nullable = Option<P2WPKHBytes>;
}
impl SeriesValue for P2WSHAddressIndex {
    type Nullable = Option<P2WSHAddressIndex>;
}
impl SeriesValue for P2WSHBytes {
    type Nullable = Option<P2WSHBytes>;
}
impl SeriesValue for PoolSlug {
    type Nullable = Option<PoolSlug>;
}
impl SeriesValue for RawLockTime {
    type Nullable = Option<RawLockTime>;
}
impl SeriesValue for Sats {
    type Nullable = Option<Sats>;
}
impl SeriesValue for Seconds {
    type Nullable = Option<Seconds>;
}
impl SeriesValue for SigOps {
    type Nullable = Option<SigOps>;
}
impl SeriesValue for Timestamp {
    type Nullable = Option<Timestamp>;
}
impl SeriesValue for TxInIndex {
    type Nullable = Option<TxInIndex>;
}
impl SeriesValue for TxIndex {
    type Nullable = Option<TxIndex>;
}
impl SeriesValue for TxOutIndex {
    type Nullable = Option<TxOutIndex>;
}
impl SeriesValue for TxVersion {
    type Nullable = Option<TxVersion>;
}
impl SeriesValue for Txid {
    type Nullable = Option<Txid>;
}
impl SeriesValue for TypeIndex {
    type Nullable = Option<TypeIndex>;
}
impl SeriesValue for UnknownOutputIndex {
    type Nullable = Option<UnknownOutputIndex>;
}
impl SeriesValue for VSize {
    type Nullable = Option<VSize>;
}
impl SeriesValue for Week1 {
    type Nullable = Option<Week1>;
}
impl SeriesValue for Weight {
    type Nullable = Option<Weight>;
}
impl SeriesValue for Year1 {
    type Nullable = Option<Year1>;
}
impl SeriesValue for Year10 {
    type Nullable = Option<Year10>;
}
/// The series tree's node types, one generic struct per shape.
pub mod tree {
    use super::*;

    shape! { Meter at "series().rarity_meter.cycle" {
        p0_1: SeriesPattern1<Option<Dollars>> = "*_p0_1",
        p0_5: SeriesPattern1<Option<Dollars>> = "*_p0_5",
        p1: SeriesPattern1<Option<Dollars>> = "*_p1",
        p2: SeriesPattern1<Option<Dollars>> = "*_p2",
        p5: SeriesPattern1<Option<Dollars>> = "*_p5",
        p10: SeriesPattern1<Option<Dollars>> = "*_p10",
        p20: SeriesPattern1<Option<Dollars>> = "*_p20",
        p30: SeriesPattern1<Option<Dollars>> = "*_p30",
        p40: SeriesPattern1<Option<Dollars>> = "*_p40",
        median: SeriesPattern1<Option<Dollars>> = "*_median",
        p60: SeriesPattern1<Option<Dollars>> = "*_p60",
        p70: SeriesPattern1<Option<Dollars>> = "*_p70",
        p80: SeriesPattern1<Option<Dollars>> = "*_p80",
        p90: SeriesPattern1<Option<Dollars>> = "*_p90",
        p95: SeriesPattern1<Option<Dollars>> = "*_p95",
        p98: SeriesPattern1<Option<Dollars>> = "*_p98",
        p99: SeriesPattern1<Option<Dollars>> = "*_p99",
        p99_5: SeriesPattern1<Option<Dollars>> = "*_p99_5",
        p99_9: SeriesPattern1<Option<Dollars>> = "*_p99_9",
        level: SeriesPattern1<Option<Score>> = "*_level",
        score: SeriesPattern1<Option<Score>> = "*_score",
    } }
    shape! { Extreme<A> at "series().rarity_meter.extremes.realized_loss_sum_24h" {
        block: SeriesPattern1<A> = "*",
        p0_1: SeriesPattern1<A> = "*_rarity_p0_1",
        p0_5: SeriesPattern1<A> = "*_rarity_p0_5",
        p1: SeriesPattern1<A> = "*_rarity_p1",
        p2: SeriesPattern1<A> = "*_rarity_p2",
        p5: SeriesPattern1<A> = "*_rarity_p5",
        p10: SeriesPattern1<A> = "*_rarity_p10",
        p20: SeriesPattern1<A> = "*_rarity_p20",
        p30: SeriesPattern1<A> = "*_rarity_p30",
        p40: SeriesPattern1<A> = "*_rarity_p40",
        median: SeriesPattern1<A> = "*_rarity_median",
        p60: SeriesPattern1<A> = "*_rarity_p60",
        p70: SeriesPattern1<A> = "*_rarity_p70",
        p80: SeriesPattern1<A> = "*_rarity_p80",
        p90: SeriesPattern1<A> = "*_rarity_p90",
        p95: SeriesPattern1<A> = "*_rarity_p95",
        p98: SeriesPattern1<A> = "*_rarity_p98",
        p99: SeriesPattern1<A> = "*_rarity_p99",
        p99_5: SeriesPattern1<A> = "*_rarity_p99_5",
        p99_9: SeriesPattern1<A> = "*_rarity_p99_9",
    } }
    shape! { Extremes at "series().rarity_meter.extremes" {
        supply_in_loss: Extreme<Option<Bitcoin>> = "supply_in_loss",
        realized_profit_sum_24h: Extreme<Option<Dollars>> = "realized_profit_sum_*",
        realized_loss_sum_24h: Extreme<Option<Dollars>> = "realized_loss_sum_*",
        realized_peak_regret_sum_24h: Extreme<Option<Dollars>> = "realized_peak_regret_sum_*",
        sell_side_risk_ratio_24h: Extreme<Option<Ratio>> = "sell_side_risk_ratio_*",
    } }
    shape! { MobileMedianCapitalizedPriceMedian at "series().rarity_meter.components.mobile.median_capitalized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "mobile_median_capitalized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "mobile_median_capitalized_price_*",
    } }
    shape! { MobileMedianRealizedPriceMedian at "series().rarity_meter.components.mobile.median_realized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "mobile_median_realized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "mobile_median_realized_price_*",
    } }
    shape! { MobileRealizedPriceMedian at "series().rarity_meter.components.mobile.realized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "mobile_realized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "mobile_realized_price_*",
    } }
    shape! { AwakeRealizedPriceMedian at "series().rarity_meter.components.awake.realized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "awake_realized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "awake_realized_price_*",
    } }
    shape! { Under6mCapitalizedPriceMedian at "series().rarity_meter.components.under_6m.capitalized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "age_under_6m_capitalized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "age_under_6m_capitalized_price_*",
    } }
    shape! { Under6mRealizedPriceMedian at "series().rarity_meter.components.under_6m.realized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "age_under_6m_realized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "age_under_6m_realized_price_*",
    } }
    shape! { SthMedianCapitalizedPriceMedian at "series().rarity_meter.components.sth.median_capitalized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "sth_median_capitalized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "sth_median_capitalized_price_*",
    } }
    shape! { SthMedianRealizedPriceMedian at "series().rarity_meter.components.sth.median_realized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "sth_median_realized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "sth_median_realized_price_*",
    } }
    shape! { SthCapitalizedPriceMedian at "series().rarity_meter.components.sth.capitalized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "sth_capitalized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "sth_capitalized_price_*",
    } }
    shape! { SthRealizedPriceMedian at "series().rarity_meter.components.sth.realized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "sth_realized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "sth_realized_price_*",
    } }
    shape! { TrueMarketMeanMedian at "series().rarity_meter.components.all.true_market_mean.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "true_market_mean_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "true_market_mean_*",
    } }
    shape! { VaultedPriceMedian at "series().rarity_meter.components.all.vaulted_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "vaulted_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "vaulted_price_*",
    } }
    shape! { AllMedianCapitalizedPriceMedian at "series().rarity_meter.components.all.median_capitalized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "median_capitalized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "median_capitalized_price_*",
    } }
    shape! { AllMedianRealizedPriceMedian at "series().rarity_meter.components.all.median_realized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "median_realized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "median_realized_price_*",
    } }
    shape! { MedianComponent<A> at "series().rarity_meter.components.all.median_capitalized_price" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Multiple>> = "*_ratio",
        p0_1: A = "rarity_p0_1",
        p0_5: A = "rarity_p0_5",
        p1: A = "rarity_p1",
        p2: A = "rarity_p2",
        p5: A = "rarity_p5",
        p10: A = "rarity_p10",
        p20: A = "rarity_p20",
        p30: A = "rarity_p30",
        p40: A = "rarity_p40",
        median: A = "rarity_median",
        p60: A = "rarity_p60",
        p70: A = "rarity_p70",
        p80: A = "rarity_p80",
        p90: A = "rarity_p90",
        p95: A = "rarity_p95",
        p98: A = "rarity_p98",
        p99: A = "rarity_p99",
        p99_5: A = "rarity_p99_5",
        p99_9: A = "rarity_p99_9",
    } }
    shape! { AllCapitalizedPriceMedian at "series().rarity_meter.components.all.capitalized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "capitalized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "capitalized_price_*",
    } }
    shape! { AllRealizedPriceMedian at "series().rarity_meter.components.all.realized_price.median" {
        ratio: SeriesPattern1<Option<Multiple>> = "realized_price_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "realized_price_*",
    } }
    shape! { RarityPercentiles<A> at "series().rarity_meter.components.all.capitalized_price" {
        p0_1: A = "*_p0_1",
        p0_5: A = "*_p0_5",
        p1: A = "*_p1",
        p2: A = "*_p2",
        p5: A = "*_p5",
        p10: A = "*_p10",
        p20: A = "*_p20",
        p30: A = "*_p30",
        p40: A = "*_p40",
        median: A = "*_median",
        p60: A = "*_p60",
        p70: A = "*_p70",
        p80: A = "*_p80",
        p90: A = "*_p90",
        p95: A = "*_p95",
        p98: A = "*_p98",
        p99: A = "*_p99",
        p99_5: A = "*_p99_5",
        p99_9: A = "*_p99_9",
    } }
    shape! { MobileComponents at "series().rarity_meter.components.mobile" {
        realized_price: RarityPercentiles<MobileRealizedPriceMedian> = "rarity",
        median_realized_price: MedianComponent<MobileMedianRealizedPriceMedian> = "*_realized_price",
        median_capitalized_price: MedianComponent<MobileMedianCapitalizedPriceMedian> = "*_capitalized_price",
    } }
    shape! { AwakeComponents at "series().rarity_meter.components.awake" {
        realized_price: RarityPercentiles<AwakeRealizedPriceMedian> = "*",
    } }
    shape! { MeanComponents at "series().rarity_meter.components.under_6m" {
        realized_price: RarityPercentiles<Under6mRealizedPriceMedian> = "*",
        capitalized_price: RarityPercentiles<Under6mCapitalizedPriceMedian> = "*",
    } }
    shape! { CohortComponents at "series().rarity_meter.components.sth" {
        realized_price: RarityPercentiles<SthRealizedPriceMedian> = "*",
        capitalized_price: RarityPercentiles<SthCapitalizedPriceMedian> = "*",
        median_realized_price: MedianComponent<SthMedianRealizedPriceMedian> = "sth_median_realized_price",
        median_capitalized_price: MedianComponent<SthMedianCapitalizedPriceMedian> = "sth_median_capitalized_price",
    } }
    shape! { AllComponents at "series().rarity_meter.components.all" {
        realized_price: RarityPercentiles<AllRealizedPriceMedian> = "*",
        capitalized_price: RarityPercentiles<AllCapitalizedPriceMedian> = "*",
        median_realized_price: MedianComponent<AllMedianRealizedPriceMedian> = "median_realized_price",
        median_capitalized_price: MedianComponent<AllMedianCapitalizedPriceMedian> = "median_capitalized_price",
        vaulted_price: RarityPercentiles<VaultedPriceMedian> = "*",
        true_market_mean: RarityPercentiles<TrueMarketMeanMedian> = "*",
    } }
    shape! { Components at "series().rarity_meter.components" {
        all: AllComponents = "*",
        sth: CohortComponents = "*",
        under_6m: MeanComponents = "*",
        awake: AwakeComponents = "*",
        mobile: MobileComponents = "mobile_median",
    } }
    shape! { RarityMeter at "series().rarity_meter" {
        components: Components = "*",
        extremes: Extremes = "24h",
        full: Meter = "*_meter",
        local: Meter = "local_*_meter",
        cycle: Meter = "cycle_*_meter",
    } }
    shape! { FloorCapital at "series().bedrock.awake.floor_capital" {
        p95_to_p98: SeriesPattern1<Option<Dollars>> = "*_p95_to_p98",
        p98_to_p99: SeriesPattern1<Option<Dollars>> = "*_p98_to_p99",
        p99_to_p99_5: SeriesPattern1<Option<Dollars>> = "*_p99_to_p99_5",
        p99_5_to_p99_9: SeriesPattern1<Option<Dollars>> = "*_p99_5_to_p99_9",
        below_p99_9: SeriesPattern1<Option<Dollars>> = "*_below_p99_9",
    } }
    shape! { Levels at "series().bedrock.awake.cost_basis_above_floor" {
        p10: SeriesPattern1<Option<Dollars>> = "*_p10",
        p20: SeriesPattern1<Option<Dollars>> = "*_p20",
        p30: SeriesPattern1<Option<Dollars>> = "*_p30",
        p40: SeriesPattern1<Option<Dollars>> = "*_p40",
        median: SeriesPattern1<Option<Dollars>> = "*_median",
        p60: SeriesPattern1<Option<Dollars>> = "*_p60",
        p70: SeriesPattern1<Option<Dollars>> = "*_p70",
        p80: SeriesPattern1<Option<Dollars>> = "*_p80",
        p90: SeriesPattern1<Option<Dollars>> = "*_p90",
    } }
    shape! { Percentiles<A> at "series().bedrock.awake.floor" {
        p95: SeriesPattern1<A> = "*_p95",
        p98: SeriesPattern1<A> = "*_p98",
        p99: SeriesPattern1<A> = "*_p99",
        p99_5: SeriesPattern1<A> = "*_p99_5",
        p99_9: SeriesPattern1<A> = "*_p99_9",
    } }
    shape! { CointimePriceWithRatio at "series().cointime.prices.true_market_mean" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Multiple>> = "*_ratio",
        aviv: SeriesPattern1<Option<Multiple>> = "aviv",
        aviv_nupl: SeriesPattern1<Option<Ratio>> = "aviv_nupl",
    } }
    shape! { CointimePriceWithMvrv at "series().cointime.prices.active" {
        block: SeriesPattern1<Option<Dollars>> = "*_price",
        ratio: SeriesPattern1<Option<Multiple>> = "*_price_ratio",
        mvrv: SeriesPattern1<Option<Multiple>> = "*_mvrv",
    } }
    shape! { Prices at "series().cointime.prices" {
        vaulted: CointimePriceWithMvrv = "*",
        active: CointimePriceWithMvrv = "active",
        true_market_mean: CointimePriceWithRatio = "true_market_mean",
        cointime: CointimePriceWithMvrv = "cointime",
    } }
    shape! { Caps at "series().cointime.caps" {
        thermo: SeriesPattern1<Option<Dollars>> = "thermo_*",
        investor: SeriesPattern1<Option<Dollars>> = "investor_*",
        active: SeriesPattern1<Option<Dollars>> = "active_*",
        vaulted: SeriesPattern1<Option<Dollars>> = "vaulted_*",
        cointime: SeriesPattern1<Option<Dollars>> = "cointime_*",
        investorness: SeriesPattern1<Option<Percent>> = "investorness",
        producerness: SeriesPattern1<Option<Percent>> = "producerness",
    } }
    shape! { MobileCapital at "series().coinflow.cohorts.all.mobile.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
    } }
    shape! { InLoss at "series().coinflow.cohorts.all.mobile.supply.in_loss" {
        share: SeriesPattern1<Option<Percent>> = "*",
    } }
    shape! { Velocity at "series().supply.velocity" {
        btc: SeriesPattern1<Option<Ratio>> = "*_btc",
        usd: SeriesPattern1<Option<Ratio>> = "*_usd",
    } }
    shape! { Adjusted at "series().cointime.adjusted" {
        inflation_rate: SeriesPattern1<Option<Percent>> = "*_inflation_rate",
        velocity: Velocity = "*_velocity",
    } }
    shape! { BandCapital at "series().profitability.all.loss_0_to_10_percent.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        share: SeriesPattern1<Option<Percent>> = "*_capital_share",
    } }
    shape! { Density at "series().holders.all.cost_basis.capital_density" {
        total: SeriesPattern1<Option<Percent>> = "*",
        in_profit: SeriesPattern1<Option<Percent>> = "*_in_profit",
        in_loss: SeriesPattern1<Option<Percent>> = "*_in_loss",
    } }
    shape! { PriceWithMvrv at "series().holders.all.cost_basis.by_coin.avg" {
        block: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_avg",
        ratio: SeriesPattern1<Option<Multiple>> = "*_cost_basis_by_coin_avg_ratio",
        mvrv: SeriesPattern1<Option<Multiple>> = "*_mvrv",
    } }
    shape! { CostBasisByCoin at "series().holders.all.cost_basis.by_coin" {
        avg: PriceWithMvrv = "*",
        realized_price: SeriesPattern1<Option<Dollars>> = "*_realized_price",
        p5: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p5",
        p10: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p10",
        p15: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p15",
        p20: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p20",
        p25: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p25",
        p30: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p30",
        p35: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p35",
        p40: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p40",
        p45: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p45",
        median: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_median",
        p55: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p55",
        p60: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p60",
        p65: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p65",
        p70: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p70",
        p75: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p75",
        p80: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p80",
        p85: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p85",
        p90: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p90",
        p95: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_p95",
        median_realized_price: SeriesPattern1<Option<Dollars>> = "*_median_realized_price",
    } }
    shape! { Unrealized at "series().holders.all.unrealized" {
        profit: SeriesPattern1<Option<Dollars>> = "*_unrealized_profit",
        loss: SeriesPattern1<Option<Dollars>> = "*_unrealized_loss",
        net_pnl: SeriesPattern1<Option<Dollars>> = "*_net_unrealized_pnl",
        gross_pnl: SeriesPattern1<Option<Dollars>> = "*_gross_unrealized_pnl",
        nupl: SeriesPattern1<Option<Ratio>> = "*_nupl",
    } }
    shape! { Part at "series().holders.all.capital.in_loss" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { SpotValue at "series().holders.all.supply.total" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "market_cap",
    } }
    shape! { FundedTotal at "series().addresses.exposed.count" {
        funded: SeriesPattern1<Count> = "*_funded_exposed_address_count",
        total: SeriesPattern1<Count> = "*_total_exposed_address_count",
    } }
    shape! { RespentCount at "series().addresses.respent.count" {
        funded: SeriesPattern1<Count> = "*_funded_respent_address_count",
        total: SeriesPattern1<Count> = "*_total_respent_address_count",
    } }
    shape! { PredicateSupply at "series().addresses.exposed.supply" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "*_usd",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { Exposed at "series().addresses.exposed" {
        count: FundedTotal = "*",
        supply: PredicateSupply = "*_exposed_address_supply",
    } }
    shape! { ReusedCount at "series().addresses.reused.count" {
        funded: SeriesPattern1<Count> = "*_funded_reused_address_count",
        total: SeriesPattern1<Count> = "*_total_reused_address_count",
    } }
    shape! { ByCoin at "series().entry.rookie.cost_basis.by_coin" {
        avg: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_coin_avg",
        realized_price: SeriesPattern1<Option<Dollars>> = "*_realized_price",
    } }
    shape! { CohortCostBasis at "series().entry.rookie.cost_basis" {
        by_coin: ByCoin = "*",
    } }
    shape! { CreationUnrealized at "series().entry.rookie.unrealized" {
        profit: SeriesPattern1<Option<Dollars>> = "*_unrealized_profit",
        loss: SeriesPattern1<Option<Dollars>> = "*_unrealized_loss",
        net_pnl: SeriesPattern1<Option<Dollars>> = "*_net_unrealized_pnl",
    } }
    shape! { AddressesWindows at "series().addresses.capital.delta.percent" {
        _24h: SeriesPattern1<Option<Percent>> = "*_24h_percent",
        _1w: SeriesPattern1<Option<Percent>> = "*_1w_percent",
        _1m: SeriesPattern1<Option<Percent>> = "*_1m_percent",
        _1y: SeriesPattern1<Option<Percent>> = "*_1y_percent",
    } }
    shape! { MarketPriceWithRatio at "series().market.sma._200d" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Multiple>> = "*_ratio",
        mayer_multiple: SeriesPattern1<Option<Multiple>> = "mayer_multiple",
    } }
    shape! { PriceWithRatio at "series().market.ema._13d" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Multiple>> = "*_ratio",
    } }
    shape! { CostBasisByDollar at "series().holders.all.cost_basis.by_dollar" {
        avg: PriceWithRatio = "*_cost_basis_by_dollar_avg",
        capitalized_price: SeriesPattern1<Option<Dollars>> = "*_capitalized_price",
        p5: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p5",
        p10: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p10",
        p15: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p15",
        p20: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p20",
        p25: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p25",
        p30: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p30",
        p35: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p35",
        p40: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p40",
        p45: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p45",
        median: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_median",
        p55: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p55",
        p60: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p60",
        p65: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p65",
        p70: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p70",
        p75: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p75",
        p80: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p80",
        p85: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p85",
        p90: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p90",
        p95: SeriesPattern1<Option<Dollars>> = "*_cost_basis_by_dollar_p95",
        median_capitalized_price: SeriesPattern1<Option<Dollars>> = "*_median_capitalized_price",
    } }
    shape! { CostBasis at "series().coinflow.cohorts.all.mobile.cost_basis" {
        by_coin: CostBasisByCoin = "*",
        by_dollar: CostBasisByDollar = "*",
        supply_density: Density = "*_supply_density",
        capital_density: Density = "*_capital_density",
    } }
    shape! { HoldersCostBasis at "series().holders.all.cost_basis" {
        by_coin: CostBasisByCoin = "*",
        by_dollar: CostBasisByDollar = "*",
        min: SeriesPattern1<Option<Dollars>> = "*_cost_basis_min",
        max: SeriesPattern1<Option<Dollars>> = "*_cost_basis_max",
        supply_density: Density = "*_supply_density",
        capital_density: Density = "*_capital_density",
    } }
    shape! { Ema at "series().market.ema" {
        _1w: PriceWithRatio = "*_1w",
        _8d: PriceWithRatio = "*_8d",
        _13d: PriceWithRatio = "*_13d",
        _21d: PriceWithRatio = "*_21d",
        _1m: PriceWithRatio = "*_1m",
        _34d: PriceWithRatio = "*_34d",
        _55d: PriceWithRatio = "*_55d",
        _89d: PriceWithRatio = "*_89d",
        _144d: PriceWithRatio = "*_144d",
        _200d: PriceWithRatio = "*_200d",
        _1y: PriceWithRatio = "*_1y",
        _2y: PriceWithRatio = "*_2y",
        _200w: PriceWithRatio = "*_200w",
        _4y: PriceWithRatio = "*_4y",
    } }
    shape! { Sma at "series().market.sma" {
        _1w: PriceWithRatio = "*_1w",
        _8d: PriceWithRatio = "*_8d",
        _13d: PriceWithRatio = "*_13d",
        _21d: PriceWithRatio = "*_21d",
        _1m: PriceWithRatio = "*_1m",
        _34d: PriceWithRatio = "*_34d",
        _50d: PriceWithRatio = "*_50d",
        _55d: PriceWithRatio = "*_55d",
        _89d: PriceWithRatio = "*_89d",
        _144d: PriceWithRatio = "*_144d",
        _200d: MarketPriceWithRatio = "*_200d",
        _1y: PriceWithRatio = "*_1y",
        _2y: PriceWithRatio = "*_2y",
        _200w: PriceWithRatio = "*_200w",
        _4y: PriceWithRatio = "*_4y",
    } }
    shape! { PriceMinMax at "series().market.range.max" {
        _1w: SeriesPattern1<Option<Dollars>> = "*_1w",
        _1m: SeriesPattern1<Option<Dollars>> = "*_1m",
        _1y: SeriesPattern1<Option<Dollars>> = "*_1y",
    } }
    shape! { MarketRange at "series().market.range" {
        min: PriceMinMax = "*_min",
        max: PriceMinMax = "*_max",
        true_range: SeriesPattern1<Option<Dollars>> = "*_true_range",
    } }
    shape! { ByLookbackPeriod at "series().market.returns" {
        _24h: SeriesPattern1<Option<Percent>> = "*_24h",
        _1w: SeriesPattern1<Option<Percent>> = "*_1w",
        _1m: SeriesPattern1<Option<Percent>> = "*_1m",
        _3m: SeriesPattern1<Option<Percent>> = "*_3m",
        _6m: SeriesPattern1<Option<Percent>> = "*_6m",
        _1y: SeriesPattern1<Option<Percent>> = "*_1y",
        _2y: SeriesPattern1<Option<Percent>> = "*_2y",
        _3y: SeriesPattern1<Option<Percent>> = "*_3y",
        _4y: SeriesPattern1<Option<Percent>> = "*_4y",
        _5y: SeriesPattern1<Option<Percent>> = "*_5y",
        _6y: SeriesPattern1<Option<Percent>> = "*_6y",
        _8y: SeriesPattern1<Option<Percent>> = "*_8y",
        _10y: SeriesPattern1<Option<Percent>> = "*_10y",
    } }
    shape! { Ath at "series().market.ath" {
        high: SeriesPattern1<Option<Dollars>> = "*_ath",
        drawdown: SeriesPattern1<Option<Percent>> = "*_drawdown",
        days_since: SeriesPattern1<Option<Days>> = "days_since_*_ath",
        days_between_max: SeriesPattern1<Option<Days>> = "days_between_*_ath_max",
    } }
    shape! { Market at "series().market" {
        ath: Ath = "*",
        returns: ByLookbackPeriod = "*_return",
        range: MarketRange = "*",
        sma: Sma = "*_sma",
        ema: Ema = "*_ema",
    } }
    shape! { UtxoSetSeries at "series().utxo_set" {
        supply: SeriesPattern1<Option<Bitcoin>> = "*",
        count: SeriesPattern1<Count> = "utxo_count",
    } }
    shape! { HashPriceValue<A> at "series().mining.hashrate.price" {
        block: SeriesPattern1<A> = "*",
        atl: SeriesPattern1<A> = "*_atl",
        rebound: SeriesPattern1<Option<Percent>> = "*_rebound",
    } }
    shape! { HashRateSma at "series().mining.hashrate.rate.sma" {
        _1w: SeriesPattern1<Option<Hashrate>> = "*_1w",
        _1m: SeriesPattern1<Option<Hashrate>> = "*_1m",
        _2m: SeriesPattern1<Option<Hashrate>> = "*_2m",
        _1y: SeriesPattern1<Option<Hashrate>> = "*_1y",
    } }
    shape! { Rate at "series().mining.hashrate.rate" {
        block: SeriesPattern1<Option<Hashrate>> = "*",
        sma: HashRateSma = "*_sma",
        ath: SeriesPattern1<Option<Hashrate>> = "*_ath",
        drawdown: SeriesPattern1<Option<Percent>> = "*_drawdown",
    } }
    shape! { MiningHashrate at "series().mining.hashrate" {
        rate: Rate = "hashrate",
        price: HashPriceValue<Option<HashPrice>> = "*_price",
        value: HashPriceValue<Option<HashValue>> = "*_value",
    } }
    shape! { PerTxDistribution<A> at "series().transactions.vsize" {
        tx: SeriesPattern19<A> = "*",
        min: SeriesPattern1<A> = "*_min",
        max: SeriesPattern1<A> = "*_max",
        p10: SeriesPattern1<A> = "*_p10",
        p25: SeriesPattern1<A> = "*_p25",
        median: SeriesPattern1<A> = "*_median",
        p75: SeriesPattern1<A> = "*_p75",
        p90: SeriesPattern1<A> = "*_p90",
    } }
    shape! { Value at "series().addresses.avg_balance" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "*_usd",
    } }
    shape! { FloorSupply at "series().bedrock.awake.floor_supply" {
        p95_to_p98: Value = "*_p95_to_p98",
        p98_to_p99: Value = "*_p98_to_p99",
        p99_to_p99_5: Value = "*_p99_to_p99_5",
        p99_5_to_p99_9: Value = "*_p99_5_to_p99_9",
        below_p99_9: Value = "*_below_p99_9",
    } }
    shape! { Mode at "series().bedrock.awake" {
        supply_in_loss_share_threshold: Percentiles<Option<Percent>> = "*_supply_in_loss_share_threshold",
        floor: Percentiles<Option<Dollars>> = "*_floor",
        cost_basis_above_floor: Levels = "*_cost_basis_above_floor",
        floor_supply: FloorSupply = "*_floor_supply",
        floor_capital: FloorCapital = "*_floor_capital",
    } }
    shape! { Bedrock at "series().bedrock" {
        awake: Mode = "*_awake",
        mobile: Mode = "*_mobile",
        unweighted: Mode = "*_unweighted",
    } }
    shape! { CointimeSupply at "series().cointime.supply" {
        vaulted: Value = "vaulted_*",
        hodled_or_lost: Value = "hodled_or_lost_*",
        active: Value = "active_*",
    } }
    shape! { MobileSupply at "series().coinflow.cohorts.all.mobile.supply" {
        total: Value = "*",
        in_loss: InLoss = "*_in_loss_share",
    } }
    shape! { Awake at "series().coinflow.cohorts.all.mobile" {
        supply: MobileSupply = "*_supply",
        capital: MobileCapital = "*",
        cost_basis: CostBasis = "*",
    } }
    shape! { Side at "series().coinflow.age_ranges._10y_to_12y.immobile" {
        supply: Value = "*",
    } }
    shape! { CoinflowCohort at "series().coinflow.cohorts.all" {
        mobile: Awake = "*_mobile",
        immobile: Side = "*_immobile_supply",
    } }
    shape! { CoinflowRange at "series().coinflow.age_ranges._10y_to_12y" {
        spending_rate: SeriesPattern1<Option<PerDay>> = "*_spending_rate",
        spending_exposure: SeriesPattern1<Option<Float>> = "*_spending_exposure",
        mobility: SeriesPattern1<Option<Ratio>> = "*_mobility",
        mobile: Side = "*_mobile_supply",
        immobile: Side = "*_immobile_supply",
    } }
    shape! { CointimeCohort at "series().cointime.cohorts.all" {
        awake: Awake = "*_awake",
        dormant: Side = "*_dormant_supply",
    } }
    shape! { BandSupply at "series().profitability.all.loss_0_to_10_percent.supply" {
        total: Value = "*",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { Band at "series().profitability.all.loss_0_to_10_percent" {
        supply: BandSupply = "*_supply",
        capital: BandCapital = "*",
    } }
    shape! { ProfitabilityRange at "series().profitability.all" {
        profit_over_1000_percent: Band = "*_profit_over_1000_percent",
        profit_500_to_1000_percent: Band = "*_profit_500_to_1000_percent",
        profit_300_to_500_percent: Band = "*_profit_300_to_500_percent",
        profit_200_to_300_percent: Band = "*_profit_200_to_300_percent",
        profit_100_to_200_percent: Band = "*_profit_100_to_200_percent",
        profit_90_to_100_percent: Band = "*_profit_90_to_100_percent",
        profit_80_to_90_percent: Band = "*_profit_80_to_90_percent",
        profit_70_to_80_percent: Band = "*_profit_70_to_80_percent",
        profit_60_to_70_percent: Band = "*_profit_60_to_70_percent",
        profit_50_to_60_percent: Band = "*_profit_50_to_60_percent",
        profit_40_to_50_percent: Band = "*_profit_40_to_50_percent",
        profit_30_to_40_percent: Band = "*_profit_30_to_40_percent",
        profit_20_to_30_percent: Band = "*_profit_20_to_30_percent",
        profit_10_to_20_percent: Band = "*_profit_10_to_20_percent",
        profit_0_to_10_percent: Band = "*_profit_0_to_10_percent",
        loss_0_to_10_percent: Band = "*_loss_0_to_10_percent",
        loss_10_to_20_percent: Band = "*_loss_10_to_20_percent",
        loss_20_to_30_percent: Band = "*_loss_20_to_30_percent",
        loss_30_to_40_percent: Band = "*_loss_30_to_40_percent",
        loss_40_to_50_percent: Band = "*_loss_40_to_50_percent",
        loss_50_to_60_percent: Band = "*_loss_50_to_60_percent",
        loss_60_to_70_percent: Band = "*_loss_60_to_70_percent",
        loss_70_to_80_percent: Band = "*_loss_70_to_80_percent",
        loss_80_to_90_percent: Band = "*_loss_80_to_90_percent",
        loss_90_to_100_percent: Band = "*_loss_90_to_100_percent",
    } }
    shape! { Profitability at "series().profitability" {
        all: ProfitabilityRange = "",
        sth: ProfitabilityRange = "*",
        lth: ProfitabilityRange = "lth",
    } }
    shape! { OpReturnWindows at "series().op_return.fees.sum" {
        _24h: Value = "*_24h",
        _1w: Value = "*_1w",
        _1m: Value = "*_1m",
        _1y: Value = "*_1y",
    } }
    shape! { PoolFees at "series().pools._175btc.fees" {
        avg: OpReturnWindows = "*",
    } }
    shape! { Coinbase at "series().pools._175btc.coinbase" {
        cumulative: Value = "*_cumulative",
        sum: OpReturnWindows = "*_sum",
    } }
    shape! { OpReturnValue at "series().op_return.fees.block" {
        btc: SeriesPattern18<Option<Bitcoin>> = "*",
        usd: SeriesPattern18<Option<Dollars>> = "*_usd",
    } }
    shape! { ValueFlow at "series().outputs.op_return_value" {
        block: OpReturnValue = "*",
        cumulative: Value = "*_cumulative",
        sum: OpReturnWindows = "*_sum",
    } }
    shape! { Activity at "series().addresses.balances._100_to_1k_btc.activity" {
        transfer_volume: ValueFlow = "*",
    } }
    shape! { HoldersValueFlow at "series().holders.all.activity.transfer_volume" {
        block: OpReturnValue = "*",
        cumulative: Value = "*_cumulative",
        sum: OpReturnWindows = "*_sum",
        in_profit: ValueFlow = "*_in_profit",
        in_loss: ValueFlow = "*_in_loss",
    } }
    shape! { Share at "series().op_return.tx_share" {
        cumulative: SeriesPattern1<Option<Percent>> = "*_cumulative",
        _24h: SeriesPattern1<Option<Percent>> = "*_24h",
        _1w: SeriesPattern1<Option<Percent>> = "*_1w",
        _1m: SeriesPattern1<Option<Percent>> = "*_1m",
        _1y: SeriesPattern1<Option<Percent>> = "*_1y",
    } }
    shape! { ValueFlowStats at "series().mining.rewards.fees" {
        block: OpReturnValue = "*",
        cumulative: Value = "*_cumulative",
        sum: OpReturnWindows = "*_sum",
        avg: OpReturnWindows = "*_avg",
        min: OpReturnWindows = "*_min",
        max: OpReturnWindows = "*_max",
        p10: OpReturnWindows = "*_p10",
        p25: OpReturnWindows = "*_p25",
        median: OpReturnWindows = "*_median",
        p75: OpReturnWindows = "*_p75",
        p90: OpReturnWindows = "*_p90",
        share: Share = "fee_share",
    } }
    shape! { MiningValueFlow at "series().mining.rewards.subsidy" {
        block: OpReturnValue = "*",
        cumulative: Value = "*_cumulative",
        sum: OpReturnWindows = "*_sum",
        share: Share = "*_share",
    } }
    shape! { Rewards at "series().mining.rewards" {
        coinbase: ValueFlow = "*",
        subsidy: MiningValueFlow = "subsidy",
        fees: ValueFlowStats = "fees",
        unclaimed: ValueFlow = "unclaimed_rewards",
    } }
    shape! { Mining at "series().mining" {
        rewards: Rewards = "*",
        hashrate: MiningHashrate = "hash",
    } }
    shape! { FeesSeries at "series().op_return.fees" {
        block: OpReturnValue = "*_fees",
        cumulative: Value = "*_fees_cumulative",
        sum: OpReturnWindows = "*_fees_sum",
        chain_share: Share = "*_fee_chain_share",
    } }
    shape! { BlocksHalving at "series().blocks.halving" {
        epoch: SeriesPattern1<Halving> = "*_epoch",
        blocks_to_halving: SeriesPattern1<Count> = "blocks_to_*",
        days_to_halving: SeriesPattern1<Option<Days>> = "days_to_*",
    } }
    shape! { BlocksDifficulty at "series().blocks.difficulty" {
        block: SeriesPattern1<Option<Difficulty>> = "*",
        hashrate: SeriesPattern1<Option<Hashrate>> = "*_hashrate",
        adjustment: SeriesPattern1<Option<Percent>> = "*_adjustment",
        epoch: SeriesPattern1<Epoch> = "*_epoch",
        blocks_to_retarget: SeriesPattern1<Count> = "blocks_to_retarget",
        days_to_retarget: SeriesPattern1<Option<Days>> = "days_to_retarget",
    } }
    shape! { Windows<A> at "series().inputs.per_second" {
        _24h: SeriesPattern1<A> = "*_24h",
        _1w: SeriesPattern1<A> = "*_1w",
        _1m: SeriesPattern1<A> = "*_1m",
        _1y: SeriesPattern1<A> = "*_1y",
    } }
    shape! { NewAddressCount at "series().addresses.new" {
        block: SeriesPattern18<Count> = "*",
        sum: Windows<Count> = "*_sum",
    } }
    shape! { Delta<A> at "series().addresses.capital.delta" {
        absolute: Windows<A> = "*",
        percent: AddressesWindows = "*",
    } }
    shape! { LthSupply at "series().holders.lth.supply" {
        total: Value = "*",
        share: SeriesPattern1<Option<Percent>> = "*_share",
        in_profit: PredicateSupply = "*_in_profit",
        in_loss: PredicateSupply = "*_in_loss",
        delta: Delta<Option<Bitcoin>> = "*_delta",
    } }
    shape! { Capital at "series().holders.all.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        share: SeriesPattern1<Option<Percent>> = "*_capital_share",
        in_profit: Part = "*_capital_in_profit",
        in_loss: Part = "*_capital_in_loss",
        delta: Delta<Option<Dollars>> = "*_capital_delta",
    } }
    shape! { AllSupply at "series().holders.all.supply" {
        total: SpotValue = "circulating_*",
        share: SeriesPattern1<Option<Percent>> = "*_share",
        in_profit: PredicateSupply = "*_in_profit",
        in_loss: PredicateSupply = "*_in_loss",
        delta: Delta<Option<Bitcoin>> = "*_delta",
    } }
    shape! { CapitalViews at "series().addresses.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        delta: Delta<Option<Dollars>> = "*_capital_delta",
        share: SeriesPattern1<Option<Percent>> = "*_capital_share",
    } }
    shape! { CohortSupply at "series().addresses.supply" {
        total: Value = "*",
        delta: Delta<Option<Bitcoin>> = "*_delta",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { CreationSupply at "series().entry.rookie.supply" {
        total: Value = "*",
        delta: Delta<Option<Bitcoin>> = "*_delta",
        share: SeriesPattern1<Option<Percent>> = "*_share",
        in_profit: Value = "*_in_profit",
        in_loss: Value = "*_in_loss",
    } }
    shape! { WithDelta<A, B> at "series().addresses.funded" {
        block: SeriesPattern1<A> = "*",
        delta: Delta<B> = "*_delta",
    } }
    shape! { Supply at "series().supply" {
        circulating: SeriesPattern1<Option<Bitcoin>> = "*",
        burned: ValueFlow = "burned",
        inflation_rate: SeriesPattern1<Option<Percent>> = "inflation_rate",
        velocity: Velocity = "velocity",
        market_cap: WithDelta<Option<Dollars>, Option<Dollars>> = "market_cap",
    } }
    shape! { BalanceUtxos at "series().addresses.balances._0_sats.utxos" {
        count: WithDelta<Count, CountSigned> = "*",
    } }
    shape! { ZeroBalance at "series().addresses.balances._0_sats" {
        funded: WithDelta<Count, CountSigned> = "*_address_count",
        utxos: BalanceUtxos = "*_utxo_count",
    } }
    shape! { CreationCapital at "series().entry.rookie.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        delta: Delta<Option<Dollars>> = "*_capital_delta",
        share: SeriesPattern1<Option<Percent>> = "*_capital_share",
        in_profit: SeriesPattern1<Option<Dollars>> = "*_capital_in_profit",
        in_loss: SeriesPattern1<Option<Dollars>> = "*_capital_in_loss",
    } }
    shape! { AgeCreationSupply at "series().age.ranges._10y_to_12y.supply" {
        total: Value = "*_supply",
        delta: Delta<Option<Bitcoin>> = "*_supply_delta",
        share: SeriesPattern1<Option<Percent>> = "*_supply_share",
        in_profit: Value = "*_supply_in_profit",
        in_loss: Value = "*_supply_in_loss",
        matured: ValueFlow = "*_matured_supply",
    } }
    shape! { BlockCount at "series().pools._175btc.block_count" {
        cumulative: SeriesPattern1<Count> = "*_cumulative",
        sum: Windows<Count> = "*_sum",
    } }
    shape! { Pool at "series().pools._175btc" {
        block_count: BlockCount = "*_block_count",
        block_share: Share = "*_block_share",
        coinbase: Coinbase = "*_coinbase",
        fees: PoolFees = "*_fees_avg",
        hashrate: Windows<Option<Hashrate>> = "*_hashrate",
    } }
    shape! { Pools at "series().pools" {
        block_pool: SeriesPattern18<PoolSlug> = "block_*",
        unknown: Pool = "*_unknown",
        blockfills: Pool = "*_blockfills",
        ultimuspool: Pool = "*_ultimuspool",
        terrapool: Pool = "*_terrapool",
        luxor: Pool = "*_luxor",
        _1thash: Pool = "*_1thash",
        btccom: Pool = "*_btccom",
        bitfarms: Pool = "*_bitfarms",
        huobipool: Pool = "*_huobipool",
        wayicn: Pool = "*_wayicn",
        canoepool: Pool = "*_canoepool",
        btctop: Pool = "*_btctop",
        bitcoincom: Pool = "*_bitcoincom",
        _175btc: Pool = "*_175btc",
        gbminers: Pool = "*_gbminers",
        axbt: Pool = "*_axbt",
        asicminer: Pool = "*_asicminer",
        bitminter: Pool = "*_bitminter",
        bitcoinrussia: Pool = "*_bitcoinrussia",
        btcserv: Pool = "*_btcserv",
        simplecoinus: Pool = "*_simplecoinus",
        btcguild: Pool = "*_btcguild",
        eligius: Pool = "*_eligius",
        ozcoin: Pool = "*_ozcoin",
        eclipsemc: Pool = "*_eclipsemc",
        maxbtc: Pool = "*_maxbtc",
        triplemining: Pool = "*_triplemining",
        coinlab: Pool = "*_coinlab",
        _50btc: Pool = "*_50btc",
        ghashio: Pool = "*_ghashio",
        stminingcorp: Pool = "*_stminingcorp",
        bitparking: Pool = "*_bitparking",
        mmpool: Pool = "*_mmpool",
        polmine: Pool = "*_polmine",
        kncminer: Pool = "*_kncminer",
        bitalo: Pool = "*_bitalo",
        f2pool: Pool = "*_f2pool",
        hhtt: Pool = "*_hhtt",
        megabigpower: Pool = "*_megabigpower",
        mtred: Pool = "*_mtred",
        nmcbit: Pool = "*_nmcbit",
        yourbtcnet: Pool = "*_yourbtcnet",
        givemecoins: Pool = "*_givemecoins",
        braiinspool: Pool = "*_braiinspool",
        antpool: Pool = "*_antpool",
        multicoinco: Pool = "*_multicoinco",
        bcpoolio: Pool = "*_bcpoolio",
        cointerra: Pool = "*_cointerra",
        kanopool: Pool = "*_kanopool",
        solock: Pool = "*_solock",
        ckpool: Pool = "*_ckpool",
        nicehash: Pool = "*_nicehash",
        bitclub: Pool = "*_bitclub",
        bitcoinaffiliatenetwork: Pool = "*_bitcoinaffiliatenetwork",
        btcc: Pool = "*_btcc",
        bwpool: Pool = "*_bwpool",
        exxbw: Pool = "*_exxbw",
        bitsolo: Pool = "*_bitsolo",
        bitfury: Pool = "*_bitfury",
        _21inc: Pool = "*_21inc",
        digitalbtc: Pool = "*_digitalbtc",
        _8baochi: Pool = "*_8baochi",
        mybtccoinpool: Pool = "*_mybtccoinpool",
        tbdice: Pool = "*_tbdice",
        hashpool: Pool = "*_hashpool",
        nexious: Pool = "*_nexious",
        bravomining: Pool = "*_bravomining",
        hotpool: Pool = "*_hotpool",
        okexpool: Pool = "*_okexpool",
        bcmonster: Pool = "*_bcmonster",
        _1hash: Pool = "*_1hash",
        bixin: Pool = "*_bixin",
        tatmaspool: Pool = "*_tatmaspool",
        viabtc: Pool = "*_viabtc",
        connectbtc: Pool = "*_connectbtc",
        batpool: Pool = "*_batpool",
        waterhole: Pool = "*_waterhole",
        dcexploration: Pool = "*_dcexploration",
        dcex: Pool = "*_dcex",
        btpool: Pool = "*_btpool",
        _58coin: Pool = "*_58coin",
        bitcoinindia: Pool = "*_bitcoinindia",
        shawnp0wers: Pool = "*_shawnp0wers",
        phashio: Pool = "*_phashio",
        rigpool: Pool = "*_rigpool",
        haozhuzhu: Pool = "*_haozhuzhu",
        _7pool: Pool = "*_7pool",
        miningkings: Pool = "*_miningkings",
        hashbx: Pool = "*_hashbx",
        dpool: Pool = "*_dpool",
        rawpool: Pool = "*_rawpool",
        haominer: Pool = "*_haominer",
        helix: Pool = "*_helix",
        bitcoinukraine: Pool = "*_bitcoinukraine",
        poolin: Pool = "*_poolin",
        secretsuperstar: Pool = "*_secretsuperstar",
        tigerpoolnet: Pool = "*_tigerpoolnet",
        sigmapoolcom: Pool = "*_sigmapoolcom",
        okpooltop: Pool = "*_okpooltop",
        hummerpool: Pool = "*_hummerpool",
        tangpool: Pool = "*_tangpool",
        bytepool: Pool = "*_bytepool",
        spiderpool: Pool = "*_spiderpool",
        novablock: Pool = "*_novablock",
        miningcity: Pool = "*_miningcity",
        binancepool: Pool = "*_binancepool",
        minerium: Pool = "*_minerium",
        lubiancom: Pool = "*_lubiancom",
        okkong: Pool = "*_okkong",
        aaopool: Pool = "*_aaopool",
        emcdpool: Pool = "*_emcdpool",
        foundryusa: Pool = "*_foundryusa",
        sbicrypto: Pool = "*_sbicrypto",
        arkpool: Pool = "*_arkpool",
        purebtccom: Pool = "*_purebtccom",
        marapool: Pool = "*_marapool",
        kucoinpool: Pool = "*_kucoinpool",
        entrustcharitypool: Pool = "*_entrustcharitypool",
        okminer: Pool = "*_okminer",
        titan: Pool = "*_titan",
        pegapool: Pool = "*_pegapool",
        btcnuggets: Pool = "*_btcnuggets",
        cloudhashing: Pool = "*_cloudhashing",
        digitalxmintsy: Pool = "*_digitalxmintsy",
        telco214: Pool = "*_telco214",
        btcpoolparty: Pool = "*_btcpoolparty",
        multipool: Pool = "*_multipool",
        transactioncoinmining: Pool = "*_transactioncoinmining",
        btcdig: Pool = "*_btcdig",
        trickysbtcpool: Pool = "*_trickysbtcpool",
        btcmp: Pool = "*_btcmp",
        eobot: Pool = "*_eobot",
        unomp: Pool = "*_unomp",
        patels: Pool = "*_patels",
        gogreenlight: Pool = "*_gogreenlight",
        bitcoinindiapool: Pool = "*_bitcoinindiapool",
        ekanembtc: Pool = "*_ekanembtc",
        canoe: Pool = "*_canoe",
        tiger: Pool = "*_tiger",
        _1m1x: Pool = "*_1m1x",
        zulupool: Pool = "*_zulupool",
        secpool: Pool = "*_secpool",
        ocean: Pool = "*_ocean",
        whitepool: Pool = "*_whitepool",
        wiz: Pool = "*_wiz",
        wk057: Pool = "*_wk057",
        futurebitapollosolo: Pool = "*_futurebitapollosolo",
        carbonnegative: Pool = "*_carbonnegative",
        portlandhodl: Pool = "*_portlandhodl",
        phoenix: Pool = "*_phoenix",
        neopool: Pool = "*_neopool",
        maxipool: Pool = "*_maxipool",
        bitfufupool: Pool = "*_bitfufupool",
        gdpool: Pool = "*_gdpool",
        miningdutch: Pool = "*_miningdutch",
        publicpool: Pool = "*_publicpool",
        miningsquared: Pool = "*_miningsquared",
        innopolistech: Pool = "*_innopolistech",
        btclab: Pool = "*_btclab",
        parasite: Pool = "*_parasite",
        redrockpool: Pool = "*_redrockpool",
        est3lar: Pool = "*_est3lar",
        braiinssolo: Pool = "*_braiinssolo",
        solopoolcom: Pool = "*_solopoolcom",
        noderunners: Pool = "*_noderunners",
        dmnd: Pool = "*_dmnd",
    } }
    shape! { DataBytesSeries at "series().op_return.data_bytes" {
        block: SeriesPattern18<Bytes> = "*_bytes",
        cumulative: SeriesPattern1<Bytes> = "*_bytes_cumulative",
        sum: Windows<Bytes> = "*_bytes_sum",
        chain_share: Share = "*_chain_share",
    } }
    shape! { Inscription at "series().transactions.inscription" {
        flag: SeriesPattern19<Boolean> = "has_*",
        block: SeriesPattern18<Count> = "*_tx_count",
        cumulative: SeriesPattern1<Count> = "*_tx_count_cumulative",
        sum: Windows<Count> = "*_tx_count_sum",
        share: Share = "*_tx_share",
        fees: FeesSeries = "*",
    } }
    shape! { TransactionsFlagged at "series().transactions.nonstandard" {
        flag: SeriesPattern19<Boolean> = "is_*",
        block: SeriesPattern18<Count> = "*_tx_count",
        cumulative: SeriesPattern1<Count> = "*_tx_count_cumulative",
        sum: Windows<Count> = "*_tx_count_sum",
        share: Share = "*_tx_share",
    } }
    shape! { Patterns at "series().transactions.patterns" {
        coinjoin: TransactionsFlagged = "*",
        consolidation: TransactionsFlagged = "consolidation",
        batch_payout: TransactionsFlagged = "batch_payout",
    } }
    shape! { Fees at "series().transactions.fees" {
        fee: PerTxDistribution<Sats> = "tx_*",
        fee_rate: SeriesPattern19<Option<FeeRate>> = "*_rate",
        effective_fee_rate: PerTxDistribution<Option<FeeRate>> = "effective_*_rate",
        cpfp_parent: TransactionsFlagged = "cpfp_parent",
        cpfp_child: TransactionsFlagged = "cpfp_child",
    } }
    shape! { TxCount at "series().transactions.features.one_input" {
        block: SeriesPattern18<Count> = "*_count",
        cumulative: SeriesPattern1<Count> = "*_count_cumulative",
        sum: Windows<Count> = "*_count_sum",
        share: Share = "*_share",
    } }
    shape! { Flagged at "series().transactions.features.dust_output" {
        flag: SeriesPattern19<Boolean> = "has_*",
        block: SeriesPattern18<Count> = "*_tx_count",
        cumulative: SeriesPattern1<Count> = "*_tx_count_cumulative",
        sum: Windows<Count> = "*_tx_count_sum",
        share: Share = "*_tx_share",
    } }
    shape! { Signatures at "series().transactions.features.signatures" {
        annex: Flagged = "annex",
        sighash_all: Flagged = "*_all",
        sighash_none: Flagged = "*_none",
        sighash_single: Flagged = "*_single",
        sighash_default: Flagged = "*_default",
        sighash_anyone_can_pay: Flagged = "*_anyone_can_pay",
    } }
    shape! { TransactionsScripts at "series().transactions.features.scripts" {
        p2pk: Flagged = "p2pk",
        p2ms: Flagged = "p2ms",
        p2pkh: Flagged = "p2pkh",
        p2sh: Flagged = "p2sh",
        p2wpkh: Flagged = "p2wpkh",
        p2wsh: Flagged = "p2wsh",
        p2tr: Flagged = "p2tr",
        p2a: Flagged = "p2a",
        empty_script: Flagged = "empty_*",
        unknown_script: Flagged = "unknown_*",
        fake_pubkey: Flagged = "fake_pubkey",
        fake_scripthash: Flagged = "fake_scripthash",
    } }
    shape! { Features at "series().transactions.features" {
        scripts: TransactionsScripts = "script",
        signatures: Signatures = "sighash",
        segwit: TxCount = "segwit_*",
        rbf_signaling: TransactionsFlagged = "rbf_signaling",
        dust_output: Flagged = "dust_output",
        epobc: TransactionsFlagged = "epobc",
        one_input: TxCount = "one_input_*",
        one_output: TxCount = "one_output_*",
    } }
    shape! { FlowStats<A, B> at "series().blocks.size" {
        block: SeriesPattern18<A> = "*",
        cumulative: SeriesPattern1<A> = "*_cumulative",
        sum: Windows<A> = "*_sum",
        avg: Windows<B> = "*_avg",
        min: Windows<A> = "*_min",
        max: Windows<A> = "*_max",
        p10: Windows<A> = "*_p10",
        p25: Windows<A> = "*_p25",
        median: Windows<A> = "*_median",
        p75: Windows<A> = "*_p75",
        p90: Windows<A> = "*_p90",
    } }
    shape! { Averaged<A, B> at "series().blocks.interval" {
        block: SeriesPattern18<A> = "*",
        avg: Windows<B> = "*_avg",
    } }
    shape! { AddressesActivity at "series().addresses.activity" {
        reactivated: Averaged<Count, Option<CountFract>> = "*_reactivated_address_count",
        sending: Averaged<Count, Option<CountFract>> = "*_sending_address_count",
        receiving: Averaged<Count, Option<CountFract>> = "*_receiving_address_count",
        bidirectional: Averaged<Count, Option<CountFract>> = "*_bidirectional_address_count",
        active: Averaged<Count, Option<CountFract>> = "*_active_address_count",
    } }
    shape! { Flow<A> at "series().blocks.count" {
        block: SeriesPattern18<A> = "*",
        cumulative: SeriesPattern1<A> = "*_cumulative",
        sum: Windows<A> = "*_sum",
    } }
    shape! { ReserveRisk at "series().cointime.reserve_risk" {
        block: SeriesPattern1<Option<Float>> = "*",
        hodl_bank: SeriesPattern1<Option<Float>> = "hodl_bank",
        vocdd: Flow<Option<DollarDaysPerCoin>> = "vocdd",
    } }
    shape! { CointimeValue at "series().cointime.value" {
        destroyed: Flow<Option<DollarBlocks>> = "*_destroyed",
        created: Flow<Option<DollarBlocks>> = "*_created",
        stored: Flow<Option<DollarBlocks>> = "*_stored",
    } }
    shape! { CointimeRange at "series().cointime.age_ranges._10y_to_12y" {
        coindays_created: Flow<Option<CoinDays>> = "*_coindays_created",
        coindays_consumed: Flow<Option<CoinDays>> = "*_coindays_consumed",
        wakefulness: SeriesPattern1<Option<Ratio>> = "*_wakefulness",
        awake_to_dormant: SeriesPattern1<Option<Multiple>> = "*_awake_to_dormant",
        awake: Side = "*_awake_supply",
        dormant: Side = "*_dormant_supply",
    } }
    shape! { CointimeActivity at "series().cointime.activity" {
        coinblocks_created: Flow<Option<CoinBlocks>> = "*_created",
        coinblocks_destroyed: Flow<Option<CoinBlocks>> = "*_destroyed",
        coinblocks_stored: Flow<Option<CoinBlocks>> = "*_stored",
        liveliness: SeriesPattern1<Option<Ratio>> = "liveliness",
        vaultedness: SeriesPattern1<Option<Ratio>> = "vaultedness",
        liveliness_to_vaultedness: SeriesPattern1<Option<Multiple>> = "liveliness_to_vaultedness",
        concurrent_liveliness: Windows<Option<Ratio>> = "concurrent_liveliness",
    } }
    shape! { AdjustedSopr at "series().holders.all.realized.adjusted_sopr" {
        _24h: SeriesPattern1<Option<Multiple>> = "*_sopr_24h",
        _1w: SeriesPattern1<Option<Multiple>> = "*_sopr_1w",
        _1m: SeriesPattern1<Option<Multiple>> = "*_sopr_1m",
        _1y: SeriesPattern1<Option<Multiple>> = "*_sopr_1y",
        value_created: Flow<Option<Dollars>> = "*_value_created",
        value_destroyed: Flow<Option<Dollars>> = "*_value_destroyed",
    } }
    shape! { HoldersRealized at "series().holders.all.realized" {
        profit: Flow<Option<Dollars>> = "*_realized_profit",
        loss: Flow<Option<Dollars>> = "*_realized_loss",
        net_pnl: Flow<Option<Dollars>> = "*_net_realized_pnl",
        gross_pnl: Flow<Option<Dollars>> = "*_gross_realized_pnl",
        profit_to_loss: Windows<Option<Multiple>> = "*_realized_profit_to_loss",
        value_destroyed: Flow<Option<Dollars>> = "*_value_destroyed",
        peak_regret: Flow<Option<Dollars>> = "*_realized_peak_regret",
        sopr: Windows<Option<Multiple>> = "*_sopr",
        adjusted_sopr: AdjustedSopr = "*_adjusted",
        sell_side_risk_ratio: Windows<Option<Ratio>> = "*_sell_side_risk_ratio",
    } }
    shape! { HoldersActivity at "series().holders.all.activity" {
        transfer_volume: HoldersValueFlow = "*_transfer_volume",
        coindays_destroyed: Flow<Option<CoinDays>> = "*_coindays_destroyed",
        dormancy: Windows<Option<Days>> = "*_dormancy",
    } }
    shape! { EventCount<A> at "series().addresses.respent.events.active" {
        count: A = "*_count",
        share: Share = "*_share",
    } }
    shape! { ReuseEvents at "series().addresses.respent.events" {
        outputs: EventCount<Flow<Count>> = "*_output_to_respent_address",
        inputs: EventCount<Flow<Count>> = "*_input_from_respent_address",
        active: EventCount<Averaged<Count, Option<CountFract>>> = "*_active_respent_address",
    } }
    shape! { Respent at "series().addresses.respent" {
        count: RespentCount = "*",
        supply: PredicateSupply = "*_respent_address_supply",
        events: ReuseEvents = "*",
    } }
    shape! { AddressesReuseEvents at "series().addresses.reused.events" {
        outputs: EventCount<Flow<Count>> = "*_output_to_reused_address",
        inputs: EventCount<Flow<Count>> = "*_input_from_reused_address",
        active: EventCount<Averaged<Count, Option<CountFract>>> = "*_active_reused_address",
    } }
    shape! { Reused at "series().addresses.reused" {
        count: ReusedCount = "*",
        supply: PredicateSupply = "*_reused_address_supply",
        events: AddressesReuseEvents = "*",
    } }
    shape! { AddressSeries at "series().addresses.types.p2pk" {
        funded: WithDelta<Count, CountSigned> = "*_funded_address_count",
        empty: SeriesPattern1<Count> = "*_empty_address_count",
        total: SeriesPattern1<Count> = "*_total_address_count",
        new: NewAddressCount = "*_new_address_count",
        activity: AddressesActivity = "*",
        avg_balance: Value = "*_address_balance_avg",
        reused: Reused = "*",
        respent: Respent = "*",
        exposed: Exposed = "*",
    } }
    shape! { AddressType at "series().addresses.types" {
        p2pk: AddressSeries = "*",
        p2pkh: AddressSeries = "p2pkh",
        p2sh: AddressSeries = "p2sh",
        p2wpkh: AddressSeries = "p2wpkh",
        p2wsh: AddressSeries = "p2wsh",
        p2tr: AddressSeries = "p2tr",
    } }
    shape! { Realized at "series().addresses.balances._100_to_1k_btc.realized" {
        profit: Flow<Option<Dollars>> = "*_profit",
        loss: Flow<Option<Dollars>> = "*_loss",
    } }
    shape! { Balance at "series().addresses.balances._100_to_1k_btc" {
        funded: WithDelta<Count, CountSigned> = "*_address_count",
        supply: CohortSupply = "*_supply",
        capital: CapitalViews = "*",
        utxos: BalanceUtxos = "*_utxo_count",
        activity: Activity = "*_transfer_volume",
        realized: Realized = "*_realized",
    } }
    shape! { CreationActivity at "series().entry.rookie.activity" {
        transfer_volume: ValueFlow = "*_transfer_volume",
        coindays_destroyed: Flow<Option<CoinDays>> = "*_coindays_destroyed",
    } }
    shape! { CreationRealized at "series().entry.rookie.realized" {
        profit: Flow<Option<Dollars>> = "*_realized_profit",
        loss: Flow<Option<Dollars>> = "*_realized_loss",
        net_pnl: Flow<Option<Dollars>> = "*_net_realized_pnl",
        value_destroyed: Flow<Option<Dollars>> = "*_value_destroyed",
    } }
    shape! { AgeCreationActivity at "series().age.ranges._10y_to_12y.activity" {
        transfer_volume: HoldersValueFlow = "*_transfer_volume",
        coindays_destroyed: Flow<Option<CoinDays>> = "*_coindays_destroyed",
        coindays_created: Flow<Option<CoinDays>> = "*_coindays_created",
    } }
    shape! { CohortUtxos at "series().entry.rookie.utxos" {
        count: WithDelta<Count, CountSigned> = "*_utxo_count",
        spent_count: Flow<Count> = "*_spent_utxo_count",
    } }
    shape! { Cohort at "series().utxos.amounts._100_to_1k_btc" {
        supply: CohortSupply = "*_supply",
        capital: CapitalViews = "*",
        utxos: CohortUtxos = "*",
        activity: Activity = "*_transfer_volume",
        realized: Realized = "*_realized",
        cost_basis: CohortCostBasis = "*",
    } }
    shape! { UtxosSpendableType at "series().utxos.types" {
        p2pk: Cohort = "p2pk",
        p2pkh: Cohort = "p2pkh",
        p2ms: Cohort = "p2ms",
        p2sh: Cohort = "p2sh",
        p2wpkh: Cohort = "p2wpkh",
        p2wsh: Cohort = "p2wsh",
        p2tr: Cohort = "p2tr",
        p2a: Cohort = "p2a",
        unknown: Cohort = "unknown_*",
        empty: Cohort = "empty_*",
    } }
    shape! { Zero at "series().utxos.amounts._0_sats" {
        utxos: CohortUtxos = "*",
    } }
    shape! { AmountRange<A, B> at "series().addresses.balances" {
        _0_sats: A = "*_0_sats",
        _1_to_10_sats: B = "*_1_to_10_sats",
        _10_to_100_sats: B = "*_10_to_100_sats",
        _100_to_1k_sats: B = "*_100_to_1k_sats",
        _1k_to_10k_sats: B = "*_1k_to_10k_sats",
        _10k_to_100k_sats: B = "*_10k_to_100k_sats",
        _100k_to_1m_sats: B = "*_100k_to_1m_sats",
        _1m_to_10m_sats: B = "*_1m_to_10m_sats",
        _10m_to_100m_sats: B = "*_10m_to_100m_sats",
        _1_to_10_btc: B = "*_1_to_10_btc",
        _10_to_100_btc: B = "*_10_to_100_btc",
        _100_to_1k_btc: B = "*_100_to_1k_btc",
        _1k_to_10k_btc: B = "*_1k_to_10k_btc",
        _10k_to_100k_btc: B = "*_10k_to_100k_btc",
        over_100k_btc: B = "*_over_100k_btc",
    } }
    shape! { Addresses at "series().addresses" {
        funded: WithDelta<Count, CountSigned> = "funded_*_count",
        empty: SeriesPattern1<Count> = "empty_*_count",
        total: SeriesPattern1<Count> = "total_*_count",
        new: NewAddressCount = "new_*_count",
        activity: AddressesActivity = "",
        avg_balance: Value = "*_balance_avg",
        reused: Reused = "",
        respent: Respent = "",
        exposed: Exposed = "",
        supply: CohortSupply = "*_supply",
        capital: CapitalViews = "*",
        types: AddressType = "p2pk",
        balances: AmountRange<ZeroBalance, Balance> = "balance",
    } }
    shape! { Utxos at "series().utxos" {
        amounts: AmountRange<Zero, Cohort> = "*",
        types: UtxosSpendableType = "output",
    } }
    shape! { CreationCohort<A, B, C, D, E, F> at "series().entry.rookie" {
        supply: A = "*_supply",
        capital: B = "*",
        utxos: CohortUtxos = "*",
        activity: C = "*",
        realized: D = "*",
        unrealized: E = "*",
        cost_basis: F = "*",
    } }
    shape! { Entry at "series().entry" {
        veteran: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*",
        rookie: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "rookie",
    } }
    shape! { Holders<A, B> at "series().holders" {
        all: A = "",
        sth: B = "sth",
        lth: B = "lth",
        under_4m: B = "*_under_4m",
        under_6m: B = "*_under_6m",
        over_4m: B = "*_over_4m",
        over_6m: B = "*_over_6m",
    } }
    shape! { Vintage at "series().age.vintages" {
        _2009: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2009",
        _2010: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2010",
        _2011: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2011",
        _2012: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2012",
        _2013: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2013",
        _2014: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2014",
        _2015: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2015",
        _2016: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2016",
        _2017: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2017",
        _2018: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2018",
        _2019: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2019",
        _2020: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2020",
        _2021: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2021",
        _2022: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2022",
        _2023: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2023",
        _2024: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2024",
        _2025: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2025",
        _2026: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2026",
        _2027: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2027",
    } }
    shape! { ByHalvingEpoch at "series().age.halving_epochs" {
        _0: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_0",
        _1: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_1",
        _2: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_2",
        _3: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_3",
        _4: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_4",
        _5: CreationCohort<CreationSupply, CreationCapital, CreationActivity, CreationRealized, CreationUnrealized, CohortCostBasis> = "*_5",
    } }
    shape! { Range at "series().age.ranges._10y_to_12y" {
        supply: AgeCreationSupply = "*",
        capital: CreationCapital = "*",
        utxos: CohortUtxos = "*",
        activity: AgeCreationActivity = "*",
        realized: CreationRealized = "*",
        unrealized: CreationUnrealized = "*",
        cost_basis: CohortCostBasis = "*",
    } }
    shape! { AgeRange<A> at "series().age.ranges" {
        under_1h: A = "*_under_1h",
        _1h_to_1d: A = "*_1h_to_1d",
        _1d_to_1w: A = "*_1d_to_1w",
        _1w_to_1m: A = "*_1w_to_1m",
        _1m_to_2m: A = "*_1m_to_2m",
        _2m_to_3m: A = "*_2m_to_3m",
        _3m_to_4m: A = "*_3m_to_4m",
        _4m_to_5m: A = "*_4m_to_5m",
        _5m_to_6m: A = "*_5m_to_6m",
        _6m_to_9m: A = "*_6m_to_9m",
        _9m_to_1y: A = "*_9m_to_1y",
        _1y_to_18m: A = "*_1y_to_18m",
        _18m_to_2y: A = "*_18m_to_2y",
        _2y_to_3y: A = "*_2y_to_3y",
        _3y_to_4y: A = "*_3y_to_4y",
        _4y_to_5y: A = "*_4y_to_5y",
        _5y_to_6y: A = "*_5y_to_6y",
        _6y_to_7y: A = "*_6y_to_7y",
        _7y_to_8y: A = "*_7y_to_8y",
        _8y_to_10y: A = "*_8y_to_10y",
        _10y_to_12y: A = "*_10y_to_12y",
        _12y_to_15y: A = "*_12y_to_15y",
        over_15y: A = "*_over_15y",
    } }
    shape! { Coinflow at "series().coinflow" {
        age_ranges: AgeRange<CoinflowRange> = "*",
        cohorts: Holders<CoinflowCohort, CoinflowCohort> = "*",
    } }
    shape! { Cointime at "series().cointime" {
        activity: CointimeActivity = "coinblocks",
        age_ranges: AgeRange<CointimeRange> = "*",
        cohorts: Holders<CointimeCohort, CointimeCohort> = "*",
        supply: CointimeSupply = "supply",
        value: CointimeValue = "cointime_value",
        caps: Caps = "cap",
        prices: Prices = "vaulted",
        adjusted: Adjusted = "cointime_adjusted",
        reserve_risk: ReserveRisk = "reserve_risk",
    } }
    shape! { Age at "series().age" {
        ranges: AgeRange<Range> = "*",
        halving_epochs: ByHalvingEpoch = "halving_epoch",
        vintages: Vintage = "vintage",
    } }
    shape! { Bucket at "series().op_return.policies.multiple" {
        output_count: Flow<Count> = "*_output_count",
        data_bytes: DataBytesSeries = "*_data",
        tx_count: Flow<Count> = "*_tx_count",
        tx_share: Share = "*_tx_share",
        tx_vsize: Flow<VSize> = "*_tx_vsize",
        fees: FeesSeries = "*",
    } }
    shape! { Policy at "series().op_return.policies" {
        pre_v30_standard: Bucket = "*_pre_v30_standard",
        pre_v30_nonstandard: Bucket = "*_pre_v30_nonstandard",
        oversized: Bucket = "*_oversized",
        multiple: Bucket = "*_multiple",
    } }
    shape! { ByKind at "series().op_return.protocols" {
        runes: Bucket = "*_runes",
        witness_commitment: Bucket = "*_witness_commitment",
        veriblock: Bucket = "*_veriblock",
        omni: Bucket = "*_omni",
        stacks: Bucket = "*_stacks",
        blockstack: Bucket = "*_blockstack",
        colu: Bucket = "*_colu",
        open_assets: Bucket = "*_open_assets",
        komodo: Bucket = "*_komodo",
        coinspark: Bucket = "*_coinspark",
        poet: Bucket = "*_poet",
        docproof: Bucket = "*_docproof",
        factom: Bucket = "*_factom",
        eternity_wall: Bucket = "*_eternity_wall",
        memo: Bucket = "*_memo",
        bitproof: Bucket = "*_bitproof",
        ascribe: Bucket = "*_ascribe",
        stampery: Bucket = "*_stampery",
        bare_hash: Bucket = "*_bare_hash",
        text: Bucket = "*_text",
        empty: Bucket = "*_empty",
        unknown: Bucket = "*_unknown",
    } }
    shape! { OpReturn at "series().op_return" {
        flag: SeriesPattern19<Boolean> = "has_*",
        output_count: Flow<Count> = "*_output_count",
        data_bytes: DataBytesSeries = "*_data",
        tx_count: Flow<Count> = "*_tx_count",
        tx_share: Share = "*_tx_share",
        tx_vsize: Flow<VSize> = "*_tx_vsize",
        fees: FeesSeries = "*",
        protocols: ByKind = "*",
        policies: Policy = "*",
    } }
    shape! { OutputTypeSeries at "series().inputs.types.empty" {
        count: Flow<Count> = "*_count",
        share: Share = "*_share",
        tx_count: Flow<Count> = "*_tx_count",
        tx_share: Share = "*_tx_share",
    } }
    shape! { ByType at "series().outputs.types" {
        p2pk: OutputTypeSeries = "p2pk_*",
        p2pkh: OutputTypeSeries = "p2pkh_*",
        p2ms: OutputTypeSeries = "p2ms_*",
        p2sh: OutputTypeSeries = "p2sh_*",
        p2wpkh: OutputTypeSeries = "p2wpkh_*",
        p2wsh: OutputTypeSeries = "p2wsh_*",
        p2tr: OutputTypeSeries = "p2tr_*",
        p2a: OutputTypeSeries = "p2a_*",
        unknown: OutputTypeSeries = "unknown_*",
        empty: OutputTypeSeries = "empty_*",
        op_return: OutputTypeSeries = "op_return_*",
    } }
    shape! { Outputs at "series().outputs" {
        spending_txin_index: SeriesPattern23<TxInIndex> = "spending_txin_index",
        count: FlowStats<Count, Option<CountFract>> = "*_count",
        per_second: Windows<Option<PerSecond>> = "outputs_per_second",
        spendable_count: Flow<Count> = "spendable_*_count",
        types: ByType = "*",
        op_return_value: ValueFlow = "op_return_value",
    } }
    shape! { SpendableType at "series().inputs.types" {
        p2pk: OutputTypeSeries = "p2pk_*",
        p2pkh: OutputTypeSeries = "p2pkh_*",
        p2ms: OutputTypeSeries = "p2ms_*",
        p2sh: OutputTypeSeries = "p2sh_*",
        p2wpkh: OutputTypeSeries = "p2wpkh_*",
        p2wsh: OutputTypeSeries = "p2wsh_*",
        p2tr: OutputTypeSeries = "p2tr_*",
        p2a: OutputTypeSeries = "p2a_*",
        unknown: OutputTypeSeries = "unknown_*",
        empty: OutputTypeSeries = "empty_*",
    } }
    shape! { Inputs at "series().inputs" {
        count: FlowStats<Count, Option<CountFract>> = "*_count",
        per_second: Windows<Option<PerSecond>> = "inputs_per_second",
        types: SpendableType = "*",
    } }
    shape! { Blocks at "series().blocks" {
        count: Flow<Count> = "*_count",
        interval: Averaged<Seconds, Option<SecondsFract>> = "*_interval",
        size: FlowStats<Bytes, Option<BytesFract>> = "*_size",
        weight: FlowStats<Weight, Option<WeightFract>> = "*_weight",
        fullness: SeriesPattern18<Option<Percent>> = "*_fullness",
        difficulty: BlocksDifficulty = "difficulty",
        halving: BlocksHalving = "halving",
    } }
    shape! { Price at "series().price" {
        open: SeriesPattern2<Option<Dollars>> = "*_open",
        high: SeriesPattern2<Option<Dollars>> = "*_high",
        low: SeriesPattern2<Option<Dollars>> = "*_low",
        close: SeriesPattern2<Option<Dollars>> = "*_close",
        ohlc: SeriesPattern2<OHLCDollars> = "*_ohlc",
        spot: SeriesPattern1<Option<Dollars>> = "*",
        sats_per_dollar: SeriesPattern1<Sats> = "sats_per_dollar",
    } }
    shape! { Timestamps at "series().mappings.timestamp" {
        monotonic: SeriesPattern18<Timestamp> = "*_monotonic",
        period_start: SeriesPattern2<Timestamp> = "*",
    } }
    shape! { TxoutIndex at "series().mappings.txout_index" {
        identity: SeriesPattern22<TxOutIndex> = "*",
    } }
    shape! { TxinIndex at "series().mappings.txin_index" {
        identity: SeriesPattern20<TxInIndex> = "*",
    } }
    shape! { MappingsTxIndex at "series().mappings.tx_index" {
        identity: SeriesPattern19<TxIndex> = "*_index",
        input_count: SeriesPattern19<Count> = "*_input_count",
        output_count: SeriesPattern19<Count> = "*_output_count",
    } }
    shape! { MappingsYear10 at "series().mappings.year10" {
        date: SeriesPattern15<Date> = "*",
        first_height: SeriesPattern15<Height> = "first_height",
    } }
    shape! { MappingsYear1 at "series().mappings.year1" {
        date: SeriesPattern14<Date> = "*",
        first_height: SeriesPattern14<Height> = "first_height",
    } }
    shape! { MappingsMonth6 at "series().mappings.month6" {
        date: SeriesPattern13<Date> = "*",
        first_height: SeriesPattern13<Height> = "first_height",
    } }
    shape! { MappingsMonth3 at "series().mappings.month3" {
        date: SeriesPattern12<Date> = "*",
        first_height: SeriesPattern12<Height> = "first_height",
    } }
    shape! { MappingsMonth1 at "series().mappings.month1" {
        date: SeriesPattern11<Date> = "*",
        first_height: SeriesPattern11<Height> = "first_height",
    } }
    shape! { MappingsWeek1 at "series().mappings.week1" {
        date: SeriesPattern10<Date> = "*",
        first_height: SeriesPattern10<Height> = "first_height",
    } }
    shape! { MappingsDay3 at "series().mappings.day3" {
        date: SeriesPattern9<Date> = "*",
        first_height: SeriesPattern9<Height> = "first_height",
    } }
    shape! { MappingsDay1 at "series().mappings.day1" {
        date: SeriesPattern8<Date> = "*",
        first_height: SeriesPattern8<Height> = "first_height",
    } }
    shape! { MappingsHour12 at "series().mappings.hour12" {
        first_height: SeriesPattern7<Height> = "*",
    } }
    shape! { MappingsHour4 at "series().mappings.hour4" {
        first_height: SeriesPattern6<Height> = "*",
    } }
    shape! { MappingsHour1 at "series().mappings.hour1" {
        first_height: SeriesPattern5<Height> = "*",
    } }
    shape! { MappingsMinute30 at "series().mappings.minute30" {
        first_height: SeriesPattern4<Height> = "*",
    } }
    shape! { MappingsMinute10 at "series().mappings.minute10" {
        first_height: SeriesPattern3<Height> = "*",
    } }
    shape! { MappingsHalving at "series().mappings.halving" {
        first_height: SeriesPattern16<Height> = "*",
    } }
    shape! { MappingsEpoch at "series().mappings.epoch" {
        first_height: SeriesPattern17<Height> = "*",
    } }
    shape! { MappingsHeight at "series().mappings.height" {
        minute10: SeriesPattern18<Minute10> = "*_minute10",
        minute30: SeriesPattern18<Minute30> = "*_minute30",
        hour1: SeriesPattern18<Hour1> = "*_hour1",
        hour4: SeriesPattern18<Hour4> = "*_hour4",
        hour12: SeriesPattern18<Hour12> = "*_hour12",
        day1: SeriesPattern18<Day1> = "*_day1",
        day3: SeriesPattern18<Day3> = "*_day3",
        epoch: SeriesPattern18<Epoch> = "difficulty_epoch",
        halving: SeriesPattern18<Halving> = "halving_epoch",
        week1: SeriesPattern18<Week1> = "*_week1",
        month1: SeriesPattern18<Month1> = "*_month1",
        month3: SeriesPattern18<Month3> = "*_month3",
        month6: SeriesPattern18<Month6> = "*_month6",
        year1: SeriesPattern18<Year1> = "*_year1",
        year10: SeriesPattern18<Year10> = "*_year10",
    } }
    shape! { MappingsOpReturn at "series().mappings.outputs.op_return" {
        identity: SeriesPattern25<OpReturnIndex> = "*",
    } }
    shape! { MappingsUnknown at "series().mappings.outputs.unknown" {
        identity: SeriesPattern35<UnknownOutputIndex> = "*",
    } }
    shape! { Empty at "series().mappings.outputs.empty" {
        identity: SeriesPattern24<EmptyOutputIndex> = "*",
    } }
    shape! { MappingsP2ms at "series().mappings.outputs.p2ms" {
        identity: SeriesPattern27<P2MSOutputIndex> = "*",
    } }
    shape! { Output at "series().mappings.outputs" {
        p2ms: MappingsP2ms = "p2ms_output_*",
        empty: Empty = "empty_output_*",
        unknown: MappingsUnknown = "unknown_output_*",
        op_return: MappingsOpReturn = "op_return_*",
    } }
    shape! { MappingsP2a at "series().mappings.addresses.p2a" {
        identity: SeriesPattern26<P2AAddressIndex> = "*_index",
        address: SeriesPattern26<Address> = "*",
    } }
    shape! { MappingsP2wsh at "series().mappings.addresses.p2wsh" {
        identity: SeriesPattern34<P2WSHAddressIndex> = "*_index",
        address: SeriesPattern34<Address> = "*",
    } }
    shape! { MappingsP2wpkh at "series().mappings.addresses.p2wpkh" {
        identity: SeriesPattern33<P2WPKHAddressIndex> = "*_index",
        address: SeriesPattern33<Address> = "*",
    } }
    shape! { MappingsP2tr at "series().mappings.addresses.p2tr" {
        identity: SeriesPattern32<P2TRAddressIndex> = "*_index",
        address: SeriesPattern32<Address> = "*",
    } }
    shape! { MappingsP2sh at "series().mappings.addresses.p2sh" {
        identity: SeriesPattern31<P2SHAddressIndex> = "*_index",
        address: SeriesPattern31<Address> = "*",
    } }
    shape! { MappingsP2pkh at "series().mappings.addresses.p2pkh" {
        identity: SeriesPattern30<P2PKHAddressIndex> = "*_index",
        address: SeriesPattern30<Address> = "*",
    } }
    shape! { MappingsP2pk65 at "series().mappings.addresses.p2pk65" {
        identity: SeriesPattern29<P2PK65AddressIndex> = "*_index",
        address: SeriesPattern29<Address> = "*",
    } }
    shape! { MappingsP2pk33 at "series().mappings.addresses.p2pk33" {
        identity: SeriesPattern28<P2PK33AddressIndex> = "*_index",
        address: SeriesPattern28<Address> = "*",
    } }
    shape! { MappingsAddresses at "series().mappings.addresses" {
        p2pk33: MappingsP2pk33 = "p2pk33_*",
        p2pk65: MappingsP2pk65 = "p2pk65_*",
        p2pkh: MappingsP2pkh = "p2pkh_*",
        p2sh: MappingsP2sh = "p2sh_*",
        p2tr: MappingsP2tr = "p2tr_*",
        p2wpkh: MappingsP2wpkh = "p2wpkh_*",
        p2wsh: MappingsP2wsh = "p2wsh_*",
        p2a: MappingsP2a = "p2a_*",
    } }
    shape! { Mappings at "series().mappings" {
        addresses: MappingsAddresses = "address",
        outputs: Output = "index",
        height: MappingsHeight = "block",
        epoch: MappingsEpoch = "first_height",
        halving: MappingsHalving = "first_height",
        minute10: MappingsMinute10 = "first_height",
        minute30: MappingsMinute30 = "first_height",
        hour1: MappingsHour1 = "first_height",
        hour4: MappingsHour4 = "first_height",
        hour12: MappingsHour12 = "first_height",
        day1: MappingsDay1 = "*",
        day3: MappingsDay3 = "*",
        week1: MappingsWeek1 = "*",
        month1: MappingsMonth1 = "*",
        month3: MappingsMonth3 = "*",
        month6: MappingsMonth6 = "*",
        year1: MappingsYear1 = "*",
        year10: MappingsYear10 = "*",
        tx_index: MappingsTxIndex = "tx",
        txin_index: TxinIndex = "txin_index",
        txout_index: TxoutIndex = "txout_index",
        timestamp: Timestamps = "timestamp",
    } }
    shape! { IndexerOpReturn at "series().indexer.op_return" {
        first_index: SeriesPattern18<OpReturnIndex> = "first_*_index",
        tx_index: SeriesPattern25<TxIndex> = "*_tx_index",
        protocol: SeriesPattern25<OpReturnKind> = "*_protocol",
        post_op_return_bytes: SeriesPattern25<Bytes> = "post_*_bytes",
    } }
    shape! { IndexerUnknown at "series().indexer.scripts.unknown" {
        first_index: SeriesPattern18<UnknownOutputIndex> = "first_*_output_index",
        tx_index: SeriesPattern35<TxIndex> = "*_output_tx_index",
        legacy_sigop_cost: SeriesPattern35<SigOps> = "*_legacy_sigop_cost",
    } }
    shape! { IndexerP2ms at "series().indexer.scripts.p2ms" {
        first_index: SeriesPattern18<P2MSOutputIndex> = "first_*_output_index",
        tx_index: SeriesPattern27<TxIndex> = "*_output_tx_index",
        legacy_sigop_cost: SeriesPattern27<SigOps> = "*_legacy_sigop_cost",
    } }
    shape! { ScriptType at "series().indexer.scripts.empty" {
        first_index: SeriesPattern18<EmptyOutputIndex> = "first_*_index",
        tx_index: SeriesPattern24<TxIndex> = "*_tx_index",
    } }
    shape! { Scripts at "series().indexer.scripts" {
        empty: ScriptType = "*",
        p2ms: IndexerP2ms = "p2ms",
        unknown: IndexerUnknown = "unknown",
    } }
    shape! { IndexerP2a at "series().indexer.addresses.p2a" {
        first_index: SeriesPattern18<P2AAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern26<P2ABytes> = "*_bytes",
    } }
    shape! { IndexerP2tr at "series().indexer.addresses.p2tr" {
        first_index: SeriesPattern18<P2TRAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern32<P2TRBytes> = "*_bytes",
    } }
    shape! { IndexerP2wsh at "series().indexer.addresses.p2wsh" {
        first_index: SeriesPattern18<P2WSHAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern34<P2WSHBytes> = "*_bytes",
    } }
    shape! { IndexerP2wpkh at "series().indexer.addresses.p2wpkh" {
        first_index: SeriesPattern18<P2WPKHAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern33<P2WPKHBytes> = "*_bytes",
    } }
    shape! { IndexerP2sh at "series().indexer.addresses.p2sh" {
        first_index: SeriesPattern18<P2SHAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern31<P2SHBytes> = "*_bytes",
    } }
    shape! { IndexerP2pkh at "series().indexer.addresses.p2pkh" {
        first_index: SeriesPattern18<P2PKHAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern30<P2PKHBytes> = "*_bytes",
    } }
    shape! { IndexerP2pk33 at "series().indexer.addresses.p2pk33" {
        first_index: SeriesPattern18<P2PK33AddressIndex> = "first_*_address_index",
        bytes: SeriesPattern28<P2PK33Bytes> = "*_bytes",
    } }
    shape! { IndexerP2pk65 at "series().indexer.addresses.p2pk65" {
        first_index: SeriesPattern18<P2PK65AddressIndex> = "first_*_address_index",
        bytes: SeriesPattern29<P2PK65Bytes> = "*_bytes",
    } }
    shape! { Addrs at "series().indexer.addresses" {
        p2pk65: IndexerP2pk65 = "*",
        p2pk33: IndexerP2pk33 = "p2pk33",
        p2pkh: IndexerP2pkh = "p2pkh",
        p2sh: IndexerP2sh = "p2sh",
        p2wpkh: IndexerP2wpkh = "p2wpkh",
        p2wsh: IndexerP2wsh = "p2wsh",
        p2tr: IndexerP2tr = "p2tr",
        p2a: IndexerP2a = "p2a",
    } }
    shape! { IndexerOutputs at "series().indexer.outputs" {
        first_txout_index: SeriesPattern18<TxOutIndex> = "first_txout_index",
        value: SeriesPattern22<Sats> = "*_value",
        output_type: SeriesPattern22<OutputType> = "*_type",
        type_index: SeriesPattern22<TypeIndex> = "type_index",
    } }
    shape! { IndexerInputs at "series().indexer.inputs" {
        first_txin_index: SeriesPattern18<TxInIndex> = "first_txin_*",
        outpoint: SeriesPattern21<OutPoint> = "outpoint",
        txout_index: SeriesPattern21<TxOutIndex> = "spent_txout_*",
        tx_index: SeriesPattern20<TxIndex> = "input_tx_*",
        output_type: SeriesPattern20<OutputType> = "output_type",
        type_index: SeriesPattern21<TypeIndex> = "type_*",
        value: SeriesPattern20<Sats> = "input_value",
    } }
    shape! { CountSeries at "series().indexer.transactions.features.one_input" {
        block: SeriesPattern18<Count> = "*",
    } }
    shape! { VersionCount<A> at "series().transactions.versions" {
        v1: A = "v1_*",
        v2: A = "v2_*",
        v3: A = "v3_*",
        other: A = "other_version_*",
    } }
    shape! { Transactions at "series().transactions" {
        count: FlowStats<Count, Option<CountFract>> = "*_count",
        features: Features = "*",
        vsize: PerTxDistribution<VSize> = "*_vsize",
        fees: Fees = "fee",
        inscription: Inscription = "inscription",
        patterns: Patterns = "coinjoin",
        nonstandard: TransactionsFlagged = "nonstandard",
        sigop_cost: Flow<SigOps> = "block_sigop_cost",
        versions: VersionCount<TxCount> = "*",
        volume: ValueFlow = "*_volume",
        per_second: Windows<Option<PerSecond>> = "*_per_second",
    } }
    shape! { IndexerFeature at "series().indexer.transactions.features.epobc" {
        flag: SeriesPattern19<Boolean> = "is_*",
        block: SeriesPattern18<Count> = "*_tx_count",
    } }
    shape! { Feature at "series().indexer.transactions.features.annex" {
        flag: SeriesPattern19<Boolean> = "has_*",
        block: SeriesPattern18<Count> = "*_tx_count",
    } }
    shape! { TransactionFeatures at "series().indexer.transactions.features" {
        p2pk: Feature = "p2pk",
        p2ms: Feature = "p2ms",
        p2pkh: Feature = "p2pkh",
        p2sh: Feature = "p2sh",
        p2wpkh: Feature = "p2wpkh",
        p2wsh: Feature = "p2wsh",
        p2tr: Feature = "p2tr",
        p2a: Feature = "p2a",
        op_return: Feature = "op_return",
        empty_script: Feature = "empty_script",
        unknown_script: Feature = "unknown_script",
        fake_pubkey: Feature = "fake_pubkey",
        fake_scripthash: Feature = "fake_scripthash",
        inscription: Feature = "inscription",
        annex: Feature = "annex",
        sighash_all: Feature = "*_all",
        sighash_none: Feature = "*_none",
        sighash_single: Feature = "*_single",
        sighash_default: Feature = "*_default",
        sighash_anyone_can_pay: Feature = "*_anyone_can_pay",
        rbf_signaling: IndexerFeature = "rbf_signaling",
        dust_output: Feature = "dust_output",
        epobc: IndexerFeature = "epobc",
        one_input: CountSeries = "one_input_tx_count",
        one_output: CountSeries = "one_output_tx_count",
    } }
    shape! { IndexerTransactions at "series().indexer.transactions" {
        first_tx_index: SeriesPattern18<TxIndex> = "first_*_index",
        txid: SeriesPattern19<Txid> = "txid",
        version: SeriesPattern19<TxVersion> = "*_version",
        locktime: SeriesPattern19<RawLockTime> = "*_locktime",
        weight: SeriesPattern19<Weight> = "*_weight",
        size: SeriesPattern19<Bytes> = "*_size",
        sigop_cost: SeriesPattern19<SigOps> = "*_sigop_cost",
        first_txin_index: SeriesPattern19<TxInIndex> = "first_txin_index",
        first_txout_index: SeriesPattern19<TxOutIndex> = "first_txout_index",
        features: TransactionFeatures = "sighash",
        versions: VersionCount<CountSeries> = "*_count",
    } }
    shape! { IndexerBlocks at "series().indexer.blocks" {
        blockhash: SeriesPattern18<BlockHash> = "blockhash",
        coinbase_tag: SeriesPattern18<CoinbaseTag> = "coinbase_tag",
        timestamp: SeriesPattern18<Timestamp> = "timestamp",
        median_time: SeriesPattern18<Timestamp> = "median_time",
        size: SeriesPattern18<Bytes> = "block_size",
        weight: SeriesPattern18<Weight> = "block_weight",
        segwit_tx_count: SeriesPattern18<Count> = "*_count",
        segwit_tx_size: SeriesPattern18<Bytes> = "*_size",
        segwit_tx_weight: SeriesPattern18<Weight> = "*_weight",
    } }
    shape! { Indexer at "series().indexer" {
        blocks: IndexerBlocks = "segwit_*",
        transactions: IndexerTransactions = "*",
        inputs: IndexerInputs = "index",
        outputs: IndexerOutputs = "output",
        addresses: Addrs = "p2pk65",
        scripts: Scripts = "empty_output",
        op_return: IndexerOpReturn = "op_return",
    } }
    shape! { SeriesTree at "series()" {
        indexer: Indexer = "tx",
        mappings: Mappings = "date",
        price: Price = "price",
        blocks: Blocks = "block",
        inputs: Inputs = "input",
        outputs: Outputs = "output",
        transactions: Transactions = "tx",
        mining: Mining = "coinbase",
        op_return: OpReturn = "op_return",
        pools: Pools = "pool",
        utxo_set: UtxoSetSeries = "circulating_supply",
        market: Market = "price",
        age: Age = "age",
        utxos: Utxos = "amount",
        addresses: Addresses = "address",
        holders: Holders<CreationCohort<AllSupply, Capital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis>, CreationCohort<LthSupply, Capital, HoldersActivity, HoldersRealized, Unrealized, HoldersCostBasis>> = "age",
        entry: Entry = "veteran",
        profitability: Profitability = "sth",
        supply: Supply = "circulating_supply",
        cointime: Cointime = "age",
        coinflow: Coinflow = "age",
        bedrock: Bedrock = "bedrock",
        rarity_meter: RarityMeter = "rarity",
    } }
}
pub use tree::SeriesTree;
/// A method per API endpoint, and the typed series tree.
pub struct BitviewClient {
    base: Arc<BitviewClientBase>,
    series: OnceLock<Box<SeriesTree>>,
}

impl BitviewClient {
    pub const VERSION: &'static str = "v0.12.2";

    pub fn new(base_url: impl Into<String>) -> Self {
        let base = Arc::new(BitviewClientBase::new(base_url));
        Self {
            base,
            series: OnceLock::new(),
        }
    }

    pub fn with_options(options: BitviewClientOptions) -> Self {
        let base = Arc::new(BitviewClientBase::with_options(options));
        Self {
            base,
            series: OnceLock::new(),
        }
    }

    /// The typed series tree, built on first use.
    pub fn series(&self) -> &SeriesTree {
        self.series
            .get_or_init(|| Box::new(SeriesTree::build(self.base.clone(), String::new())))
    }

    /// A series by a name known only at runtime; [`series`](Self::series) has the typed paths.
    pub fn series_endpoint(
        &self,
        series: impl Into<SeriesName>,
        index: Index,
    ) -> SeriesEndpoint<serde_json::Value> {
        SeriesEndpoint::new(self.base.clone(), Arc::from(series.into().as_str()), index)
    }

    /// Like [`series_endpoint`](Self::series_endpoint) with date selectors; `Err` if the index is
    /// not date-based.
    pub fn date_series_endpoint(
        &self,
        series: impl Into<SeriesName>,
        index: Index,
    ) -> Result<DateSeriesEndpoint<serde_json::Value>> {
        if !index.is_date_based() {
            return Err(BitviewError::new(format!(
                "{} is not a date-based index",
                index.name()
            )));
        }
        Ok(SeriesEndpoint::new(
            self.base.clone(),
            Arc::from(series.into().as_str()),
            index,
        ))
    }

    /// Fetch address hash-prefix matches from raw payload bytes matching `address_type` length.
    pub fn get_address_payload_hash_prefix_matches(
        &self,
        address_type: OutputType,
        payload: &[u8],
        nibbles: usize,
    ) -> Result<AddressHashPrefixMatches> {
        validate_address_payload_for_type(address_type, payload)?;
        let prefix = address_payload_hash_prefix(payload, nibbles)?;
        self.get_address_hash_prefix_matches(address_type, &prefix)
    }

    /// Fetch address hash-prefix matches for a mainnet Bitcoin address.
    pub fn get_address_hash_prefix_matches_for_address(
        &self,
        address: &str,
        nibbles: usize,
    ) -> Result<AddressHashPrefixMatches> {
        let hashed = address_hash_prefix(address, nibbles)?;
        self.get_address_hash_prefix_matches(hashed.address_type, &hashed.prefix)
    }

    /// Health check
    ///
    /// Local health and query-readiness check. Returns server identity, uptime, and a coherent local sync snapshot without a bitcoind round-trip. Reads the published prefix during processing; an empty index returns 503 `state_updating` at once. Responses are not cached. For chain-tip catch-up, request `GET /api/server/sync`.
    ///
    /// Endpoint: `GET /health`
    pub fn get_health(&self) -> Result<Health> {
        self.base.get_json(&format!("/health"))
    }

    /// API version
    ///
    /// Endpoint: `GET /version`
    pub fn get_version(&self) -> Result<String> {
        self.base.get_json(&format!("/version"))
    }

    /// Sync status
    ///
    /// Returns a coherent local index snapshot and a separately observed Bitcoin Core tip height. The two heights can differ during indexing or a reorg. Conditional requests refresh these observations before validation.
    ///
    /// Endpoint: `GET /api/server/sync`
    pub fn get_sync_status(&self) -> Result<SyncStatus> {
        self.base.get_json(&format!("/api/server/sync"))
    }

    /// Disk usage
    ///
    /// Returns allocated file bytes (file lengths on non-Unix platforms) for BRK and Bitcoin data. Each request scans both trees; these are independent observations, not an atomic filesystem snapshot. Conditional requests validate the newly observed totals. Directory-link cycles and excessive nesting fail without returning partial totals.
    ///
    /// Endpoint: `GET /api/server/disk`
    pub fn get_disk_usage(&self) -> Result<DiskUsage> {
        self.base.get_json(&format!("/api/server/disk"))
    }

    /// Series catalog
    ///
    /// The available series as a tree of categories and subcategories.
    ///
    /// Endpoint: `GET /api/series`
    pub fn get_series_tree(&self) -> Result<TreeNode> {
        self.base.get_json(&format!("/api/series"))
    }

    /// Series count
    ///
    /// Returns the number of series available per index type.
    ///
    /// Endpoint: `GET /api/series/count`
    pub fn get_series_count(&self) -> Result<DetailedSeriesCount> {
        self.base.get_json(&format!("/api/series/count"))
    }

    /// List available indexes
    ///
    /// Returns all available indexes with their accepted REST query aliases. REST series queries accept any alias; MCP tools take the canonical `index` name only.
    ///
    /// Endpoint: `GET /api/series/indexes`
    pub fn get_indexes(&self) -> Result<Vec<IndexInfo>> {
        self.base.get_json(&format!("/api/series/indexes"))
    }

    /// Series list
    ///
    /// Paginated flat list of all available series names.
    ///
    /// Endpoint: `GET /api/series/list`
    pub fn list_series(&self, page: Option<i64>, per_page: Option<i64>) -> Result<PaginatedSeries> {
        let mut query = Vec::new();
        if let Some(v) = page {
            query.push(query_pair("page", v));
        }
        if let Some(v) = per_page {
            query.push(query_pair("per_page", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/series/list{}", query_str);
        self.base.get_json(&path)
    }

    /// Search series
    ///
    /// Search series by name or descriptive terms. Results prioritize whole query words in names, then descriptions, then fuzzy names, then fuzzy descriptions. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to 1024 UTF-8 bytes.
    ///
    /// Endpoint: `GET /api/series/search`
    pub fn search_series(&self, q: &str, limit: Option<Limit>) -> Result<Vec<SeriesName>> {
        let mut query = Vec::new();
        query.push(query_pair("q", q));
        if let Some(v) = limit {
            query.push(query_pair("limit", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/series/search{}", query_str);
        self.base.get_json(&path)
    }

    /// Get series info
    ///
    /// Returns the optional description, supported indexes, the indexes whose values can be null, the value type, and the optional unit (what the value type measures) for the specified series. The decoded series name is limited to 1024 UTF-8 bytes.
    ///
    /// Endpoint: `GET /api/series/{series}`
    pub fn get_series_info(&self, series: SeriesName) -> Result<SeriesInfo> {
        self.base
            .get_json(&format!("/api/series/{}", path_segment(series)))
    }

    /// Get series data
    ///
    /// Rows selected by a range of positions, dates or timestamps, as JSON or CSV.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}`
    pub fn get_series(
        &self,
        series: SeriesName,
        index: Index,
        start: Option<RangeIndex>,
        end: Option<RangeIndex>,
        limit: Option<Limit>,
        format: Option<Format>,
    ) -> Result<FormatResponse<SeriesData>> {
        let mut query = Vec::new();
        if let Some(v) = start {
            query.push(query_pair("start", v));
        }
        if let Some(v) = end {
            query.push(query_pair("end", v));
        }
        if let Some(v) = limit {
            query.push(query_pair("limit", v));
        }
        if let Some(v) = format {
            query.push(query_pair("format", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!(
            "/api/series/{}/{}{}",
            path_segment(series),
            index.name(),
            query_str
        );
        if format == Some(Format::CSV) {
            self.base.get_text(&path).map(FormatResponse::Csv)
        } else {
            self.base.get_json(&path).map(FormatResponse::Json)
        }
    }

    /// Get raw series data
    ///
    /// The data array without the `SeriesData` wrapper. Supports the same range and format parameters as `GET /api/series/{series}/{index}`.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/data`
    pub fn get_series_data(
        &self,
        series: SeriesName,
        index: Index,
        start: Option<RangeIndex>,
        end: Option<RangeIndex>,
        limit: Option<Limit>,
        format: Option<Format>,
    ) -> Result<FormatResponse<Vec<serde_json::Value>>> {
        let mut query = Vec::new();
        if let Some(v) = start {
            query.push(query_pair("start", v));
        }
        if let Some(v) = end {
            query.push(query_pair("end", v));
        }
        if let Some(v) = limit {
            query.push(query_pair("limit", v));
        }
        if let Some(v) = format {
            query.push(query_pair("format", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!(
            "/api/series/{}/{}/data{}",
            path_segment(series),
            index.name(),
            query_str
        );
        if format == Some(Format::CSV) {
            self.base.get_text(&path).map(FormatResponse::Csv)
        } else {
            self.base.get_json(&path).map(FormatResponse::Json)
        }
    }

    /// Get latest series value
    ///
    /// Returns the single most recent value for a series, unwrapped (not inside a SeriesData object).
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/latest`
    pub fn get_series_latest(&self, series: SeriesName, index: Index) -> Result<serde_json::Value> {
        self.base.get_json(&format!(
            "/api/series/{}/{}/latest",
            path_segment(series),
            index.name()
        ))
    }

    /// Get series data length
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/len`
    pub fn get_series_len(&self, series: SeriesName, index: Index) -> Result<i64> {
        self.base.get_json(&format!(
            "/api/series/{}/{}/len",
            path_segment(series),
            index.name()
        ))
    }

    /// Get series version
    ///
    /// Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.
    ///
    /// Endpoint: `GET /api/series/{series}/{index}/version`
    pub fn get_series_version(&self, series: SeriesName, index: Index) -> Result<u32> {
        self.base.get_json(&format!(
            "/api/series/{}/{}/version",
            path_segment(series),
            index.name()
        ))
    }

    /// Bulk series data
    ///
    /// Fetch multiple series in a single request. Supports filtering by index and date range. Returns an array of SeriesData objects. For a single series, use `get_series` instead.
    ///
    /// Endpoint: `GET /api/series/bulk`
    pub fn get_series_bulk(
        &self,
        series: SeriesList,
        index: Index,
        start: Option<RangeIndex>,
        end: Option<RangeIndex>,
        limit: Option<Limit>,
        format: Option<Format>,
    ) -> Result<FormatResponse<Vec<SeriesData>>> {
        let mut query = Vec::new();
        query.push(query_pair("series", series));
        query.push(query_pair("index", index.name()));
        if let Some(v) = start {
            query.push(query_pair("start", v));
        }
        if let Some(v) = end {
            query.push(query_pair("end", v));
        }
        if let Some(v) = limit {
            query.push(query_pair("limit", v));
        }
        if let Some(v) = format {
            query.push(query_pair("format", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/series/bulk{}", query_str);
        if format == Some(Format::CSV) {
            self.base.get_text(&path).map(FormatResponse::Csv)
        } else {
            self.base.get_json(&path).map(FormatResponse::Json)
        }
    }

    /// Available URPD cohorts
    ///
    /// Every URPD cohort once a UTXO set is published (none before): the age aggregates (`all`, `sth`, `lth`, `age_under_4m`, ...), then the age bands from the youngest (`age_under_1h`, `age_1h_to_1d`, ...).
    ///
    /// Endpoint: `GET /api/urpd`
    pub fn list_urpd_cohorts(&self) -> Result<Vec<UrpdCohort>> {
        self.base.get_json(&format!("/api/urpd"))
    }

    /// Latest URPD
    ///
    /// URPD for the latest published block; `date` echoes the date served. Fields, units, price-grid rounding and weighting as in `GET /api/urpd/{cohort}/{point}`.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}`
    pub fn get_urpd(
        &self,
        cohort: UrpdCohort,
        agg: Option<UrpdAggregation>,
        weight: Option<UrpdWeight>,
    ) -> Result<Urpd> {
        let mut query = Vec::new();
        if let Some(v) = agg {
            query.push(query_pair("agg", v));
        }
        if let Some(v) = weight {
            query.push(query_pair("weight", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/urpd/{}{}", path_segment(cohort), query_str);
        self.base.get_json(&path)
    }

    /// URPD at block height or date
    ///
    /// URPD for a cohort at a block height or the last block of a UTC day. Returns `{ height, hash, date, cohort, weight, aggregation, close, supply, buckets }` where `buckets` is columns, `{ price, supply, realized_cap, unrealized_pnl }`, one entry per bucket in ascending price: the same block header and columnar layout as `GET /api/utxo-set/{point}`. `close`, `price`, `realized_cap`, and `unrealized_pnl` are USD; `supply` is BTC. `unrealized_pnl` can be negative. Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation price on the price grid (four significant digits of the price in cents, within 0.05% of the exact price), rounded to the cent per bucket, so the buckets need not sum to the realized cap series. With a `wakefulness` or `mobility` `weight`, `supply` is the weighted supply (whole satoshis) and the bucket values follow it.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}/{point}`
    pub fn get_urpd_at(
        &self,
        cohort: UrpdCohort,
        point: HeightOrDate,
        agg: Option<UrpdAggregation>,
        weight: Option<UrpdWeight>,
    ) -> Result<Urpd> {
        let mut query = Vec::new();
        if let Some(v) = agg {
            query.push(query_pair("agg", v));
        }
        if let Some(v) = weight {
            query.push(query_pair("weight", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!(
            "/api/urpd/{}/{}{}",
            path_segment(cohort),
            path_segment(point),
            query_str
        );
        self.base.get_json(&path)
    }

    /// Difficulty adjustment
    ///
    /// Get current difficulty adjustment progress and estimates.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustment)*
    ///
    /// Endpoint: `GET /api/v1/difficulty-adjustment`
    pub fn get_difficulty_adjustment(&self) -> Result<DifficultyAdjustment> {
        self.base
            .get_json(&format!("/api/v1/difficulty-adjustment"))
    }

    /// Current BTC price
    ///
    /// Returns bitcoin latest price (on-chain derived, USD only).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-price)*
    ///
    /// Endpoint: `GET /api/v1/prices`
    pub fn get_prices(&self) -> Result<PricePoint> {
        self.base.get_json(&format!("/api/v1/prices"))
    }

    /// Historical price
    ///
    /// Completed four-hour BTC/USD closes, oldest first, labeled by interval end, from the first priced interval (2010-08-16). With a UNIX timestamp, returns the latest nonempty completed close at or before it; before the first close returns an empty list. The current partial interval is excluded. USD only (`currency` accepts `USD`); exchangeRates is empty.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-historical-price)*
    ///
    /// Endpoint: `GET /api/v1/historical-price`
    pub fn get_historical_price(
        &self,
        timestamp: Option<Timestamp>,
        currency: Option<Currency>,
    ) -> Result<HistoricalPrice> {
        let mut query = Vec::new();
        if let Some(v) = timestamp {
            query.push(query_pair("timestamp", v));
        }
        if let Some(v) = currency {
            query.push(query_pair("currency", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/v1/historical-price{}", query_str);
        self.base.get_json(&path)
    }

    /// Address hash-prefix matches
    ///
    /// Find addresses by address type and by the first 1-16 hex nibbles of RapidHash v3 over the raw address payload bytes. Intended for privacy-preserving client-side wallet discovery without sending raw addresses or xpubs. Fetch metadata with `GET /api/address/{address}`.
    ///
    /// Endpoint: `GET /api/address/hash-prefix/{address_type}/{prefix}`
    pub fn get_address_hash_prefix_matches(
        &self,
        address_type: OutputType,
        prefix: &str,
    ) -> Result<AddressHashPrefixMatches> {
        self.base.get_json(&format!(
            "/api/address/hash-prefix/{}/{}",
            path_segment(address_type),
            path_segment(prefix)
        ))
    }

    /// Address information
    ///
    /// Balance and transaction counts. Supports P2PKH, P2SH, P2WPKH, P2WSH, P2TR and P2A addresses, and P2PK public keys in hex (33 or 65 bytes).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address)*
    ///
    /// Endpoint: `GET /api/address/{address}`
    pub fn get_address(&self, address: Address) -> Result<AddressStats> {
        self.base
            .get_json(&format!("/api/address/{}", path_segment(address)))
    }

    /// Address transactions
    ///
    /// Get transaction history for an address, newest first. Returns up to 50 mempool transactions plus a confirmed page sized to fill the response to 50 total (chain floor of 25, so 25-50 confirmed depending on mempool weight). To paginate further confirmed history, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs`
    pub fn get_address_txs(&self, address: Address) -> Result<Vec<Transaction>> {
        self.base
            .get_json(&format!("/api/address/{}/txs", path_segment(address)))
    }

    /// Address confirmed transactions
    ///
    /// Get the first 25 confirmed transactions for an address. For pagination, request `GET /api/address/{address}/txs/chain/{after_txid}` with the last returned txid.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/chain`
    pub fn get_address_confirmed_txs(&self, address: Address) -> Result<Vec<Transaction>> {
        self.base
            .get_json(&format!("/api/address/{}/txs/chain", path_segment(address)))
    }

    /// Address confirmed transactions (paginated)
    ///
    /// Get the next 25 confirmed transactions strictly older than `after_txid` (Esplora-canonical pagination form, matches mempool.space).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-chain)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/chain/{after_txid}`
    pub fn get_address_confirmed_txs_after(
        &self,
        address: Address,
        after_txid: Txid,
    ) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!(
            "/api/address/{}/txs/chain/{}",
            path_segment(address),
            path_segment(after_txid)
        ))
    }

    /// Address mempool transactions
    ///
    /// Get unconfirmed transactions for an address from the mempool, newest first (up to 50).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-transactions-mempool)*
    ///
    /// Endpoint: `GET /api/address/{address}/txs/mempool`
    pub fn get_address_mempool_txs(&self, address: Address) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!(
            "/api/address/{}/txs/mempool",
            path_segment(address)
        ))
    }

    /// Address UTXOs
    ///
    /// Get unspent transaction outputs (UTXOs) for an address, including unconfirmed ones; outputs that mempool transactions spend are left out.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-utxo)*
    ///
    /// Endpoint: `GET /api/address/{address}/utxo`
    pub fn get_address_utxos(&self, address: Address) -> Result<Vec<Utxo>> {
        self.base
            .get_json(&format!("/api/address/{}/utxo", path_segment(address)))
    }

    /// Validate address
    ///
    /// Validate a Bitcoin address and get information about its type and scriptPubKey. Returns `isvalid: false` with an error message for invalid addresses.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-address-validate)*
    ///
    /// Endpoint: `GET /api/v1/validate-address/{address}`
    pub fn validate_address(&self, address: &str) -> Result<AddressValidation> {
        self.base.get_json(&format!(
            "/api/v1/validate-address/{}",
            path_segment(address)
        ))
    }

    /// Block information
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block)*
    ///
    /// Endpoint: `GET /api/block/{hash}`
    pub fn get_block(&self, hash: BlockHash) -> Result<BlockInfo> {
        self.base
            .get_json(&format!("/api/block/{}", path_segment(hash)))
    }

    /// Block (v1)
    ///
    /// Returns block details with extras by hash.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-v1)*
    ///
    /// Endpoint: `GET /api/v1/block/{hash}`
    pub fn get_block_v1(&self, hash: BlockHash) -> Result<BlockInfoV1> {
        self.base
            .get_json(&format!("/api/v1/block/{}", path_segment(hash)))
    }

    /// Block header
    ///
    /// Returns the hex-encoded 80-byte block header.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-header)*
    ///
    /// Endpoint: `GET /api/block/{hash}/header`
    pub fn get_block_header(&self, hash: BlockHash) -> Result<Hex> {
        self.base
            .get_text(&format!("/api/block/{}/header", path_segment(hash)))
            .and_then(parse_text)
    }

    /// Block hash by height
    ///
    /// Block hash as plain text.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-height)*
    ///
    /// Endpoint: `GET /api/block-height/{height}`
    pub fn get_block_by_height(&self, height: Height) -> Result<BlockHash> {
        self.base
            .get_text(&format!("/api/block-height/{}", path_segment(height)))
            .and_then(parse_text)
    }

    /// Block by timestamp
    ///
    /// Find the block with the greatest header timestamp at or before the given UNIX timestamp, choosing the earliest height on ties.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-timestamp)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/timestamp/{timestamp}`
    pub fn get_block_by_timestamp(&self, timestamp: Timestamp) -> Result<BlockTimestamp> {
        self.base.get_json(&format!(
            "/api/v1/mining/blocks/timestamp/{}",
            path_segment(timestamp)
        ))
    }

    /// Raw block
    ///
    /// Returns the raw block data in binary format.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-raw)*
    ///
    /// Endpoint: `GET /api/block/{hash}/raw`
    pub fn get_block_raw(&self, hash: BlockHash) -> Result<Vec<u8>> {
        self.base
            .get_bytes(&format!("/api/block/{}/raw", path_segment(hash)))
    }

    /// Block status
    ///
    /// Whether the block is in the best chain and, if so, its height and the hash of the next block. A block outside the best chain (orphaned or unknown) reports `in_best_chain: false`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-status)*
    ///
    /// Endpoint: `GET /api/block/{hash}/status`
    pub fn get_block_status(&self, hash: BlockHash) -> Result<BlockStatus> {
        self.base
            .get_json(&format!("/api/block/{}/status", path_segment(hash)))
    }

    /// Block tip height
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-height)*
    ///
    /// Endpoint: `GET /api/blocks/tip/height`
    pub fn get_block_tip_height(&self) -> Result<Height> {
        self.base.get_json(&format!("/api/blocks/tip/height"))
    }

    /// Block tip hash
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-tip-hash)*
    ///
    /// Endpoint: `GET /api/blocks/tip/hash`
    pub fn get_block_tip_hash(&self) -> Result<BlockHash> {
        self.base
            .get_text(&format!("/api/blocks/tip/hash"))
            .and_then(parse_text)
    }

    /// Transaction ID at index
    ///
    /// Retrieve a single transaction ID at a specific index within a block. Returns plain text txid.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-id)*
    ///
    /// Endpoint: `GET /api/block/{hash}/txid/{index}`
    pub fn get_block_txid(&self, hash: BlockHash, index: BlockTxIndex) -> Result<Txid> {
        self.base
            .get_text(&format!(
                "/api/block/{}/txid/{}",
                path_segment(hash),
                path_segment(index)
            ))
            .and_then(parse_text)
    }

    /// Block transaction IDs
    ///
    /// Transaction IDs in block order.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transaction-ids)*
    ///
    /// Endpoint: `GET /api/block/{hash}/txids`
    pub fn get_block_txids(&self, hash: BlockHash) -> Result<Vec<Txid>> {
        self.base
            .get_json(&format!("/api/block/{}/txids", path_segment(hash)))
    }

    /// Block transactions
    ///
    /// Up to 25 transactions starting at index 0.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*
    ///
    /// Endpoint: `GET /api/block/{hash}/txs`
    pub fn get_block_txs(&self, hash: BlockHash) -> Result<Vec<Transaction>> {
        self.base
            .get_json(&format!("/api/block/{}/txs", path_segment(hash)))
    }

    /// Block transactions (paginated)
    ///
    /// Up to 25 transactions starting at `start_index`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-transactions)*
    ///
    /// Endpoint: `GET /api/block/{hash}/txs/{start_index}`
    pub fn get_block_txs_from_index(
        &self,
        hash: BlockHash,
        start_index: BlockTxIndex,
    ) -> Result<Vec<Transaction>> {
        self.base.get_json(&format!(
            "/api/block/{}/txs/{}",
            path_segment(hash),
            path_segment(start_index)
        ))
    }

    /// Recent blocks
    ///
    /// The last 10 blocks.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*
    ///
    /// Endpoint: `GET /api/blocks`
    pub fn get_blocks(&self) -> Result<Vec<BlockInfo>> {
        self.base.get_json(&format!("/api/blocks"))
    }

    /// Blocks from height
    ///
    /// Retrieve up to 10 blocks going backwards from the given height. For example, height=100 returns blocks 100, 99, 98, ..., 91. Height=0 returns only block 0. A height past the tip is not found.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks)*
    ///
    /// Endpoint: `GET /api/blocks/{height}`
    pub fn get_blocks_from_height(&self, height: Height) -> Result<Vec<BlockInfo>> {
        self.base
            .get_json(&format!("/api/blocks/{}", path_segment(height)))
    }

    /// Recent blocks with extras
    ///
    /// Retrieve the last 15 blocks with extended data including pool identification and fee statistics.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*
    ///
    /// Endpoint: `GET /api/v1/blocks`
    pub fn get_blocks_v1(&self) -> Result<Vec<BlockInfoV1>> {
        self.base.get_json(&format!("/api/v1/blocks"))
    }

    /// Blocks from height with extras
    ///
    /// Retrieve up to 15 blocks with extended data going backwards from the given height: the published blocks among the 15 heights ending at it, so fewer (or none) past the tip.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-blocks-v1)*
    ///
    /// Endpoint: `GET /api/v1/blocks/{height}`
    pub fn get_blocks_v1_from_height(&self, height: Height) -> Result<Vec<BlockInfoV1>> {
        self.base
            .get_json(&format!("/api/v1/blocks/{}", path_segment(height)))
    }

    /// List all mining pools
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*
    ///
    /// Endpoint: `GET /api/v1/mining/pools`
    pub fn get_pools(&self) -> Result<Vec<PoolInfo>> {
        self.base.get_json(&format!("/api/v1/mining/pools"))
    }

    /// Mining pool statistics
    ///
    /// Get mining pool statistics for a time period.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pools)*
    ///
    /// Endpoint: `GET /api/v1/mining/pools/{time_period}`
    pub fn get_pool_stats(&self, time_period: TimePeriod) -> Result<PoolsSummary> {
        self.base.get_json(&format!(
            "/api/v1/mining/pools/{}",
            path_segment(time_period)
        ))
    }

    /// Mining pool details
    ///
    /// Block counts and mining shares by time period for one pool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}`
    pub fn get_pool(&self, slug: PoolSlug) -> Result<PoolDetail> {
        self.base
            .get_json(&format!("/api/v1/mining/pool/{}", path_segment(slug)))
    }

    /// All pools hashrate (all time)
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate/pools`
    pub fn get_pools_hashrate(&self) -> Result<Vec<PoolHashrateEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/hashrate/pools"))
    }

    /// All pools hashrate
    ///
    /// Get hashrate data for all mining pools for a time period.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrates)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate/pools/{time_period}`
    pub fn get_pools_hashrate_by_period(
        &self,
        time_period: TimePeriod,
    ) -> Result<Vec<PoolHashrateEntry>> {
        self.base.get_json(&format!(
            "/api/v1/mining/hashrate/pools/{}",
            path_segment(time_period)
        ))
    }

    /// Mining pool hashrate
    ///
    /// Get hashrate history for a specific mining pool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-hashrate)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/hashrate`
    pub fn get_pool_hashrate(&self, slug: PoolSlug) -> Result<Vec<PoolHashrateEntry>> {
        self.base.get_json(&format!(
            "/api/v1/mining/pool/{}/hashrate",
            path_segment(slug)
        ))
    }

    /// Mining pool blocks
    ///
    /// Get up to 100 recent blocks mined by a specific pool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/blocks`
    pub fn get_pool_blocks(&self, slug: PoolSlug) -> Result<Vec<BlockInfoV1>> {
        self.base.get_json(&format!(
            "/api/v1/mining/pool/{}/blocks",
            path_segment(slug)
        ))
    }

    /// Mining pool blocks before height
    ///
    /// Get up to 100 blocks mined by a specific pool strictly below the given height (pass the last height of a page to get the next one).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mining-pool-blocks)*
    ///
    /// Endpoint: `GET /api/v1/mining/pool/{slug}/blocks/{height}`
    pub fn get_pool_blocks_from(&self, slug: PoolSlug, height: Height) -> Result<Vec<BlockInfoV1>> {
        self.base.get_json(&format!(
            "/api/v1/mining/pool/{}/blocks/{}",
            path_segment(slug),
            path_segment(height)
        ))
    }

    /// Network hashrate (all time)
    ///
    /// Get network hashrate and difficulty data for all time.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate`
    pub fn get_hashrate(&self) -> Result<HashrateSummary> {
        self.base.get_json(&format!("/api/v1/mining/hashrate"))
    }

    /// Network hashrate
    ///
    /// Get network hashrate and difficulty data for a time period.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-hashrate)*
    ///
    /// Endpoint: `GET /api/v1/mining/hashrate/{time_period}`
    pub fn get_hashrate_by_period(&self, time_period: TimePeriod) -> Result<HashrateSummary> {
        self.base.get_json(&format!(
            "/api/v1/mining/hashrate/{}",
            path_segment(time_period)
        ))
    }

    /// Difficulty adjustments (all time)
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
    ///
    /// Endpoint: `GET /api/v1/mining/difficulty-adjustments`
    pub fn get_difficulty_adjustments(&self) -> Result<Vec<DifficultyAdjustmentEntry>> {
        self.base
            .get_json(&format!("/api/v1/mining/difficulty-adjustments"))
    }

    /// Difficulty adjustments
    ///
    /// Get historical difficulty adjustments for a time period.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-difficulty-adjustments)*
    ///
    /// Endpoint: `GET /api/v1/mining/difficulty-adjustments/{time_period}`
    pub fn get_difficulty_adjustments_by_period(
        &self,
        time_period: TimePeriod,
    ) -> Result<Vec<DifficultyAdjustmentEntry>> {
        self.base.get_json(&format!(
            "/api/v1/mining/difficulty-adjustments/{}",
            path_segment(time_period)
        ))
    }

    /// Mining reward statistics
    ///
    /// Get mining reward statistics for the last N blocks including total rewards, fees, and transaction count.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-reward-stats)*
    ///
    /// Endpoint: `GET /api/v1/mining/reward-stats/{block_count}`
    pub fn get_reward_stats(&self, block_count: i64) -> Result<RewardStats> {
        self.base.get_json(&format!(
            "/api/v1/mining/reward-stats/{}",
            path_segment(block_count)
        ))
    }

    /// Block fees
    ///
    /// Get average total fees per block for a time period.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-fees)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/fees/{time_period}`
    pub fn get_block_fees(&self, time_period: TimePeriod) -> Result<Vec<BlockFeesEntry>> {
        self.base.get_json(&format!(
            "/api/v1/mining/blocks/fees/{}",
            path_segment(time_period)
        ))
    }

    /// Block rewards
    ///
    /// Get average coinbase reward (subsidy + fees) per block for a time period.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-rewards)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/rewards/{time_period}`
    pub fn get_block_rewards(&self, time_period: TimePeriod) -> Result<Vec<BlockRewardsEntry>> {
        self.base.get_json(&format!(
            "/api/v1/mining/blocks/rewards/{}",
            path_segment(time_period)
        ))
    }

    /// Block fee rates
    ///
    /// Get block fee rate percentiles (min, 10th, 25th, median, 75th, 90th, max) for a time period.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-block-feerates)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/fee-rates/{time_period}`
    pub fn get_block_fee_rates(&self, time_period: TimePeriod) -> Result<Vec<BlockFeeRatesEntry>> {
        self.base.get_json(&format!(
            "/api/v1/mining/blocks/fee-rates/{}",
            path_segment(time_period)
        ))
    }

    /// Block sizes and weights
    ///
    /// Get average block sizes and weights for a time period.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-sizes-weights)*
    ///
    /// Endpoint: `GET /api/v1/mining/blocks/sizes-weights/{time_period}`
    pub fn get_block_sizes_weights(&self, time_period: TimePeriod) -> Result<BlockSizesWeights> {
        self.base.get_json(&format!(
            "/api/v1/mining/blocks/sizes-weights/{}",
            path_segment(time_period)
        ))
    }

    /// Projected mempool blocks
    ///
    /// Projected blocks for fee estimation. Block 0 reflects Bitcoin Core's actual next-block selection; blocks 1+ are a fee-tier approximation.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-blocks-fees)*
    ///
    /// Endpoint: `GET /api/v1/fees/mempool-blocks`
    pub fn get_mempool_blocks(&self) -> Result<Vec<MempoolBlock>> {
        self.base.get_json(&format!("/api/v1/fees/mempool-blocks"))
    }

    /// Recommended fees
    ///
    /// Recommended fee rates by confirmation target, in whole sat/vB.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees)*
    ///
    /// Endpoint: `GET /api/v1/fees/recommended`
    pub fn get_recommended_fees(&self) -> Result<RecommendedFees> {
        self.base.get_json(&format!("/api/v1/fees/recommended"))
    }

    /// Recommended fee rates (precise)
    ///
    /// Recommended fee rates by confirmation target, with up to three decimal places and support for sub-sat/vB rates.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-recommended-fees-precise)*
    ///
    /// Endpoint: `GET /api/v1/fees/precise`
    pub fn get_precise_fees(&self) -> Result<PreciseRecommendedFees> {
        self.base.get_json(&format!("/api/v1/fees/precise"))
    }

    /// Mempool statistics
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool)*
    ///
    /// Endpoint: `GET /api/mempool`
    pub fn get_mempool(&self) -> Result<MempoolInfo> {
        self.base.get_json(&format!("/api/mempool"))
    }

    /// Mempool content hash
    ///
    /// Returns an opaque content token for the published projected next block, including statistics and transaction bodies. This is not the HTTP ETag. An unchanged token means unchanged content, not necessarily a stalled sync loop.
    ///
    /// Endpoint: `GET /api/mempool/hash`
    pub fn get_mempool_hash(&self) -> Result<NextBlockHash> {
        self.base.get_json(&format!("/api/mempool/hash"))
    }

    /// Mempool transaction IDs
    ///
    /// Get all transaction IDs currently in the mempool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-transaction-ids)*
    ///
    /// Endpoint: `GET /api/mempool/txids`
    pub fn get_mempool_txids(&self) -> Result<Vec<Txid>> {
        self.base.get_json(&format!("/api/mempool/txids"))
    }

    /// Recent mempool transactions
    ///
    /// Get the last 10 transactions to enter the mempool.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-mempool-recent)*
    ///
    /// Endpoint: `GET /api/mempool/recent`
    pub fn get_mempool_recent(&self) -> Result<Vec<MempoolRecentTx>> {
        self.base.get_json(&format!("/api/mempool/recent"))
    }

    /// Recent RBF replacements
    ///
    /// Returns up to 25 most-recent RBF replacement trees across the whole mempool. Each entry has the shape of `replacements` in `GET /api/v1/tx/{txid}/rbf`.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-replacements)*
    ///
    /// Endpoint: `GET /api/v1/replacements`
    pub fn get_replacements(&self) -> Result<Vec<ReplacementNode>> {
        self.base.get_json(&format!("/api/v1/replacements"))
    }

    /// Recent full-RBF replacements
    ///
    /// Same response shape as `GET /api/v1/replacements`, but limited to trees where at least one predecessor was non-signaling (full-RBF).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-fullrbf-replacements)*
    ///
    /// Endpoint: `GET /api/v1/fullrbf/replacements`
    pub fn get_fullrbf_replacements(&self) -> Result<Vec<ReplacementNode>> {
        self.base.get_json(&format!("/api/v1/fullrbf/replacements"))
    }

    /// Projected next block template
    ///
    /// Bitcoin Core's `getblocktemplate` selection: full transaction bodies in GBT order with aggregate stats. The returned `hash` is an opaque content token; pass it to `GET /api/v1/mempool/block-template/diff/{hash}` to fetch deltas instead of refetching the whole template.
    ///
    /// Endpoint: `GET /api/v1/mempool/block-template`
    pub fn get_block_template(&self) -> Result<BlockTemplate> {
        self.base
            .get_json(&format!("/api/v1/mempool/block-template"))
    }

    /// Block template diff since hash
    ///
    /// Delta of the projected next block since `<hash>`. `order` is the full new template in order: each entry is either a number (index into the prior template the client cached at `<hash>`) or a transaction object (new body to insert at this position). Walk `order` once to rebuild and replace the cached `stats` with the response's; `removed` is a convenience list of txids that left so clients can evict cached bodies. After applying, use the response `hash` as `<hash>` on the next call to keep iterating. Returns `404` when `<hash>` has aged out of server history; clients should fall back to `GET /api/v1/mempool/block-template`.
    ///
    /// Endpoint: `GET /api/v1/mempool/block-template/diff/{hash}`
    pub fn get_block_template_diff(&self, hash: NextBlockHash) -> Result<BlockTemplateDiff> {
        self.base.get_json(&format!(
            "/api/v1/mempool/block-template/diff/{}",
            path_segment(hash)
        ))
    }

    /// Live BTC/USD price
    ///
    /// Current BTC/USD price in dollars, derived from on-chain round-dollar output patterns in the oracle's recent-block window, with the mempool's outputs blended in when available.
    ///
    /// Endpoint: `GET /api/mempool/price`
    pub fn get_live_price(&self) -> Result<Dollars> {
        self.base.get_json(&format!("/api/mempool/price"))
    }

    /// Txid by index
    ///
    /// Retrieve the transaction ID (txid) at a given global transaction index. Returns the txid as plain text.
    ///
    /// Endpoint: `GET /api/tx-index/{index}`
    pub fn get_tx_by_index(&self, index: TxIndex) -> Result<Txid> {
        self.base
            .get_text(&format!("/api/tx-index/{}", path_segment(index)))
            .and_then(parse_text)
    }

    /// CPFP info
    ///
    /// Returns ancestors and descendants for a CPFP (Child Pays For Parent) transaction, including the effective fee rate of the package.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-children-pay-for-parent)*
    ///
    /// Endpoint: `GET /api/v1/cpfp/{txid}`
    pub fn get_cpfp(&self, txid: Txid) -> Result<CpfpInfo> {
        self.base
            .get_json(&format!("/api/v1/cpfp/{}", path_segment(txid)))
    }

    /// RBF replacement history
    ///
    /// Returns the RBF replacement tree for a transaction, if any. Both `replacements` and `replaces` are null when the tx has no known RBF history within the mempool monitor's retention window.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-rbf-history)*
    ///
    /// Endpoint: `GET /api/v1/tx/{txid}/rbf`
    pub fn get_tx_rbf(&self, txid: Txid) -> Result<RbfResponse> {
        self.base
            .get_json(&format!("/api/v1/tx/{}/rbf", path_segment(txid)))
    }

    /// Transaction information
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction)*
    ///
    /// Endpoint: `GET /api/tx/{txid}`
    pub fn get_tx(&self, txid: Txid) -> Result<Transaction> {
        self.base
            .get_json(&format!("/api/tx/{}", path_segment(txid)))
    }

    /// Transaction hex
    ///
    /// Hex-encoded serialized transaction.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-hex)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/hex`
    pub fn get_tx_hex(&self, txid: Txid) -> Result<Hex> {
        self.base
            .get_text(&format!("/api/tx/{}/hex", path_segment(txid)))
            .and_then(parse_text)
    }

    /// Transaction merkleblock proof
    ///
    /// Get the merkleblock proof for a transaction (BIP37 format, hex encoded).
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkleblock-proof)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/merkleblock-proof`
    pub fn get_tx_merkleblock_proof(&self, txid: Txid) -> Result<Hex> {
        self.base
            .get_text(&format!("/api/tx/{}/merkleblock-proof", path_segment(txid)))
            .and_then(parse_text)
    }

    /// Transaction merkle proof
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-merkle-proof)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/merkle-proof`
    pub fn get_tx_merkle_proof(&self, txid: Txid) -> Result<MerkleProof> {
        self.base
            .get_json(&format!("/api/tx/{}/merkle-proof", path_segment(txid)))
    }

    /// Output spend status
    ///
    /// Whether the output has been spent and, if so, the spending transaction details.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspend)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/outspend/{vout}`
    pub fn get_tx_outspend(&self, txid: Txid, vout: Vout) -> Result<TxOutspend> {
        self.base.get_json(&format!(
            "/api/tx/{}/outspend/{}",
            path_segment(txid),
            path_segment(vout)
        ))
    }

    /// All output spend statuses
    ///
    /// Spend statuses in output-index order.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-outspends)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/outspends`
    pub fn get_tx_outspends(&self, txid: Txid) -> Result<Vec<TxOutspend>> {
        self.base
            .get_json(&format!("/api/tx/{}/outspends", path_segment(txid)))
    }

    /// Transaction raw
    ///
    /// Returns a transaction as binary data.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-raw)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/raw`
    pub fn get_tx_raw(&self, txid: Txid) -> Result<Vec<u8>> {
        self.base
            .get_bytes(&format!("/api/tx/{}/raw", path_segment(txid)))
    }

    /// Transaction status
    ///
    /// Retrieve the confirmation status of a transaction. Returns whether the transaction is confirmed and, if so, the block height, hash, and timestamp.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-status)*
    ///
    /// Endpoint: `GET /api/tx/{txid}/status`
    pub fn get_tx_status(&self, txid: Txid) -> Result<TxStatus> {
        self.base
            .get_json(&format!("/api/tx/{}/status", path_segment(txid)))
    }

    /// Transaction first-seen times
    ///
    /// First-seen UNIX timestamps in seconds, in request order, of transactions in the mempool or that recently left it other than by replacement (mined ones included); 0 otherwise.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#get-transaction-times)*
    ///
    /// Endpoint: `GET /api/v1/transaction-times`
    pub fn get_transaction_times(&self, txId: &[Txid]) -> Result<Vec<Timestamp>> {
        let mut query = Vec::new();
        for v in txId {
            query.push(query_pair("txId[]", v));
        }
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        let path = format!("/api/v1/transaction-times{}", query_str);
        self.base.get_json(&path)
    }

    /// Broadcast transaction
    ///
    /// Submit a raw transaction as hexadecimal text (at most 8,000,000 request bytes, including whitespace). Returns its txid as plain text. No responses are cached. Cancellation or a transport error after dispatch may leave the submission outcome unknown; do not automatically retry.
    ///
    /// *[Mempool.space docs](https://mempool.space/docs/api/rest#post-transaction)*
    ///
    /// Endpoint: `POST /api/tx`
    pub fn post_tx(&self, body: &str) -> Result<Txid> {
        self.base
            .post_text(&format!("/api/tx"), body)
            .and_then(parse_text)
    }

    /// Latest UTXO set
    ///
    /// The UTXO set after the latest published block, in the layout of `GET /api/utxo-set/{point}`.
    ///
    /// Endpoint: `GET /api/utxo-set`
    pub fn get_utxo_set_latest(&self) -> Result<UtxoSet> {
        self.base.get_json(&format!("/api/utxo-set"))
    }

    /// UTXO set at block height or date
    ///
    /// The UTXO set after a block (`840000`) or after the last published block of a UTC day (`YYYY-MM-DD`), by creation height. Returns `{ height, hash, date, count, supply, origins }`; entry `i` of `origins.count` and `origins.supply` (BTC) is what remains unspent of block `i`'s outputs.
    ///
    /// Endpoint: `GET /api/utxo-set/{point}`
    pub fn get_utxo_set(&self, point: HeightOrDate) -> Result<UtxoSet> {
        self.base
            .get_json(&format!("/api/utxo-set/{}", path_segment(point)))
    }

    /// UTXO set changes of a block or date
    ///
    /// What a block (`840000`) or a UTC day's published blocks (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: `created` has one row per block, `spent` one row per creation height, each as columnar `height`, `count` and `supply` (BTC). The set after block `first - 1` plus `created` minus `spent` is the set after block `last`.
    ///
    /// Endpoint: `GET /api/utxo-set/{point}/diff`
    pub fn get_utxo_set_diff(&self, point: HeightOrDate) -> Result<UtxoSetDiff> {
        self.base
            .get_json(&format!("/api/utxo-set/{}/diff", path_segment(point)))
    }

    /// Live BTC/USD price
    ///
    /// Current BTC/USD price in dollars. Same value as `GET /api/mempool/price`. Confirmed per-height history is available at `GET /api/series/price/height`.
    ///
    /// Endpoint: `GET /api/oracle/price`
    pub fn get_oracle_price(&self) -> Result<Dollars> {
        self.base.get_json(&format!("/api/oracle/price"))
    }

    /// Live payment output histogram
    ///
    /// Live smoothed histogram of oracle-eligible payment outputs, binned by output value on the oracle log scale. It combines the committed oracle window with the complete mempool's eligible outputs from a matching chain publication. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/payments/live`
    pub fn get_oracle_histogram_payments_live(&self) -> Result<Vec<i64>> {
        self.base
            .get_json(&format!("/api/oracle/histogram/payments/live"))
    }

    /// Payment output histogram at height or day
    ///
    /// Smoothed histogram of oracle-eligible payment outputs for a confirmed point. A block height (`840000`) gives that block's oracle payment histogram; a calendar date (`YYYY-MM-DD`) gives the average of that day's per-block payment histograms. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/payments/{point}`
    pub fn get_oracle_histogram_payments(&self, point: HeightOrDate) -> Result<Vec<i64>> {
        self.base.get_json(&format!(
            "/api/oracle/histogram/payments/{}",
            path_segment(point)
        ))
    }

    /// Live output value histogram
    ///
    /// Live unfiltered output value histogram for the complete published mempool. Nonzero outputs within the histogram's bin range are counted by value on the oracle log scale; no oracle payment filters are applied. A flat array of log-scale bins, all zero when no mempool is configured.
    ///
    /// Endpoint: `GET /api/oracle/histogram/outputs/live`
    pub fn get_oracle_histogram_outputs_live(&self) -> Result<Vec<i64>> {
        self.base
            .get_json(&format!("/api/oracle/histogram/outputs/live"))
    }

    /// Output value histogram at height or day
    ///
    /// Unfiltered output value histogram for a confirmed point. A block height (`840000`) counts that block's nonzero outputs within the histogram's bin range, coinbase included, by value on the oracle log scale; a calendar date (`YYYY-MM-DD`) sums every block that day. A flat array of log-scale bins.
    ///
    /// Endpoint: `GET /api/oracle/histogram/outputs/{point}`
    pub fn get_oracle_histogram_outputs(&self, point: HeightOrDate) -> Result<Vec<i64>> {
        self.base.get_json(&format!(
            "/api/oracle/histogram/outputs/{}",
            path_segment(point)
        ))
    }

    /// OpenAPI specification
    ///
    /// Full OpenAPI 3.1 specification for this API. Build tools from this document.
    ///
    /// Endpoint: `GET /openapi.json`
    pub fn get_openapi(&self) -> Result<serde_json::Value> {
        self.base.get_json(&format!("/openapi.json"))
    }

    /// Compact API summary
    ///
    /// Compact custom summary of the OpenAPI specification for LLM reading: types instead of schemas, `returns` instead of responses, no examples; parameters keep their enum, default and bounds. Not valid OpenAPI: build tools from `GET /openapi.json`.
    ///
    /// Endpoint: `GET /api.json`
    pub fn get_api(&self) -> Result<serde_json::Value> {
        self.base.get_json(&format!("/api.json"))
    }
}
