"""GET /api/block/{hash}/header"""

import re

import pytest

from bitview_client import BitviewError

from _lib import show


HEX_RE = re.compile(r"^[0-9a-f]{160}$")


def test_block_header(brk, mempool, block):
    """80-byte hex block header must be identical for every height in the fixture."""
    path = f"/api/block/{block.hash}/header"
    b = brk.get_block_header(block.hash)
    m = mempool.get_text(path)
    show("GET", path, b, m)
    assert HEX_RE.match(b), f"brk header is not 160 lowercase hex chars: {b!r}"
    assert b == m


def test_block_header_genesis(brk, mempool):
    """Genesis header is byte-deterministic — must match mempool.space exactly."""
    genesis_hash = mempool.get_text("/api/block-height/0")
    path = f"/api/block/{genesis_hash}/header"
    b = brk.get_block_header(genesis_hash)
    m = mempool.get_text(path)
    show("GET", path, b, m)
    assert HEX_RE.match(b)
    assert b == m


def test_block_header_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_header("notavalidhash")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_header_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_header(unknown)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
