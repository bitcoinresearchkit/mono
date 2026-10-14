use bitview_distribution::families::{CountWithDeltas, Supply};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, PartsPerMillion32};
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, CumulativeSource, LazyIndexedVec, LazyPerBlock, LazyPercentCumulativeRolling,
    LazyPercentPerBlock, LazyPreviousDeltaVec, LazyRollingSumsFromHeight, LazyWindowStartVec,
    PerBlockCumulativeAverage, PerBlockCumulativeRolling, import_cached, ops::Quotient,
    shapes::Windows,
};
use brk_error::Result;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{
    BinaryTransform, Database, Ident, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Rw,
    StorageMode, WritableVec,
};

use super::{BlockActivityCounts, ExposedState, MemberState, ReuseState};

/// What every member's import shares.
pub struct ImportContext<'a> {
    pub db: &'a Database,
    pub version: Version,
    pub mappings: &'a MappingsVecs,
    pub windows: &'a Windows<&'a LazyWindowStartVec>,
    pub spot: &'a ReadableBoxedVec<Height, Cents>,
}

/// One address type's series, or every type's together (the root). A type's ids start with
/// its key (`p2pk_funded_address_count`); the root's start with the metric.
#[derive(Traversable)]
pub struct AddressVecs<M: StorageMode = Rw> {
    /// Addresses that hold at least one unspent output at the represented block.
    pub funded: CountWithDeltas<M>,
    /// Previously seen addresses that hold no unspent outputs at the represented
    /// block.
    pub empty: StoredCount<M>,
    /// All previously seen addresses, funded or empty.
    pub total: LazyPerBlock<Count>,
    /// Addresses first observed in each block: the increase in the total address
    /// count.
    pub new: NewAddressCount,
    pub activity: ActivityVecs<M>,
    /// Mean balance of a funded address: the supply held by the addresses divided
    /// by the funded address count. Undefined (null) while no address is funded.
    pub avg_balance: AvgBalance,
    /// Addresses that have received more than one output over their lifetime. The two coinbase
    /// outputs overwritten by BIP30 duplicates at heights 91,842 and 91,880 do not count as
    /// received.
    pub reused: ReuseVecs<M>,
    /// Addresses from which more than one output has been spent over their
    /// lifetime.
    pub respent: ReuseVecs<M>,
    /// Addresses whose public key or spending script has appeared on-chain. P2PK
    /// and P2TR are exposed when funded; hashed script types become exposed when
    /// spent.
    pub exposed: ExposedVecs<M>,
    /// The supply the addresses hold, and the outputs they received and inputs they spent:
    /// the shares' and average balance's denominators.
    #[traversable(hidden)]
    supply: CachedSeries<Height, Sats, M>,
    #[traversable(hidden)]
    outputs: CumulativeSource<Count, M>,
    #[traversable(hidden)]
    inputs: CumulativeSource<Count, M>,
}

/// A stored count of addresses.
#[derive(Traversable)]
pub struct StoredCount<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub value: LazyPerBlock<Count>,
    #[traversable(hidden)]
    pub stored: CachedSeries<Height, Count, M>,
}

#[derive(Clone, Traversable)]
pub struct NewAddressCount {
    /// Value for the represented block.
    pub block: LazyPreviousDeltaVec<Height, Count>,
    pub sum: LazyRollingSumsFromHeight<Count>,
}

#[derive(Traversable)]
pub struct ActivityVecs<M: StorageMode = Rw> {
    /// Distinct previously seen addresses that received bitcoin in the
    /// represented block while holding no unspent balance immediately before
    /// that receive was applied.
    pub reactivated: PerBlockCumulativeAverage<Count, M>,
    /// Distinct addresses that sent bitcoin in the represented block.
    pub sending: PerBlockCumulativeAverage<Count, M>,
    /// Distinct addresses that received bitcoin in the represented block.
    pub receiving: PerBlockCumulativeAverage<Count, M>,
    /// Distinct addresses that both sent and received bitcoin in the
    /// represented block.
    pub bidirectional: PerBlockCumulativeAverage<Count, M>,
    /// Distinct addresses active in the represented block: sending plus
    /// receiving minus bidirectional addresses.
    pub active: PerBlockCumulativeAverage<Count, M>,
}

#[derive(Clone, Traversable)]
pub struct AvgBalance {
    /// Reported in BTC.
    pub btc: LazyPerBlock<Bitcoin>,
    /// Reported in US dollars: the BTC mean at the spot price.
    pub usd: LazyPerBlock<Dollars>,
}

/// Addresses meeting a reuse predicate: receive-based reuse (more than one lifetime receive)
/// or spend-based respending (more than one lifetime spend).
#[derive(Traversable)]
pub struct ReuseVecs<M: StorageMode = Rw> {
    pub count: FundedTotal<M>,
    /// Balance held at the represented block by funded addresses meeting the
    /// predicate.
    pub supply: PredicateSupply<M>,
    pub events: ReuseEvents<M>,
}

