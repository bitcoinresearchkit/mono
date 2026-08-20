#!/usr/bin/env python3
"""Determine which mining pools qualify as "major" based on dominance.

A pool is major if ANY window's dominance exceeds its threshold:
  all-time >= 1%, 1y >= 1%, 1m >= 0.75%, 1w >= 0.5%

Computes dominance from blocks_mined_cumulative / block_count_cumulative,
so it works for ALL pools (major and minor alike).

Usage:
    python3 scripts/pool_major_threshold.py
    python3 scripts/pool_major_threshold.py --all-time 5 --1y 1 --1m 0.75 --1w 0.5
"""

import argparse
import json
import re
import urllib.request
import concurrent.futures
from pathlib import Path

API_BASE = "https://bitview.space/api/series"
POOLSLUG_PATH = Path(__file__).resolve().parent.parent / "crates/brk_types/src/pool_slug.rs"
HEADERS = {"User-Agent": "pool-threshold-script"}
WINDOWS = {"1w": 7, "1m": 30, "1y": 365}


def parse_pool_variants():
    """Return [(VariantName, serialized_slug), ...] from the PoolSlug enum."""
    src = POOLSLUG_PATH.read_text()
    m = re.search(r"pub enum PoolSlug\s*\{(.*?)^\}", src, re.DOTALL | re.MULTILINE)
    if not m:
        raise RuntimeError("Could not find PoolSlug enum")
    body = m.group(1)
    variants = []
    renamed_slug = None
    for line in body.splitlines():
        line = line.strip().rstrip(",")
        rename = re.fullmatch(r'#\[serde\(rename = "([^"]+)"\)\]', line)
        if rename:
            renamed_slug = rename.group(1)
            continue
        if not line or line.startswith("#[") or line.startswith("//"):
            continue
        name = line.split("(")[0].split("{")[0].strip()
        if not name or not name[0].isupper():
            continue
        if name.startswith("Dummy"):
            continue
        variants.append((name, renamed_slug or name.lower()))
        renamed_slug = None
    return variants


def fetch_json(url):
    try:
        req = urllib.request.Request(url, headers=HEADERS)
        with urllib.request.urlopen(req, timeout=15) as resp:
            return json.loads(resp.read())
    except Exception:
        return None


def fetch_cumulative(slug, days):
    url = f"{API_BASE}/{slug}_blocks_mined_cumulative/day1?start=-{days}"
    return fetch_json(url)


def fetch_total_cumulative(days):
    url = f"{API_BASE}/block_count_cumulative/day1?start=-{days}"
    return fetch_json(url)


def is_major(doms, thresholds):
    """Check if any window meets its threshold."""
    for label, thresh in thresholds.items():
        v = doms.get(label)
        if v is not None and v >= thresh:
            return True
    return False


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--all-time", type=float, default=1.0, help="All-time dominance threshold %% (default: 1.0)")
    parser.add_argument("--1y", type=float, default=1.0, dest="t1y", help="1y rolling dominance threshold %% (default: 1.0)")
    parser.add_argument("--1m", type=float, default=0.75, dest="t1m", help="1m rolling dominance threshold %% (default: 0.75)")
    parser.add_argument("--1w", type=float, default=0.5, dest="t1w", help="1w rolling dominance threshold %% (default: 0.5)")
    args = parser.parse_args()

    thresholds = {
        "all-time": args.all_time,
        "1y": args.t1y,
        "1m": args.t1m,
        "1w": args.t1w,
    }

    variants = parse_pool_variants()
    print(f"Found {len(variants)} pool variants in {POOLSLUG_PATH.name}")
    print(f"Thresholds: {', '.join(f'{k}>={v}%' for k, v in thresholds.items())}")

    max_days = max(WINDOWS.values()) + 1
    print(f"Fetching blocks_mined_cumulative for all pools...")

    total_data = fetch_total_cumulative(max_days)
    if not total_data:
        print("ERROR: Could not fetch block_count_cumulative")
        return
    total_cum = total_data["data"]

    pool_cum = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=20) as ex:
        futures = {ex.submit(fetch_cumulative, slug, max_days): (name, slug)
                   for name, slug in variants}
        for fut in concurrent.futures.as_completed(futures):
            name, slug = futures[fut]
            result = fut.result()
            if result and result.get("data"):
                pool_cum[name] = {"slug": slug, "data": result["data"]}

    results = []
    for name, info in pool_cum.items():
        pool_data = info["data"]
        n = len(pool_data)
        doms = {}

        if n > 0 and len(total_cum) > 0:
            doms["all-time"] = pool_data[-1] / total_cum[-1] * 100 if total_cum[-1] > 0 else 0

        for label, days in WINDOWS.items():
            if n > days and len(total_cum) > days:
                pool_diff = pool_data[-1] - pool_data[-(days + 1)]
                total_diff = total_cum[-1] - total_cum[-(days + 1)]
                doms[label] = pool_diff / total_diff * 100 if total_diff > 0 else 0
            else:
                doms[label] = None

        values = [v for v in doms.values() if v is not None]
        max_dom = max(values) if values else None
        major = is_major(doms, thresholds)
        results.append((name, info["slug"], doms, max_dom, major))

    results.sort(key=lambda x: -(x[3] or 0))

    def fmt(v):
        return f"{v:8.4f}%" if v is not None else "      N/A"

    header = f"{'Pool':<30} {'all-time':>9} {'1w':>9} {'1m':>9} {'1y':>9}  Major?"
    thr_line = f"{'threshold:':<30} {'>=' + str(thresholds['all-time']) + '%':>9} {'>=' + str(thresholds['1w']) + '%':>9} {'>=' + str(thresholds['1m']) + '%':>9} {'>=' + str(thresholds['1y']) + '%':>9}"
    print(f"\n{header}")
    print(thr_line)
    print("-" * len(header))
    for name, slug, doms, max_dom, major in results:
        at = fmt(doms.get("all-time"))
        w1w = fmt(doms.get("1w"))
        w1m = fmt(doms.get("1m"))
        w1y = fmt(doms.get("1y"))
        marker = "***" if major else ""
        print(f"{name:<30} {at} {w1w} {w1m} {w1y}  {marker}")

    major_list = [(n, s, d, m, mj) for n, s, d, m, mj in results if mj]
    print(f"\n--- {len(major_list)} major pools ---")

    print(f"\n--- Qualifying windows ---")
    for name, slug, doms, max_dom, _ in major_list:
        qualifying = []
        for label, thresh in thresholds.items():
            v = doms.get(label)
            if v is not None and v >= thresh:
                qualifying.append(f"{label}={v:.2f}%")
        print(f"  {name:<30} ({', '.join(qualifying)})")

    major_names = sorted(set(["Unknown"] + [n for n, _, _, _, _ in major_list]))

    thresholds_str = ", ".join(f"{k}>={v}%" for k, v in thresholds.items())
    print(f"\n--- Rust is_major() match arms ---\n")
    print(f"    /// Pools with dominance above per-window thresholds get full metrics.")
    print(f"    /// Thresholds: {thresholds_str}.")
    print(f"    /// Generated by `scripts/pool_major_threshold.py`.")
    print(f"    pub fn is_major(&self) -> bool {{")
    print(f"        matches!(")
    print(f"            self,")
    for i, name in enumerate(major_names):
        if i == 0:
            print(f"            Self::{name}", end="")
        else:
            print(f"\n                | Self::{name}", end="")
    print()
    print(f"        )")
    print(f"    }}")


if __name__ == "__main__":
    main()
