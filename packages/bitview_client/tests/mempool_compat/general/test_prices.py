"""GET /api/v1/prices"""

import time

from _lib import assert_same_structure, show


def test_prices_shape(brk, mempool):
    """Brk's typed response must carry every key mempool.space returns (modulo intentional fiat skips)."""
    path = "/api/v1/prices"
    b = brk.get_prices()
    m = mempool.get_json(path)
    show("GET", path, b, m)
    assert_same_structure(b, m)
    assert "USD" in b and "time" in b


def test_prices_invariants(brk):
    """`time` is a recent unix-seconds value and `USD` is a sane spot price."""
    d = brk.get_prices()
    now = int(time.time())

    assert isinstance(d["time"], int), f"time must be int, got {type(d['time']).__name__}"
    assert 1_500_000_000 < d["time"] < now + 10, (
        f"time={d['time']} not within sane bounds (post-2017 and not in the future)"
    )

    assert isinstance(d["USD"], (int, float)), (
        f"USD must be numeric, got {type(d['USD']).__name__}"
    )
    assert 1_000 < d["USD"] < 10_000_000, (
        f"USD={d['USD']} outside protocol-realistic spot bounds"
    )


def test_prices_close_to_mempool(brk, mempool):
    """Brk's USD must track mempool.space's within 10% (covers feed divergence, not market drift)."""
    b = brk.get_prices()
    m = mempool.get_json("/api/v1/prices")
    drift = abs(b["USD"] - m["USD"]) / m["USD"]
    assert drift < 0.10, (
        f"USD diverges from mempool.space by {drift:.1%}: brk={b['USD']} vs mempool={m['USD']}"
    )
