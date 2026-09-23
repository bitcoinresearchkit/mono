import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const html = await readFile(new URL("../wave.html", import.meta.url), "utf8");
function section(start, end) {
  const from = html.indexOf(start), to = html.indexOf(end, from);
  assert.ok(from >= 0 && to > from);
  return html.slice(from, to);
}

function fixture(override) {
  const calls = [];
  const ranges = [{ id: "young" }, { id: "old" }];
  const values = {
    supply: [20, 80], realized_cap: [200, 400],
    wakefulness: [0.75, 0.25], mobility: [0.5, 0.125],
  };
  const fetch = async (url) => {
    calls.push(url);
    const [, band, suffix] = url.match(/utxos_(young|old)_(\w+)\/day1/);
    const value = values[suffix][band === "young" ? 0 : 1];
    const data = override?.(band, suffix, calls.length) ?? { start: 5, end: 6, data: [value] };
    return { ok: true, json: async () => url.endsWith("/latest") ? value : data };
  };
  const api = new Function("fetch", "ranges", `
    const API = "https://example.test/api";
    const cache = new Map(), seriesCache = new Map();
    const dayToDate = day => day;
    ${section("        const bases =", "        const PALETTE")}
    ${section("        function fetchSeries(", "        async function loadTiny(")}
    return {fetchMode, modes, modeKeyFor, visibleRow};
  `)(fetch, ranges);
  return { ...api, calls };
}

test("fetches raw series on demand and reuses weights across complements and bases", async () => {
  const { fetchMode, calls, modes } = fixture();
  await fetchMode("supply");
  assert.equal(calls.length, 2);
  await fetchMode("cointime");
  assert.equal(calls.length, 4);
  await fetchMode("dormant");
  assert.equal(calls.length, 4);
  await fetchMode("capital-cointime");
  assert.equal(calls.length, 6);
  await fetchMode("capital-dormant");
  assert.equal(calls.length, 6);
  await fetchMode("coinflow");
  assert.equal(calls.length, 8);
  for (const mode of Object.keys(modes)) await fetchMode(mode);
  assert.equal(calls.length, 8);
  assert.equal(new Set(calls).size, calls.length);
  assert.ok(calls.every(url => /_(supply|realized_cap|wakefulness|mobility)\/day1$/.test(url)));
});

test("computes weighted supply and capital, complements, and both displays locally", async () => {
  const { fetchMode, calls } = fixture();
  for (const [mode, expected, complement, rawTotal] of [
    ["supply", [20, 80], 0, 100],
    ["cointime", [15, 20], 65, 100],
    ["dormant", [5, 60], 35, 100],
    ["coinflow", [10, 10], 80, 100],
    ["immobile", [10, 70], 20, 100],
    ["capital", [200, 400], 0, 600],
    ["capital-cointime", [150, 100], 350, 600],
    ["capital-dormant", [50, 300], 250, 600],
    ["capital-coinflow", [100, 50], 450, 600],
    ["capital-immobile", [100, 350], 150, 600],
  ]) {
    const [row] = await fetchMode(mode);
    assert.deepEqual(row.weights, expected);
    assert.equal(row.complementValue, complement);
    assert.equal(row.total + complement, rawTotal);
    assert.equal(row.absoluteValues.at(-1), rawTotal);
    assert.equal(row.shareValues.at(-1), 100);
    assert.deepEqual(row.shares, expected.map(value => value / rawTotal * 100));
    const count = calls.length;
    assert.equal((await fetchMode(mode))[0], row);
    assert.equal(calls.length, count);
  }
});

test("aligns dates before multiplying and skips missing weights for nonempty bands", async () => {
  const { fetchMode } = fixture((band, suffix) => suffix === "wakefulness"
    ? { start: 6, end: 9, data: [null, 0.5, 1] }
    : { start: 5, end: 8, data: [2, 4, 8] });
  const rows = await fetchMode("dormant");
  assert.deepEqual(rows.map(row => row.time), [7]);
  assert.deepEqual(rows[0].weights, [4, 4]);
  assert.equal(rows[0].complementValue, 8);
});

