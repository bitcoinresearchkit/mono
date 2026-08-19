"""GET /api/block/{hash}/raw"""

import pytest

from bitview_client import BitviewError

from _lib import show


def test_block_raw(brk, mempool, block):
    """Raw block bytes must be byte-identical to mempool.space and start with the /header bytes."""
    path = f"/api/block/{block.hash}/raw"
    b = brk.get_block_raw(block.hash)
    m = mempool.get_bytes(path)
    show("GET", path, f"<{len(b)} bytes>", f"<{len(m)} bytes>")
    assert b == m
    assert b[:80].hex() == brk.get_block_header(block.hash), (
        "first 80 bytes of /raw must match /header response"
    )


def test_block_raw_genesis(brk, mempool):
    """Genesis raw block is byte-deterministic — must match exactly."""
    genesis_hash = mempool.get_text("/api/block-height/0")
    path = f"/api/block/{genesis_hash}/raw"
    b = brk.get_block_raw(genesis_hash)
    m = mempool.get_bytes(path)
    show("GET", path, f"<{len(b)} bytes>", f"<{len(m)} bytes>")
    assert b == m
    assert b[:80].hex() == brk.get_block_header(genesis_hash)


def test_block_raw_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_raw("notavalidhash")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_raw_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_raw(unknown)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
