"""
Shared fixtures for mempool.space compatibility tests.

Helper functions live in `_lib.py`; this file holds only fixtures so pytest
can discover them throughout the subtree. Each subtree test imports helpers
with `from _lib import ...` — the conftest puts this directory on sys.path.

Usage:
    cd packages/bitview_client
    uv run pytest tests/mempool_compat -sv                              # all
    uv run pytest tests/mempool_compat/blocks -sv                       # one category
    uv run pytest tests/mempool_compat/blocks/test_block.py -sv         # one endpoint
    BRK_URL=http://host:port uv run pytest tests/mempool_compat -sv     # custom server

Environment variables:
    BRK_URL      brk server base URL                       (default: http://localhost:3110)
    MEMPOOL_URL  mempool.space base URL                     (default: https://mempool.space)
    RATE_LIMIT   seconds between mempool.space requests     (default: 0.5)
"""

import os
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import pytest
import requests

from bitview_client import BitviewClient

# Make `_lib` and `_endpoints` importable from any nested test file.
sys.path.insert(0, str(Path(__file__).parent))

BRK_BASE = os.environ.get("BRK_URL", "http://localhost:3110")
MEMPOOL_BASE = os.environ.get("MEMPOOL_URL", "https://mempool.space")
RATE_LIMIT = float(os.environ.get("RATE_LIMIT", "0.5"))


class ApiClient:
    """HTTP client for a single API server with optional rate limiting."""

    def __init__(self, base_url: str, name: str, rate_limit: float = 0.0):
        self.base_url = base_url.rstrip("/")
        self.name = name
        self.rate_limit = rate_limit
        self._last_request = 0.0
        self.session = requests.Session()
        self.session.headers["User-Agent"] = "brk-compat-test/1.0"

    def _wait(self):
        if self.rate_limit > 0:
            elapsed = time.monotonic() - self._last_request
            if elapsed < self.rate_limit:
                time.sleep(self.rate_limit - elapsed)
        self._last_request = time.monotonic()

    def get(self, path: str, params=None, timeout: int = 30) -> requests.Response:
        self._wait()
        url = f"{self.base_url}{path}"
        for _ in range(3):
            resp = self.session.get(url, params=params, timeout=timeout)
            if resp.status_code == 429:
                wait = int(resp.headers.get("Retry-After", 5))
                time.sleep(wait)
                continue
            resp.raise_for_status()
            return resp
        resp.raise_for_status()
        return resp

    def get_raw(self, path: str, params=None, timeout: int = 30) -> requests.Response:
        """Like `get` but does not raise on non-2xx — returns the raw response."""
        self._wait()
        url = f"{self.base_url}{path}"
        return self.session.get(url, params=params, timeout=timeout)

    def get_json(self, path: str, params=None, timeout: int = 30) -> Any:
        return self.get(path, params=params, timeout=timeout).json()

    def get_text(self, path: str, params=None, timeout: int = 30) -> str:
        return self.get(path, params=params, timeout=timeout).text

    def get_bytes(self, path: str, params=None, timeout: int = 30) -> bytes:
        return self.get(path, params=params, timeout=timeout).content


# Absolute heights for well-known eras + relative depths for recent blocks.
# Covers: genesis-era, early, mid, post-halving, taproot-era, recent, near-tip.
FIXED_HEIGHTS = [100, 100_000, 400_000, 630_000, 800_000]
RELATIVE_DEPTHS = [1000, 100, 10]


@dataclass
class BlockData:
    """A discovered block with associated txids."""

    height: int
    hash: str
    txid: str
    coinbase_txid: str


@dataclass
class LiveData:
    """Live blockchain data discovered at session start."""

    tip_height: int
    tip_hash: str
    blocks: list  # list[BlockData] — multiple depths for parametrized tests
    addresses: dict  # dict[str, str] — keyed by scriptpubkey_type
    stable_height: int
    stable_hash: str
    stable_block: dict
    sample_txid: str
    coinbase_txid: str
    sample_address: str


