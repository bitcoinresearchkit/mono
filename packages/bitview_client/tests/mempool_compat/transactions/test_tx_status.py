"""GET /api/tx/{txid}/status"""

import pytest
from bitview_client import BitviewError

from _lib import assert_same_values, show


def test_tx_status_value_parity(brk, mempool, block):
    """Status must match for a confirmed regular tx (multi-era)."""
    path = f"/api/tx/{block.txid}/status"
    b = brk.get_tx_status(block.txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_status_coinbase_value_parity(brk, mempool, block):
    """Status must match for a coinbase tx (multi-era)."""
    path = f"/api/tx/{block.coinbase_txid}/status"
    b = brk.get_tx_status(block.coinbase_txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_status_invariants(brk, live):
    """Recent confirmed tx: confirmed=True, height/hash/time match the block."""
    sample = live.blocks[-1]
    s = brk.get_tx_status(sample.txid)
    show("GET", f"/api/tx/{sample.txid}/status", s, "-")
    assert s["confirmed"] is True
    assert int(s["block_height"]) == sample.height
    assert s["block_hash"] == sample.hash
    assert int(s["block_time"]) > 0


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_status_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/status")
    assert exc_info.value.status == 400


def test_tx_status_malformed_unknown(brk):
    """Valid 64-char hex with no matching tx must produce BitviewError(status=404)."""
    bad = "0" * 64
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/status")
    assert exc_info.value.status == 404


def test_tx_status_mempool_unconfirmed(brk, mempool):
    """Unconfirmed mempool tx: status must be confirmed=false with no block fields."""
    txids = mempool.get_json("/api/mempool/txids")
    if not txids:
        pytest.skip("mempool.space mempool currently empty")

    for txid in txids[:25]:
        try:
            b = brk.get_tx_status(txid)
        except BitviewError:
            continue
        if b.get("confirmed"):
            continue
        try:
            m = mempool.get_json(f"/api/tx/{txid}/status")
        except Exception:
            continue
        if m.get("confirmed"):
            continue
        show("GET", f"/api/tx/{txid}/status", b, m)
        assert_same_values(b, m)
        assert b["confirmed"] is False
        return
    pytest.skip("no shared unconfirmed tx between brk and mempool.space")
