"""GET /api/address/{address}/utxo"""

import pytest

from bitview_client import BitviewError

from _lib import assert_same_values, show


# Inactive historical addresses with stable, comparable UTXO sets.
STABLE_ADDRS = [
    ("p2pkh", "12cbQLTFMXRnSzktFkuoG3eHoMeFtpTu3S"),
    ("p2sh", "3D2oetdNuZUqQHPJmcMDDHYoqkyNVsFk9r"),
]

# Genesis pubkey-hash address: tens of thousands of dust UTXOs — exceeds both
# brk's 1000-cap and mempool.space's 500-cap, so both indexers must 400.
HEAVY_ADDR = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"


@pytest.mark.parametrize("atype,addr", STABLE_ADDRS, ids=[a for a, _ in STABLE_ADDRS])
def test_address_utxo_static(brk, mempool, atype, addr):
    """Exact UTXO parity (txid+vout+value+status) for stable historical addresses."""
    path = f"/api/address/{addr}/utxo"
    b = brk.get_address_utxos(addr)
    m = mempool.get_json(path)
    show("GET", f"{path}  [{atype}]", f"({len(b)} utxos)", f"({len(m)} utxos)")
    key = lambda u: (u["txid"], u["vout"])
    assert_same_values(sorted(b, key=key), sorted(m, key=key))


def test_address_utxo_discovered(brk, mempool, live_addrs):
    """Same exact-parity contract over each live-discovered scriptpubkey type."""
    for atype, addr in live_addrs:
        path = f"/api/address/{addr}/utxo"
        b = brk.get_address_utxos(addr)
        m = mempool.get_json(path)
        show("GET", f"{path}  [{atype}]", f"({len(b)} utxos)", f"({len(m)} utxos)")
        key = lambda u: (u["txid"], u["vout"])
        assert_same_values(sorted(b, key=key), sorted(m, key=key))


@pytest.mark.parametrize("atype,addr", STABLE_ADDRS, ids=[a for a, _ in STABLE_ADDRS])
def test_address_utxo_all_confirmed(brk, atype, addr):
    """brk's /utxo only returns confirmed UTXOs (mempool-funded ones are excluded by design)."""
    b = brk.get_address_utxos(addr)
    if not b:
        pytest.skip(f"{addr} has no utxos in brk")
    unconfirmed = [u for u in b if not u["status"]["confirmed"]]
    assert not unconfirmed, (
        f"{addr}: {len(unconfirmed)} unconfirmed UTXO(s) returned: "
        f"{[(u['txid'], u['vout']) for u in unconfirmed[:3]]}"
    )


def test_address_utxo_too_many(brk):
    """Heavy address (>1000 UTXOs) must produce BitviewError(status=400, code=too_many_utxos)."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_address_utxos(HEAVY_ADDR)
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )


def test_address_utxo_invalid(brk):
    """Garbage input must produce a BitviewError carrying HTTP 400."""
    with pytest.raises(BitviewError) as exc_info:
        brk.get_address_utxos("abc")
    assert exc_info.value.status == 400, (
        f"expected status=400, got {exc_info.value.status}"
    )
