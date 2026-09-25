use bitcoin::{
    Network, OutPoint as BitcoinOutPoint, Txid as BitcoinTxid, blockdata::constants::genesis_block,
    hashes::Hash,
};
use brk_types::{Block, Height, OutputType, TypeIndex, Version};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, WritableVec};

use super::*;
use crate::{Lengths, Readers, Stores, Vecs, processor::BlockBuffers, test_cache::init_cache};

fn txid(value: u64) -> Txid {
    let mut bytes = [0; 32];
    bytes[..8].copy_from_slice(&value.to_le_bytes());
    BitcoinTxid::from_byte_array(bytes).into()
}

#[test]
fn parent_cache_collisions_invalidation_and_chain_continuity() {
    let mut buffers = BlockBuffers::default();
    let a = TxidPrefix::from(txid(1));
    let b = TxidPrefix::from(txid(1 + (1 << 16)));
    let read = ParentRead {
        tx_index: TxIndex::new(7),
        first_txout_index: TxOutIndex::new(11),
    };
    let cache = &mut buffers.inputs.cache;
    assert_eq!(cache.get(a), None);
    cache.insert(a, read);
    cache.insert(b, read);
    assert_eq!(cache.get(a), None);
    assert_eq!(cache.get(b), Some(read));
    cache.invalidate(a);
    assert_eq!(cache.get(b), Some(read));
    let block = Block::from((Height::ZERO, genesis_block(Network::Bitcoin)));
    buffers.finish_block(*block.hash());
    buffers.continue_from(Some(*block.hash()));
    assert_eq!(buffers.inputs.cache.get(b), Some(read));
    buffers.continue_from(None);
    assert_eq!(buffers.inputs.cache.get(b), None);
}

