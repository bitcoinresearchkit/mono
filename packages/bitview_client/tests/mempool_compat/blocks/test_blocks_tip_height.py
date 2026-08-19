"""GET /api/blocks/tip/height"""

import re

from _lib import show


HEX_HASH_RE = re.compile(r"^[0-9a-f]{64}$")


def test_blocks_tip_height_close(brk, mempool):
    """brk and mempool tips must be within 3 blocks (live-race tolerance)."""
    path = "/api/blocks/tip/height"
    b = brk.get_block_tip_height()
    m = int(mempool.get_text(path))
    show("GET", path, b, m)
    assert isinstance(b, int) and b >= 0, f"tip height not a non-negative int: {b!r}"
    assert abs(b - m) <= 3, f"tip heights differ by {abs(b - m)}: brk={b} mempool={m}"


def test_blocks_tip_height_resolves_to_hash(brk):
    """tip/height must resolve to a 64-char hex hash via /api/block-height/{tip}."""
    h = brk.get_block_tip_height()
    bh = brk.get_block_by_height(h)
    show("GET", "/api/blocks/tip/height", h, bh)
    assert HEX_HASH_RE.match(bh), f"block-height/{h} returned non-hash: {bh!r}"


def test_blocks_tip_height_matches_tip_hash(brk):
    """tip/height and tip/hash must point to the same block."""
    h = brk.get_block_tip_height()
    tip_hash = brk.get_block_tip_hash()
    blk = brk.get_block(tip_hash)
    show("GET", "/api/blocks/tip/height", h, f"tip_hash={tip_hash} block.height={blk['height']}")
    assert blk["height"] == h, (
        f"tip/height={h} but /block/{tip_hash}.height={blk['height']}"
    )


def test_blocks_tip_height_matches_recent(brk):
    """tip/height must equal /api/blocks[0].height."""
    h = brk.get_block_tip_height()
    blocks = brk.get_blocks()
    show("GET", "/api/blocks/tip/height", h, blocks[0]["height"])
    assert blocks[0]["height"] == h, (
        f"tip/height={h} but /api/blocks[0].height={blocks[0]['height']}"
    )
