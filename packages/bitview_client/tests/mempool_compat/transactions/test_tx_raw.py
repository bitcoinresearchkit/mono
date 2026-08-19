"""GET /api/tx/{txid}/raw"""

import pytest
from bitview_client import BitviewError

from _lib import show


def test_tx_raw_value_parity(brk, mempool, block):
    """Raw tx bytes must be byte-identical for a confirmed regular tx (multi-era)."""
    path = f"/api/tx/{block.txid}/raw"
    b = brk.get_tx_raw(block.txid)
    m = mempool.get_bytes(path)
    show("GET", path, f"<{len(b)} bytes>", f"<{len(m)} bytes>")
    assert b == m


def test_tx_raw_coinbase_value_parity(brk, mempool, block):
    """Coinbase tx bytes must be byte-identical (multi-era)."""
    path = f"/api/tx/{block.coinbase_txid}/raw"
    b = brk.get_tx_raw(block.coinbase_txid)
    m = mempool.get_bytes(path)
    show("GET", path, f"<{len(b)} bytes>", f"<{len(m)} bytes>")
    assert b == m


def test_tx_raw_matches_hex(brk, live):
    """Recent tx: raw bytes' hex must equal /hex endpoint output exactly."""
    sample = live.blocks[-1]
    raw = brk.get_tx_raw(sample.txid)
    hex_str = brk.get_tx_hex(sample.txid)
    show("GET", f"/api/tx/{sample.txid}/raw", f"<{len(raw)} bytes>", "-")
    assert isinstance(raw, bytes) and len(raw) > 0
    assert raw.hex() == hex_str, "raw.hex() != /hex"


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_raw_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get(f"/api/tx/{bad}/raw")
    assert exc_info.value.status == 400


def test_tx_raw_malformed_unknown(brk):
    """Valid 64-char hex with no matching tx must produce BitviewError(status=404)."""
    bad = "0" * 64
    with pytest.raises(BitviewError) as exc_info:
        brk.get(f"/api/tx/{bad}/raw")
    assert exc_info.value.status == 404
