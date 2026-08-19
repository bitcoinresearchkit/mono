"""GET /api/v1/transaction-times?txId[]=..."""

import time

import pytest
from bitview_client import BitviewError

from _lib import show


def test_transaction_times_few(brk, mempool, live):
    """First-seen timestamps must match for a few txids (confirmed → all 0)."""
    txids = [b.txid for b in live.blocks[:3]]
    params = [("txId[]", t) for t in txids]
    b = brk.get_transaction_times(txids)
    m = mempool.get_json("/api/v1/transaction-times", params=params)
    show("GET", f"/api/v1/transaction-times?txId[]={{{len(txids)} txids}}", b, m)
    assert isinstance(b, list) and isinstance(m, list)
    assert len(b) == len(m) == len(txids)
    assert b == m


def test_transaction_times_many(brk, mempool, live):
    """A larger batch (all sample blocks + coinbases) must match exactly."""
    txids = [b.txid for b in live.blocks] + [b.coinbase_txid for b in live.blocks]
    params = [("txId[]", t) for t in txids]
    b = brk.get_transaction_times(txids)
    m = mempool.get_json("/api/v1/transaction-times", params=params)
    show("GET", f"/api/v1/transaction-times?txId[]={{{len(txids)} txids}}",
         f"({len(b)})", f"({len(m)})")
    assert len(b) == len(m) == len(txids)
    assert b == m


def test_transaction_times_single(brk, mempool, live):
    """A single-element batch must return a 1-element list with the same value."""
    txid = live.sample_txid
    params = [("txId[]", txid)]
    b = brk.get_transaction_times([txid])
    m = mempool.get_json("/api/v1/transaction-times", params=params)
    show("GET", f"/api/v1/transaction-times?txId[]={txid[:16]}...", b, m)
    assert isinstance(b, list) and isinstance(m, list)
    assert len(b) == len(m) == 1
    assert b == m


def test_transaction_times_unknown_txid_returns_zero(brk, mempool):
    """Unknown 64-char hex must return [0] on both servers."""
    bad = "0" * 64
    params = [("txId[]", bad)]
    b = brk.get_transaction_times([bad])
    m = mempool.get_json("/api/v1/transaction-times", params=params)
    show("GET", f"/api/v1/transaction-times?txId[]={bad[:16]}...", b, m)
    assert b == [0]
    assert m == [0]


def test_transaction_times_empty_batch_rejected(brk):
    """Empty batch must produce BitviewError(status=400) (mempool returns 500, brk-only check)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_transaction_times([])
    assert exc_info.value.status == 400


def test_transaction_times_malformed_short(brk):
    """Short txid in batch must produce BitviewError(status=400) (mempool silently returns [])."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_transaction_times(["abc"])
    assert exc_info.value.status == 400


def test_transaction_times_mempool_unconfirmed(brk, mempool):
    """Unconfirmed mempool tx: first-seen timestamp must be a plausible
    Unix-second value (post-genesis, not in the future). Cross-observer
    agreement is not asserted: each server records when *it* first saw
    the tx, and rebroadcasts/restarts can put two independent observers
    days or weeks apart on the same txid."""
    txids = mempool.get_json("/api/mempool/txids")
    if not txids:
        pytest.skip("mempool.space mempool currently empty")

    GENESIS_TS = 1231006505
    now = int(time.time())
    skew = 5 * 60

    for txid in txids[:25]:
        try:
            b = brk.get_transaction_times([txid])
        except BitviewError:
            continue
        if not b or b[0] == 0:
            continue
        try:
            m = mempool.get_json(
                "/api/v1/transaction-times", params=[("txId[]", txid)]
            )
        except Exception:
            continue
        if not m or m[0] == 0:
            continue
        show("GET", f"/api/v1/transaction-times?txId[]={txid[:16]}...", b, m)
        assert GENESIS_TS <= b[0] <= now + skew, f"brk first-seen out of plausible range: {b[0]}"
        assert GENESIS_TS <= m[0] <= now + skew, f"mempool first-seen out of plausible range: {m[0]}"
        return
    pytest.skip("no shared unconfirmed tx between brk and mempool.space")