#[derive(Traversable)]
pub struct ExposedVecs<M: StorageMode = Rw> {
    pub count: FundedTotal<M>,
    /// Balance held at the represented block by funded exposed addresses.
    pub supply: PredicateSupply<M>,
}

#[derive(Traversable)]
pub struct FundedTotal<M: StorageMode = Rw> {
    /// Addresses that hold unspent outputs at the represented block and meet
    /// the predicate.
    pub funded: StoredCount<M>,
    /// Addresses that have ever met the predicate, whether or not they hold
    /// unspent outputs at the represented block.
    pub total: StoredCount<M>,
}

#[derive(Traversable)]
pub struct PredicateSupply<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub value: Supply<M>,
    /// The predicate's supply divided by all the supply the addresses hold.
    pub share: LazyPercentPerBlock<PartsPerMillion32>,
}

/// Per-block reuse events.
#[derive(Traversable)]
pub struct ReuseEvents<M: StorageMode = Rw> {
    /// Received outputs that leave their address meeting the predicate: for reuse, every output
    /// after an address's first lifetime output; for respending, outputs to an address that had
    /// already spent more than one output when the receive was applied. Multiple qualifying
    /// outputs to one address count separately. Shares are of the outputs the addresses
    /// received.
    pub outputs: EventCount<M>,
    /// Inputs spending from an address that met the predicate immediately before the input was
    /// applied. Multiple qualifying inputs of one address count separately. Shares are of the
    /// inputs the addresses spent.
    pub inputs: EventCount<M>,
    /// Distinct active addresses meeting the predicate after their block's
    /// events, and their share of active addresses, counted per block (an
    /// address active in two blocks counts twice).
    pub active: ActiveEvents<M>,
}

#[derive(Traversable)]
pub struct EventCount<M: StorageMode = Rw> {
    pub count: PerBlockCumulativeRolling<Count, M>,
    pub share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

#[derive(Traversable)]
pub struct ActiveEvents<M: StorageMode = Rw> {
    pub count: PerBlockCumulativeAverage<Count, M>,
    pub share: LazyPercentCumulativeRolling<PartsPerMillion32>,
}

impl AddressVecs {
    /// `prefix` is the member's id prefix (`p2pk_`, empty for the root).
    pub fn import(ctx: &ImportContext<'_>, prefix: &str) -> Result<Self> {
        let name = |metric: &str| format!("{prefix}{metric}");
        let count_version = ctx.version + Version::ONE;
        let supply = import_cached(ctx.db, &name("address_supply_sats"), count_version)?;
        let outputs = CumulativeSource::import(
            ctx.db,
            &name("address_output_count_cumulative"),
            count_version,
        )?;
        let inputs = CumulativeSource::import(
            ctx.db,
            &name("address_input_count_cumulative"),
            count_version,
        )?;
        let funded = CountWithDeltas::import(
            ctx.db,
            &name("funded_address_count"),
            count_version,
            ctx.mappings,
            ctx.windows,
        )?;
        let empty = StoredCount::import(ctx, &name("empty_address_count"), count_version)?;
        let total_name = name("total_address_count");
        let total_source = LazyIndexedVec::new(
            &format!("{total_name}_source"),
            count_version,
            &funded.stored,
            &empty.stored,
            |_, funded, empty| funded + empty,
        );
        let total = LazyPerBlock::from_height_source::<Ident>(
            &total_name,
            count_version,
            &total_source,
            ctx.mappings,
        );
        let new = NewAddressCount::new(ctx, &name("new_address_count"), &total_source);
        let activity = ActivityVecs::import(ctx, prefix)?;
        let avg_balance =
            AvgBalance::new(ctx, &name("avg_address_balance"), &supply, &funded.stored);
        let reused = ReuseVecs::import(
            ctx,
            prefix,
            "reused",
            &activity,
            &supply,
            outputs.cumulative_source(),
            inputs.cumulative_source(),
        )?;
        let respent = ReuseVecs::import(
            ctx,
            prefix,
            "respent",
            &activity,
            &supply,
            outputs.cumulative_source(),
            inputs.cumulative_source(),
        )?;
        let exposed = ExposedVecs {
            count: FundedTotal::import(ctx, prefix, "exposed")?,
            supply: PredicateSupply::import(ctx, &name("exposed_address_supply"), &supply)?,
        };
        Ok(Self {
            funded,
            empty,
            total,
            new,
            activity,
            avg_balance,
            reused,
            respent,
            exposed,
            supply,
            outputs,
            inputs,
        })
    }

    /// The supply the addresses hold.
    pub fn supply(&self) -> &CachedSeries<Height, Sats> {
        &self.supply
    }

