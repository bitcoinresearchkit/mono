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

    shape! { Cycle at "series().rarity_meter.cycle" {
        pct0_1: SeriesPattern1<Option<Dollars>> = "*_pct0_1",
        pct0_5: SeriesPattern1<Option<Dollars>> = "*_pct0_5",
        pct1: SeriesPattern1<Option<Dollars>> = "*_pct1",
        pct2: SeriesPattern1<Option<Dollars>> = "*_pct2",
        pct5: SeriesPattern1<Option<Dollars>> = "*_pct5",
        pct10: SeriesPattern1<Option<Dollars>> = "*_pct10",
        pct20: SeriesPattern1<Option<Dollars>> = "*_pct20",
        pct30: SeriesPattern1<Option<Dollars>> = "*_pct30",
        pct40: SeriesPattern1<Option<Dollars>> = "*_pct40",
        median: SeriesPattern1<Option<Dollars>> = "*_median",
        pct60: SeriesPattern1<Option<Dollars>> = "*_pct60",
        pct70: SeriesPattern1<Option<Dollars>> = "*_pct70",
        pct80: SeriesPattern1<Option<Dollars>> = "*_pct80",
        pct90: SeriesPattern1<Option<Dollars>> = "*_pct90",
        pct95: SeriesPattern1<Option<Dollars>> = "*_pct95",
        pct98: SeriesPattern1<Option<Dollars>> = "*_pct98",
        pct99: SeriesPattern1<Option<Dollars>> = "*_pct99",
        pct99_5: SeriesPattern1<Option<Dollars>> = "*_pct99_5",
        pct99_9: SeriesPattern1<Option<Dollars>> = "*_pct99_9",
        level: SeriesPattern1<Option<Score>> = "*_level",
        score: SeriesPattern1<Option<Score>> = "*_score",
    } }
    shape! { Thresholds<A> at "series().rarity_meter.extremes.realized_loss_24h.thresholds" {
        tail0_1: SeriesPattern1<A> = "*_1",
        tail0_05: SeriesPattern1<A> = "*_05",
        tail0_025: SeriesPattern1<A> = "*_025",
    } }
    shape! { RealizedLoss24h<A> at "series().rarity_meter.extremes.realized_loss_24h" {
        thresholds: Thresholds<A> = "*_threshold_tail0",
        tail_share: SeriesPattern1<Option<Percent>> = "*_tail_share",
    } }
    shape! { Extremes at "series().rarity_meter.extremes" {
        supply_in_loss: RealizedLoss24h<Option<Bitcoin>> = "*_supply_in_loss",
        realized_profit_24h: RealizedLoss24h<Option<Dollars>> = "*_realized_profit_24h",
        realized_loss_24h: RealizedLoss24h<Option<Dollars>> = "*_realized_loss_24h",
        realized_peak_regret_24h: RealizedLoss24h<Option<Dollars>> = "*_realized_peak_regret_24h",
        sell_side_risk_ratio_24h: RealizedLoss24h<Option<Ratio>> = "*_sell_side_risk_ratio_24h",
    } }
    shape! { TrueMarketMeanMedian at "series().rarity_meter.components.true_market_mean.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "true_market_mean_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "true_market_mean_rarity_*",
    } }
    shape! { VaultedPriceMedian at "series().rarity_meter.components.vaulted_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "vaulted_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "vaulted_price_rarity_*",
    } }
    shape! { MobileMedianCapitalizedPriceMedian at "series().rarity_meter.components.mobile.median_capitalized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "mobile_median_capitalized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "mobile_median_capitalized_price_rarity_*",
    } }
    shape! { MobileMedianRealizedPriceMedian at "series().rarity_meter.components.mobile.median_realized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "mobile_median_realized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "mobile_median_realized_price_rarity_*",
    } }
    shape! { MobileRealizedPriceMedian at "series().rarity_meter.components.mobile.realized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "mobile_realized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "mobile_realized_price_rarity_*",
    } }
    shape! { AwakeRealizedPriceMedian at "series().rarity_meter.components.awake.realized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "awake_realized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "awake_realized_price_rarity_*",
    } }
    shape! { Under6mCapitalizedPriceMedian at "series().rarity_meter.components.under_6m.capitalized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "utxos_under_6m_old_capitalized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "utxos_under_6m_old_capitalized_price_rarity_*",
    } }
    shape! { Under6mRealizedPriceMedian at "series().rarity_meter.components.under_6m.realized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "utxos_under_6m_old_realized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "utxos_under_6m_old_realized_price_rarity_*",
    } }
    shape! { SthMedianCapitalizedPriceMedian at "series().rarity_meter.components.sth.median_capitalized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "sth_median_capitalized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "sth_median_capitalized_price_rarity_*",
    } }
    shape! { SthMedianRealizedPriceMedian at "series().rarity_meter.components.sth.median_realized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "sth_median_realized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "sth_median_realized_price_rarity_*",
    } }
    shape! { SthCapitalizedPriceMedian at "series().rarity_meter.components.sth.capitalized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "sth_capitalized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "sth_capitalized_price_rarity_*",
    } }
    shape! { SthRealizedPriceMedian at "series().rarity_meter.components.sth.realized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "sth_realized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "sth_realized_price_rarity_*",
    } }
    shape! { AllMedianCapitalizedPriceMedian at "series().rarity_meter.components.all.median_capitalized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "median_capitalized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "median_capitalized_price_rarity_*",
    } }
    shape! { AllMedianRealizedPriceMedian at "series().rarity_meter.components.all.median_realized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "median_realized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "median_realized_price_rarity_*",
    } }
    shape! { MedianCapitalizedPrice<A> at "series().rarity_meter.components.all.median_capitalized_price" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
        pct0_1: A = "pct0_1",
        pct0_5: A = "pct0_5",
        pct1: A = "pct1",
        pct2: A = "pct2",
        pct5: A = "pct5",
        pct10: A = "pct10",
        pct20: A = "pct20",
        pct30: A = "pct30",
        pct40: A = "pct40",
        median: A = "median",
        pct60: A = "pct60",
        pct70: A = "pct70",
        pct80: A = "pct80",
        pct90: A = "pct90",
        pct95: A = "pct95",
        pct98: A = "pct98",
        pct99: A = "pct99",
        pct99_5: A = "pct99_5",
        pct99_9: A = "pct99_9",
    } }
    shape! { AllCapitalizedPriceMedian at "series().rarity_meter.components.all.capitalized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "capitalized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "capitalized_price_rarity_*",
    } }
    shape! { AllRealizedPriceMedian at "series().rarity_meter.components.all.realized_price.median" {
        ratio: SeriesPattern1<Option<Ratio>> = "realized_price_rarity_ratio_*",
        band: SeriesPattern1<Option<Dollars>> = "realized_price_rarity_*",
    } }
    shape! { ComponentsTrueMarketMean<A> at "series().rarity_meter.components.true_market_mean" {
        pct0_1: A = "pct0_1",
        pct0_5: A = "pct0_5",
        pct1: A = "pct1",
        pct2: A = "pct2",
        pct5: A = "pct5",
        pct10: A = "pct10",
        pct20: A = "pct20",
        pct30: A = "pct30",
        pct40: A = "pct40",
        median: A = "median",
        pct60: A = "pct60",
        pct70: A = "pct70",
        pct80: A = "pct80",
        pct90: A = "pct90",
        pct95: A = "pct95",
        pct98: A = "pct98",
        pct99: A = "*",
        pct99_5: A = "*_5",
        pct99_9: A = "*_9",
    } }
    shape! { ComponentsMobile at "series().rarity_meter.components.mobile" {
        realized_price: ComponentsTrueMarketMean<MobileRealizedPriceMedian> = "pct99",
        median_realized_price: MedianCapitalizedPrice<MobileMedianRealizedPriceMedian> = "*_realized_price",
        median_capitalized_price: MedianCapitalizedPrice<MobileMedianCapitalizedPriceMedian> = "*_capitalized_price",
    } }
    shape! { ComponentsAwake at "series().rarity_meter.components.awake" {
        realized_price: ComponentsTrueMarketMean<AwakeRealizedPriceMedian> = "*",
    } }
    shape! { Under6m at "series().rarity_meter.components.under_6m" {
        realized_price: ComponentsTrueMarketMean<Under6mRealizedPriceMedian> = "*",
        capitalized_price: ComponentsTrueMarketMean<Under6mCapitalizedPriceMedian> = "*",
    } }
    shape! { ComponentsAll<A, B, C, D> at "series().rarity_meter.components.all" {
        realized_price: ComponentsTrueMarketMean<A> = "pct99",
        capitalized_price: ComponentsTrueMarketMean<B> = "pct99",
        median_realized_price: MedianCapitalizedPrice<C> = "*_realized_price",
        median_capitalized_price: MedianCapitalizedPrice<D> = "*_capitalized_price",
    } }
    shape! { Components at "series().rarity_meter.components" {
        all: ComponentsAll<AllRealizedPriceMedian, AllCapitalizedPriceMedian, AllMedianRealizedPriceMedian, AllMedianCapitalizedPriceMedian> = "median",
        sth: ComponentsAll<SthRealizedPriceMedian, SthCapitalizedPriceMedian, SthMedianRealizedPriceMedian, SthMedianCapitalizedPriceMedian> = "sth_median",
        under_6m: Under6m = "*",
        awake: ComponentsAwake = "*",
        mobile: ComponentsMobile = "mobile_median",
        vaulted_price: ComponentsTrueMarketMean<VaultedPriceMedian> = "*",
        true_market_mean: ComponentsTrueMarketMean<TrueMarketMeanMedian> = "*",
    } }
    shape! { RarityMeter at "series().rarity_meter" {
        components: Components = "pct99",
        extremes: Extremes = "*",
        full: Cycle = "*",
        local: Cycle = "local_*",
        cycle: Cycle = "cycle_*",
    } }
    shape! { CostBasisAboveFloor at "series().bedrock.awake.cost_basis_above_floor" {
        pct10: SeriesPattern1<Option<Dollars>> = "*_pct10",
        pct20: SeriesPattern1<Option<Dollars>> = "*_pct20",
        pct30: SeriesPattern1<Option<Dollars>> = "*_pct30",
        pct40: SeriesPattern1<Option<Dollars>> = "*_pct40",
        median: SeriesPattern1<Option<Dollars>> = "*_median",
        pct60: SeriesPattern1<Option<Dollars>> = "*_pct60",
        pct70: SeriesPattern1<Option<Dollars>> = "*_pct70",
        pct80: SeriesPattern1<Option<Dollars>> = "*_pct80",
        pct90: SeriesPattern1<Option<Dollars>> = "*_pct90",
    } }
    shape! { Floor<A> at "series().bedrock.awake.floor" {
        pct95: SeriesPattern1<A> = "*_pct95",
        pct98: SeriesPattern1<A> = "*_pct98",
        pct99: SeriesPattern1<A> = "*_pct99",
        pct99_5: SeriesPattern1<A> = "*_pct99_5",
        pct99_9: SeriesPattern1<A> = "*_pct99_9",
    } }
    shape! { BedrockAwake at "series().bedrock.awake" {
        supply_in_loss_share_threshold: Floor<Option<Percent>> = "*_supply_in_loss_share_threshold",
        floor: Floor<Option<Dollars>> = "*_floor",
        cost_basis_above_floor: CostBasisAboveFloor = "*_cost_basis_above_floor",
    } }
    shape! { Bedrock at "series().bedrock" {
        unweighted: BedrockAwake = "*_unweighted",
        awake: BedrockAwake = "*_awake",
        mobile: BedrockAwake = "*_mobile",
    } }
    shape! { PricesTrueMarketMean at "series().cointime.prices.true_market_mean" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
        aviv: SeriesPattern1<Option<Ratio>> = "aviv",
        aviv_nupl: SeriesPattern1<Option<Ratio>> = "aviv_nupl",
    } }
    shape! { PricesActive at "series().cointime.prices.active" {
        block: SeriesPattern1<Option<Dollars>> = "*_price",
        ratio: SeriesPattern1<Option<Ratio>> = "*_price_ratio",
        mvrv: SeriesPattern1<Option<Ratio>> = "*_mvrv",
    } }
    shape! { Prices at "series().cointime.prices" {
        vaulted: PricesActive = "*",
        active: PricesActive = "active",
        true_market_mean: PricesTrueMarketMean = "true_market_mean",
        cointime: PricesActive = "cointime",
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
    shape! { SupplyInLoss at "series().coinflow.cohorts.all.mobile.supply.in_loss" {
        share: SeriesPattern1<Option<Percent>> = "*",
    } }
    shape! { Velocity at "series().supply.velocity" {
        btc: SeriesPattern1<Option<Ratio64>> = "*_btc",
        usd: SeriesPattern1<Option<Ratio64>> = "*_usd",
    } }
    shape! { Adjusted at "series().cointime.adjusted" {
        inflation_rate: SeriesPattern1<Option<Percent>> = "*_inflation_rate",
        velocity: Velocity = "*_velocity",
    } }
    shape! { Loss0To10PercentCapital at "series().profitability.all.loss_0_to_10_percent.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        share: SeriesPattern1<Option<Percent>> = "*_capital_share",
    } }
    shape! { CapitalDensity at "series().holders.all.cost_basis.capital_density" {
        total: SeriesPattern1<Option<Percent>> = "*",
        in_profit: SeriesPattern1<Option<Percent>> = "*_in_profit",
        in_loss: SeriesPattern1<Option<Percent>> = "*_in_loss",
    } }
    shape! { Avg at "series().holders.all.cost_basis.per_coin.avg" {
        block: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_avg",
        ratio: SeriesPattern1<Option<Ratio>> = "*_cost_basis_per_coin_avg_ratio",
        mvrv: SeriesPattern1<Option<Ratio>> = "*_mvrv",
    } }
    shape! { AllCostBasisPerCoin at "series().holders.all.cost_basis.per_coin" {
        avg: Avg = "*",
        realized_price: SeriesPattern1<Option<Dollars>> = "*_realized_price",
        pct5: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct5",
        pct10: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct10",
        pct15: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct15",
        pct20: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct20",
        pct25: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct25",
        pct30: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct30",
        pct35: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct35",
        pct40: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct40",
        pct45: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct45",
        median: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_median",
        pct55: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct55",
        pct60: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct60",
        pct65: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct65",
        pct70: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct70",
        pct75: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct75",
        pct80: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct80",
        pct85: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct85",
        pct90: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct90",
        pct95: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_pct95",
        median_realized_price: SeriesPattern1<Option<Dollars>> = "*_median_realized_price",
    } }
    shape! { AllUnrealized at "series().holders.all.unrealized" {
        profit: SeriesPattern1<Option<Dollars>> = "*_unrealized_profit",
        loss: SeriesPattern1<Option<Dollars>> = "*_unrealized_loss",
        net_pnl: SeriesPattern1<Option<Dollars>> = "*_net_unrealized_pnl",
        gross_pnl: SeriesPattern1<Option<Dollars>> = "*_gross_unrealized_pnl",
        nupl: SeriesPattern1<Option<Ratio>> = "*_nupl",
    } }
    shape! { CapitalInLoss at "series().holders.all.capital.in_loss" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { Total at "series().holders.all.supply.total" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "market_cap",
    } }
    shape! { ExposedCount at "series().addresses.exposed.count" {
        funded: SeriesPattern1<Count> = "*_funded_exposed_address_count",
        total: SeriesPattern1<Count> = "*_total_exposed_address_count",
    } }
    shape! { RespentCount at "series().addresses.respent.count" {
        funded: SeriesPattern1<Count> = "*_funded_respent_address_count",
        total: SeriesPattern1<Count> = "*_total_respent_address_count",
    } }
    shape! { ExposedSupply at "series().addresses.exposed.supply" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "*_usd",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { Exposed at "series().addresses.exposed" {
        count: ExposedCount = "*",
        supply: ExposedSupply = "*_exposed_address_supply",
    } }
    shape! { ReusedCount at "series().addresses.reused.count" {
        funded: SeriesPattern1<Count> = "*_funded_reused_address_count",
        total: SeriesPattern1<Count> = "*_total_reused_address_count",
    } }
    shape! { RookieCostBasisPerCoin at "series().entry.rookie.cost_basis.per_coin" {
        avg: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_coin_avg",
        realized_price: SeriesPattern1<Option<Dollars>> = "*_realized_price",
    } }
    shape! { RookieCostBasis at "series().entry.rookie.cost_basis" {
        per_coin: RookieCostBasisPerCoin = "*",
    } }
    shape! { RookieUnrealized at "series().entry.rookie.unrealized" {
        profit: SeriesPattern1<Option<Dollars>> = "*_unrealized_profit",
        loss: SeriesPattern1<Option<Dollars>> = "*_unrealized_loss",
        net_pnl: SeriesPattern1<Option<Dollars>> = "*_net_unrealized_pnl",
    } }
    shape! { DeltaRate at "series().addresses.capital.delta.rate" {
        _24h: SeriesPattern1<Option<Percent>> = "*_24h_rate",
        _1w: SeriesPattern1<Option<Percent>> = "*_1w_rate",
        _1m: SeriesPattern1<Option<Percent>> = "*_1m_rate",
        _1y: SeriesPattern1<Option<Percent>> = "*_1y_rate",
    } }
    shape! { Sma200d at "series().market.sma._200d" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
        mayer_multiple: SeriesPattern1<Option<Ratio>> = "mayer_multiple",
    } }
    shape! { Ema13d at "series().market.ema._13d" {
        block: SeriesPattern1<Option<Dollars>> = "*",
        ratio: SeriesPattern1<Option<Ratio>> = "*_ratio",
    } }
    shape! { PerDollar at "series().holders.all.cost_basis.per_dollar" {
        avg: Ema13d = "*_cost_basis_per_dollar_avg",
        capitalized_price: SeriesPattern1<Option<Dollars>> = "*_capitalized_price",
        pct5: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct5",
        pct10: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct10",
        pct15: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct15",
        pct20: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct20",
        pct25: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct25",
        pct30: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct30",
        pct35: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct35",
        pct40: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct40",
        pct45: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct45",
        median: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_median",
        pct55: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct55",
        pct60: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct60",
        pct65: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct65",
        pct70: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct70",
        pct75: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct75",
        pct80: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct80",
        pct85: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct85",
        pct90: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct90",
        pct95: SeriesPattern1<Option<Dollars>> = "*_cost_basis_per_dollar_pct95",
        median_capitalized_price: SeriesPattern1<Option<Dollars>> = "*_median_capitalized_price",
    } }
    shape! { MobileCostBasis at "series().coinflow.cohorts.all.mobile.cost_basis" {
        per_coin: AllCostBasisPerCoin = "*",
        per_dollar: PerDollar = "*",
        supply_density: CapitalDensity = "*_supply_density",
        capital_density: CapitalDensity = "*_capital_density",
    } }
    shape! { AllCostBasis at "series().holders.all.cost_basis" {
        per_coin: AllCostBasisPerCoin = "*",
        per_dollar: PerDollar = "*",
        min: SeriesPattern1<Option<Dollars>> = "*_cost_basis_min",
        max: SeriesPattern1<Option<Dollars>> = "*_cost_basis_max",
        supply_density: CapitalDensity = "*_supply_density",
        capital_density: CapitalDensity = "*_capital_density",
    } }
    shape! { Ema at "series().market.ema" {
        _1w: Ema13d = "*_1w",
        _8d: Ema13d = "*_8d",
        _13d: Ema13d = "*_13d",
        _21d: Ema13d = "*_21d",
        _1m: Ema13d = "*_1m",
        _34d: Ema13d = "*_34d",
        _55d: Ema13d = "*_55d",
        _89d: Ema13d = "*_89d",
        _144d: Ema13d = "*_144d",
        _200d: Ema13d = "*_200d",
        _1y: Ema13d = "*_1y",
        _2y: Ema13d = "*_2y",
        _200w: Ema13d = "*_200w",
        _4y: Ema13d = "*_4y",
    } }
    shape! { MarketSma at "series().market.sma" {
        _1w: Ema13d = "*_1w",
        _8d: Ema13d = "*_8d",
        _13d: Ema13d = "*_13d",
        _21d: Ema13d = "*_21d",
        _1m: Ema13d = "*_1m",
        _34d: Ema13d = "*_34d",
        _50d: Ema13d = "*_50d",
        _55d: Ema13d = "*_55d",
        _89d: Ema13d = "*_89d",
        _144d: Ema13d = "*_144d",
        _200d: Sma200d = "*_200d",
        _1y: Ema13d = "*_1y",
        _2y: Ema13d = "*_2y",
        _200w: Ema13d = "*_200w",
        _4y: Ema13d = "*_4y",
    } }
    shape! { Max at "series().market.range.max" {
        _1w: SeriesPattern1<Option<Dollars>> = "*_1w",
        _1m: SeriesPattern1<Option<Dollars>> = "*_1m",
        _1y: SeriesPattern1<Option<Dollars>> = "*_1y",
    } }
    shape! { Range at "series().market.range" {
        min: Max = "*_min",
        max: Max = "*_max",
        true_range: SeriesPattern1<Option<Dollars>> = "*_true_range",
    } }
    shape! { Returns at "series().market.returns" {
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
        max_days_between: SeriesPattern1<Option<Days>> = "max_days_between_*_ath",
    } }
    shape! { Market at "series().market" {
        ath: Ath = "*",
        returns: Returns = "*_return",
        range: Range = "*",
        sma: MarketSma = "*_sma",
        ema: Ema = "*_ema",
    } }
    shape! { UtxoSet2 at "series().utxo_set" {
        supply: SeriesPattern1<Option<Bitcoin>> = "*",
        count: SeriesPattern1<Count> = "utxo_count",
    } }
    shape! { HashratePrice<A> at "series().mining.hashrate.price" {
        block: SeriesPattern1<A> = "*",
        atl: SeriesPattern1<A> = "*_atl",
        rebound: SeriesPattern1<Option<Percent>> = "*_rebound",
    } }
    shape! { RateSma at "series().mining.hashrate.rate.sma" {
        _1w: SeriesPattern1<Option<Hashrate>> = "*_1w",
        _1m: SeriesPattern1<Option<Hashrate>> = "*_1m",
        _2m: SeriesPattern1<Option<Hashrate>> = "*_2m",
        _1y: SeriesPattern1<Option<Hashrate>> = "*_1y",
    } }
    shape! { HashrateRate at "series().mining.hashrate.rate" {
        block: SeriesPattern1<Option<Hashrate>> = "*",
        sma: RateSma = "*_sma",
        ath: SeriesPattern1<Option<Hashrate>> = "*_ath",
        drawdown: SeriesPattern1<Option<Percent>> = "*_drawdown",
    } }
    shape! { MiningHashrate at "series().mining.hashrate" {
        rate: HashrateRate = "hashrate",
        price: HashratePrice<Option<HashPrice>> = "*_price",
        value: HashratePrice<Option<HashValue>> = "*_value",
    } }
    shape! { Vsize<A> at "series().transactions.vsize" {
        tx: SeriesPattern19<A> = "*",
        min: SeriesPattern1<A> = "*_min",
        max: SeriesPattern1<A> = "*_max",
        pct10: SeriesPattern1<A> = "*_pct10",
        pct25: SeriesPattern1<A> = "*_pct25",
        median: SeriesPattern1<A> = "*_median",
        pct75: SeriesPattern1<A> = "*_pct75",
        pct90: SeriesPattern1<A> = "*_pct90",
    } }
    shape! { AvgBalance at "series().addresses.avg_balance" {
        btc: SeriesPattern1<Option<Bitcoin>> = "*",
        usd: SeriesPattern1<Option<Dollars>> = "*_usd",
    } }
    shape! { CointimeSupply at "series().cointime.supply" {
        vaulted: AvgBalance = "vaulted_*",
        hodled_or_lost: AvgBalance = "hodled_or_lost_*",
        active: AvgBalance = "active_*",
    } }
    shape! { MobileSupply at "series().coinflow.cohorts.all.mobile.supply" {
        total: AvgBalance = "*",
        in_loss: SupplyInLoss = "*_in_loss_share",
    } }
    shape! { AllMobile at "series().coinflow.cohorts.all.mobile" {
        supply: MobileSupply = "*_supply",
        capital: MobileCapital = "*",
        cost_basis: MobileCostBasis = "*",
    } }
    shape! { Immobile at "series().coinflow.age_ranges._10y_to_12y.immobile" {
        supply: AvgBalance = "*",
    } }
    shape! { CoinflowCohortsAll at "series().coinflow.cohorts.all" {
        mobile: AllMobile = "*_mobile",
        immobile: Immobile = "*_immobile_supply",
    } }
    shape! { CoinflowAgeRanges10yTo12y at "series().coinflow.age_ranges._10y_to_12y" {
        spending_rate: SeriesPattern1<Option<PerDay>> = "*_spending_rate",
        spending_exposure: SeriesPattern1<Option<Float64>> = "*_spending_exposure",
        mobility: SeriesPattern1<Option<Ratio64>> = "*_mobility",
        mobile: Immobile = "*_mobile_supply",
        immobile: Immobile = "*_immobile_supply",
    } }
    shape! { CointimeCohortsAll at "series().cointime.cohorts.all" {
        awake: AllMobile = "*_awake",
        dormant: Immobile = "*_dormant_supply",
    } }
    shape! { Loss0To10PercentSupply at "series().profitability.all.loss_0_to_10_percent.supply" {
        total: AvgBalance = "*",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { Loss0To10Percent at "series().profitability.all.loss_0_to_10_percent" {
        supply: Loss0To10PercentSupply = "*_supply",
        capital: Loss0To10PercentCapital = "*",
    } }
    shape! { ProfitabilityAll at "series().profitability.all" {
        profit_over_1000_percent: Loss0To10Percent = "*_profit_over_1000_percent",
        profit_500_to_1000_percent: Loss0To10Percent = "*_profit_500_to_1000_percent",
        profit_300_to_500_percent: Loss0To10Percent = "*_profit_300_to_500_percent",
        profit_200_to_300_percent: Loss0To10Percent = "*_profit_200_to_300_percent",
        profit_100_to_200_percent: Loss0To10Percent = "*_profit_100_to_200_percent",
        profit_90_to_100_percent: Loss0To10Percent = "*_profit_90_to_100_percent",
        profit_80_to_90_percent: Loss0To10Percent = "*_profit_80_to_90_percent",
        profit_70_to_80_percent: Loss0To10Percent = "*_profit_70_to_80_percent",
        profit_60_to_70_percent: Loss0To10Percent = "*_profit_60_to_70_percent",
        profit_50_to_60_percent: Loss0To10Percent = "*_profit_50_to_60_percent",
        profit_40_to_50_percent: Loss0To10Percent = "*_profit_40_to_50_percent",
        profit_30_to_40_percent: Loss0To10Percent = "*_profit_30_to_40_percent",
        profit_20_to_30_percent: Loss0To10Percent = "*_profit_20_to_30_percent",
        profit_10_to_20_percent: Loss0To10Percent = "*_profit_10_to_20_percent",
        profit_0_to_10_percent: Loss0To10Percent = "*_profit_0_to_10_percent",
        loss_0_to_10_percent: Loss0To10Percent = "*_loss_0_to_10_percent",
        loss_10_to_20_percent: Loss0To10Percent = "*_loss_10_to_20_percent",
        loss_20_to_30_percent: Loss0To10Percent = "*_loss_20_to_30_percent",
        loss_30_to_40_percent: Loss0To10Percent = "*_loss_30_to_40_percent",
        loss_40_to_50_percent: Loss0To10Percent = "*_loss_40_to_50_percent",
        loss_50_to_60_percent: Loss0To10Percent = "*_loss_50_to_60_percent",
        loss_60_to_70_percent: Loss0To10Percent = "*_loss_60_to_70_percent",
        loss_70_to_80_percent: Loss0To10Percent = "*_loss_70_to_80_percent",
        loss_80_to_90_percent: Loss0To10Percent = "*_loss_80_to_90_percent",
        loss_90_to_100_percent: Loss0To10Percent = "*_loss_90_to_100_percent",
    } }
    shape! { Profitability at "series().profitability" {
        all: ProfitabilityAll = "",
        sth: ProfitabilityAll = "*",
        lth: ProfitabilityAll = "lth",
    } }
    shape! { Sum at "series().op_return.fees.sum" {
        _24h: AvgBalance = "*_24h",
        _1w: AvgBalance = "*_1w",
        _1m: AvgBalance = "*_1m",
        _1y: AvgBalance = "*_1y",
    } }
    shape! { Pools175btcRewards at "series().pools._175btc.rewards" {
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
    } }
    shape! { Block at "series().op_return.fees.block" {
        btc: SeriesPattern18<Option<Bitcoin>> = "*",
        usd: SeriesPattern18<Option<Dollars>> = "*_usd",
    } }
    shape! { OpReturnValue at "series().outputs.op_return_value" {
        block: Block = "*",
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
    } }
    shape! { Balances100btcTo1kBtcActivity at "series().addresses.balances._100btc_to_1k_btc.activity" {
        transfer_volume: OpReturnValue = "*",
    } }
    shape! { TransferVolume at "series().holders.all.activity.transfer_volume" {
        block: Block = "*",
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
        in_profit: OpReturnValue = "*_in_profit",
        in_loss: OpReturnValue = "*_in_loss",
    } }
    shape! { ChainShare at "series().op_return.data_bytes.chain_share" {
        cumulative: SeriesPattern1<Option<Percent>> = "*_cumulative",
        _24h: SeriesPattern1<Option<Percent>> = "*_24h",
        _1w: SeriesPattern1<Option<Percent>> = "*_1w",
        _1m: SeriesPattern1<Option<Percent>> = "*_1m",
        _1y: SeriesPattern1<Option<Percent>> = "*_1y",
    } }
    shape! { RewardsFees at "series().mining.rewards.fees" {
        block: Block = "*",
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
        avg: Sum = "*_avg",
        min: Sum = "*_min",
        max: Sum = "*_max",
        pct10: Sum = "*_pct10",
        pct25: Sum = "*_pct25",
        median: Sum = "*_median",
        pct75: Sum = "*_pct75",
        pct90: Sum = "*_pct90",
        share: ChainShare = "fee_share",
    } }
    shape! { Subsidy at "series().mining.rewards.subsidy" {
        block: Block = "*",
        cumulative: AvgBalance = "*_cumulative",
        sum: Sum = "*_sum",
        share: ChainShare = "*_share",
    } }
    shape! { MiningRewards at "series().mining.rewards" {
        coinbase: OpReturnValue = "*",
        subsidy: Subsidy = "subsidy",
        fees: RewardsFees = "fees",
        unclaimed: OpReturnValue = "unclaimed_rewards",
    } }
    shape! { Mining at "series().mining" {
        rewards: MiningRewards = "*",
        hashrate: MiningHashrate = "hash",
    } }
    shape! { OpReturnFees at "series().op_return.fees" {
        block: Block = "*_fees",
        cumulative: AvgBalance = "*_fees_cumulative",
        sum: Sum = "*_fees_sum",
        chain_share: ChainShare = "*_fee_chain_share",
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
    shape! { InputsPerSecond<A> at "series().inputs.per_second" {
        _24h: SeriesPattern1<A> = "*_24h",
        _1w: SeriesPattern1<A> = "*_1w",
        _1m: SeriesPattern1<A> = "*_1m",
        _1y: SeriesPattern1<A> = "*_1y",
    } }
    shape! { New at "series().addresses.new" {
        block: SeriesPattern18<Count> = "*",
        sum: InputsPerSecond<Count> = "*_sum",
    } }
    shape! { Delta<A> at "series().addresses.capital.delta" {
        absolute: InputsPerSecond<A> = "*",
        rate: DeltaRate = "*",
    } }
    shape! { LthSupply at "series().holders.lth.supply" {
        total: AvgBalance = "*",
        share: SeriesPattern1<Option<Percent>> = "*_share",
        in_profit: ExposedSupply = "*_in_profit",
        in_loss: ExposedSupply = "*_in_loss",
        delta: Delta<Option<Bitcoin>> = "*_delta",
    } }
    shape! { AllCapital at "series().holders.all.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        share: SeriesPattern1<Option<Percent>> = "*_capital_share",
        in_profit: CapitalInLoss = "*_capital_in_profit",
        in_loss: CapitalInLoss = "*_capital_in_loss",
        delta: Delta<Option<Dollars>> = "*_capital_delta",
    } }
    shape! { AllSupply at "series().holders.all.supply" {
        total: Total = "circulating_*",
        share: SeriesPattern1<Option<Percent>> = "*_share",
        in_profit: ExposedSupply = "*_in_profit",
        in_loss: ExposedSupply = "*_in_loss",
        delta: Delta<Option<Bitcoin>> = "*_delta",
    } }
    shape! { AddressesCapital at "series().addresses.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        delta: Delta<Option<Dollars>> = "*_capital_delta",
        share: SeriesPattern1<Option<Percent>> = "*_capital_share",
    } }
    shape! { AddressesSupply at "series().addresses.supply" {
        total: AvgBalance = "*",
        delta: Delta<Option<Bitcoin>> = "*_delta",
        share: SeriesPattern1<Option<Percent>> = "*_share",
    } }
    shape! { RookieSupply at "series().entry.rookie.supply" {
        total: AvgBalance = "*",
        delta: Delta<Option<Bitcoin>> = "*_delta",
        share: SeriesPattern1<Option<Percent>> = "*_share",
        in_profit: AvgBalance = "*_in_profit",
        in_loss: AvgBalance = "*_in_loss",
    } }
    shape! { Funded<A, B> at "series().addresses.funded" {
        block: SeriesPattern1<A> = "*",
        delta: Delta<B> = "*_delta",
    } }
    shape! { Supply at "series().supply" {
        circulating: SeriesPattern1<Option<Bitcoin>> = "*",
        burned: OpReturnValue = "burned",
        inflation_rate: SeriesPattern1<Option<Percent>> = "inflation_rate",
        velocity: Velocity = "velocity",
        market_cap: Funded<Option<Dollars>, Option<Dollars>> = "market_cap",
    } }
    shape! { Balances0satsUtxos at "series().addresses.balances._0sats.utxos" {
        count: Funded<Count, CountSigned> = "*",
    } }
    shape! { Balances0sats at "series().addresses.balances._0sats" {
        funded: Funded<Count, CountSigned> = "*_address_count",
        utxos: Balances0satsUtxos = "*_utxo_count",
    } }
    shape! { RookieCapital at "series().entry.rookie.capital" {
        total: SeriesPattern1<Option<Dollars>> = "*_capital",
        realized_cap: SeriesPattern1<Option<Dollars>> = "*_realized_cap",
        delta: Delta<Option<Dollars>> = "*_capital_delta",
        share: SeriesPattern1<Option<Percent>> = "*_capital_share",
        in_profit: SeriesPattern1<Option<Dollars>> = "*_capital_in_profit",
        in_loss: SeriesPattern1<Option<Dollars>> = "*_capital_in_loss",
    } }
    shape! { Ranges10yTo12ySupply at "series().age.ranges._10y_to_12y.supply" {
        total: AvgBalance = "*_supply",
        delta: Delta<Option<Bitcoin>> = "*_supply_delta",
        share: SeriesPattern1<Option<Percent>> = "*_supply_share",
        in_profit: AvgBalance = "*_supply_in_profit",
        in_loss: AvgBalance = "*_supply_in_loss",
        matured: OpReturnValue = "*_matured_supply",
    } }
    shape! { BlocksMined at "series().pools._175btc.blocks_mined" {
        cumulative: SeriesPattern1<Count> = "*_cumulative",
        sum: InputsPerSecond<Count> = "*_sum",
    } }
    shape! { Pools175btc at "series().pools._175btc" {
        blocks_mined: BlocksMined = "*_blocks_mined",
        share: ChainShare = "*_share",
        rewards: Pools175btcRewards = "*_rewards",
        fees_per_block: Sum = "*_fees_per_block",
        hashrate: InputsPerSecond<Option<Hashrate>> = "*_hashrate",
    } }
    shape! { Pools at "series().pools" {
        block_pool: SeriesPattern18<PoolSlug> = "block_*",
        unknown: Pools175btc = "*_unknown",
        blockfills: Pools175btc = "*_blockfills",
        ultimuspool: Pools175btc = "*_ultimuspool",
        terrapool: Pools175btc = "*_terrapool",
        luxor: Pools175btc = "*_luxor",
        _1thash: Pools175btc = "*_1thash",
        btccom: Pools175btc = "*_btccom",
        bitfarms: Pools175btc = "*_bitfarms",
        huobipool: Pools175btc = "*_huobipool",
        wayicn: Pools175btc = "*_wayicn",
        canoepool: Pools175btc = "*_canoepool",
        btctop: Pools175btc = "*_btctop",
        bitcoincom: Pools175btc = "*_bitcoincom",
        _175btc: Pools175btc = "*_175btc",
        gbminers: Pools175btc = "*_gbminers",
        axbt: Pools175btc = "*_axbt",
        asicminer: Pools175btc = "*_asicminer",
        bitminter: Pools175btc = "*_bitminter",
        bitcoinrussia: Pools175btc = "*_bitcoinrussia",
        btcserv: Pools175btc = "*_btcserv",
        simplecoinus: Pools175btc = "*_simplecoinus",
        btcguild: Pools175btc = "*_btcguild",
        eligius: Pools175btc = "*_eligius",
        ozcoin: Pools175btc = "*_ozcoin",
        eclipsemc: Pools175btc = "*_eclipsemc",
        maxbtc: Pools175btc = "*_maxbtc",
        triplemining: Pools175btc = "*_triplemining",
        coinlab: Pools175btc = "*_coinlab",
        _50btc: Pools175btc = "*_50btc",
        ghashio: Pools175btc = "*_ghashio",
        stminingcorp: Pools175btc = "*_stminingcorp",
        bitparking: Pools175btc = "*_bitparking",
        mmpool: Pools175btc = "*_mmpool",
        polmine: Pools175btc = "*_polmine",
        kncminer: Pools175btc = "*_kncminer",
        bitalo: Pools175btc = "*_bitalo",
        f2pool: Pools175btc = "*_f2pool",
        hhtt: Pools175btc = "*_hhtt",
        megabigpower: Pools175btc = "*_megabigpower",
        mtred: Pools175btc = "*_mtred",
        nmcbit: Pools175btc = "*_nmcbit",
        yourbtcnet: Pools175btc = "*_yourbtcnet",
        givemecoins: Pools175btc = "*_givemecoins",
        braiinspool: Pools175btc = "*_braiinspool",
        antpool: Pools175btc = "*_antpool",
        multicoinco: Pools175btc = "*_multicoinco",
        bcpoolio: Pools175btc = "*_bcpoolio",
        cointerra: Pools175btc = "*_cointerra",
        kanopool: Pools175btc = "*_kanopool",
        solock: Pools175btc = "*_solock",
        ckpool: Pools175btc = "*_ckpool",
        nicehash: Pools175btc = "*_nicehash",
        bitclub: Pools175btc = "*_bitclub",
        bitcoinaffiliatenetwork: Pools175btc = "*_bitcoinaffiliatenetwork",
        btcc: Pools175btc = "*_btcc",
        bwpool: Pools175btc = "*_bwpool",
        exxbw: Pools175btc = "*_exxbw",
        bitsolo: Pools175btc = "*_bitsolo",
        bitfury: Pools175btc = "*_bitfury",
        _21inc: Pools175btc = "*_21inc",
        digitalbtc: Pools175btc = "*_digitalbtc",
        _8baochi: Pools175btc = "*_8baochi",
        mybtccoinpool: Pools175btc = "*_mybtccoinpool",
        tbdice: Pools175btc = "*_tbdice",
        hashpool: Pools175btc = "*_hashpool",
        nexious: Pools175btc = "*_nexious",
        bravomining: Pools175btc = "*_bravomining",
        hotpool: Pools175btc = "*_hotpool",
        okexpool: Pools175btc = "*_okexpool",
        bcmonster: Pools175btc = "*_bcmonster",
        _1hash: Pools175btc = "*_1hash",
        bixin: Pools175btc = "*_bixin",
        tatmaspool: Pools175btc = "*_tatmaspool",
        viabtc: Pools175btc = "*_viabtc",
        connectbtc: Pools175btc = "*_connectbtc",
        batpool: Pools175btc = "*_batpool",
        waterhole: Pools175btc = "*_waterhole",
        dcexploration: Pools175btc = "*_dcexploration",
        dcex: Pools175btc = "*_dcex",
        btpool: Pools175btc = "*_btpool",
        _58coin: Pools175btc = "*_58coin",
        bitcoinindia: Pools175btc = "*_bitcoinindia",
        shawnp0wers: Pools175btc = "*_shawnp0wers",
        phashio: Pools175btc = "*_phashio",
        rigpool: Pools175btc = "*_rigpool",
        haozhuzhu: Pools175btc = "*_haozhuzhu",
        _7pool: Pools175btc = "*_7pool",
        miningkings: Pools175btc = "*_miningkings",
        hashbx: Pools175btc = "*_hashbx",
        dpool: Pools175btc = "*_dpool",
        rawpool: Pools175btc = "*_rawpool",
        haominer: Pools175btc = "*_haominer",
        helix: Pools175btc = "*_helix",
        bitcoinukraine: Pools175btc = "*_bitcoinukraine",
        poolin: Pools175btc = "*_poolin",
        secretsuperstar: Pools175btc = "*_secretsuperstar",
        tigerpoolnet: Pools175btc = "*_tigerpoolnet",
        sigmapoolcom: Pools175btc = "*_sigmapoolcom",
        okpooltop: Pools175btc = "*_okpooltop",
        hummerpool: Pools175btc = "*_hummerpool",
        tangpool: Pools175btc = "*_tangpool",
        bytepool: Pools175btc = "*_bytepool",
        spiderpool: Pools175btc = "*_spiderpool",
        novablock: Pools175btc = "*_novablock",
        miningcity: Pools175btc = "*_miningcity",
        binancepool: Pools175btc = "*_binancepool",
        minerium: Pools175btc = "*_minerium",
        lubiancom: Pools175btc = "*_lubiancom",
        okkong: Pools175btc = "*_okkong",
        aaopool: Pools175btc = "*_aaopool",
        emcdpool: Pools175btc = "*_emcdpool",
        foundryusa: Pools175btc = "*_foundryusa",
        sbicrypto: Pools175btc = "*_sbicrypto",
        arkpool: Pools175btc = "*_arkpool",
        purebtccom: Pools175btc = "*_purebtccom",
        marapool: Pools175btc = "*_marapool",
        kucoinpool: Pools175btc = "*_kucoinpool",
        entrustcharitypool: Pools175btc = "*_entrustcharitypool",
        okminer: Pools175btc = "*_okminer",
        titan: Pools175btc = "*_titan",
        pegapool: Pools175btc = "*_pegapool",
        btcnuggets: Pools175btc = "*_btcnuggets",
        cloudhashing: Pools175btc = "*_cloudhashing",
        digitalxmintsy: Pools175btc = "*_digitalxmintsy",
        telco214: Pools175btc = "*_telco214",
        btcpoolparty: Pools175btc = "*_btcpoolparty",
        multipool: Pools175btc = "*_multipool",
        transactioncoinmining: Pools175btc = "*_transactioncoinmining",
        btcdig: Pools175btc = "*_btcdig",
        trickysbtcpool: Pools175btc = "*_trickysbtcpool",
        btcmp: Pools175btc = "*_btcmp",
        eobot: Pools175btc = "*_eobot",
        unomp: Pools175btc = "*_unomp",
        patels: Pools175btc = "*_patels",
        gogreenlight: Pools175btc = "*_gogreenlight",
        bitcoinindiapool: Pools175btc = "*_bitcoinindiapool",
        ekanembtc: Pools175btc = "*_ekanembtc",
        canoe: Pools175btc = "*_canoe",
        tiger: Pools175btc = "*_tiger",
        _1m1x: Pools175btc = "*_1m1x",
        zulupool: Pools175btc = "*_zulupool",
        secpool: Pools175btc = "*_secpool",
        ocean: Pools175btc = "*_ocean",
        whitepool: Pools175btc = "*_whitepool",
        wiz: Pools175btc = "*_wiz",
        wk057: Pools175btc = "*_wk057",
        futurebitapollosolo: Pools175btc = "*_futurebitapollosolo",
        carbonnegative: Pools175btc = "*_carbonnegative",
        portlandhodl: Pools175btc = "*_portlandhodl",
        phoenix: Pools175btc = "*_phoenix",
        neopool: Pools175btc = "*_neopool",
        maxipool: Pools175btc = "*_maxipool",
        bitfufupool: Pools175btc = "*_bitfufupool",
        gdpool: Pools175btc = "*_gdpool",
        miningdutch: Pools175btc = "*_miningdutch",
        publicpool: Pools175btc = "*_publicpool",
        miningsquared: Pools175btc = "*_miningsquared",
        innopolistech: Pools175btc = "*_innopolistech",
        btclab: Pools175btc = "*_btclab",
        parasite: Pools175btc = "*_parasite",
        redrockpool: Pools175btc = "*_redrockpool",
        est3lar: Pools175btc = "*_est3lar",
        braiinssolo: Pools175btc = "*_braiinssolo",
        solopoolcom: Pools175btc = "*_solopoolcom",
        noderunners: Pools175btc = "*_noderunners",
        dmnd: Pools175btc = "*_dmnd",
    } }
    shape! { DataBytes at "series().op_return.data_bytes" {
        block: SeriesPattern18<Bytes> = "*",
        cumulative: SeriesPattern1<Bytes> = "*_cumulative",
        sum: InputsPerSecond<Bytes> = "*_sum",
        chain_share: ChainShare = "*_chain_share",
    } }
    shape! { Nonstandard at "series().transactions.nonstandard" {
        flag: SeriesPattern19<Boolean> = "is_*",
        block: SeriesPattern18<Count> = "*_tx_count",
        cumulative: SeriesPattern1<Count> = "*_tx_count_cumulative",
        sum: InputsPerSecond<Count> = "*_tx_count_sum",
    } }
    shape! { Patterns at "series().transactions.patterns" {
        coinjoin: Nonstandard = "*",
        consolidation: Nonstandard = "consolidation",
        batch_payout: Nonstandard = "batch_payout",
    } }
    shape! { TransactionsFees at "series().transactions.fees" {
        fee: Vsize<Sats> = "tx_*",
        fee_rate: SeriesPattern19<Option<FeeRate>> = "*_rate",
        effective_fee_rate: Vsize<Option<FeeRate>> = "effective_*_rate",
        cpfp_parent: Nonstandard = "cpfp_parent",
        cpfp_child: Nonstandard = "cpfp_child",
    } }
    shape! { TransactionsFeaturesAnnex at "series().transactions.features.annex" {
        flag: SeriesPattern19<Boolean> = "has_*",
        block: SeriesPattern18<Count> = "*_tx_count",
        cumulative: SeriesPattern1<Count> = "*_tx_count_cumulative",
        sum: InputsPerSecond<Count> = "*_tx_count_sum",
    } }
    shape! { Size<A, B> at "series().blocks.size" {
        block: SeriesPattern18<A> = "*",
        cumulative: SeriesPattern1<A> = "*_cumulative",
        sum: InputsPerSecond<A> = "*_sum",
        avg: InputsPerSecond<B> = "*_avg",
        min: InputsPerSecond<A> = "*_min",
        max: InputsPerSecond<A> = "*_max",
        pct10: InputsPerSecond<A> = "*_pct10",
        pct25: InputsPerSecond<A> = "*_pct25",
        median: InputsPerSecond<A> = "*_median",
        pct75: InputsPerSecond<A> = "*_pct75",
        pct90: InputsPerSecond<A> = "*_pct90",
    } }
    shape! { Interval<A, B> at "series().blocks.interval" {
        block: SeriesPattern18<A> = "*",
        avg: InputsPerSecond<B> = "*_avg",
    } }
    shape! { AddressesActivity at "series().addresses.activity" {
        reactivated: Interval<Count, Option<CountFract>> = "*_reactivated_address_count",
        sending: Interval<Count, Option<CountFract>> = "*_sending_address_count",
        receiving: Interval<Count, Option<CountFract>> = "*_receiving_address_count",
        bidirectional: Interval<Count, Option<CountFract>> = "*_bidirectional_address_count",
        active: Interval<Count, Option<CountFract>> = "*_active_address_count",
    } }
    shape! { BlocksCount<A> at "series().blocks.count" {
        block: SeriesPattern18<A> = "*",
        cumulative: SeriesPattern1<A> = "*_cumulative",
        sum: InputsPerSecond<A> = "*_sum",
    } }
    shape! { ReserveRisk at "series().cointime.reserve_risk" {
        block: SeriesPattern1<Option<Float64>> = "*",
        hodl_bank: SeriesPattern1<Option<Float64>> = "hodl_bank",
        vocdd: BlocksCount<Option<DollarDaysPerCoin>> = "vocdd",
    } }
    shape! { Value at "series().cointime.value" {
        destroyed: BlocksCount<Option<DollarBlocks>> = "*_destroyed",
        created: BlocksCount<Option<DollarBlocks>> = "*_created",
        stored: BlocksCount<Option<DollarBlocks>> = "*_stored",
    } }
    shape! { CointimeAgeRanges10yTo12y at "series().cointime.age_ranges._10y_to_12y" {
        coindays_created: BlocksCount<Option<CoinDays>> = "*_coindays_created",
        coindays_consumed: BlocksCount<Option<CoinDays>> = "*_coindays_consumed",
        wakefulness: SeriesPattern1<Option<Ratio64>> = "*_wakefulness",
        awake_to_dormant: SeriesPattern1<Option<Ratio64>> = "*_awake_to_dormant",
        awake: Immobile = "*_awake_supply",
        dormant: Immobile = "*_dormant_supply",
    } }
    shape! { CointimeActivity at "series().cointime.activity" {
        coinblocks_created: BlocksCount<Option<CoinBlocks>> = "*_created",
        coinblocks_destroyed: BlocksCount<Option<CoinBlocks>> = "*_destroyed",
        coinblocks_stored: BlocksCount<Option<CoinBlocks>> = "*_stored",
        liveliness: SeriesPattern1<Option<Ratio64>> = "liveliness",
        vaultedness: SeriesPattern1<Option<Ratio64>> = "vaultedness",
        liveliness_to_vaultedness: SeriesPattern1<Option<Ratio64>> = "liveliness_to_vaultedness",
        concurrent_liveliness: InputsPerSecond<Option<Ratio>> = "concurrent_liveliness",
    } }
    shape! { AdjustedSopr at "series().holders.all.realized.adjusted_sopr" {
        _24h: SeriesPattern1<Option<Ratio>> = "*_sopr_24h",
        _1w: SeriesPattern1<Option<Ratio>> = "*_sopr_1w",
        _1m: SeriesPattern1<Option<Ratio>> = "*_sopr_1m",
        _1y: SeriesPattern1<Option<Ratio>> = "*_sopr_1y",
        value_created: BlocksCount<Option<Dollars>> = "*_value_created",
        value_destroyed: BlocksCount<Option<Dollars>> = "*_value_destroyed",
    } }
    shape! { AllRealized at "series().holders.all.realized" {
        profit: BlocksCount<Option<Dollars>> = "*_realized_profit",
        loss: BlocksCount<Option<Dollars>> = "*_realized_loss",
        net_pnl: BlocksCount<Option<Dollars>> = "*_net_realized_pnl",
        gross_pnl: BlocksCount<Option<Dollars>> = "*_gross_realized_pnl",
        profit_to_loss: InputsPerSecond<Option<Ratio>> = "*_realized_profit_to_loss",
        value_destroyed: BlocksCount<Option<Dollars>> = "*_value_destroyed",
        peak_regret: BlocksCount<Option<Dollars>> = "*_realized_peak_regret",
        sopr: InputsPerSecond<Option<Ratio>> = "*_sopr",
        adjusted_sopr: AdjustedSopr = "*_adjusted",
        sell_side_risk_ratio: InputsPerSecond<Option<Ratio>> = "*_sell_side_risk_ratio",
    } }
    shape! { AllActivity at "series().holders.all.activity" {
        transfer_volume: TransferVolume = "*_transfer_volume",
        coindays_destroyed: BlocksCount<Option<CoinDays>> = "*_coindays_destroyed",
        dormancy: InputsPerSecond<Option<Days>> = "*_dormancy",
    } }
    shape! { EventsActive<A> at "series().addresses.respent.events.active" {
        count: A = "*_count",
        share: ChainShare = "*_share",
    } }
    shape! { RespentEvents at "series().addresses.respent.events" {
        outputs: EventsActive<BlocksCount<Count>> = "*_output_to_respent_address",
        inputs: EventsActive<BlocksCount<Count>> = "*_input_from_respent_address",
        active: EventsActive<Interval<Count, Option<CountFract>>> = "*_active_respent_address",
    } }
    shape! { Respent at "series().addresses.respent" {
        count: RespentCount = "*",
        supply: ExposedSupply = "*_respent_address_supply",
        events: RespentEvents = "*",
    } }
    shape! { ReusedEvents at "series().addresses.reused.events" {
        outputs: EventsActive<BlocksCount<Count>> = "*_output_to_reused_address",
        inputs: EventsActive<BlocksCount<Count>> = "*_input_from_reused_address",
        active: EventsActive<Interval<Count, Option<CountFract>>> = "*_active_reused_address",
    } }
    shape! { Reused at "series().addresses.reused" {
        count: ReusedCount = "*",
        supply: ExposedSupply = "*_reused_address_supply",
        events: ReusedEvents = "*",
    } }
    shape! { P2pk at "series().addresses.types.p2pk" {
        funded: Funded<Count, CountSigned> = "*_funded_address_count",
        empty: SeriesPattern1<Count> = "*_empty_address_count",
        total: SeriesPattern1<Count> = "*_total_address_count",
        new: New = "*_new_address_count",
        activity: AddressesActivity = "*",
        avg_balance: AvgBalance = "*_avg_address_balance",
        reused: Reused = "*",
        respent: Respent = "*",
        exposed: Exposed = "*",
    } }
    shape! { AddressesTypes at "series().addresses.types" {
        p2pk: P2pk = "*",
        p2pkh: P2pk = "p2pkh",
        p2sh: P2pk = "p2sh",
        p2wpkh: P2pk = "p2wpkh",
        p2wsh: P2pk = "p2wsh",
        p2tr: P2pk = "p2tr",
    } }
    shape! { Balances100btcTo1kBtcRealized at "series().addresses.balances._100btc_to_1k_btc.realized" {
        profit: BlocksCount<Option<Dollars>> = "*_profit",
        loss: BlocksCount<Option<Dollars>> = "*_loss",
    } }
    shape! { Balances100btcTo1kBtc at "series().addresses.balances._100btc_to_1k_btc" {
        funded: Funded<Count, CountSigned> = "*_address_count",
        supply: AddressesSupply = "*_supply",
        capital: AddressesCapital = "*",
        utxos: Balances0satsUtxos = "*_utxo_count",
        activity: Balances100btcTo1kBtcActivity = "*_transfer_volume",
        realized: Balances100btcTo1kBtcRealized = "*_realized",
    } }
    shape! { RookieActivity at "series().entry.rookie.activity" {
        transfer_volume: OpReturnValue = "*_transfer_volume",
        coindays_destroyed: BlocksCount<Option<CoinDays>> = "*_coindays_destroyed",
    } }
    shape! { RookieRealized at "series().entry.rookie.realized" {
        profit: BlocksCount<Option<Dollars>> = "*_realized_profit",
        loss: BlocksCount<Option<Dollars>> = "*_realized_loss",
        net_pnl: BlocksCount<Option<Dollars>> = "*_net_realized_pnl",
        value_destroyed: BlocksCount<Option<Dollars>> = "*_value_destroyed",
    } }
    shape! { Ranges10yTo12yActivity at "series().age.ranges._10y_to_12y.activity" {
        transfer_volume: TransferVolume = "*_transfer_volume",
        coindays_destroyed: BlocksCount<Option<CoinDays>> = "*_coindays_destroyed",
        coindays_created: BlocksCount<Option<CoinDays>> = "*_coindays_created",
    } }
    shape! { RookieUtxos at "series().entry.rookie.utxos" {
        count: Funded<Count, CountSigned> = "*_utxo_count",
        spent_count: BlocksCount<Count> = "*_spent_utxo_count",
    } }
    shape! { Amounts100btcTo1kBtc at "series().utxos.amounts._100btc_to_1k_btc" {
        supply: AddressesSupply = "*_supply",
        capital: AddressesCapital = "*",
        utxos: RookieUtxos = "*",
        activity: Balances100btcTo1kBtcActivity = "*_transfer_volume",
        realized: Balances100btcTo1kBtcRealized = "*_realized",
        cost_basis: RookieCostBasis = "*",
    } }
    shape! { UtxosTypes at "series().utxos.types" {
        p2pk: Amounts100btcTo1kBtc = "p2pk",
        p2pkh: Amounts100btcTo1kBtc = "p2pkh",
        p2ms: Amounts100btcTo1kBtc = "p2ms",
        p2sh: Amounts100btcTo1kBtc = "p2sh",
        p2wpkh: Amounts100btcTo1kBtc = "p2wpkh",
        p2wsh: Amounts100btcTo1kBtc = "p2wsh",
        p2tr: Amounts100btcTo1kBtc = "p2tr",
        p2a: Amounts100btcTo1kBtc = "p2a",
        unknown: Amounts100btcTo1kBtc = "unknown_*",
        empty: Amounts100btcTo1kBtc = "empty_*",
    } }
    shape! { Amounts0sats at "series().utxos.amounts._0sats" {
        utxos: RookieUtxos = "*",
    } }
    shape! { Balances<A, B> at "series().addresses.balances" {
        _0sats: A = "*_0sats",
        _1sat_to_10sats: B = "*_1sat_to_10sats",
        _10sats_to_100sats: B = "*_10sats_to_100sats",
        _100sats_to_1k_sats: B = "*_100sats_to_1k_sats",
        _1k_sats_to_10k_sats: B = "*_1k_sats_to_10k_sats",
        _10k_sats_to_100k_sats: B = "*_10k_sats_to_100k_sats",
        _100k_sats_to_1m_sats: B = "*_100k_sats_to_1m_sats",
        _1m_sats_to_10m_sats: B = "*_1m_sats_to_10m_sats",
        _10m_sats_to_1btc: B = "*_10m_sats_to_1btc",
        _1btc_to_10btc: B = "*_1btc_to_10btc",
        _10btc_to_100btc: B = "*_10btc_to_100btc",
        _100btc_to_1k_btc: B = "*_100btc_to_1k_btc",
        _1k_btc_to_10k_btc: B = "*_1k_btc_to_10k_btc",
        _10k_btc_to_100k_btc: B = "*_10k_btc_to_100k_btc",
        over_100k_btc: B = "*_over_100k_btc",
    } }
    shape! { Addresses at "series().addresses" {
        funded: Funded<Count, CountSigned> = "funded_*_count",
        empty: SeriesPattern1<Count> = "empty_*_count",
        total: SeriesPattern1<Count> = "total_*_count",
        new: New = "new_*_count",
        activity: AddressesActivity = "",
        avg_balance: AvgBalance = "avg_*_balance",
        reused: Reused = "",
        respent: Respent = "",
        exposed: Exposed = "",
        supply: AddressesSupply = "*_supply",
        capital: AddressesCapital = "*",
        types: AddressesTypes = "p2pk",
        balances: Balances<Balances0sats, Balances100btcTo1kBtc> = "balance",
    } }
    shape! { Utxos at "series().utxos" {
        amounts: Balances<Amounts0sats, Amounts100btcTo1kBtc> = "*",
        types: UtxosTypes = "output",
    } }
    shape! { Rookie<A, B, C, D, E, F> at "series().entry.rookie" {
        supply: A = "*_supply",
        capital: B = "*",
        utxos: RookieUtxos = "*",
        activity: C = "*",
        realized: D = "*",
        unrealized: E = "*",
        cost_basis: F = "*",
    } }
    shape! { Entry at "series().entry" {
        veteran: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*",
        rookie: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "rookie",
    } }
    shape! { Holders<A, B> at "series().holders" {
        all: A = "",
        sth: B = "sth",
        lth: B = "lth",
        under_4m: B = "*_under_4m_old",
        under_6m: B = "*_under_6m_old",
        over_4m: B = "*_over_4m_old",
        over_6m: B = "*_over_6m_old",
    } }
    shape! { Vintages at "series().age.vintages" {
        _2009: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2009",
        _2010: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2010",
        _2011: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2011",
        _2012: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2012",
        _2013: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2013",
        _2014: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2014",
        _2015: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2015",
        _2016: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2016",
        _2017: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2017",
        _2018: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2018",
        _2019: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2019",
        _2020: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2020",
        _2021: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2021",
        _2022: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2022",
        _2023: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2023",
        _2024: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2024",
        _2025: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2025",
        _2026: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2026",
        _2027: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2027",
    } }
    shape! { HalvingEpochs at "series().age.halving_epochs" {
        _0: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_0",
        _1: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_1",
        _2: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_2",
        _3: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_3",
        _4: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_4",
        _5: Rookie<RookieSupply, RookieCapital, RookieActivity, RookieRealized, RookieUnrealized, RookieCostBasis> = "*_5",
    } }
    shape! { Ranges10yTo12y at "series().age.ranges._10y_to_12y" {
        supply: Ranges10yTo12ySupply = "*",
        capital: RookieCapital = "*",
        utxos: RookieUtxos = "*",
        activity: Ranges10yTo12yActivity = "*",
        realized: RookieRealized = "*",
        unrealized: RookieUnrealized = "*",
        cost_basis: RookieCostBasis = "*",
    } }
    shape! { Ranges<A> at "series().age.ranges" {
        under_1h: A = "*_under_1h_old",
        _1h_to_1d: A = "*_1h_to_1d_old",
        _1d_to_1w: A = "*_1d_to_1w_old",
        _1w_to_1m: A = "*_1w_to_1m_old",
        _1m_to_2m: A = "*_1m_to_2m_old",
        _2m_to_3m: A = "*_2m_to_3m_old",
        _3m_to_4m: A = "*_3m_to_4m_old",
        _4m_to_5m: A = "*_4m_to_5m_old",
        _5m_to_6m: A = "*_5m_to_6m_old",
        _6m_to_9m: A = "*_6m_to_9m_old",
        _9m_to_1y: A = "*_9m_to_1y_old",
        _1y_to_18m: A = "*_1y_to_18m_old",
        _18m_to_2y: A = "*_18m_to_2y_old",
        _2y_to_3y: A = "*_2y_to_3y_old",
        _3y_to_4y: A = "*_3y_to_4y_old",
        _4y_to_5y: A = "*_4y_to_5y_old",
        _5y_to_6y: A = "*_5y_to_6y_old",
        _6y_to_7y: A = "*_6y_to_7y_old",
        _7y_to_8y: A = "*_7y_to_8y_old",
        _8y_to_10y: A = "*_8y_to_10y_old",
        _10y_to_12y: A = "*_10y_to_12y_old",
        _12y_to_15y: A = "*_12y_to_15y_old",
        over_15y: A = "*_over_15y_old",
    } }
    shape! { Coinflow at "series().coinflow" {
        age_ranges: Ranges<CoinflowAgeRanges10yTo12y> = "*",
        cohorts: Holders<CoinflowCohortsAll, CoinflowCohortsAll> = "*",
    } }
    shape! { Cointime at "series().cointime" {
        activity: CointimeActivity = "coinblocks",
        age_ranges: Ranges<CointimeAgeRanges10yTo12y> = "*",
        cohorts: Holders<CointimeCohortsAll, CointimeCohortsAll> = "*",
        supply: CointimeSupply = "supply",
        value: Value = "cointime_value",
        caps: Caps = "cap",
        prices: Prices = "vaulted",
        adjusted: Adjusted = "cointime_adjusted",
        reserve_risk: ReserveRisk = "reserve_risk",
    } }
    shape! { Age at "series().age" {
        ranges: Ranges<Ranges10yTo12y> = "*",
        halving_epochs: HalvingEpochs = "halving_epoch",
        vintages: Vintages = "vintage",
    } }
    shape! { Multiple at "series().op_return.policies.multiple" {
        output_count: BlocksCount<Count> = "*_output_count",
        data_bytes: DataBytes = "*_data_bytes",
        tx_count: BlocksCount<Count> = "*_tx_count",
        tx_vsize: BlocksCount<VSize> = "*_tx_vsize",
        fees: OpReturnFees = "*",
    } }
    shape! { Policies at "series().op_return.policies" {
        pre_v30_standard: Multiple = "*_pre_v30_standard",
        pre_v30_nonstandard: Multiple = "*_pre_v30_nonstandard",
        oversized: Multiple = "*_oversized",
        multiple: Multiple = "*_multiple",
    } }
    shape! { Protocols at "series().op_return.protocols" {
        runes: Multiple = "*_runes",
        witness_commitment: Multiple = "*_witness_commitment",
        veri_block: Multiple = "*_veri_block",
        omni: Multiple = "*_omni",
        stacks: Multiple = "*_stacks",
        blockstack: Multiple = "*_blockstack",
        colu: Multiple = "*_colu",
        open_assets: Multiple = "*_open_assets",
        komodo: Multiple = "*_komodo",
        coin_spark: Multiple = "*_coin_spark",
        poet: Multiple = "*_poet",
        docproof: Multiple = "*_docproof",
        open_timestamps: Multiple = "*_open_timestamps",
        factom: Multiple = "*_factom",
        eternity_wall: Multiple = "*_eternity_wall",
        memo: Multiple = "*_memo",
        bitproof: Multiple = "*_bitproof",
        ascribe: Multiple = "*_ascribe",
        stampery: Multiple = "*_stampery",
        epobc: Multiple = "*_epobc",
        bare_hash: Multiple = "*_bare_hash",
        text: Multiple = "*_text",
        empty: Multiple = "*_empty",
        unknown: Multiple = "*_unknown",
    } }
    shape! { OpReturn at "series().op_return" {
        flag: SeriesPattern19<Boolean> = "has_*",
        output_count: BlocksCount<Count> = "*_output_count",
        data_bytes: DataBytes = "*_data_bytes",
        tx_count: BlocksCount<Count> = "*_tx_count",
        tx_vsize: BlocksCount<VSize> = "*_tx_vsize",
        fees: OpReturnFees = "*",
        protocols: Protocols = "*",
        policies: Policies = "*",
    } }
    shape! { Inscription at "series().transactions.inscription" {
        flag: SeriesPattern19<Boolean> = "has_*",
        count: BlocksCount<Count> = "*_tx_count",
        fees: OpReturnFees = "*",
    } }
    shape! { TransactionsFeatures at "series().transactions.features" {
        p2pk: TransactionsFeaturesAnnex = "p2pk",
        p2ms: TransactionsFeaturesAnnex = "p2ms",
        p2pkh: TransactionsFeaturesAnnex = "p2pkh",
        p2sh: TransactionsFeaturesAnnex = "p2sh",
        p2wpkh: TransactionsFeaturesAnnex = "p2wpkh",
        p2wsh: TransactionsFeaturesAnnex = "p2wsh",
        p2tr: TransactionsFeaturesAnnex = "p2tr",
        p2a: TransactionsFeaturesAnnex = "p2a",
        empty_script: TransactionsFeaturesAnnex = "empty_script",
        unknown_script: TransactionsFeaturesAnnex = "unknown_script",
        fake_pubkey: TransactionsFeaturesAnnex = "fake_pubkey",
        fake_scripthash: TransactionsFeaturesAnnex = "fake_scripthash",
        segwit: BlocksCount<Count> = "segwit_tx_count",
        annex: TransactionsFeaturesAnnex = "annex",
        sighash_all: TransactionsFeaturesAnnex = "*_all",
        sighash_none: TransactionsFeaturesAnnex = "*_none",
        sighash_single: TransactionsFeaturesAnnex = "*_single",
        sighash_default: TransactionsFeaturesAnnex = "*_default",
        sighash_anyone_can_pay: TransactionsFeaturesAnnex = "*_anyone_can_pay",
        explicitly_rbf: Nonstandard = "explicitly_rbf",
        dust_output: TransactionsFeaturesAnnex = "dust_output",
        one_input: BlocksCount<Count> = "one_input_tx_count",
        one_output: BlocksCount<Count> = "one_output_tx_count",
    } }
    shape! { TypesEmpty at "series().inputs.types.empty" {
        count: BlocksCount<Count> = "*_count",
        share: ChainShare = "*_share",
        tx_count: BlocksCount<Count> = "*_tx_count",
        tx_share: ChainShare = "*_tx_share",
    } }
    shape! { OutputsTypes at "series().outputs.types" {
        p2pk: TypesEmpty = "p2pk_*",
        p2pkh: TypesEmpty = "p2pkh_*",
        p2ms: TypesEmpty = "p2ms_*",
        p2sh: TypesEmpty = "p2sh_*",
        p2wpkh: TypesEmpty = "p2wpkh_*",
        p2wsh: TypesEmpty = "p2wsh_*",
        p2tr: TypesEmpty = "p2tr_*",
        p2a: TypesEmpty = "p2a_*",
        unknown: TypesEmpty = "unknown_*",
        empty: TypesEmpty = "empty_*",
        op_return: TypesEmpty = "op_return_*",
    } }
    shape! { Outputs at "series().outputs" {
        spending_txin_index: SeriesPattern23<TxInIndex> = "spending_txin_index",
        count: Size<Count, Option<CountFract>> = "*_count",
        per_second: InputsPerSecond<Option<PerSecond>> = "outputs_per_second",
        spendable_count: BlocksCount<Count> = "spendable_*_count",
        types: OutputsTypes = "*",
        op_return_value: OpReturnValue = "op_return_value",
    } }
    shape! { InputsTypes at "series().inputs.types" {
        p2pk: TypesEmpty = "p2pk_*",
        p2pkh: TypesEmpty = "p2pkh_*",
        p2ms: TypesEmpty = "p2ms_*",
        p2sh: TypesEmpty = "p2sh_*",
        p2wpkh: TypesEmpty = "p2wpkh_*",
        p2wsh: TypesEmpty = "p2wsh_*",
        p2tr: TypesEmpty = "p2tr_*",
        p2a: TypesEmpty = "p2a_*",
        unknown: TypesEmpty = "unknown_*",
        empty: TypesEmpty = "empty_*",
    } }
    shape! { Inputs at "series().inputs" {
        count: Size<Count, Option<CountFract>> = "*_count",
        per_second: InputsPerSecond<Option<PerSecond>> = "inputs_per_second",
        types: InputsTypes = "*",
    } }
    shape! { Blocks at "series().blocks" {
        count: BlocksCount<Count> = "*_count",
        interval: Interval<Seconds, Option<SecondsFract>> = "*_interval",
        size: Size<Bytes, Option<BytesFract>> = "*_size",
        weight: Size<Weight, Option<WeightFract>> = "*_weight",
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
    shape! { MappingsTimestamp at "series().mappings.timestamp" {
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
    shape! { OutputsOpReturn at "series().mappings.outputs.op_return" {
        identity: SeriesPattern25<OpReturnIndex> = "*",
    } }
    shape! { OutputsUnknown at "series().mappings.outputs.unknown" {
        identity: SeriesPattern35<UnknownOutputIndex> = "*",
    } }
    shape! { OutputsEmpty at "series().mappings.outputs.empty" {
        identity: SeriesPattern24<EmptyOutputIndex> = "*",
    } }
    shape! { OutputsP2ms at "series().mappings.outputs.p2ms" {
        identity: SeriesPattern27<P2MSOutputIndex> = "*",
    } }
    shape! { MappingsOutputs at "series().mappings.outputs" {
        p2ms: OutputsP2ms = "p2ms_output_*",
        empty: OutputsEmpty = "empty_output_*",
        unknown: OutputsUnknown = "unknown_output_*",
        op_return: OutputsOpReturn = "op_return_*",
    } }
    shape! { MappingsAddressesP2a at "series().mappings.addresses.p2a" {
        identity: SeriesPattern26<P2AAddressIndex> = "*_index",
        address: SeriesPattern26<Address> = "*",
    } }
    shape! { MappingsAddressesP2wsh at "series().mappings.addresses.p2wsh" {
        identity: SeriesPattern34<P2WSHAddressIndex> = "*_index",
        address: SeriesPattern34<Address> = "*",
    } }
    shape! { MappingsAddressesP2wpkh at "series().mappings.addresses.p2wpkh" {
        identity: SeriesPattern33<P2WPKHAddressIndex> = "*_index",
        address: SeriesPattern33<Address> = "*",
    } }
    shape! { MappingsAddressesP2tr at "series().mappings.addresses.p2tr" {
        identity: SeriesPattern32<P2TRAddressIndex> = "*_index",
        address: SeriesPattern32<Address> = "*",
    } }
    shape! { MappingsAddressesP2sh at "series().mappings.addresses.p2sh" {
        identity: SeriesPattern31<P2SHAddressIndex> = "*_index",
        address: SeriesPattern31<Address> = "*",
    } }
    shape! { MappingsAddressesP2pkh at "series().mappings.addresses.p2pkh" {
        identity: SeriesPattern30<P2PKHAddressIndex> = "*_index",
        address: SeriesPattern30<Address> = "*",
    } }
    shape! { MappingsAddressesP2pk65 at "series().mappings.addresses.p2pk65" {
        identity: SeriesPattern29<P2PK65AddressIndex> = "*_index",
        address: SeriesPattern29<Address> = "*",
    } }
    shape! { MappingsAddressesP2pk33 at "series().mappings.addresses.p2pk33" {
        identity: SeriesPattern28<P2PK33AddressIndex> = "*_index",
        address: SeriesPattern28<Address> = "*",
    } }
    shape! { MappingsAddresses at "series().mappings.addresses" {
        p2pk33: MappingsAddressesP2pk33 = "p2pk33_*",
        p2pk65: MappingsAddressesP2pk65 = "p2pk65_*",
        p2pkh: MappingsAddressesP2pkh = "p2pkh_*",
        p2sh: MappingsAddressesP2sh = "p2sh_*",
        p2tr: MappingsAddressesP2tr = "p2tr_*",
        p2wpkh: MappingsAddressesP2wpkh = "p2wpkh_*",
        p2wsh: MappingsAddressesP2wsh = "p2wsh_*",
        p2a: MappingsAddressesP2a = "p2a_*",
    } }
    shape! { Mappings at "series().mappings" {
        addresses: MappingsAddresses = "address",
        outputs: MappingsOutputs = "index",
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
        timestamp: MappingsTimestamp = "timestamp",
    } }
    shape! { IndexerOpReturn at "series().indexer.op_return" {
        first_index: SeriesPattern18<OpReturnIndex> = "first_*_index",
        tx_index: SeriesPattern25<TxIndex> = "*_tx_index",
        protocol: SeriesPattern25<OpReturnKind> = "*_protocol",
        post_op_return_bytes: SeriesPattern25<Bytes> = "post_*_bytes",
    } }
    shape! { ScriptsUnknown at "series().indexer.scripts.unknown" {
        first_index: SeriesPattern18<UnknownOutputIndex> = "first_*_output_index",
        tx_index: SeriesPattern35<TxIndex> = "*_output_tx_index",
        legacy_sigop_cost: SeriesPattern35<SigOps> = "*_legacy_sigop_cost",
    } }
    shape! { ScriptsP2ms at "series().indexer.scripts.p2ms" {
        first_index: SeriesPattern18<P2MSOutputIndex> = "first_*_output_index",
        tx_index: SeriesPattern27<TxIndex> = "*_output_tx_index",
        legacy_sigop_cost: SeriesPattern27<SigOps> = "*_legacy_sigop_cost",
    } }
    shape! { ScriptsEmpty at "series().indexer.scripts.empty" {
        first_index: SeriesPattern18<EmptyOutputIndex> = "first_*_index",
        tx_index: SeriesPattern24<TxIndex> = "*_tx_index",
    } }
    shape! { Scripts at "series().indexer.scripts" {
        empty: ScriptsEmpty = "*",
        p2ms: ScriptsP2ms = "p2ms",
        unknown: ScriptsUnknown = "unknown",
    } }
    shape! { IndexerAddressesP2a at "series().indexer.addresses.p2a" {
        first_index: SeriesPattern18<P2AAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern26<P2ABytes> = "*_bytes",
    } }
    shape! { IndexerAddressesP2tr at "series().indexer.addresses.p2tr" {
        first_index: SeriesPattern18<P2TRAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern32<P2TRBytes> = "*_bytes",
    } }
    shape! { IndexerAddressesP2wsh at "series().indexer.addresses.p2wsh" {
        first_index: SeriesPattern18<P2WSHAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern34<P2WSHBytes> = "*_bytes",
    } }
    shape! { IndexerAddressesP2wpkh at "series().indexer.addresses.p2wpkh" {
        first_index: SeriesPattern18<P2WPKHAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern33<P2WPKHBytes> = "*_bytes",
    } }
    shape! { IndexerAddressesP2sh at "series().indexer.addresses.p2sh" {
        first_index: SeriesPattern18<P2SHAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern31<P2SHBytes> = "*_bytes",
    } }
    shape! { IndexerAddressesP2pkh at "series().indexer.addresses.p2pkh" {
        first_index: SeriesPattern18<P2PKHAddressIndex> = "first_*_address_index",
        bytes: SeriesPattern30<P2PKHBytes> = "*_bytes",
    } }
    shape! { IndexerAddressesP2pk33 at "series().indexer.addresses.p2pk33" {
        first_index: SeriesPattern18<P2PK33AddressIndex> = "first_*_address_index",
        bytes: SeriesPattern28<P2PK33Bytes> = "*_bytes",
    } }
    shape! { IndexerAddressesP2pk65 at "series().indexer.addresses.p2pk65" {
        first_index: SeriesPattern18<P2PK65AddressIndex> = "first_*_address_index",
        bytes: SeriesPattern29<P2PK65Bytes> = "*_bytes",
    } }
    shape! { IndexerAddresses at "series().indexer.addresses" {
        p2pk65: IndexerAddressesP2pk65 = "*",
        p2pk33: IndexerAddressesP2pk33 = "p2pk33",
        p2pkh: IndexerAddressesP2pkh = "p2pkh",
        p2sh: IndexerAddressesP2sh = "p2sh",
        p2wpkh: IndexerAddressesP2wpkh = "p2wpkh",
        p2wsh: IndexerAddressesP2wsh = "p2wsh",
        p2tr: IndexerAddressesP2tr = "p2tr",
        p2a: IndexerAddressesP2a = "p2a",
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
    shape! { OneInput at "series().indexer.transactions.features.one_input" {
        block: SeriesPattern18<Count> = "*",
    } }
    shape! { Versions<A> at "series().transactions.versions" {
        v1: A = "v1_*",
        v2: A = "v2_*",
        v3: A = "v3_*",
        other: A = "other_version_*",
    } }
    shape! { Transactions at "series().transactions" {
        count: Size<Count, Option<CountFract>> = "*_count",
        features: TransactionsFeatures = "sighash",
        vsize: Vsize<VSize> = "*_vsize",
        fees: TransactionsFees = "fee",
        inscription: Inscription = "inscription",
        patterns: Patterns = "coinjoin",
        nonstandard: Nonstandard = "nonstandard",
        sigop_cost: BlocksCount<SigOps> = "block_sigop_cost",
        versions: Versions<BlocksCount<Count>> = "*_count",
        volume: OpReturnValue = "*_volume",
        per_second: InputsPerSecond<Option<PerSecond>> = "*_per_second",
    } }
    shape! { ExplicitlyRbf at "series().indexer.transactions.features.explicitly_rbf" {
        flag: SeriesPattern19<Boolean> = "is_*",
        block: SeriesPattern18<Count> = "*_tx_count",
    } }
    shape! { IndexerTransactionsFeaturesAnnex at "series().indexer.transactions.features.annex" {
        flag: SeriesPattern19<Boolean> = "has_*",
        block: SeriesPattern18<Count> = "*_tx_count",
    } }
    shape! { IndexerTransactionsFeatures at "series().indexer.transactions.features" {
        p2pk: IndexerTransactionsFeaturesAnnex = "p2pk",
        p2ms: IndexerTransactionsFeaturesAnnex = "p2ms",
        p2pkh: IndexerTransactionsFeaturesAnnex = "p2pkh",
        p2sh: IndexerTransactionsFeaturesAnnex = "p2sh",
        p2wpkh: IndexerTransactionsFeaturesAnnex = "p2wpkh",
        p2wsh: IndexerTransactionsFeaturesAnnex = "p2wsh",
        p2tr: IndexerTransactionsFeaturesAnnex = "p2tr",
        p2a: IndexerTransactionsFeaturesAnnex = "p2a",
        op_return: IndexerTransactionsFeaturesAnnex = "op_return",
        empty_script: IndexerTransactionsFeaturesAnnex = "empty_script",
        unknown_script: IndexerTransactionsFeaturesAnnex = "unknown_script",
        fake_pubkey: IndexerTransactionsFeaturesAnnex = "fake_pubkey",
        fake_scripthash: IndexerTransactionsFeaturesAnnex = "fake_scripthash",
        inscription: IndexerTransactionsFeaturesAnnex = "inscription",
        annex: IndexerTransactionsFeaturesAnnex = "annex",
        sighash_all: IndexerTransactionsFeaturesAnnex = "*_all",
        sighash_none: IndexerTransactionsFeaturesAnnex = "*_none",
        sighash_single: IndexerTransactionsFeaturesAnnex = "*_single",
        sighash_default: IndexerTransactionsFeaturesAnnex = "*_default",
        sighash_anyone_can_pay: IndexerTransactionsFeaturesAnnex = "*_anyone_can_pay",
        explicitly_rbf: ExplicitlyRbf = "explicitly_rbf",
        dust_output: IndexerTransactionsFeaturesAnnex = "dust_output",
        one_input: OneInput = "one_input_tx_count",
        one_output: OneInput = "one_output_tx_count",
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
        features: IndexerTransactionsFeatures = "sighash",
        versions: Versions<OneInput> = "*_count",
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
        addresses: IndexerAddresses = "p2pk65",
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
        utxo_set: UtxoSet2 = "circulating_supply",
        market: Market = "price",
        age: Age = "utxos",
        utxos: Utxos = "utxos",
        addresses: Addresses = "address",
        holders: Holders<Rookie<AllSupply, AllCapital, AllActivity, AllRealized, AllUnrealized, AllCostBasis>, Rookie<LthSupply, AllCapital, AllActivity, AllRealized, AllUnrealized, AllCostBasis>> = "utxos",
        entry: Entry = "veteran",
        profitability: Profitability = "sth",
        supply: Supply = "circulating_supply",
        cointime: Cointime = "utxos",
        coinflow: Coinflow = "utxos",
        bedrock: Bedrock = "bedrock",
        rarity_meter: RarityMeter = "rarity_meter",
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
    pub fn search_series(&self, q: SeriesName, limit: Option<Limit>) -> Result<Vec<String>> {
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
    /// Cohorts for which URPD data is available. Returns names like `all`, `sth`, `lth`, `under_4m`, `under_6m`, `over_4m`, `over_6m`, `utxos_under_1h_old`.
    ///
    /// Endpoint: `GET /api/urpd`
    pub fn list_urpd_cohorts(&self) -> Result<Vec<Cohort>> {
        self.base.get_json(&format!("/api/urpd"))
    }

    /// Latest URPD
    ///
    /// URPD for the latest published block; `date` echoes the date served. Fields, units, price-grid rounding and weighting as in `GET /api/urpd/{cohort}/{point}`.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}`
    pub fn get_urpd(
        &self,
        cohort: Cohort,
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
    /// URPD for a cohort at a block height or the last block of a UTC day. Returns `{ height, hash, date, cohort, weight, aggregation, close, supply, buckets }` where `buckets` is columns, `{ price, supply, realized_cap, unrealized_pnl }`, one entry per bucket in ascending price: the same block header and columnar layout as `GET /api/utxo-set/{point}`. `close`, `price`, `realized_cap`, and `unrealized_pnl` are USD; `supply` is BTC. `unrealized_pnl` can be negative. Bucket `realized_cap` and `unrealized_pnl` value each coin at its creation price on the price grid (four significant digits of the price in cents, within 0.05% of the exact price), rounded to the cent per bucket, so the buckets need not sum to the realized cap series. With a `cointime` or `coinflow` `weight`, `supply` is the weighted supply (whole satoshis) and the bucket values follow it.
    ///
    /// Endpoint: `GET /api/urpd/{cohort}/{point}`
    pub fn get_urpd_at(
        &self,
        cohort: Cohort,
        point: &str,
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
    pub fn get_transaction_times(&self, txId: &[Txid]) -> Result<Vec<i64>> {
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
    pub fn get_utxo_set(&self, point: &str) -> Result<UtxoSet> {
        self.base
            .get_json(&format!("/api/utxo-set/{}", path_segment(point)))
    }

    /// UTXO set changes of a block or date
    ///
    /// What a block (`840000`) or a UTC day's published blocks (`YYYY-MM-DD`) changed in the UTXO set. Returns `{ first, last, hash, date, created, spent }`: `created` has one row per block, `spent` one row per creation height, each as columnar `height`, `count` and `supply` (BTC). The set after block `first - 1` plus `created` minus `spent` is the set after block `last`.
    ///
    /// Endpoint: `GET /api/utxo-set/{point}/diff`
    pub fn get_utxo_set_diff(&self, point: &str) -> Result<UtxoSetDiff> {
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
    pub fn get_oracle_histogram_payments(&self, point: &str) -> Result<Vec<i64>> {
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
    pub fn get_oracle_histogram_outputs(&self, point: &str) -> Result<Vec<i64>> {
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
