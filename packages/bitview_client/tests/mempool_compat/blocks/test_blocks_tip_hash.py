"""GET /api/blocks/tip/hash"""

import re

from _lib import show


HEX_HASH_RE = re.compile(r"^[0-9a-f]{64}$")


def test_blocks_tip_hash_format(brk, mempool):
    """Tip hash on both servers must be a 64-char lowercase hex string."""
    path = "/api/blocks/tip/hash"
    b = brk.get_block_tip_hash()
    m = mempool.get_text(path)
    show("GET", path, b, m)
    assert HEX_HASH_RE.match(b), f"brk tip hash not 64-char hex: {b!r}"
    assert HEX_HASH_RE.match(m), f"mempool tip hash not 64-char hex: {m!r}"


def test_blocks_tip_hash_resolves(brk):
    """tip/hash must resolve to a real block whose .id matches."""
    tip_hash = brk.get_block_tip_hash()
    blk = brk.get_block(tip_hash)
    show("GET", "/api/blocks/tip/hash", tip_hash, f"block.id={blk['id']} h={blk['height']}")
    assert blk["id"] == tip_hash, f"round-trip mismatch: {blk['id']!r} vs {tip_hash!r}"
    assert blk["height"] >= 0


def test_blocks_tip_hash_matches_height(brk):
    """tip/hash and tip/height must point to the same block (race-free direction)."""
    tip_hash = brk.get_block_tip_hash()
    blk = brk.get_block(tip_hash)
    tip_height = brk.get_block_tip_height()
    show("GET", "/api/blocks/tip/hash", tip_hash, f"block.height={blk['height']} tip/height={tip_height}")
    assert tip_height - blk["height"] in (0, 1), (
        f"tip/hash@{blk['height']} not within 1 block of tip/height={tip_height}"
    )


def test_blocks_tip_hash_matches_recent(brk):
    """tip/hash must equal /api/blocks[0].id."""
    tip_hash = brk.get_block_tip_hash()
    blocks = brk.get_blocks()
    show("GET", "/api/blocks/tip/hash", tip_hash, blocks[0]["id"])
    assert blocks[0]["id"] == tip_hash, (
        f"tip/hash={tip_hash} but /api/blocks[0].id={blocks[0]['id']}"
    )
