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
