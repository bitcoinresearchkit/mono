use std::{env, time::Duration};

use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::Height;

use crate::{Blocks, State};

fn offline_client() -> Client {
    Client::new_with("http://127.0.0.1:1", Auth::None, 0, Duration::ZERO).unwrap()
}

#[test]
fn reversed_ranges_skip_both_sources() {
    let client = offline_client();
    let reader = Reader::new_without_rlimit(
        env::temp_dir().join("brk-iterator-unused-block-directory"),
        &client,
    );
    let blocks = Blocks::new(&client, &reader);
    for mut iter in [
        blocks.range(Height::new(5), Height::new(4)).unwrap(),
        blocks.range(Height::new(u32::MAX), Height::ZERO).unwrap(),
    ] {
        assert!(matches!(iter.0, State::Empty));
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}

#[test]
fn public_factory_preserves_height_boundaries() {
    let client = offline_client();
    let reader = Reader::new_without_rlimit(env::temp_dir(), &client);
    let blocks = Blocks::new(&client, &reader);
    for height in [0, u32::MAX] {
        let iter = blocks
            .range(Height::new(height), Height::new(height))
            .unwrap();
        let State::Rpc { mut heights, .. } = iter.0 else {
            panic!("expected RPC")
        };
        assert_eq!(heights.next(), Some(height));
        assert_eq!(heights.next(), None);
        assert_eq!(heights.next(), None);
    }

    assert!(
        blocks.range(Height::ZERO, Height::new(u32::MAX)).is_err(),
        "the full height domain must select the reader, not wrap into a small RPC range"
    );
}
