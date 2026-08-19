"""GET /api/tx/{txid}/outspend/{vout}"""

import pytest
from bitview_client import BitviewError

from _lib import assert_same_values, show


HEX = set("0123456789abcdef")


def test_tx_outspend_first_value_parity(brk, mempool, block):
    """Spending status of vout 0 must match (multi-era)."""
    path = f"/api/tx/{block.txid}/outspend/0"
    b = brk.get_tx_outspend(block.txid, 0)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_outspend_last_value_parity(brk, mempool, block):
    """Spending status of the last vout must match (multi-era)."""
    tx = mempool.get_json(f"/api/tx/{block.txid}")
    last_vout = len(tx["vout"]) - 1
    path = f"/api/tx/{block.txid}/outspend/{last_vout}"
    b = brk.get_tx_outspend(block.txid, last_vout)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_outspend_coinbase_value_parity(brk, mempool, block):
    """Coinbase vout 0 spending status must match (multi-era)."""
    path = f"/api/tx/{block.coinbase_txid}/outspend/0"
    b = brk.get_tx_outspend(block.coinbase_txid, 0)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_outspend_out_of_range(brk, mempool, block):
    """Past-the-end vout returns {spent: false} on both servers (no 404)."""
    tx = mempool.get_json(f"/api/tx/{block.txid}")
    bad_vout = len(tx["vout"]) + 100
    path = f"/api/tx/{block.txid}/outspend/{bad_vout}"
    b = brk.get_tx_outspend(block.txid, bad_vout)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert b == m


def test_tx_outspend_invariants_spent(brk, live):
    """An old (h100) tx output: if spent, validate the spending-tx envelope."""
    h100 = next((b for b in live.blocks if b.height == 100), None)
    if h100 is None:
        pytest.skip("h100 not discovered")
    o = brk.get_tx_outspend(h100.txid, 0)
    show("GET", f"/api/tx/{h100.txid}/outspend/0", o, "-")
    if o["spent"] is True:
        assert isinstance(o["txid"], str) and len(o["txid"]) == 64 and set(o["txid"]) <= HEX
        assert int(o["vin"]) >= 0
        assert o["status"]["confirmed"] is True
        assert int(o["status"]["block_height"]) >= h100.height


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_outspend_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/outspend/0")
    assert exc_info.value.status == 400


def test_tx_outspend_malformed_unknown_tx(brk):
    """Valid 64-char hex with no matching tx must produce BitviewError(status=404)."""
    bad = "0" * 64
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/outspend/0")
    assert exc_info.value.status == 404
