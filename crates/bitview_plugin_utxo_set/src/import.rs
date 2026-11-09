use std::path::PathBuf;

use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_vecs::{LazyPerBlock, PerBlock, import_cached};
use brk_error::Result;
use brk_types::Version;
use statedb::History;
use vecdb::Convert;

use crate::{STORAGE, Vecs};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &Mappings,
        spends_path: PathBuf,
        creations_path: PathBuf,
    ) -> Result<Self> {
        STORAGE.import(context, 20_000_000, |db, version| {
            let supply = import_cached(&db, "unspent_sats", version)?;
            let circulating_supply = LazyPerBlock::from_height_source::<Convert>(
                "circulating_supply",
                version,
                &supply,
                mappings,
            );
            Ok(Self {
                supply,
                circulating_supply,
                count: PerBlock::import(&db, "utxo_count", version + Version::ONE, mappings)?,
                history: History::open(db.path())?,
                spends_path,
                creations_path,
                db,
            })
        })
    }
}
