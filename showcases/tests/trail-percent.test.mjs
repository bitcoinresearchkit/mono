import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const html = await readFile(new URL("../trail/index.html", import.meta.url), "utf8");
function section(start, end) {
  const from = html.indexOf(start), to = html.indexOf(end, from);
  assert.ok(from >= 0 && to > from);
  return html.slice(from, to);
}

function fixture() {
  const state = { metric: "supply", metricCdfCache: new WeakMap() };
  const label = new Function("state", `
    const priceAtTerrainCoordinate = coordinate => coordinate;
    const priceCoordinate = price => price;
    const formatAxisPrice = price => "$" + price;
    ${section("        function metricSource(", "        function sampleProfile(")}
    ${section("        function cumulativeMetricPercent(", "        function elevationColor(")}
    return pricePillLabel;
  `)(state);
  return { state, label };
}

test("price and hover labels follow metric changes on the same cached profile", () => {
  const { state, label } = fixture();
  const profile = {
    close: 150,
    prices: [0, 100, 200],
    supplies: [2, 3, 5],
    realizedCaps: [0, 300, 1000],
    unrealizedPnls: [400, 300, -500],
  };
  for (const [metric, percent] of [
    ["supply", 50], ["realized_cap", 23], ["unrealized_pnl", 43],
    ["supply", 50], ["unrealized_pnl", 43], ["realized_cap", 23],
  ]) {
    state.metric = metric;
    assert.equal(label(profile, 100), `$100\n${percent}%`);
    assert.equal(label(profile, 100, "$150"), `$150\n${percent}%`);
    assert.equal(label(profile, 100, "$150", true), `$150\n${metric === "unrealized_pnl" ? 0 : percent}%`);
    assert.equal(label(profile, 200), "$200\n100%");
  }
});

test("zero or missing metric data has no percentage", () => {
  const { state, label } = fixture();
  const profile = { prices: [0], supplies: [10], realizedCaps: [0] };
  assert.equal(label(profile, 0), "$0\n100%");
  state.metric = "realized_cap";
  assert.equal(label(profile, 0), "$0\n—");
  state.metric = "unrealized_pnl";
  assert.equal(label(profile, 0), "$0\n—");
  assert.equal(label(null, 0), "$0\n—");
});

test("P&L percentages normalize profit and loss independently", () => {
  const { state, label } = fixture();
  state.metric = "unrealized_pnl";
  const profile = {
    close: 150,
    prices: [0, 100, 200, 300],
    unrealizedPnls: [100, 300, -200, -800],
  };
  assert.equal(label(profile, 0), "$0\n100%");
  assert.equal(label(profile, 100), "$100\n75%");
  assert.equal(label(profile, 150), "$150\n0%");
  assert.equal(label(profile, 200), "$200\n20%");
  assert.equal(label(profile, 300), "$300\n100%");
  assert.equal(label({ ...profile, unrealizedPnls: [100, 300, 0, 0] }, 200), "$200\n—");
  assert.equal(label({ ...profile, unrealizedPnls: [0, 0, -200, -800] }, 100), "$100\n—");
});
