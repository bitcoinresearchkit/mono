use brk_traversable::Traversable;
use brk_types::{Bitcoin, Dollars, Height, StoredF32, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    DeltaChange, DeltaRate, LazyDeltaVec, LazyVec, ReadOnlyClone, ReadableCloneableVec, VecValue,
};

use crate::{
    indexes,
    internal::{
        AmountType, CachedWindowStartVec, DerivedResolutions, FiatType, FixedRatio, LazyPerBlock,
        NumericValue, Percent, Resolutions, Windows,
    },
};

/// Generic single-slot lazy delta: a `LazyDeltaVec` + resolution views.
///
/// Used as building block for both change and rate deltas.
/// - Change: `LazyDeltaFromHeight<S, C, DeltaChange>`
/// - Rate: `LazyDeltaFromHeight<S, B, DeltaRate>`
#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyDeltaFromHeight<S, T, Op: 'static>
where
    S: VecValue,
    T: NumericValue + JsonSchema,
{
    pub height: LazyDeltaVec<Height, S, T, Op>,
    #[traversable(flatten)]
    pub resolutions: Box<Resolutions<T>>,
}

/// Single-slot lazy delta percent: fixed-point delta + lazy ratio + lazy percent views.
///
/// Mirrors `PercentPerBlock<B>` but with a lazy delta source.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct LazyDeltaPercentFromHeight<S, B>(
    pub Percent<LazyDeltaFromHeight<S, B, DeltaRate>, LazyPerBlock<StoredF32, B>>,
)
where
    S: VecValue,
    B: FixedRatio;

/// Lazy rolling deltas for all 4 window durations (24h, 1w, 1m, 1y).
///
/// Tree shape: `absolute._24h/...`, `rate._24h/...` — matches old `RollingDelta`.
///
/// Replaces `RollingDelta`, `RollingDelta1m`, and `RollingDeltaExcept1m` — since
/// there is no storage cost, all 4 windows are always available.
#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasFromHeight<S, C, B>
where
    S: VecValue,
    C: NumericValue + JsonSchema,
    B: FixedRatio,
{
    pub absolute: Windows<LazyDeltaFromHeight<S, C, DeltaChange>>,
    pub rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<S, C, B> LazyRollingDeltasFromHeight<S, C, B>
where
    S: VecValue + Into<f64>,
    C: NumericValue + JsonSchema + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let src = source.read_only_boxed_clone();

        let make_slot = |suffix: &str, cached_start: &&CachedWindowStartVec| {
            let full_name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();

            // Absolute change: source[h] - source[ago] as C (via f64)
            let change_vec = LazyDeltaVec::<Height, S, C, DeltaChange>::new(
                &full_name,
                version,
                src.clone(),
                starts_version,
                {
                    let cached = cached.clone();
                    move || cached.snapshot()
                },
            );
            let change_resolutions =
                Resolutions::forced_import(&full_name, change_vec.clone(), version, indexes);
            let absolute = LazyDeltaFromHeight {
                height: change_vec,
                resolutions: Box::new(change_resolutions),
            };

            // Rate: (source[h] - source[ago]) / source[ago] as B (via f64)
            let rate_raw_name = format!("{full_name}_rate_{}", B::SUFFIX);
            let rate_vec = LazyDeltaVec::<Height, S, B, DeltaRate>::new(
                &rate_raw_name,
                version,
                src.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let rate_resolutions =
                Resolutions::forced_import(&rate_raw_name, rate_vec.clone(), version, indexes);
            let ppm = LazyDeltaFromHeight {
                height: rate_vec,
                resolutions: Box::new(rate_resolutions),
            };

            let rate_ratio_name = format!("{full_name}_rate_ratio");
            let ratio = LazyPerBlock {
                height: LazyVec::transformed::<B::ToRatio>(
                    &rate_ratio_name,
                    version,
                    ppm.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<B::ToRatio>(
                    &rate_ratio_name,
                    version,
                    &ppm.resolutions,
                )),
            };

            let rate_name = format!("{full_name}_rate");
            let percent = LazyPerBlock {
                height: LazyVec::transformed::<B::ToPercent>(
                    &rate_name,
                    version,
                    ppm.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<B::ToPercent>(
                    &rate_name,
                    version,
                    &ppm.resolutions,
                )),
            };

            let rate = LazyDeltaPercentFromHeight(Percent {
                ppm,
                ratio,
                percent,
            });

            (absolute, rate)
        };

        let (absolute, rate) = cached_starts.map_with_suffix(make_slot).unzip();

        Self { absolute, rate }
    }
}

