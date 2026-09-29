use std::ops::Range;

use brk_error::Error;
use brk_types::{Cents, Height, Timestamp, Version};
use statedb::{Amount, Creations, History, Spends};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportableVec, LazyVec, ReadableCloneableVec, WritableVec,
};

use crate::{OriginUrpd, Replay, ReplayInputs};

fn check(replay: &mut Replay, range: Range<usize>, inputs: ReplayInputs<'_>) {
    let prices = inputs.prices.collect_range_dyn(0, inputs.history.len());
    let timestamps = inputs.timestamps.collect_range_dyn(0, inputs.history.len());
    let mut seen = 0;
    replay
        .for_each(range.clone(), inputs, |height, close, source| {
            let h = usize::from(height);
            assert_eq!(h, range.start + seen);
            assert_eq!(close, prices[h]);
            let state = inputs.history.state_at(h + 1)?;
            let rebuilt = OriginUrpd::new(&state, &prices, &timestamps)?;
            // A resident replay can have fewer known future price buckets. Compare
            // its nonzero entries rather than the layout of empty scratch buckets.
            assert_eq!(
                source.iter().collect::<Vec<_>>(),
                rebuilt.iter().collect::<Vec<_>>()
            );
            seen += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(seen, range.len());
}

#[test]
fn replay_matches_reconstruction_after_resume_reprice_reorg_and_failure() {
    let dir = tempdir().unwrap();
    let db = Database::open(&dir.path().join("vecs")).unwrap();
    let mut prices = BytesVec::<Height, Cents>::import(&db, "prices", Version::ONE).unwrap();
    let mut timestamps =
        BytesVec::<Height, Timestamp>::import(&db, "timestamps", Version::ONE).unwrap();
    let mut history = History::open(dir.path()).unwrap();
    let mut spends = Spends::open(dir.path()).unwrap();
    let mut creations = Creations::open(dir.path()).unwrap();
    for h in 0..12 {
        prices.push(Cents::new((h + 1) as u64 * 150));
        timestamps.push(Timestamp::new(1_000_000_000 + h as u32 * 86400));
        creations
            .push(
                [h as u8; 32],
                Amount {
                    sats: 100 + h as u64 * 7,
                    count: 2,
                },
                None,
            )
            .unwrap();
        let rows = (h > 0).then(|| ((h - 1) as u32, Amount { sats: 30, count: 1 }));
        spends.push([h as u8; 32], rows).unwrap();
    }
    prices.write().unwrap();
    timestamps.write().unwrap();
    creations.commit().unwrap();
    spends.commit().unwrap();
    history
        .advance(0, 12, &spends, &creations, |_, _| Ok(()))
        .unwrap();
    let mut replay = Replay::default();
    {
        let reader = history.reader(&spends, &creations).unwrap();
        let inputs = ReplayInputs {
            history: &reader,
            prices: &prices,
            timestamps: &timestamps,
        };
        check(&mut replay, 0..4, inputs);
        check(&mut replay, 4..4, inputs);
        check(&mut replay, 4..8, inputs);
        check(&mut Replay::default(), 8..12, inputs);
        let result = replay.for_each(8..12, inputs, |h, _, _| {
            if usize::from(h) == 10 {
                Err(Error::Internal("injected consumer failure"))
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        check(&mut replay, 8..12, inputs);
        assert!(
            replay
                .for_each(12..13, inputs, |_, _, _| unreachable!())
                .is_err()
        );
        check(&mut replay, 0..4, inputs);
        let repriced = LazyVec::<Height, Cents, Height, Cents>::init(
            "repriced",
            Version::TWO,
            prices.read_only_boxed_clone(),
            |_, p| Cents::new(u64::from(p) * 2),
        );
        check(
            &mut replay,
            4..8,
            ReplayInputs {
                prices: &repriced,
                ..inputs
            },
        );
        check(&mut replay, 0..4, inputs);
    }
    // Replace the cached state's last block without changing the source versions.
    spends.truncate(3).unwrap();
    creations.truncate(3).unwrap();
    for h in 3..6 {
        creations
            .push(
                [100 + h as u8; 32],
                Amount {
                    sats: 900,
                    count: 2,
                },
                None,
            )
            .unwrap();
        spends.push([100 + h as u8; 32], []).unwrap();
    }
    creations.commit().unwrap();
    spends.commit().unwrap();
    history
        .advance(3, 6, &spends, &creations, |_, _| Ok(()))
        .unwrap();
    let reader = history.reader(&spends, &creations).unwrap();
    check(
        &mut replay,
        4..6,
        ReplayInputs {
            history: &reader,
            prices: &prices,
            timestamps: &timestamps,
        },
    );
}
