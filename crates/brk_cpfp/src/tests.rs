use brk_types::{CpfpClusterTxIndex, FeeRate, Sats, VSize};

use crate::{ChunkInput, linearize};

fn input(fee: u64, parents: &[CpfpClusterTxIndex]) -> ChunkInput<'_> {
    ChunkInput {
        fee: Sats::new(fee),
        vsize: VSize::from(10u64),
        parents,
    }
}

#[test]
fn chunk_feerates_never_increase() {
    let p = [CpfpClusterTxIndex::from(1u32)];
    // `a` alone pays 0.7; `p` pays nothing, but with both its children the three pay 0.867.
    // Either child with `p` pays 0.65, so a chunk grown one ancestor set at a time stops
    // after `a` and leaves a later, richer chunk: the order must merge them.
    let items = [input(7, &[]), input(0, &[]), input(13, &p), input(13, &p)];
    let chunks = linearize(&items);
    assert_eq!(chunks.len(), 1);
    assert_eq!(
        chunks[0].txs,
        (0..4u32).map(CpfpClusterTxIndex::from).collect::<Vec<_>>()
    );
    assert_eq!(
        chunks[0].feerate,
        FeeRate::from((Sats::new(33), VSize::from(40u64)))
    );

    // Independent transactions keep their own chunks, richest first.
    let items = [input(5, &[]), input(10, &[])];
    let chunks = linearize(&items);
    let txs: Vec<_> = chunks.iter().map(|chunk| chunk.txs.clone()).collect();
    assert_eq!(
        txs,
        [
            [CpfpClusterTxIndex::from(1u32)],
            [CpfpClusterTxIndex::from(0u32)]
        ]
    );
}
