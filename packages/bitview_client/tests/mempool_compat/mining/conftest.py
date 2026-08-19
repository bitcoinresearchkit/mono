"""Mining-specific fixtures shared by every mining test in this folder."""

import pytest


@pytest.fixture(scope="module")
def pool_slugs(mempool):
    """Top 3 active pool slugs from the last week."""
    data = mempool.get_json("/api/v1/mining/pools/1w")
    pools = data.get("pools", []) if isinstance(data, dict) else []
    slugs = [p["slug"] for p in pools if p.get("blockCount", 0) > 0][:3]
    return slugs or ["foundryusa"]


@pytest.fixture(scope="module")
def pool_slug(pool_slugs):
    return pool_slugs[0]