// ---------------------------------------------------------------------------
// Amount delta types (sats change + lazy BTC + rate)
// ---------------------------------------------------------------------------

/// Single-slot amount delta change: sats delta + lazy BTC.
#[derive(Clone, Traversable)]
pub struct LazyDeltaAmountFromHeight<S, C>
where
    S: VecValue,
    C: AmountType,
{
    pub btc: LazyPerBlock<Bitcoin, C>,
    pub sats: LazyDeltaFromHeight<S, C, DeltaChange>,
}

/// Lazy amount rolling deltas for all 4 windows.
///
/// Tree shape: `absolute._24h.{sats,btc}/...`, `rate._24h/...` — mirrors
/// `LazyRollingDeltasFiatFromHeight` but stores sats instead of cents and
/// derives a lazy BTC view alongside (free, since sats → btc is a scalar
/// transform).
#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasAmountFromHeight<S, C, B>
where
    S: VecValue,
    C: AmountType,
    B: FixedRatio,
{
    pub absolute: Windows<LazyDeltaAmountFromHeight<S, C>>,
    pub rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<S, C, B> LazyRollingDeltasAmountFromHeight<S, C, B>
where
    S: VecValue + Into<f64>,
    C: AmountType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let src = source.read_only_boxed_clone();

        let make_slot = |suffix: &str, cached_start: &&CachedWindowStartVec| {
            let full_name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();

            // Absolute change (sats): source[h] - source[ago] as C (via f64)
            let sats_name = format!("{full_name}_sats");
            let change_vec = LazyDeltaVec::<Height, S, C, DeltaChange>::new(
                &sats_name,
                version,
                src.clone(),
                starts_version,
                {
                    let cached = cached.clone();
                    move || cached.snapshot()
                },
            );
            let change_resolutions =
                Resolutions::forced_import(&sats_name, change_vec.clone(), version, indexes);
            let sats = LazyDeltaFromHeight {
                height: change_vec,
                resolutions: Box::new(change_resolutions),
            };

            // Absolute change (btc): lazy from sats delta
            let btc = LazyPerBlock {
                height: LazyVec::transformed::<C::ToBitcoin>(
                    &full_name,
                    version,
                    sats.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<C::ToBitcoin>(
                    &full_name,
                    version,
                    &sats.resolutions,
                )),
            };

            let absolute = LazyDeltaAmountFromHeight { btc, sats };

            // Rate: (source[h] - source[ago]) / source[ago] as B (via f64)
            let rate_raw_name = format!("{full_name}_rate_{}", B::SUFFIX);
            let rate_vec = LazyDeltaVec::<Height, S, B, DeltaRate>::new(
                &rate_raw_name,
                version,
                src.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let rate_resolutions =
                Resolutions::forced_import(&rate_raw_name, rate_vec.clone(), version, indexes);
            let ppm = LazyDeltaFromHeight {
                height: rate_vec,
                resolutions: Box::new(rate_resolutions),
            };

            let rate_ratio_name = format!("{full_name}_rate_ratio");
            let ratio = LazyPerBlock {
                height: LazyVec::transformed::<B::ToRatio>(
                    &rate_ratio_name,
                    version,
                    ppm.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<B::ToRatio>(
                    &rate_ratio_name,
                    version,
                    &ppm.resolutions,
                )),
            };

            let rate_name = format!("{full_name}_rate");
            let percent = LazyPerBlock {
                height: LazyVec::transformed::<B::ToPercent>(
                    &rate_name,
                    version,
                    ppm.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<B::ToPercent>(
                    &rate_name,
                    version,
                    &ppm.resolutions,
                )),
            };

            let rate = LazyDeltaPercentFromHeight(Percent {
                ppm,
                ratio,
                percent,
            });

            (absolute, rate)
        };

        let (absolute, rate) = cached_starts.map_with_suffix(make_slot).unzip();

        Self { absolute, rate }
    }
}

// ---------------------------------------------------------------------------
// Fiat delta types (cents change + lazy USD + rate)
// ---------------------------------------------------------------------------