test("deduplicates concurrent requests and retries failed raw series", async () => {
  let fail = true;
  const { fetchMode, calls } = fixture((band, suffix) => {
    if (fail && band === "young" && suffix === "supply") throw new Error("offline");
  });
  await assert.rejects(fetchMode("supply"), /offline/);
  fail = false;
  await Promise.all([fetchMode("cointime"), fetchMode("dormant")]);
  assert.equal(calls.length, 5);
});

test("tiny previews use only latest raw values and share the same calculations", async () => {
  const { fetchMode, calls } = fixture();
  const [row] = await fetchMode("capital-dormant", undefined, true);
  assert.deepEqual(row.weights, [50, 300]);
  assert.equal(row.complementValue, 250);
  assert.equal(calls.length, 4);
  assert.ok(calls.every(url => url.endsWith("/latest")));
});


test("hiding the complement renormalizes visible bands without fetching or mutating cached data", async () => {
  const { fetchMode, visibleRow, calls } = fixture();
  const [raw] = await fetchMode("cointime");
  const original = structuredClone(raw);
  const count = calls.length;
  const row = visibleRow(raw, new Set(["Dormant"]));
  assert.deepEqual(row.weights, [15, 20]);
  assert.deepEqual(row.absoluteValues, [0, 15, 35, 35]);
  assert.deepEqual(row.shares, [15 / 35 * 100, 20 / 35 * 100]);
  assert.equal(row.shareValues.at(-1), 100);
  assert.equal(row.complementShare, 0);
  assert.equal(row.complementValue, 0);
  assert.equal(calls.length, count);
  assert.deepEqual(raw, original);
  assert.deepEqual(visibleRow(raw), original);
});

test("age bands and complements can all be hidden and restored for either basis", async () => {
  const { fetchMode, visibleRow } = fixture();
  for (const mode of ["cointime", "capital-cointime"]) {
    const [raw] = await fetchMode(mode);
    const single = visibleRow(raw, new Set(["young", "Dormant"]));
    assert.deepEqual(single.shares, [0, 100]);
    assert.equal(single.weights[1], raw.weights[1]);
    assert.equal(single.total, raw.weights[1]);
    const empty = visibleRow(raw, new Set(["young", "old", "Dormant"]));
    assert.ok(empty.shareValues.every(value => value === 0));
    assert.ok(empty.absoluteValues.every(value => value === 0));
    assert.equal(empty.total, 0);
    assert.deepEqual(visibleRow(raw), raw);
    // Hiding Dormant must not hide a different complement in another mode.
    const [dormant] = await fetchMode(mode.replace("cointime", "dormant"));
    assert.equal(visibleRow(dormant, new Set(["Dormant"])).complementValue, dormant.complementValue);
  }
});

test("legend clicks and keyboard activation toggle visibility and rerender locally", () => {
  const listeners = {};
  const hiddenBands = new Set();
  let renders = 0;
  const item = { dataset: { band: "Dormant" } };
  const legendElement = {
    contains: target => target === item,
    addEventListener: (name, callback) => { listeners[name] = callback; },
  };
  new Function("legendElement", "hiddenBands", "renderActiveRows",
    section("        function toggleBand(", "        function updateControls("),
  )(legendElement, hiddenBands, () => renders++);
  const event = { target: { closest: () => item }, preventDefault() {} };
  listeners.click(event);
  assert.ok(hiddenBands.has("Dormant"));
  listeners.keydown({ ...event, key: "Enter" });
  assert.ok(!hiddenBands.has("Dormant"));
  listeners.keydown({ ...event, key: " " });
  assert.ok(hiddenBands.has("Dormant"));
  listeners.keydown({ ...event, key: "ArrowDown" });
  assert.equal(renders, 3);
});
