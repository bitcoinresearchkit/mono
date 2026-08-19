"""GET /api/blocks (most recent confirmed blocks)"""

import re

from _lib import assert_same_structure, assert_same_values, show


HEX_HASH_RE = re.compile(r"^[0-9a-f]{64}$")
EXPECTED_COUNT = 10


def test_blocks_recent_shape(brk, mempool):
    """Recent blocks list must have the same length and element structure as mempool.space."""
    path = "/api/blocks"
    b = brk.get_blocks()
    m = mempool.get_json(path)
    show(
        "GET", path,
        f"({len(b)} blocks, {b[-1]['height']}-{b[0]['height']})",
        f"({len(m)} blocks, {m[-1]['height']}-{m[0]['height']})",
    )
    assert len(b) == EXPECTED_COUNT, f"expected {EXPECTED_COUNT}, got {len(b)}"
    assert len(b) == len(m), f"length mismatch: brk={len(b)} vs mempool={len(m)}"
    assert_same_structure(b, m)


def test_blocks_recent_chain(brk):
    """Tip-first order, no duplicates, and previousblockhash links each block to its successor."""
    b = brk.get_blocks()
    heights = [blk["height"] for blk in b]
    show("GET", "/api/blocks", f"heights={heights}", "-")
    assert heights == sorted(heights, reverse=True), f"not tip-first: {heights}"
    assert len(set(heights)) == len(heights), "duplicate heights"
    for blk in b:
        assert HEX_HASH_RE.match(blk["id"]), f"id is not 64 lowercase hex: {blk['id']!r}"
    for i in range(len(b) - 1):
        assert b[i]["previousblockhash"] == b[i + 1]["id"], (
            f"chain break at index {i}: prev={b[i]['previousblockhash']!r} "
            f"vs next.id={b[i + 1]['id']!r}"
        )


def test_blocks_recent_tip(brk):
    """The first element of /api/blocks must be the tip."""
    b = brk.get_blocks()
    tip_hash = brk.get_block_tip_hash()
    tip_height = brk.get_block_tip_height()
    show("GET", "/api/blocks[0]", b[0], f"tip={tip_hash} h={tip_height}")
    assert b[0]["id"] == tip_hash, f"head mismatch: {b[0]['id']!r} vs tip={tip_hash!r}"
    assert b[0]["height"] == tip_height


def test_blocks_recent_canonical(brk, mempool):
    """The floor block (least likely to race vs mempool's tip) must value-match mempool."""
    b = brk.get_blocks()
    floor = b[-1]
    path = f"/api/block/{floor['id']}"
    m = mempool.get_json(path)
    show("GET", path, floor, m, max_lines=20)
    assert_same_values(floor, m)