    #[inline(always)]
    pub fn push(&mut self, state: &MemberState) {
        self.funded.push(Count::from(state.funded));
        self.empty.push(state.empty);
        self.supply.push(state.supply);
        self.outputs.push_block(Count::from(state.outputs));
        self.inputs.push_block(Count::from(state.inputs));
        self.activity.push(&state.activity);
        self.reused.push(&state.reused);
        self.respent.push(&state.respent);
        let ExposedState {
            funded,
            total,
            supply,
        } = state.exposed;
        self.exposed.count.push(funded, total);
        self.exposed.supply.value.push(supply);
    }

    /// The running counters at the end of `height`, `None` without that block.
    pub fn restore(&self, height: Height) -> Option<MemberState> {
        let count = |vec: &CachedSeries<Height, Count>| vec.collect_one(height).map(u64::from);
        let supply = |supply: &PredicateSupply| supply.value.stored.collect_one(height);
        let reuse = |vecs: &ReuseVecs| -> Option<ReuseState> {
            Some(ReuseState {
                funded: count(&vecs.count.funded.stored)?,
                total: count(&vecs.count.total.stored)?,
                supply: supply(&vecs.supply)?,
                ..ReuseState::default()
            })
        };
        Some(MemberState {
            funded: count(&self.funded.stored)?,
            empty: count(&self.empty.stored)?,
            supply: self.supply.collect_one(height)?,
            outputs: 0,
            inputs: 0,
            activity: BlockActivityCounts::default(),
            reused: reuse(&self.reused)?,
            respent: reuse(&self.respent)?,
            exposed: ExposedState {
                funded: count(&self.exposed.count.funded.stored)?,
                total: count(&self.exposed.count.total.stored)?,
                supply: supply(&self.exposed.supply)?,
            },
        })
    }
}

impl StoredCount {
    fn import(ctx: &ImportContext<'_>, name: &str, version: Version) -> Result<Self> {
        let stored = import_cached(ctx.db, name, version)?;
        let value = LazyPerBlock::from_height_source::<Ident>(name, version, &stored, ctx.mappings);
        Ok(Self { value, stored })
    }

    #[inline(always)]
    fn push(&mut self, count: u64) {
        self.stored.push(Count::from(count));
    }
}

impl NewAddressCount {
    fn new(
        ctx: &ImportContext<'_>,
        name: &str,
        total: &impl ReadableCloneableVec<Height, Count>,
    ) -> Self {
        Self {
            block: LazyPreviousDeltaVec::new(name, ctx.version, total),
            sum: LazyRollingSumsFromHeight::new(
                &format!("{name}_sum"),
                ctx.version,
                total,
                ctx.windows,
                ctx.mappings,
            ),
        }
    }
}

impl ActivityVecs {
    fn import(ctx: &ImportContext<'_>, prefix: &str) -> Result<Self> {
        let import = |kind: &str| {
            PerBlockCumulativeAverage::import(
                ctx.db,
                &format!("{prefix}{kind}_address_count"),
                ctx.version + Version::TWO,
                ctx.mappings,
                ctx.windows,
            )
        };
        Ok(Self {
            reactivated: import("reactivated")?,
            sending: import("sending")?,
            receiving: import("receiving")?,
            bidirectional: import("bidirectional")?,
            active: import("active")?,
        })
    }