@pytest.fixture(scope="session")
def brk():
    """Typed bitview_client SDK pointed at the brk server under test.

    All tests must go through this fixture's typed methods (e.g.
    `brk.get_difficulty_adjustment()`). If a typed method is missing or
    broken, that's an SDK finding — fix the bindgen before adapting the
    test. Never reach into raw HTTP from a compat test.
    """
    client = BitviewClient(BRK_BASE)
    yield client
    client.close()


@pytest.fixture(scope="session")
def mempool():
    return ApiClient(MEMPOOL_BASE, "mempool.space", rate_limit=RATE_LIMIT)


@pytest.fixture(scope="session", autouse=True)
def check_servers(mempool):
    """Fail fast if either server is unreachable."""
    try:
        BitviewClient(BRK_BASE).get_text("/api/blocks/tip/height")
    except Exception as e:
        pytest.exit(f"brk server not reachable at {BRK_BASE}: {e}")
    try:
        mempool.get("/api/blocks/tip/height")
    except Exception as e:
        pytest.exit(f"mempool.space not reachable at {mempool.base_url}: {e}")


@pytest.fixture(scope="session")
def live(mempool) -> LiveData:
    """Discover live blockchain data once per session.

    Picks blocks at multiple depths and extracts addresses of different
    scriptpubkey types so parametrized tests cover varied real data.
    """
    tip_height = int(mempool.get_text("/api/blocks/tip/height"))
    tip_hash = mempool.get_text("/api/blocks/tip/hash")

    heights = FIXED_HEIGHTS + [tip_height - d for d in RELATIVE_DEPTHS]
    heights.sort()

    blocks: list[BlockData] = []
    addresses: dict[str, str] = {}

    for h in heights:
        bh = mempool.get_text(f"/api/block-height/{h}")
        txids = mempool.get_json(f"/api/block/{bh}/txids")
        coinbase = txids[0]
        sample = txids[min(1, len(txids) - 1)]
        blocks.append(BlockData(height=h, hash=bh, txid=sample, coinbase_txid=coinbase))

        if len(addresses) < 8:
            tx = mempool.get_json(f"/api/tx/{sample}")
            for vout in tx.get("vout", []):
                atype = vout.get("scriptpubkey_type")
                addr = vout.get("scriptpubkey_address")
                if addr and atype and atype not in addresses:
                    addresses[atype] = addr

    stable = blocks[0]
    stable_block = mempool.get_json(f"/api/block/{stable.hash}")
    sample_address = next(iter(addresses.values()), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")

    data = LiveData(
        tip_height=tip_height,
        tip_hash=tip_hash,
        blocks=blocks,
        addresses=addresses,
        stable_height=stable.height,
        stable_hash=stable.hash,
        stable_block=stable_block,
        sample_txid=stable.txid,
        coinbase_txid=stable.coinbase_txid,
        sample_address=sample_address,
    )

    print(f"\n{'=' * 70}")
    print(f"  LIVE TEST DATA  (from {MEMPOOL_BASE})")
    print(f"{'=' * 70}")
    print(f"  tip       {data.tip_height}  {data.tip_hash[:20]}...")
    for i, b in enumerate(blocks):
        print(f"  block[{i}]  {b.height}  {b.hash[:20]}...  tx={b.txid[:16]}...")
    for atype, addr in addresses.items():
        print(f"  addr      {atype:12s}  {addr}")
    print(f"{'=' * 70}\n")
    return data


@pytest.fixture(params=range(8), ids=[
    "h100", "h100k", "h400k", "h630k", "h800k", "recent1k", "recent100", "recent10",
])
def block(request, live):
    """One BlockData per id — skip if not discovered for this session."""
    i = request.param
    if i >= len(live.blocks):
        pytest.skip("block not discovered")
    return live.blocks[i]


@pytest.fixture()
def live_addrs(live):
    """All dynamically discovered addresses, keyed by scriptpubkey_type."""
    return list(live.addresses.items())
