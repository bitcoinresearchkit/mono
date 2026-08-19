"""GET /api/mempool/recent"""

from _lib import assert_same_structure, show


HEX = set("0123456789abcdef")
MAX_RECENT = 10


def test_mempool_recent_structure(brk, mempool):
    """Recent mempool txs envelope must match across the full list."""
    path = "/api/mempool/recent"
    b = brk.get_mempool_recent()
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert isinstance(b, list) and isinstance(m, list)
    assert len(b) > 0, "brk recent list is empty"
    assert_same_structure(b, m)


def test_mempool_recent_invariants(brk):
    """Length cap, txid format, positive fee/vsize/value, unique txids."""
    b = brk.get_mempool_recent()
    show("GET", "/api/mempool/recent", b, "-")
    assert 1 <= len(b) <= MAX_RECENT, f"recent length out of range: {len(b)}"
    txids = []
    for i, tx in enumerate(b):
        txid = tx["txid"]
        assert isinstance(txid, str) and len(txid) == 64 and set(txid) <= HEX, (
            f"entry {i} txid malformed: {txid!r}"
        )
        assert int(tx["fee"]) >= 0, f"entry {i} negative fee: {tx['fee']}"
        assert int(tx["vsize"]) > 0, f"entry {i} non-positive vsize: {tx['vsize']}"
        assert int(tx["value"]) > 0, f"entry {i} non-positive value: {tx['value']}"
        txids.append(txid)
    assert len(txids) == len(set(txids)), f"duplicate txids in recent: {txids}"
