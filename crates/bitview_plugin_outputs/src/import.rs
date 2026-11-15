use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{
    LazyPerSecondWindows, LazyWindowStartVec, PerBlockFullFromCumulative,
    ValuePerBlockCumulativeRolling, import_cached, shapes::Windows,
};
use brk_error::Result;
use brk_types::Version;
use statedb::Creations;

use super::{Coinbase, Pubkeys, STORAGE, Vecs, by_type, spent};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();

        let spent = spent::Vecs::import(&db, version)?;
        let count = PerBlockFullFromCumulative::import(
            &db,
            "output_count",
            version,
            &mappings.output_count_source(),
            mappings,
            window_starts,
        )?;
        let per_second =
            LazyPerSecondWindows::new("outputs_per_second", version, &count.rolling.sum);
        let types = by_type::Vecs::import(&db, version, mappings, window_starts)?;
        let unspendable_value = import_cached(&db, "unspendable_output_value", version)?;
        let non_op_return_count = by_type::non_op_return_output_count(
            version + Version::TWO,
            &types.types.unspendable.op_return.count.cumulative.height,
            mappings,
            window_starts,
        );
        let op_return_value = ValuePerBlockCumulativeRolling::import(
            &db,
            "op_return_value",
            version,
            mappings,
            window_starts,
        )?;

        let coinbase = Coinbase::import(&db, version, mappings, window_starts)?;
        let pubkeys = Pubkeys::import(&db, version, mappings, window_starts)?;
        let creations = Creations::open(db.path())?;
        let this = Self {
            creations,
            db,
            spent,
            count,
            per_second,
            non_op_return_count,
            unspendable_value,
            types,
            op_return_value,
            coinbase,
            pubkeys,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