    #[inline(always)]
    fn push(&mut self, counts: &BlockActivityCounts) {
        self.reactivated
            .push_block(Count::from(u64::from(counts.reactivated)));
        self.sending
            .push_block(Count::from(u64::from(counts.sending)));
        self.receiving
            .push_block(Count::from(u64::from(counts.receiving)));
        self.bidirectional
            .push_block(Count::from(u64::from(counts.bidirectional)));
        self.active
            .push_block(Count::from(u64::from(counts.active())));
    }
}

impl AvgBalance {
    fn new(
        ctx: &ImportContext<'_>,
        name: &str,
        supply: &CachedSeries<Height, Sats>,
        funded: &CachedSeries<Height, Count>,
    ) -> Self {
        let version = ctx.version + Version::TWO;
        let btc = LazyIndexedVec::new(
            &format!("{name}_source"),
            version,
            supply,
            funded,
            // No funded address divides by zero: a non-finite mean, served as null.
            |_, supply, count| {
                Bitcoin::from(f64::from(Bitcoin::from(supply)) / u64::from(count) as f64)
            },
        );
        let usd = LazyIndexedVec::new(
            &format!("{name}_usd_source"),
            version,
            &btc,
            ctx.spot,
            |_, btc, spot| Dollars::from(f64::from(btc) * f64::from(Dollars::from(spot))),
        );
        Self {
            btc: LazyPerBlock::from_height_source::<Ident>(name, version, &btc, ctx.mappings),
            usd: LazyPerBlock::from_height_source::<Ident>(
                &format!("{name}_usd"),
                version,
                &usd,
                ctx.mappings,
            ),
        }
    }
}

impl ReuseVecs {
    fn import(
        ctx: &ImportContext<'_>,
        prefix: &str,
        predicate: &str,
        activity: &ActivityVecs,
        address_supply: &CachedSeries<Height, Sats>,
        outputs: &impl ReadableCloneableVec<Height, Count>,
        inputs: &impl ReadableCloneableVec<Height, Count>,
    ) -> Result<Self> {
        let name = |metric: &str| format!("{prefix}{metric}");
        let count = |metric: &str| {
            PerBlockCumulativeRolling::import(
                ctx.db,
                &name(&format!("{metric}_count")),
                ctx.version + Version::ONE,
                ctx.mappings,
                ctx.windows,
            )
        };
        let output_metric = format!("output_to_{predicate}_address");
        let output_count = count(&output_metric)?;
        let output_share = event_share(
            ctx,
            &name(&format!("{output_metric}_share")),
            output_count.cumulative_source(),
            outputs,
        );
        let input_metric = format!("input_from_{predicate}_address");
        let input_count = count(&input_metric)?;
        let input_share = event_share(
            ctx,
            &name(&format!("{input_metric}_share")),
            input_count.cumulative_source(),
            inputs,
        );
        let active_count = PerBlockCumulativeAverage::import(
            ctx.db,
            &name(&format!("active_{predicate}_address_count")),
            ctx.version,
            ctx.mappings,
            ctx.windows,
        )?;
        let active_share = event_share(
            ctx,
            &name(&format!("active_{predicate}_address_share")),
            active_count.cumulative_source(),
            activity.active.cumulative_source(),
        );
        Ok(Self {
            count: FundedTotal::import(ctx, prefix, predicate)?,
            supply: PredicateSupply::import(
                ctx,
                &name(&format!("{predicate}_address_supply")),
                address_supply,
            )?,
            events: ReuseEvents {
                outputs: EventCount {
                    count: output_count,
                    share: output_share,
                },
                inputs: EventCount {
                    count: input_count,
                    share: input_share,
                },
                active: ActiveEvents {
                    count: active_count,
                    share: active_share,
                },
            },
        })
    }

    #[inline(always)]
    fn push(&mut self, state: &ReuseState) {
        self.count.push(state.funded, state.total);
        self.supply.value.push(state.supply);
        let events = &mut self.events;
        events
            .outputs
            .count
            .push_block(Count::from(state.output_events));
        events
            .inputs
            .count
            .push_block(Count::from(state.input_events));
        events.active.count.push_block(Count::from(state.active));
    }
}

impl FundedTotal {
    fn import(ctx: &ImportContext<'_>, prefix: &str, predicate: &str) -> Result<Self> {
        let version = ctx.version + Version::ONE;
        Ok(Self {
            funded: StoredCount::import(
                ctx,
                &format!("{prefix}funded_{predicate}_address_count"),
                version,
            )?,
            total: StoredCount::import(
                ctx,
                &format!("{prefix}total_{predicate}_address_count"),
                version,
            )?,
        })
    }

    #[inline(always)]
    fn push(&mut self, funded: u64, total: u64) {
        self.funded.push(funded);
        self.total.push(total);
    }
}

impl PredicateSupply {
    /// `address_supply` is the supply all the member's addresses hold, the share's denominator.
    fn import(
        ctx: &ImportContext<'_>,
        name: &str,
        address_supply: &CachedSeries<Height, Sats>,
    ) -> Result<Self> {
        let version = ctx.version + Version::ONE;
        let value = Supply::import(ctx.db, name, version, ctx.mappings, ctx.spot)?;
        let share_name = format!("{name}_share");
        let source = LazyIndexedVec::new(
            &format!("{share_name}_ppm"),
            version,
            &value.stored,
            address_supply,
            |_, part, total| Quotient::<PartsPerMillion32>::apply(part, total),
        );
        let share =
            LazyPercentPerBlock::from_height_source(&share_name, version, &source, ctx.mappings);
        Ok(Self { value, share })
    }
}

/// A cumulative or windowed share of two cumulative counts.
fn event_share(
    ctx: &ImportContext<'_>,
    name: &str,
    numerator: &impl ReadableCloneableVec<Height, Count>,
    denominator: &impl ReadableCloneableVec<Height, Count>,
) -> LazyPercentCumulativeRolling<PartsPerMillion32> {
    LazyPercentCumulativeRolling::from_cumulative_ratio::<Count, Count, Quotient<PartsPerMillion32>>(
        name,
        ctx.version,
        numerator,
        denominator,
        ctx.windows,
        ctx.mappings,
    )
}
