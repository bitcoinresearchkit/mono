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
