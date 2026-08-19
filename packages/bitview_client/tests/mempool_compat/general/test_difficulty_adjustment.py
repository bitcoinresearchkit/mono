"""GET /api/v1/difficulty-adjustment"""

import time

from _lib import assert_same_structure, assert_same_values, show


REQUIRED_KEYS = {
    "progressPercent", "difficultyChange", "estimatedRetargetDate",
    "remainingBlocks", "remainingTime", "previousRetarget",
    "previousTime", "nextRetargetHeight", "timeAvg",
    "adjustedTimeAvg", "timeOffset", "expectedBlocks",
}

# Fields derived purely from chain state (heights and confirmed block
# timestamps) — these must match mempool.space within float tolerance.
# All other fields depend on the wall clock at request time and will drift.
CHAIN_DETERMINISTIC_FIELDS = {
    "progressPercent", "remainingBlocks", "nextRetargetHeight",
    "previousTime", "previousRetarget",
}


def test_difficulty_adjustment_shape(brk, mempool):
    """Response must have every key mempool.space returns, with matching types."""
    path = "/api/v1/difficulty-adjustment"
    b = brk.get_difficulty_adjustment()
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_structure(b, m)
    missing = REQUIRED_KEYS - set(b.keys())
    assert not missing, f"brk missing keys: {missing}"


def test_difficulty_adjustment_chain_values_match_mempool(brk, mempool):
    """Chain-derived fields must equal mempool.space's within float tolerance."""
    b = brk.get_difficulty_adjustment()
    m = mempool.get_json("/api/v1/difficulty-adjustment")
    assert_same_values(
        {k: b[k] for k in CHAIN_DETERMINISTIC_FIELDS},
        {k: m[k] for k in CHAIN_DETERMINISTIC_FIELDS},
    )


def test_difficulty_adjustment_invariants(brk):
    """Cross-field invariants and protocol-level bounds."""
    d = brk.get_difficulty_adjustment()
    now_ms = int(time.time() * 1000)

    assert 0 <= d["progressPercent"] <= 100
    assert 0 <= d["remainingBlocks"] <= 2016
    assert d["nextRetargetHeight"] % 2016 == 0

    blocks_done = 2016 - d["remainingBlocks"]
    assert abs(d["progressPercent"] - blocks_done / 2016 * 100) < 1e-6

    # Bitcoin protocol clamps a single retarget to a 4× factor in either
    # direction → difficulty change ∈ [-75%, +300%]. previousRetarget reports
    # the same quantity historically, so the same bound applies.
    assert -75.0 <= d["difficultyChange"] <= 300.0
    assert -75.0 <= d["previousRetarget"] <= 300.0

    # expectedBlocks: wall-clock-derived count of blocks that should have
    # arrived in the current epoch. Bounded above by ~2 epochs in any sane
    # state (a much larger value would mean clock skew or epoch-start bug).
    assert 0 <= d["expectedBlocks"] <= 2 * 2016

    # timeAvg in milliseconds. Sanity: between 1s and 1h per block.
    assert 1_000 <= d["timeAvg"] <= 3_600_000
    assert 1_000 <= d["adjustedTimeAvg"] <= 3_600_000

    # remainingTime is remainingBlocks * adjustedTimeAvg (matches mempool.space).
    assert d["remainingTime"] == d["remainingBlocks"] * d["adjustedTimeAvg"]

    assert d["estimatedRetargetDate"] > now_ms
    assert d["previousTime"] * 1000 < now_ms


def test_difficulty_adjustment_previous_time_matches_chain(brk):
    """previousTime must be the timestamp of the block at the most recent retarget."""
    d = brk.get_difficulty_adjustment()
    epoch_start_height = d["nextRetargetHeight"] - 2016
    epoch_start_hash = brk.get_block_by_height(epoch_start_height)
    epoch_start_block = brk.get_block(epoch_start_hash)
    assert d["previousTime"] == epoch_start_block["timestamp"], (
        f"previousTime={d['previousTime']} but block at height "
        f"{epoch_start_height} has timestamp={epoch_start_block['timestamp']}"
    )


def test_difficulty_adjustment_next_retarget_aligned_with_tip(brk):
    """The tip must sit inside the epoch ending at nextRetargetHeight."""
    d = brk.get_difficulty_adjustment()
    tip = int(brk.get_block_tip_height())
    assert d["nextRetargetHeight"] - 2016 <= tip < d["nextRetargetHeight"], (
        f"tip={tip} not in current epoch "
        f"[{d['nextRetargetHeight'] - 2016}, {d['nextRetargetHeight']})"
    )
    assert d["remainingBlocks"] == d["nextRetargetHeight"] - tip - 1 \
        or d["remainingBlocks"] == d["nextRetargetHeight"] - tip, (
            "remainingBlocks must equal blocks left to next retarget "
            "(off-by-one tolerated for tip-race during request)"
        )
