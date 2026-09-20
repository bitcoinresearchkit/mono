use brk_error::Result;

use brk_types::Version;
use vecdb::{BytesVec, Database, ImportOptions, ImportableVec, MutableVec};

use super::Vecs;

pub fn forced_import(db: &Database, version: Version) -> Result<Vecs> {
    Ok(Vecs {
        // Stamps now count completed blocks. Rebuild vectors using the old
        // height stamps or missing rollback records.
        txin_index: MutableVec::<BytesVec<_, _>>::forced_import_with(
            ImportOptions::new(db, "txin_index", version + Version::ONE)
                .with_saved_stamped_changes(10),
        )?,
    })
}
