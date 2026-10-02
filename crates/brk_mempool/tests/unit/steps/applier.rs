use brk_types::{FeeRate, Sats, TxOut, Txid, VSize};

use super::*;
use crate::{
    cycle::CycleDiff,
    steps::preparer::{TxAddition, TxsPulled},
    test_support::{fake_entry_info, fake_tx, p2wpkh_script},
};

fn fresh_addition(seed: u8, fee: u64, vsize: u64) -> (TxAddition, Txid) {
    let prev = Some(TxOut::from((p2wpkh_script(seed), Sats::from(2_500u64))));
    let tx = fake_tx(seed, &[prev], &[(p2wpkh_script(seed + 1), 1_234)]);
    let txid = tx.txid;
    let info = fake_entry_info(txid, fee, vsize);
    let entry = TxEntry::new(&info, vsize, false);
    (TxAddition::Fresh { tx, entry }, txid)
}

fn fresh_pulled(addition: TxAddition) -> TxsPulled {
    TxsPulled {
        live_len: 1,
        added: vec![addition],
        removed: vec![],
    }
}

#[test]
fn revived_path_exhumes_body_from_graveyard() {
    let mut lock = State::default();
    let snapshot = Snapshot::default();
    let (addition, txid) = fresh_addition(0xC1, 300, 100);
    let TxAddition::Fresh { tx, entry } = addition else {
        unreachable!();
    };
    // Pre-load the graveyard with this tx, then submit a Revived
    // addition that re-publishes it without a raw body.
    let rate = FeeRate::from((entry.fee, entry.vsize));
    lock.graveyard
        .bury(tx, entry.clone(), rate, TxRemoval::Vanished);

    let mut diff = CycleDiff::default();
    apply(
        &mut lock,
        &snapshot,
        fresh_pulled(TxAddition::Revived { entry }),
        &mut diff,
    );

    let state = &lock;
    assert!(state.txs.contains(&txid), "revived tx republished");
    assert!(state.graveyard.get(&txid).is_none(), "tomb consumed");
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0].txid, txid);
    assert_eq!(diff.added[0].fee, Sats::new(300));
}

#[test]
fn bury_preserves_chunk_rate_from_snapshot_or_falls_back_to_isolated_rate() {
    for has_snapshot in [false, true] {
        let mut lock = State::default();
        let (addition, txid) = fresh_addition(0xC2, 100, 100);
        let mut diff = CycleDiff::default();
        apply(
            &mut lock,
            &Snapshot::default(),
            fresh_pulled(addition),
            &mut diff,
        );
        assert!(lock.txs.contains(&txid));
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].txid, txid);

        let isolated_rate = FeeRate::from((Sats::from(100u64), VSize::from(100u64)));
        let cpfp_rate = FeeRate::from((Sats::from(500u64), VSize::from(100u64)));
        let prefix = TxidPrefix::from(&txid);
        let (snapshot, expected) = if has_snapshot {
            (
                Snapshot::for_test_with_chunk_rates(&[(prefix, cpfp_rate, txid)]),
                cpfp_rate,
            )
        } else {
            (Snapshot::default(), isolated_rate)
        };

        let mut diff = CycleDiff::default();
        apply(
            &mut lock,
            &snapshot,
            TxsPulled {
                live_len: 0,
                added: vec![],
                removed: vec![(prefix, TxRemoval::Vanished)],
            },
            &mut diff,
        );
        assert_eq!(diff.removed.len(), 1);
        assert_eq!(diff.removed[0].chunk_rate, expected);
        assert_eq!(lock.graveyard.get(&txid).unwrap().chunk_rate, expected);
    }
}
