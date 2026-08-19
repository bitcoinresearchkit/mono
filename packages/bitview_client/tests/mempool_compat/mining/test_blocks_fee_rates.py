"""GET /api/v1/mining/blocks/fee-rates/{time_period}

Note: there is no values_match test here. brk emits float bucket means; mempool
stores integer per-block percentiles, averages, then casts to INT. Beyond
rounding, the per-block max (avgFee_100) also drifts by several sat/vB,
indicating a real methodology difference (effective fee-rate definition,
RBF/ancestor-fee handling) that no test tolerance should hide."""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_structure, show, summary


PERIODS = ["24h", "3d", "1w", "1m", "3m", "6m", "1y", "2y", "3y", "all"]
PERCENTILES = ["avgFee_0", "avgFee_10", "avgFee_25", "avgFee_50", "avgFee_75", "avgFee_90", "avgFee_100"]


@pytest.mark.parametrize("period", PERIODS)
def test_mining_blocks_fee_rates_structure(brk, mempool, period):
    """Block fee-rate percentiles envelope must match across all periods."""
    path = f"/api/v1/mining/blocks/fee-rates/{period}"
    b = brk.get_block_fee_rates(period)
    m = mempool.get_json(path)
    show("GET", path, summary(b), summary(m))
    assert isinstance(b, list) and isinstance(m, list)
    assert_same_structure(b, m)


def test_mining_blocks_fee_rates_invariants(brk):
    """Series ordering, percentile monotonicity, non-negative rates (period=1m)."""
    period = "1m"
    b = brk.get_block_fee_rates(period)
    show("GET", f"/api/v1/mining/blocks/fee-rates/{period}", summary(b), "-")
    assert len(b) > 0, "expected non-empty fee-rates series for 1m"
    heights = [entry["avgHeight"] for entry in b]
    timestamps = [entry["timestamp"] for entry in b]
    assert heights == sorted(heights), "avgHeight not ascending"
    assert timestamps == sorted(timestamps), "timestamps not ascending"
    assert len(set(heights)) == len(heights), "duplicate avgHeight in series"
    for entry in b:
        values = [entry[k] for k in PERCENTILES]
        assert values == sorted(values), (
            f"percentiles not monotonically non-decreasing at height {entry['avgHeight']}: {values}"
        )
        for k in PERCENTILES:
            assert entry[k] >= 0, f"negative fee rate {k}={entry[k]} at {entry['avgHeight']}"


@pytest.mark.parametrize("bad", ["9000y", "abc", "1d"])
def test_mining_blocks_fee_rates_malformed(brk, bad):
    """Unknown time period must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/v1/mining/blocks/fee-rates/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )
