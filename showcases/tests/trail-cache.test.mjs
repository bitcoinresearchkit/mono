import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const html = await readFile(new URL("../trail/index.html", import.meta.url), "utf8");
function section(start, end) {
  const from = html.indexOf(start), to = html.indexOf(end, from);
  assert.ok(from >= 0 && to > from);
  return html.slice(from, to);
}

test("a refreshed comparison on the same date invalidates sampled deltas", () => {
  const state = { metric: "supply", mode: "1w", sampleVersion: 1, binCount: 2 };
  const sample = new Function("state", `
    const MIN_TERRAIN_PRICE = 1000, PRICE_BUCKET_WIDTH = 1000;
    ${section("        function metricSource(", "        function normalizeProfile(")}
    return sampleProfile;
  `)(state);
  const profile = { prices: [1000, 2000], supplies: [10, 20] };
  const comparison = { date: "2026-01-01", prices: profile.prices, supplies: [1, 2] };
  const original = sample(profile, comparison);
  assert.deepEqual([...original], [9, 18]);
  assert.equal(sample(profile, comparison), original);
  const refreshed = sample(profile, { ...comparison, supplies: [4, 5] });
  assert.notEqual(refreshed, original);
  assert.deepEqual([...refreshed], [6, 15]);
  assert.deepEqual([...sample(profile)], [0, 0]);
  assert.equal(profile.sampledRawPeak, 0);
  state.mode = "value";
  state.sampleVersion++;
  assert.deepEqual([...sample(profile)], [10, 20]);
  assert.equal(profile.sampledRawPeak, 20);
});

test("a summary without a matching ETag is refreshed, never relabeled as current", async () => {
  const cached = { etag: "", data: { totalSupply: 10 } };
  const state = { summaryCache: new Map([["latest", cached]]) };
  const changed = new Function("state", "fetch", `
    const summaryUrl = () => "/summary";
    ${section("        async function exactSummaryChanged(", "        async function refreshLivePrice(")}
    return exactSummaryChanged;
  `)(state, async () => ({ ok: true, headers: new Headers({ etag: '"new"' }) }));
  assert.equal(await changed({ key: "latest" }), true);
  assert.equal(cached.etag, "");
  cached.etag = '"old"';
  assert.equal(await changed({ key: "latest" }), true);
  cached.etag = '"new"';
  assert.equal(await changed({ key: "latest" }), false);
});

test("comparison dates rebuild when the mode or available dates change", () => {
  const state = {
    mode: "1w", dates: ["2026-01-10"],
    availableDates: ["2026-01-02", "2026-01-10"],
  };
  const api = new Function("state", `
    const DAY_MS = 86400000, COMPARISON_DAY_OFFSETS = { "1w": 7 };
    ${section("        function lowerBound(", "        function latestAvailableDate(")}
    ${section("        function comparisonDayOffset(", "        function profileKeyForDate(")}
    return { comparisonDate, rebuildSceneProfileDates };
  `)(state);
  api.rebuildSceneProfileDates();
  assert.equal(api.comparisonDate(0), "2026-01-02");
  assert.ok(state.sceneProfileDates.has("2026-01-02"));
  state.availableDates.splice(1, 0, "2026-01-03");
  api.rebuildSceneProfileDates();
  assert.equal(api.comparisonDate(0), "2026-01-03");
  assert.ok(!state.sceneProfileDates.has("2026-01-02"));
  state.mode = "value";
  api.rebuildSceneProfileDates();
  assert.equal(api.comparisonDate(0), "");
  assert.deepEqual([...state.sceneProfileDates], state.dates);
});

test("surface paths rebuild when the neighboring row is replaced at the same revision", () => {
  class Path2D {
    points = [];
    moveTo(x, y) { this.points.push(["move", x, y]); }
    lineTo(x, y) { this.points.push(["line", x, y]); }
    closePath() { this.points.push(["close"]); }
  }
  const state = {
    metric: "supply", mode: "value", valueScale: 1,
    binCount: 2, dates: ["2026-01-01", "2026-01-08"],
    projectionCache: { key: "fixed" },
  };
  const draw = new Function("state", "Path2D", `
    const DAY_MS = 86400000;
    const clamp = (v, min, max) => Math.max(min, Math.min(max, v));
    const lerp = (a, b, t) => a + (b - a) * t;
    const visibleRange = () => ({ start: 0, end: 1 });
    const dateTimeAtIndex = index => index * DAY_MS;
    const priceCoordinate = () => null;
    const historyColorMultiplier = () => 1;
    const scaleColorPeak = () => 1;
    const dateSegmentOffsets = () => ({ firstX: 0, firstY: 0, secondX: 4, secondY: 4 });
    const surfaceShade = () => 1;
    const surfaceColorBetween = () => 1;
    const elevationColor = () => "white";
    const heightSign = () => 1;
    function drawSideWallSegment() {}
    function drawWaterCaps() {}
    function drawMeshStrip() {}
    function drawPriceTrailSegment() {}
    ${section("        function appendPolygon(", "        function drawTerrainFootprint(")}
    ${section("        function pathForFill(", "        function drawSideWallSegment(")}
    ${section("        function drawSurface(", "        function terrainChunkKey(")}
    return drawSurface;
  `)(state, Path2D);
  const layout = {
    timeX: 4, depthY: 4, timeSpan: 1, heightScale: 100,
    priceEdgeX: [0, 10, 20], priceEdgeYBase: [100, 90, 80],
  };
  const profiles = state.dates.map(date => ({ date, sampleRevision: 1 }));
  const rows = [new Float32Array([0.1, 0.5]), new Float32Array([0.2, 0.6])];
  const paths = [];
  const target = { save() {}, restore() {}, fill(path) { paths.push(path); } };
  draw(target, layout, rows, profiles, 0, 1);
  const original = paths[0];
  paths.length = 0;
  draw(target, layout, rows, profiles, 0, 1);
  assert.equal(paths[0], original);
  rows[1] = new Float32Array([0.8, 0.3]);
  paths.length = 0;
  draw(target, layout, rows, profiles, 0, 1);
  assert.notEqual(paths[0], original);
  assert.notDeepEqual(paths[0].points, original.points);
});
