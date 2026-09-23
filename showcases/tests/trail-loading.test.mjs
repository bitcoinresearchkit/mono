import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const html = await readFile(new URL("../trail/index.html", import.meta.url), "utf8");
function section(start, end) {
  const from = html.indexOf(start), to = html.indexOf(end, from);
  assert.ok(from >= 0 && to > from);
  return html.slice(from, to);
}

test("resuming restarts an interrupted initial load without duplicating an active one", () => {
  const state = { dates: [], cohort: "sth", preferredDate: "2026-01-01", loadController: new AbortController() };
  const document = { hidden: false }, calls = [];
  const resume = new Function("state", "document", "calls", `
    const startLiveUpdates = () => calls.push("live");
    const loadDates = (cohort, date) => calls.push([cohort, date]);
    function updateSummary() {}
    function scheduleRender() {}
    function scheduleTerrainChunkBuild() {}
    const scheduleWindowLoad = () => calls.push("window");
    ${section("        function resumeUpdates(", "        function createHoverModel(")}
    return resumeUpdates;
  `)(state, document, calls);
  resume();
  assert.deepEqual(calls, ["live"]);
  state.loadController.abort();
  calls.length = 0;
  resume();
  assert.deepEqual(calls, ["live", ["sth", "2026-01-01"]]);
  state.dates = ["2026-01-01"];
  calls.length = 0;
  resume();
  assert.deepEqual(calls, ["live", "window"]);
  document.hidden = true;
  calls.length = 0;
  resume();
  assert.deepEqual(calls, []);
});

test("a failed window batch aborts its other workers and stops consuming the queue", async () => {
  const controller = new AbortController();
  const state = {
    dates: Array.from({ length: 10 }, (_, i) => String(i)),
    selectedIndex: 9, mode: "value", cache: new Map(),
    loadGeneration: 1, loadController: controller,
  };
  const requests = [];
  const fetchProfileByDate = (date, signal) => {
    requests.push(date);
    if (requests.length === 1) return Promise.reject(new Error("Network failure"));
    return new Promise((resolve, reject) => {
      signal.addEventListener("abort", () => reject(signal.reason), { once: true });
    });
  };
  const load = new Function("state", "fetchProfileByDate", `
    const clamp = (v, min, max) => Math.max(min, Math.min(max, v));
    const visibleRange = () => ({ start: 0, end: 9 });
    const profileCohorts = () => ["all"];
    const profileKeyForDate = date => date;
    const comparisonDate = () => "";
    const getScaleProfile = () => ({});
    const requestConcurrency = () => 2;
    const fetchProfile = async index => state.cache.set(state.dates[index], {});
    const console = { warn() {} };
    function setPriceRange() {}
    function updateSummary() {}
    function scheduleRender() {}
    function scheduleProgressRender() {}
    function scheduleTerrainChunkBuild() {}
    ${section("        async function loadVisibleWindow(", "        function scheduleWindowLoad(")}
    return loadVisibleWindow;
  `)(state, fetchProfileByDate);
  await load(1, controller);
  assert.equal(controller.signal.aborted, true);
  assert.equal(requests.length, 2);
  assert.equal(state.loading, 0);
  assert.equal(state.loadController, null);
});