#[test]
fn parent_resolution_preserves_store_updates_bounds_errors_and_pending_reads() -> Result<()> {
    init_cache();
    let dir = tempdir()?;
    let mut vecs = Vecs::forced_import(dir.path(), Version::new(34))?;
    let mut stores = Stores::forced_import(dir.path(), Version::new(34))?;
    let prefixes: Vec<_> = (0..1002).map(|i| TxidPrefix::from(txid(i + 1))).collect();
    for (i, &prefix) in prefixes.iter().enumerate() {
        if i == 500 {
            vecs.transactions.first_txout_index.write()?;
        }
        vecs.transactions
            .first_txout_index
            .push(TxOutIndex::from(i * 2));
        stores
            .transaction_stores_mut()
            .txid_prefixes
            .insert(prefix, TxIndex::from(i));
    }
    vecs.outputs.output_type.push(OutputType::P2PKH);
    vecs.outputs.type_index.push(TypeIndex::new(16));
    vecs.outputs.output_type.write()?;
    vecs.outputs.type_index.write()?;
    let readers = Readers::new(&vecs);
    let block = Block::from((Height::ZERO, genesis_block(Network::Bitcoin)));
    let mut lengths = Lengths::default();
    let processor = BlockProcessor {
        block: &block,
        height: Height::ZERO,
        check_collisions: true,
        lengths: &mut lengths,
        vecs: &mut vecs,
        stores: &mut stores,
        readers: &readers,
    };
    let mut resolver = InputResolver::default();
    assert!(resolver.resolve(&processor, &[])?.is_empty());
    processor.lengths.tx_index = TxIndex::new(2000);
    // Single-parent and large batches, with persisted and pending offsets.
    for len in [0, 1, 1002, 1] {
        for (i, &prefix) in prefixes[..len].iter().enumerate() {
            let read = InputResolver::read_parent(&resolver.cache, &processor, prefix)?;
            assert_eq!(read.tx_index, TxIndex::from(i));
            assert_eq!(read.first_txout_index, TxOutIndex::from(i * 2));
            resolver.cache.insert(prefix, read);
        }
    }
    // Exercise the complete input resolver with repeated historical parents,
    // a same-block parent, and a coinbase, using both cold and warm caches.
    processor.vecs.outputs.output_type.push(OutputType::P2PKH);
    processor.vecs.outputs.type_index.push(TypeIndex::new(17));
    processor.lengths.tx_index = TxIndex::new(2000);
    processor.lengths.txout_index = TxOutIndex::new(3000);
    let mut spending = block.txdata[0].clone();
    let mut previous = spending.input[0].clone();
    previous.previous_output = BitcoinOutPoint::new(txid(1).into(), 1);
    let mut same_block = previous.clone();
    same_block.previous_output = BitcoinOutPoint::new(txid(88888).into(), 0);
    let mut persisted_output = previous.clone();
    persisted_output.previous_output.vout = 0;
    spending.input = vec![previous.clone(), previous, same_block, persisted_output];
    let mut txs = [
        ComputedTx::new(
            TxIndex::new(2000),
            &block.txdata[0],
            txid(88888),
            true,
            0,
            0,
        ),
        ComputedTx::new(TxIndex::new(2001), &spending, txid(88889), true, 0, 0),
    ];
    ComputedTx::set_block_offsets(&mut txs);
    resolver.clear_cache();
    let cold = format!("{:?}", resolver.resolve(&processor, &txs)?);
    let warm = resolver.resolve(&processor, &txs)?;
    assert_eq!(cold, format!("{warm:?}"));
    assert!(matches!(warm[0], InputSource::Coinbase));
    for source in &warm[1..3] {
        let InputSource::PreviousBlock {
            outpoint,
            txout_index,
            output_type,
            type_index,
            ..
        } = source
        else {
            panic!("expected historical parent")
        };
        assert_eq!(outpoint.tx_index(), TxIndex::ZERO);
        assert_eq!(*txout_index, TxOutIndex::new(1));
        assert_eq!(*output_type, OutputType::P2PKH);
        assert_eq!(*type_index, TypeIndex::new(17));
    }
    assert!(
        matches!(warm[3], InputSource::SameBlock { txout_index, .. } if txout_index == TxOutIndex::new(3000))
    );
    assert!(matches!(
        warm[4],
        InputSource::PreviousBlock { txout_index, output_type, type_index, .. }
            if txout_index == TxOutIndex::ZERO
                && output_type == OutputType::P2PKH
                && type_index == TypeIndex::new(16)
    ));

    // Failed output reads must not publish partially resolved parents to the cache.
    resolver.clear_cache();
    spending.input.truncate(1);
    spending.input[0].previous_output.vout = 2;
    let missing_output = ComputedTx::new(TxIndex::new(2000), &spending, txid(99999), true, 0, 0);
    assert!(matches!(
        resolver.resolve(&processor, &[missing_output]),
        Err(Error::Internal("Missing output_type"))
    ));
    assert_eq!(resolver.cache.get(prefixes[0]), None);
    processor.vecs.outputs.output_type.push(OutputType::P2PKH);
    let missing_index = ComputedTx::new(TxIndex::new(2000), &spending, txid(99999), true, 0, 0);
    assert!(matches!(
        resolver.resolve(&processor, &[missing_index]),
        Err(Error::Internal("Missing type_index"))
    ));
    assert_eq!(resolver.cache.get(prefixes[0]), None);
    // Preparing a new block invalidates every prefix it may replace (including BIP30).
    let cached = InputResolver::read_parent(&resolver.cache, &processor, prefixes[0])?;
    resolver.cache.insert(prefixes[0], cached);
    let replacement_id = txid(1);
    let computed = ComputedTx::new(
        TxIndex::new(1002),
        &block.txdata[0],
        replacement_id,
        true,
        0,
        0,
    );
    resolver.prepare(&[computed], TxIndex::new(1002), TxOutIndex::new(2004));
    assert_eq!(resolver.cache.get(prefixes[0]), None);
    processor
        .vecs
        .transactions
        .first_txout_index
        .push(TxOutIndex::new(9999));
    processor
        .stores
        .transaction_stores_mut()
        .txid_prefixes
        .insert(prefixes[0], TxIndex::new(1002));
    let read = InputResolver::read_parent(&resolver.cache, &processor, prefixes[0])?;
    assert_eq!(read.first_txout_index, TxOutIndex::new(9999));
    resolver.cache.insert(prefixes[0], read);
    processor.lengths.tx_index = TxIndex::new(1002);
    assert!(matches!(
        InputResolver::read_parent(&resolver.cache, &processor, prefixes[0]),
        Err(Error::UnknownTxid)
    ));
    // Rollback discards cached values before reading the changed store.
    resolver.clear_cache();
    processor.lengths.tx_index = TxIndex::new(2000);
    processor
        .stores
        .transaction_stores_mut()
        .txid_prefixes
        .remove(prefixes[0]);
    assert!(matches!(
        InputResolver::read_parent(&resolver.cache, &processor, prefixes[0]),
        Err(Error::UnknownTxid)
    ));
    assert_eq!(resolver.cache.get(prefixes[0]), None);
    Ok(())
}
