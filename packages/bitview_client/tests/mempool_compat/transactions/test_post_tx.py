"""POST /api/tx (broadcast)

Live broadcast can't be tested in CI — instead we feed every form of
*invalid* payload and verify both servers reject it identically with 400.
"""

import pytest
from bitview_client import BitviewError

from _lib import show


@pytest.mark.parametrize("label,body", [
    ("empty", ""),
    ("whitespace", "    "),
    ("padded_garbage", "  deadbeef  "),
    ("garbage_short", "deadbeef"),
    ("non_hex", "not-hex-zzzz"),
    ("single_byte", "00"),
])
def test_post_tx_invalid_body_rejected(brk, mempool, label, body):
    """Invalid body must be rejected with 400 on both servers."""
    path = "/api/tx"
    with pytest.raises(BitviewError) as ei:
        brk.post_tx(body)
    assert ei.value.status == 400, label
    mempool._wait()
    m = mempool.session.post(f"{mempool.base_url}{path}", data=body, timeout=15)
    show("POST", f"{path} ({label})", "brk=400", f"mempool={m.status_code}")
    assert m.status_code == 400, f"{label}: mempool={m.status_code}"


def test_post_tx_coinbase_rejected(brk, mempool, block):
    """Re-broadcasting a coinbase tx is rejected with 400 on both servers (multi-era)."""
    coinbase_hex = mempool.get_text(f"/api/tx/{block.coinbase_txid}/hex")
    with pytest.raises(BitviewError) as ei:
        brk.post_tx(coinbase_hex)
    assert ei.value.status == 400
    mempool._wait()
    m = mempool.session.post(f"{mempool.base_url}/api/tx", data=coinbase_hex, timeout=15)
    show("POST", f"/api/tx (coinbase h={block.height})", "brk=400", f"mempool={m.status_code}")
    assert m.status_code == 400


def test_post_tx_already_confirmed_rejected(brk, mempool, live):
    """Re-broadcasting an already-confirmed regular tx is rejected with 400 on both."""
    sample = live.blocks[-1]
    tx_hex = mempool.get_text(f"/api/tx/{sample.txid}/hex")
    with pytest.raises(BitviewError) as ei:
        brk.post_tx(tx_hex)
    assert ei.value.status == 400
    mempool._wait()
    m = mempool.session.post(f"{mempool.base_url}/api/tx", data=tx_hex, timeout=15)
    show("POST", f"/api/tx (confirmed h={sample.height})", "brk=400", f"mempool={m.status_code}")
    assert m.status_code == 400
