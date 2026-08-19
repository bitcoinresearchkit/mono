"""GET /api/v1/mining/reward-stats/{block_count}

Note: there is no values_match test here. mempool.space's reward-stats endpoint
serves results anchored to a cached/precomputed block that lags real-time tip
non-deterministically across counts, so any direct numeric comparison is flaky.
The invariants test below covers structural correctness."""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show


COUNTS = [1, 10, 100, 500, 1000]


@pytest.mark.parametrize("count", COUNTS)
def test_mining_reward_stats_structure(brk, mempool, count):
    """Reward stats envelope must match across counts."""
    path = f"/api/v1/mining/reward-stats/{count}"
    b = brk.get_reward_stats(count)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_structure(b, m)


def test_mining_reward_stats_invariants(brk):
    """Range alignment, reward >= fee, totalTx >= block count (count=1000)."""
    count = 1000
    b = brk.get_reward_stats(count)
    show("GET", f"/api/v1/mining/reward-stats/{count}", b, "-")
    start = int(b["startBlock"])
    end = int(b["endBlock"])
    total_reward = int(b["totalReward"])
    total_fee = int(b["totalFee"])
    total_tx = int(b["totalTx"])
    assert start <= end, f"startBlock {start} > endBlock {end}"
    assert end - start + 1 == count, (
        f"range mismatch: {end} - {start} + 1 = {end - start + 1}, expected {count}"
    )
    assert total_fee >= 0, f"negative totalFee: {total_fee}"
    assert total_reward >= total_fee, (
        f"totalReward {total_reward} < totalFee {total_fee} (subsidy must be non-negative)"
    )
    assert total_tx >= count, (
        f"totalTx {total_tx} < block_count {count} (each block has >=1 coinbase tx)"
    )


@pytest.mark.parametrize("bad", ["abc", "-1"])
def test_mining_reward_stats_malformed(brk, bad):
    """Non-numeric or negative block_count must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/reward-stats/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )
