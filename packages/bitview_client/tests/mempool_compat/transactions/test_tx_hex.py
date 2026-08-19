"""GET /api/tx/{txid}/hex"""

import pytest
from bitview_client import BitviewError

from _lib import show


HEX = set("0123456789abcdef")


def test_tx_hex_value_parity(brk, mempool, block):
    """Raw tx hex must be byte-identical for a confirmed regular tx (multi-era)."""
    path = f"/api/tx/{block.txid}/hex"
    b = brk.get_tx_hex(block.txid)
    m = mempool.get_text(path)
    show("GET", path, b[:80] + "...", m[:80] + "...")
    assert b == m


def test_tx_hex_coinbase_value_parity(brk, mempool, block):
    """Coinbase tx hex must be byte-identical (multi-era)."""
    path = f"/api/tx/{block.coinbase_txid}/hex"
    b = brk.get_tx_hex(block.coinbase_txid)
    m = mempool.get_text(path)
    show("GET", path, b[:80] + "...", m[:80] + "...")
    assert b == m


def test_tx_hex_invariants(brk, live):
    """Recent tx hex: non-empty, even length, strict lowercase hex."""
    sample = live.blocks[-1]
    h = brk.get_tx_hex(sample.txid)
    show("GET", f"/api/tx/{sample.txid}/hex", f"({len(h)} chars)", "-")
    assert isinstance(h, str) and len(h) > 0
    assert len(h) % 2 == 0, f"odd hex length: {len(h)}"
    assert set(h) <= HEX, f"non-hex chars present: {set(h) - HEX}"


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_hex_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/hex")
    assert exc_info.value.status == 400


def test_tx_hex_malformed_unknown(brk):
    """Valid 64-char hex with no matching tx must produce BitviewError(status=404)."""
    bad = "0" * 64
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/hex")
    assert exc_info.value.status == 404
