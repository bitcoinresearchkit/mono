"""GET /api/block/{hash}/txids"""

import re

import pytest

from bitview_client import BitviewError

from _lib import show


HEX_TXID_RE = re.compile(r"^[0-9a-f]{64}$")


def test_block_txids(brk, mempool, block):
    """Ordered txid list must match mempool.space byte-for-byte."""
    path = f"/api/block/{block.hash}/txids"
    b = brk.get_block_txids(block.hash)
    m = mempool.get_json(path)
    show("GET", path, b[:3], m[:3])
    assert b == m
    assert all(HEX_TXID_RE.match(t) for t in b), "every txid must be 64 lowercase hex chars"
    assert b[0] == brk.get_block_txid(block.hash, 0), (
        "txids[0] must equal /txid/0 (split-brain check)"
    )


def test_block_txids_genesis(brk, mempool):
    """Genesis: single-element list with the deterministic coinbase txid."""
    genesis_hash = mempool.get_text("/api/block-height/0")
    path = f"/api/block/{genesis_hash}/txids"
    b = brk.get_block_txids(genesis_hash)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert b == m
    assert b == ["4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"]


def test_block_txids_invalid_hash(brk):
    """Non-hex / wrong-length hash must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txids("notavalidhash")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_block_txids_unknown_hash(brk):
    """Syntactically valid but unknown hash must produce BitviewError(status=404)."""
    unknown = "0000000000000000000000000000000000000000000000000000000000000001"
    with pytest.raises(BitviewError) as exc_info:
        brk.get_block_txids(unknown)
    assert exc_info.value.status == 404, (
        f"expected status=404, got {exc_info.value.status}"
    )
