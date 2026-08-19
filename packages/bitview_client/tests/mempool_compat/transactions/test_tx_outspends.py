"""GET /api/tx/{txid}/outspends"""

import pytest
from bitview_client import BitviewError

from _lib import assert_same_values, show


def test_tx_outspends_value_parity(brk, mempool, block):
    """Outspends list must match for a confirmed regular tx (multi-era)."""
    path = f"/api/tx/{block.txid}/outspends"
    b = brk.get_tx_outspends(block.txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_outspends_coinbase_value_parity(brk, mempool, block):
    """Outspends list must match for a coinbase tx (multi-era)."""
    path = f"/api/tx/{block.coinbase_txid}/outspends"
    b = brk.get_tx_outspends(block.coinbase_txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m)


def test_tx_outspends_length_matches_vout(brk, live):
    """Recent tx: len(outspends) must equal len(tx.vout)."""
    sample = live.blocks[-1]
    tx = brk.get_tx(sample.txid)
    o = brk.get_tx_outspends(sample.txid)
    show("GET", f"/api/tx/{sample.txid}/outspends", f"({len(o)} entries)", "-")
    assert len(o) == len(tx["vout"]), f"outspends={len(o)} vs vout={len(tx['vout'])}"


def test_tx_outspends_matches_per_vout(brk, live):
    """Recent tx: each outspends[i] equals /outspend/{i}."""
    sample = live.blocks[-1]
    o = brk.get_tx_outspends(sample.txid)
    show("GET", f"/api/tx/{sample.txid}/outspends", f"({len(o)} entries)", "-")
    for i, expected in enumerate(o):
        single = brk.get_tx_outspend(sample.txid, i)
        assert single == expected, f"outspends[{i}] != /outspend/{i}"


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_outspends_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/outspends")
    assert exc_info.value.status == 400


def test_tx_outspends_malformed_unknown(brk):
    """Valid 64-char hex with no matching tx must produce BitviewError(status=404)."""
    bad = "0" * 64
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}/outspends")
    assert exc_info.value.status == 404
