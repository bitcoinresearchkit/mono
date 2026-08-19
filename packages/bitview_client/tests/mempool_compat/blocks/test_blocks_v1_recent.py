"""GET /api/v1/blocks (most recent confirmed blocks with extras)"""

import re

from _lib import assert_same_structure, assert_same_values, show


HEX_HASH_RE = re.compile(r"^[0-9a-f]{64}$")
EXPECTED_COUNT = 15

# Same fee-algo / rounding divergences as /api/v1/block/{hash}.
FEE_ALGO_DIFF = {"medianFee", "medianFeeAmt", "feeRange", "feePercentiles"}
ROUNDING_DIFF = {"avgFeeRate"}
EXTRAS_EXCLUDE = FEE_ALGO_DIFF | ROUNDING_DIFF


def test_blocks_v1_recent_shape(brk, mempool):
    """v1 list must have the same length and element structure as mempool.space."""
    path = "/api/v1/blocks"
    b = brk.get_blocks_v1()
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} blocks)", f"({len(m)} blocks)", max_lines=4)
    assert len(b) == EXPECTED_COUNT, f"expected {EXPECTED_COUNT}, got {len(b)}"
    assert len(b) == len(m), f"length mismatch: brk={len(b)} vs mempool={len(m)}"
    assert_same_structure(b, m)


def test_blocks_v1_recent_chain(brk):
    """Tip-first order, no duplicates, valid previousblockhash chain, stale=False, extras.price set."""
    b = brk.get_blocks_v1()
    heights = [blk["height"] for blk in b]
    show("GET", "/api/v1/blocks", f"heights={heights}", "-")
    assert heights == sorted(heights, reverse=True), f"not tip-first: {heights}"
    assert len(set(heights)) == len(heights), "duplicate heights"
    for blk in b:
        assert HEX_HASH_RE.match(blk["id"]), f"id is not 64 lowercase hex: {blk['id']!r}"
        assert blk["stale"] is False, f"confirmed block stale=True: {blk['id']}"
        assert isinstance(blk["extras"]["price"], (int, float))
        assert blk["extras"]["price"] >= 0
    for i in range(len(b) - 1):
        assert b[i]["previousblockhash"] == b[i + 1]["id"], (
            f"chain break at index {i}"
        )


def test_blocks_v1_recent_tip(brk):
    """The first element must be the tip."""
    b = brk.get_blocks_v1()
    tip_hash = brk.get_block_tip_hash()
    tip_height = brk.get_block_tip_height()
    show("GET", "/api/v1/blocks[0]", b[0]["id"], f"tip={tip_hash} h={tip_height}")
    assert b[0]["id"] == tip_hash
    assert b[0]["height"] == tip_height


def test_blocks_v1_recent_canonical(brk, mempool):
    """The floor block must value-match mempool (modulo fee-algo + rounding divergences)."""
    b = brk.get_blocks_v1()
    floor = b[-1]
    path = f"/api/v1/block/{floor['id']}"
    m = mempool.get_json(path)
    show("GET", path, floor["extras"], m["extras"], max_lines=15)
    assert_same_values(floor, m, exclude=EXTRAS_EXCLUDE)
