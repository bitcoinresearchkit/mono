"""GET /api/v1/historical-price (with and without timestamp)"""

import time

import pytest

from _lib import assert_same_structure, show


HOUR4 = 14400  # brk's bucket size for the price series

# Well-known timestamps from different eras.
HISTORICAL_TIMESTAMPS = [
    1231006505,   # genesis block (2009-01-03)
    1354116278,   # block 210000 — first halving (2012-11-28)
    1468082773,   # block 420000 — second halving (2016-07-09)
    1588788036,   # block 630000 — third halving (2020-05-11)
    1713571767,   # block 840000 — fourth halving (2024-04-20)
]


def test_historical_price_bulk_shape(brk, mempool):
    """Bulk response must structurally match mempool.space and have a non-empty `prices` list."""
    path = "/api/v1/historical-price"
    b = brk.get_historical_price()
    m = mempool.get_json(path)
    show("GET", path, b, m, max_lines=10)
    assert_same_structure(b, m)
    assert isinstance(b["prices"], list) and b["prices"], "brk returned no prices"
    assert b["exchangeRates"] == {}, "brk must not emit fiat exchange rates"


def test_historical_price_bulk_ordering(brk):
    """Brk's bulk series must be strictly ascending in `time`, span pre-2010 to within ~7 days of now."""
    d = brk.get_historical_price()
    times = [p["time"] for p in d["prices"]]
    assert times == sorted(times), "bulk prices must be ascending"
    assert len(set(times)) == len(times), "bulk prices must have unique timestamps"
    assert times[0] < 1262304000, f"first entry must be pre-2010, got {times[0]}"
    now = int(time.time())
    assert times[-1] > now - 7 * 86400, f"latest entry stale: {times[-1]} vs now {now}"


def test_historical_price_bulk_usd_sane(brk):
    """No negative or null USD values; the latest entry sits in the protocol-realistic spot band."""
    d = brk.get_historical_price()
    usds = [p["USD"] for p in d["prices"]]
    assert all(isinstance(u, (int, float)) for u in usds), "USD must be numeric"
    assert all(u >= 0 for u in usds), "USD must be non-negative"
    assert 1_000 < usds[-1] < 10_000_000, f"latest USD={usds[-1]} outside sane spot bounds"


@pytest.mark.parametrize("ts", HISTORICAL_TIMESTAMPS, ids=[
    "genesis", "halving1", "halving2", "halving3", "halving4",
])
def test_historical_price_at_era(brk, mempool, ts):
    """Single-entry response with bucket-aligned `time`, USD within 25% of mempool when both have data."""
    path = f"/api/v1/historical-price?timestamp={ts}"
    b = brk.get_historical_price(timestamp=ts)
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_structure(b, m)
    assert len(b["prices"]) == 1, f"expected 1 price entry, got {len(b['prices'])}"

    entry = b["prices"][0]
    bucket_start = (ts // HOUR4) * HOUR4
    assert entry["time"] == bucket_start, (
        f"bucket misaligned: entry time={entry['time']} vs expected {bucket_start} for ts={ts}"
    )

    if m["prices"] and entry["USD"] > 0 and m["prices"][0]["USD"] > 0:
        m_usd = m["prices"][0]["USD"]
        drift = abs(entry["USD"] - m_usd) / m_usd
        assert drift < 0.25, (
            f"USD diverges from mempool by {drift:.1%} at ts={ts}: "
            f"brk={entry['USD']} vs mempool={m_usd}"
        )


def test_historical_price_at_block(brk, live):
    """At each fixture-block timestamp brk returns one bucket-aligned entry."""
    for block in live.blocks:
        info = brk.get_block(block.hash)
        ts = info["timestamp"]
        b = brk.get_historical_price(timestamp=ts)
        assert len(b["prices"]) == 1, f"height {block.height}: expected 1 entry, got {len(b['prices'])}"
        bucket_start = (ts // HOUR4) * HOUR4
        assert b["prices"][0]["time"] == bucket_start, (
            f"height {block.height}: bucket misaligned: "
            f"got {b['prices'][0]['time']} vs expected {bucket_start} for ts={ts}"
        )


def test_historical_price_future(brk):
    """A future timestamp must not crash; brk emits a single entry whose USD is numeric."""
    ts = int(time.time()) + 86400
    b = brk.get_historical_price(timestamp=ts)
    assert len(b["prices"]) == 1
    assert isinstance(b["prices"][0]["USD"], (int, float))


def test_historical_price_pre_genesis(brk):
    """Pre-INDEX_EPOCH (2009-01-01) timestamps must return an empty list, not panic."""
    b = brk.get_historical_price(timestamp=0)
    assert b["prices"] == [], f"expected empty list for pre-EPOCH timestamp, got {b['prices']}"
    assert b["exchangeRates"] == {}
