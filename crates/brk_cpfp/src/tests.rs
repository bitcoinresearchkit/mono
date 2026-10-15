use brk_types::{CpfpClusterChunk, CpfpClusterTxIndex, FeeRate, Sats, VSize};

use crate::Cluster;

fn linearize(items: &[(u64, &[u32])]) -> Vec<CpfpClusterChunk> {
    let mut cluster = Cluster::default();
    for &(fee, parents) in items {
        cluster.push(
            Sats::new(fee),
            VSize::from(10u64),
            parents
                .iter()
                .map(|&parent| CpfpClusterTxIndex::from(parent)),
        );
    }
    cluster.chunks()
}

#[test]
fn chunk_feerates_never_increase() {
    // `a` alone pays 0.7; `p` pays nothing, but with both its children the three pay 0.867.
    // Either child with `p` pays 0.65, so a chunk grown one ancestor set at a time stops
    // after `a` and leaves a later, richer chunk: the order must merge them.
    let chunks = linearize(&[(7, &[]), (0, &[]), (13, &[1]), (13, &[1])]);
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
    let chunks = linearize(&[(5, &[]), (10, &[])]);
    let txs: Vec<_> = chunks.iter().map(|chunk| chunk.txs.clone()).collect();
    assert_eq!(
        txs,
        [
            [CpfpClusterTxIndex::from(1u32)],
            [CpfpClusterTxIndex::from(0u32)]
        ]
    );
}
