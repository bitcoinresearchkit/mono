"""GET /api/v1/mining/pools"""

from _lib import assert_same_structure, show


# Slugs present in brk's vendored pools-v2.json but reported under a different
# slug (or missing) by mempool.space. Currently the duplicate-pool collision
# case where brk preserves both `bitcoinindia` (variant 80) and
# `bitcoinindiapool` (variant 134), while mempool emits both as `bitcoinindia`.
KNOWN_BRK_ONLY_SLUGS = {"bitcoinindiapool"}

# Pools added upstream after brk's vendored pools-v2.json snapshot. Refresh
# the vendored file (and update this set) when bumping the snapshot.
KNOWN_MEMPOOL_ONLY_SLUGS = {
    "drdetroit", "emzy", "knorrium", "mononaut", "nymkappa", "rijndael",
}

EXPECTED_MIN_POOLS = 165


def test_mining_pools_list_structure(brk, mempool):
    """Pool list element schema must match (flat list, {name, slug, unique_id})."""
    path = "/api/v1/mining/pools"
    b = brk.get_pools()
    m = mempool.get_json(path)
    show("GET", path, f"({len(b)} pools)", f"({len(m)} pools)", max_lines=4)
    assert isinstance(b, list) and isinstance(m, list), "both must be flat lists"
    assert_same_structure(b, m)


def test_mining_pools_list_fields(brk):
    """Every pool entry must carry a non-empty slug + name + non-negative unique_id."""
    b = brk.get_pools()
    show("GET", "/api/v1/mining/pools", f"({len(b)} pools)", "-")
    assert len(b) >= EXPECTED_MIN_POOLS, f"expected >= {EXPECTED_MIN_POOLS} pools, got {len(b)}"
    for p in b:
        assert isinstance(p["slug"], str) and p["slug"], f"bad slug: {p!r}"
        assert isinstance(p["name"], str) and p["name"], f"bad name: {p!r}"
        assert isinstance(p["unique_id"], int) and p["unique_id"] >= 0, (
            f"bad unique_id: {p!r}"
        )


def test_mining_pools_slugs_unique(brk):
    """Pool slugs must be unique across the response."""
    b = brk.get_pools()
    slugs = [p["slug"] for p in b]
    show("GET", "/api/v1/mining/pools", f"({len(slugs)} slugs)", "-")
    assert len(slugs) == len(set(slugs)), (
        f"duplicate slugs: {len(slugs) - len(set(slugs))}"
    )


def test_mining_pools_unique_ids_unique(brk):
    """Pool unique_ids must be unique across the response."""
    b = brk.get_pools()
    ids = [p["unique_id"] for p in b]
    show("GET", "/api/v1/mining/pools", f"({len(ids)} unique_ids)", "-")
    assert len(ids) == len(set(ids)), (
        f"duplicate unique_ids: {len(ids) - len(set(ids))}"
    )


def test_mining_pools_slugs_match_mempool(brk, mempool):
    """brk's slug set must equal mempool's, modulo documented exceptions."""
    b_slugs = {p["slug"] for p in brk.get_pools()}
    m_slugs = {p["slug"] for p in mempool.get_json("/api/v1/mining/pools")}
    show(
        "GET", "/api/v1/mining/pools",
        f"brk-only={sorted(b_slugs - m_slugs)}",
        f"mempool-only={sorted(m_slugs - b_slugs)}",
    )
    unexpected_brk_only = (b_slugs - m_slugs) - KNOWN_BRK_ONLY_SLUGS
    unexpected_mempool_only = (m_slugs - b_slugs) - KNOWN_MEMPOOL_ONLY_SLUGS
    assert not unexpected_brk_only, (
        f"undocumented brk-only slugs (likely format divergence): {unexpected_brk_only}"
    )
    assert not unexpected_mempool_only, (
        f"undocumented mempool-only slugs (refresh pools-v2.json?): {unexpected_mempool_only}"
    )
