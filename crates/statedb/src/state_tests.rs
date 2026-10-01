use crate::{Amount, State};

#[test]
fn failed_block_restores_all_touched_origins_and_totals() {
    let amount = Amount { sats: 10, count: 1 };
    let mut state = State::new(vec![amount], [1; 32]).unwrap();
    let mut scratch = Vec::new();
    assert!(
        state
            .apply(
                [2; 32],
                amount,
                [(0, amount), (0, amount)].into_iter(),
                Amount { sats: 20, count: 2 },
                &mut scratch
            )
            .is_err()
    );
    assert_eq!(state.amounts(), [amount]);
    assert_eq!(state.total(), amount);
    assert_eq!(state.hash(), [1; 32]);
}

#[test]
fn state_totals_and_removal_arithmetic_reject_overflow_without_panicking() {
    let large = Amount {
        sats: u64::MAX,
        count: 1,
    };
    let small = Amount { sats: 1, count: 1 };
    assert!(State::new(vec![large, small], [0; 32]).is_err());
    assert!(
        State::new(
            vec![
                Amount {
                    sats: 0,
                    count: u64::MAX
                },
                small
            ],
            [0; 32]
        )
        .is_err()
    );
    let mut state = State::new(vec![large], [0; 32]).unwrap();
    let mut scratch = Vec::new();
    assert!(
        state
            .apply([1; 32], small, [].into_iter(), small, &mut scratch)
            .is_err()
    );
    assert_eq!(state.amounts(), [large]);
    assert_eq!(state.total(), large);
}
