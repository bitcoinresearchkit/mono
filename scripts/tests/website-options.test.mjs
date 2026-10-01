import assert from "node:assert/strict";
import { test } from "node:test";

function mockBrowser(t) {
  const names = ["document", "window", "location", "getComputedStyle", "localStorage"];
  const saved = names.map((name) => Object.getOwnPropertyDescriptor(globalThis, name));
  t.after(() => names.forEach((name, index) => {
    if (saved[index]) Object.defineProperty(globalThis, name, saved[index]);
    else delete globalThis[name];
  }));
  globalThis.document = { documentElement: { style: {} }, getElementById: () => null };
  globalThis.location = new URL("https://fixture.invalid/");
  Object.defineProperty(globalThis, "localStorage", { configurable: true, value: { getItem: () => null } });
  globalThis.window = {
    document, location, matchMedia: () => ({ matches: false, addEventListener() {} }),
  };
  globalThis.getComputedStyle = () => ({ getPropertyValue: () => "" });
}

test("active supply in loss chart resolves to a fetchable ratio series", async (t) => {
  mockBrowser(t);
  const { createCointimeSection } = await import("../../website/scripts/options/frameworks/cointime/index.js");
  function findChart(tree) {
    for (const node of tree) {
      if (node.title === "Active Supply in Loss") return node;
      const found = node.tree && findChart(node.tree);
      if (found) return found;
    }
  }
  const chart = findChart([createCointimeSection()]);
  assert.ok(chart);
  const series = chart.bottom[0].series;
  assert.equal(typeof series.name, "string");
  assert.equal(typeof series.by.height.fetch, "function");
});

test("weighted URPD charts use the current cohort endpoints", async (t) => {
  mockBrowser(t);
  const { bitview } = await import("../../website/scripts/utils/client.js");
  const { createWeightedUrpdSection } = await import("../../website/scripts/options/urpd/index.js");
  const { createAgeBoundsSection } = await import("../../website/scripts/options/urpd/age-bounds.js");
  for (const owner of ["cointime", "coinflow"]) {
    const [costBasis, density] = createWeightedUrpdSection(owner, bitview.series[owner].urpd).tree;
    for (const [name, cohort] of [
      ["All", "all"], ["STH", "sth"], ["LTH", "lth"],
      ["<4M", "under_4m"], ["<6M", "under_6m"],
      [">4M", "over_4m"], [">6M", "over_6m"],
    ]) {
      const charts = costBasis.tree.find((node) => node.name === name).tree;
      for (const [index, weight] of ["coin", "dollar"].entries()) {
        const median = charts[index].top.find((blueprint) => blueprint.title === "P50").series;
        const prefix = cohort === "all" ? "" : `${cohort}_`;
        assert.equal(median.usd.by.day1.path, `/api/series/${prefix}${owner}_cost_basis_per_${weight}_pct50/day1`);
      }
      assert.equal(charts[1].top[0].series.usd.by.day1.path,
        `/api/series/${owner}_urpd_${cohort}_capitalized_price/day1`);
    }
    const sthDensity = density.tree.find((node) => node.name === "STH (<5M)");
    assert.equal(sthDensity.bottom[1].series.by.day1.path,
      `/api/series/${owner}_urpd_sth_supply_density_total_ratio/day1`);
  }
  const sthBounds = createAgeBoundsSection(bitview.series.cohorts.urpd.ageBounds).tree[1];
  assert.equal(sthBounds.top[0].series.usd.by.day1.path,
    "/api/series/utxos_urpd_sth_cost_basis_min/day1");
});

test("every lazy chart resolves to fetchable series in the current client", async (t) => {
  mockBrowser(t);
  const { createPartialOptions } = await import("../../website/scripts/options/partial.js");
  const { collectUnusedSeries } = await import("../../website/scripts/options/unused.js");
  const { bitview } = await import("../../website/scripts/utils/client.js");
  const used = new Set();
  let charts = 0;
  function walk(tree, path = []) {
    for (const node of tree) {
      const next = [...path, node.name];
      if ("tree" in node) {
        walk(node.tree, next);
        continue;
      }
      if (!node.top && !node.bottom) continue;
      charts++;
      for (const blueprint of [...(node.top ?? []), ...(node.bottom ?? [])]) {
        const label = `${next.join(" / ")}: ${blueprint.title}`;
        assert.ok(blueprint.series, label);
        const patterns = "usd" in blueprint.series && "sats" in blueprint.series
          ? [blueprint.series.usd, blueprint.series.sats]
          : [blueprint.series];
        for (const series of patterns) {
          used.add(series.name);
          assert.equal(typeof series.indexes, "function", label);
          for (const index of series.indexes()) {
            assert.equal(typeof series.by[index].fetch, "function", label);
          }
        }
      }
    }
  }
  walk(createPartialOptions());
  assert.ok(charts > 0);
  const unused = collectUnusedSeries(bitview.series, used);
  // Keep these visible in the report: a categorical phase, day mappings, the
  // total-supply identity, and the UTXO history prototype's duplicate count.
  // Every other chartable metric must have a chart, including new client series.
  assert.deepEqual([...unused.keys()].sort(), [
    "capital_sentiment_phase",
    "date",
    "first_height",
    "supply_dominance",
    "supply_dominance_ratio",
    "utxo_count_bis",
  ]);
});

test("unused metrics compare series names and ignore unchartable representations", async () => {
  const { collectUnusedSeries } = await import("../../website/scripts/options/unused.js");
  const pattern = (name, indexes = ["day1"]) => ({ name, indexes: () => indexes });
  const series = {
    used: pattern("used"),
    alias: pattern("used"),
    missing: pattern("missing"),
    heightOnly: pattern("height_only", ["height"]),
    units: { cents: pattern("cents"), ppm: pattern("ppm") },
    constants: { zero: pattern("constant_0") },
  };
  assert.deepEqual([...collectUnusedSeries(series, new Set(["used"]))], [["missing", ["missing"]]]);
  assert.ok(collectUnusedSeries(series, new Set()).has("used"));
});
