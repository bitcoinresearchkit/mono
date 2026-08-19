"""GET /api/tx/{txid}"""

import pytest
from bitview_client import BitviewError

from _lib import assert_same_values, show


def test_tx_by_id_value_parity(brk, mempool, block):
    """Full transaction data must match for a confirmed regular tx (multi-era)."""
    path = f"/api/tx/{block.txid}"
    b = brk.get_tx(block.txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m, exclude={"sigops"})


def test_tx_coinbase_value_parity(brk, mempool, block):
    """Coinbase transaction must match (multi-era)."""
    path = f"/api/tx/{block.coinbase_txid}"
    b = brk.get_tx(block.coinbase_txid)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_values(b, m, exclude={"sigops"})


def test_tx_invariants_regular(brk, live):
    """Recent regular tx: fee accounting, weight <= 4*size, status confirmed."""
    sample = live.blocks[-1]
    if sample.txid == sample.coinbase_txid:
        pytest.skip("recent block has only coinbase")
    tx = brk.get_tx(sample.txid)
    show("GET", f"/api/tx/{sample.txid}", tx, "-")
    assert tx["txid"] == sample.txid
    assert len(tx["vin"]) > 0 and len(tx["vout"]) > 0
    assert int(tx["size"]) > 0
    assert 0 < int(tx["weight"]) <= 4 * int(tx["size"])
    sum_in = sum(int(v["prevout"]["value"]) for v in tx["vin"])
    sum_out = sum(int(o["value"]) for o in tx["vout"])
    assert sum_in - sum_out == int(tx["fee"])
    assert tx["status"]["confirmed"] is True


def test_tx_invariants_coinbase(brk, live):
    """Recent coinbase: single vin, is_coinbase, no prevout, status confirmed."""
    sample = live.blocks[-1]
    tx = brk.get_tx(sample.coinbase_txid)
    show("GET", f"/api/tx/{sample.coinbase_txid}", tx, "-")
    assert tx["txid"] == sample.coinbase_txid
    assert len(tx["vin"]) == 1
    cbin = tx["vin"][0]
    assert cbin["is_coinbase"] is True
    assert cbin["prevout"] is None
    assert int(tx["fee"]) == 0
    assert tx["status"]["confirmed"] is True


@pytest.mark.parametrize("bad", ["abc", "deadbeef"])
def test_tx_malformed_short(brk, bad):
    """Short txid must produce BitviewError(status=400)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}")
    assert exc_info.value.status == 400


def test_tx_malformed_unknown(brk):
    """Valid 64-char hex with no matching tx must produce BitviewError(status=404)."""
    bad = "0" * 64
    with pytest.raises(BitviewError) as exc_info:
        brk.get_text(f"/api/tx/{bad}")
    assert exc_info.value.status == 404
