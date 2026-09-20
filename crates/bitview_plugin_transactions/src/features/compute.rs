use bitview_plugin_indexer::Indexer;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::StoredU64;

use super::Vecs;

pub fn compute(vecs: &mut Vecs, indexer: &Indexer, exit: &Exit) -> Result<()> {
    let starting_height = indexer.safe_lengths().height;
    let source = &indexer.vecs().transaction_features.count;
    for (target, source) in [
        (&mut vecs.count.annex, &source.annex),
        (&mut vecs.count.sighash_all, &source.sighash_all),
        (&mut vecs.count.sighash_none, &source.sighash_none),
        (&mut vecs.count.sighash_single, &source.sighash_single),
        (&mut vecs.count.sighash_default, &source.sighash_default),
        (
            &mut vecs.count.sighash_anyone_can_pay,
            &source.sighash_anyone_can_pay,
        ),
        (&mut vecs.count.dust_output, &source.dust_output),
    ] {
        target.compute_cumulative_transformed(starting_height, source, StoredU64::from, exit)?;
    }
    Ok(())
}