/// Single-slot fiat delta change: cents delta + lazy USD.
#[derive(Clone, Traversable)]
pub struct LazyDeltaFiatFromHeight<S, C>
where
    S: VecValue,
    C: FiatType,
{
    pub usd: LazyPerBlock<Dollars, C>,
    pub cents: LazyDeltaFromHeight<S, C, DeltaChange>,
}

/// Lazy fiat rolling deltas for all 4 windows.
///
/// Tree shape: `absolute._24h.{cents,usd}/...`, `rate._24h/...` — matches old `FiatRollingDelta`.
///
/// Replaces `FiatRollingDelta`, `FiatRollingDelta1m`, and `FiatRollingDeltaExcept1m`.
#[derive(Clone, Traversable)]
pub struct LazyRollingDeltasFiatFromHeight<S, C, B>
where
    S: VecValue,
    C: FiatType,
    B: FixedRatio,
{
    pub absolute: Windows<LazyDeltaFiatFromHeight<S, C>>,
    pub rate: Windows<LazyDeltaPercentFromHeight<S, B>>,
}

impl<S, C, B> LazyRollingDeltasFiatFromHeight<S, C, B>
where
    S: VecValue + Into<f64>,
    C: FiatType + From<f64>,
    B: FixedRatio + From<f64>,
{
    pub fn new(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        cached_starts: &Windows<&CachedWindowStartVec>,
        indexes: &indexes::Vecs,
    ) -> Self {
        let src = source.read_only_boxed_clone();

        let make_slot = |suffix: &str, cached_start: &&CachedWindowStartVec| {
            let full_name = format!("{name}_{suffix}");
            let cached = cached_start.read_only_clone();
            let starts_version = cached.version();

            // Absolute change (cents): source[h] - source[ago] as C (via f64)
            let cents_name = format!("{full_name}_cents");
            let change_vec = LazyDeltaVec::<Height, S, C, DeltaChange>::new(
                &cents_name,
                version,
                src.clone(),
                starts_version,
                {
                    let cached = cached.clone();
                    move || cached.snapshot()
                },
            );
            let change_resolutions =
                Resolutions::forced_import(&cents_name, change_vec.clone(), version, indexes);
            let cents = LazyDeltaFromHeight {
                height: change_vec,
                resolutions: Box::new(change_resolutions),
            };

            // Absolute change (usd): lazy from cents delta
            let usd = LazyPerBlock {
                height: LazyVec::transformed::<C::ToDollars>(
                    &full_name,
                    version,
                    cents.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<C::ToDollars>(
                    &full_name,
                    version,
                    &cents.resolutions,
                )),
            };

            let absolute = LazyDeltaFiatFromHeight { usd, cents };

            // Rate: (source[h] - source[ago]) / source[ago] as B (via f64)
            let rate_raw_name = format!("{full_name}_rate_{}", B::SUFFIX);
            let rate_vec = LazyDeltaVec::<Height, S, B, DeltaRate>::new(
                &rate_raw_name,
                version,
                src.clone(),
                starts_version,
                move || cached.snapshot(),
            );
            let rate_resolutions =
                Resolutions::forced_import(&rate_raw_name, rate_vec.clone(), version, indexes);
            let ppm = LazyDeltaFromHeight {
                height: rate_vec,
                resolutions: Box::new(rate_resolutions),
            };

            let rate_ratio_name = format!("{full_name}_rate_ratio");
            let ratio = LazyPerBlock {
                height: LazyVec::transformed::<B::ToRatio>(
                    &rate_ratio_name,
                    version,
                    ppm.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<B::ToRatio>(
                    &rate_ratio_name,
                    version,
                    &ppm.resolutions,
                )),
            };

            let rate_name = format!("{full_name}_rate");
            let percent = LazyPerBlock {
                height: LazyVec::transformed::<B::ToPercent>(
                    &rate_name,
                    version,
                    ppm.height.read_only_boxed_clone(),
                ),
                resolutions: Box::new(DerivedResolutions::from_derived_computed::<B::ToPercent>(
                    &rate_name,
                    version,
                    &ppm.resolutions,
                )),
            };

            let rate = LazyDeltaPercentFromHeight(Percent {
                ppm,
                ratio,
                percent,
            });

            (absolute, rate)
        };

        let (absolute, rate) = cached_starts.map_with_suffix(make_slot).unzip();

        Self { absolute, rate }
    }
}
