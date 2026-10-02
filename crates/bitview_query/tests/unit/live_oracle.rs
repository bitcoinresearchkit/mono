use std::cell::Cell;

use bitview_plugin::Publication;
use brk_oracle::{Config, cents_to_bin};

use super::*;

#[test]
fn same_tip_publications_invalidate_the_warmed_window() {
    let cache = LiveOracle::default();
    let gate = Publication::default();
    let seed = Cell::new(1_000_000u64);
    let builds = Cell::new(0);
    let read = |tip| {
        let _guard = gate.try_read().unwrap();
        cache
            .get_or_try_init(tip, gate.revision(), || {
                builds.set(builds.get() + 1);
                Ok(Oracle::new(
                    cents_to_bin(seed.get() as f64),
                    Config::default(),
                ))
            })
            .unwrap()
            .price_cents()
            .inner()
    };
    let tip = BlockHash::default();
    assert_eq!(read(tip), seed.get());
    assert_eq!(read(tip), seed.get());
    assert_eq!(builds.get(), 1);
    for _ in 0..2 {
        gate.begin_update();
        seed.set(seed.get() + 100_000);
        gate.finish_update();
        assert_eq!(read(tip), seed.get());
        assert_eq!(read(tip), seed.get());
    }
    assert_eq!(builds.get(), 3);
    let next = "11".repeat(32).parse().unwrap();
    assert_eq!(read(next), seed.get());
    assert_eq!(builds.get(), 4);
}
