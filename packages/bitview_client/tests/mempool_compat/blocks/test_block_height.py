"""GET /api/block-height/{height}"""

import re

import pytest

from bitview_client import BitviewError

from _lib import show


HEX_HASH_RE = re.compile(r"^[0-9a-f]{64}$")


def test_block_height(brk, mempool, block):
    """Hash at the given height must match mempool.space and round-trip via /block/{hash}."""
    path = f"/api/block-height/{block.height}"
    b = brk.get_block_by_height(block.height)
    m = mempool.get_text(path)
    show("GET", path, b, m)
    assert HEX_HASH_RE.match(b), f"hash is not 64 lowercase hex chars: {b!r}"
    assert b == m
    assert b == block.hash
    assert brk.get_block(b)["height"] == block.height, "round-trip /block/{hash}.height must match"


def test_block_height_genesis(brk, mempool):
    """Height 0 returns the deterministic genesis hash."""
    path = "/api/block-height/0"
    b = brk.get_block_by_height(0)
    m = mempool.get_text(path)
    show("GET", path, b, m)
    assert b == m
    assert b == "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"


def test_block_height_tip(brk, mempool):
    """Height = tip/height returns tip/hash."""
    tip_height = int(mempool.get_text("/api/blocks/tip/height"))
    tip_hash = mempool.get_text("/api/blocks/tip/hash")
    b = brk.get_block_by_height(tip_height)
    show("GET", f"/api/block-height/{tip_height}", b, tip_hash)
    assert b == tip_hash, f"tip mismatch: brk={b!r} mempool={tip_hash!r}"


def test_block_height_out_of_range(brk):
    """Height past the tip must produce BitviewError(status=404)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_by_height(99_999_999)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )


@pytest.mark.parametrize("bad", ["-1", "abc"])
def test_block_height_malformed(brk, bad):
    """Negative or non-numeric height must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/block-height/{bad}")
    assert exc_info.value.status == 400, (
        f"expected status=400 for {bad!r}, got {exc_info.value.status}"
    )
