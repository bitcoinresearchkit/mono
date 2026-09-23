import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const html = await readFile(new URL("../trail/index.html", import.meta.url), "utf8");
function section(start, end) {
  const from = html.indexOf(start), to = html.indexOf(end, from);
  assert.ok(from >= 0 && to > from);
  return html.slice(from, to);
}

function fixture(timeAxis = null) {
  const listeners = new Map(), captured = new Set();
  const state = {
    width: 1920, selectedIndex: 0, timeDragging: false,
    timePointerId: null, pricePointerId: null,
    hoverPriceCoordinate: null,
    dates: ["2026-01-01"], availableDates: ["2026-01-01"],
    priceCoordinates: [0, 0.25, 0.5, 0.75, 1],
    hoverModel: {
      layout: { left: 100, width: 800, timeXStep: 0 },
      start: 0, end: 0, minY: 100, maxY: 500,
      timeOut: {}, axisGap: 0,
    },
  };
  if (timeAxis) {
    state.dates = Array.from({ length: 11 }, (_, i) => `2026-01-${String(i + 1).padStart(2, "0")}`);
    state.availableDates = state.dates;
    state.selectedIndex = 5;
    state.hoverModel.end = 10;
  }
  const canvas = {
    style: {},
    addEventListener(type, callback) {
      if (!listeners.has(type)) listeners.set(type, []);
      listeners.get(type).push(callback);
    },
    setPointerCapture(id) { captured.add(id); },
    hasPointerCapture(id) { return captured.has(id); },
    releasePointerCapture(id) { captured.delete(id); },
  };
  const api = new Function("state", "canvas", "timeAxis", `
    const hoverQuery = { matches: false, addEventListener() {} };
    const app = { style: { getPropertyValue: () => "0.25" } };
    const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
    const dateOffsetFromView = () => 0;
    const nearestPriceIndex = coordinate => Math.round(coordinate * 4);
    function scheduleRender() {}

    function cancelNavigationLoad() {}
    function invalidateSummaryForMovement() {}
    function updateSummary() {}
    function scheduleSettingsPersistence() {}
    function scheduleTerrainChunkBuild() {}
    function navigateToDate() {}
    ${timeAxis ? section("        function timeDateAtPointer(", "        function priceCoordinateAtPointer(") : `
    const timeDateAtPointer = (x, y, requireHit) => !requireHit || x < 80 ? "2026-01-01" : null;
    `}
    const lerp = (a, b, t) => a + (b - a) * t;
    const dateTimeAtIndex = index => Date.parse(state.dates[index]);
    const nearestDateIn = (dates, date) => dates.reduce((best, candidate) =>
      Math.abs(Date.parse(candidate) - Date.parse(date)) < Math.abs(Date.parse(best) - Date.parse(date)) ? candidate : best
    );
    const datePillHitTest = () => true;
    const projectAtTimeAxis = (layout, index) => timeAxis
      ? { x: 100 + timeAxis.x * index / 10, y: 100 + timeAxis.y * index / 10 }
      : { x: 40, y: 300 };
    const offsetPoint = point => point;
    const queueTimeDragDate = date => { state.timeDragTargetDate = date; };
    ${section("        function canHoverTerrain(", "        function scheduleRender(")}
    ${section("        function priceCoordinateAtPointer(", "        function cancelNavigationPrefetch(")}
    ${section("        function setTerrainCursor(", '        window.addEventListener("keydown"')}
    ${section('        canvas.addEventListener("pointerdown"', "        for (const input of dimensionInputs)")}
    return { pointerHitRadius, priceCoordinateAtPointer, clearHoverState };
  `)(state, canvas, timeAxis);
  function send(type, overrides = {}) {
    const event = {
      type, pointerId: 1, pointerType: "touch", isPrimary: true,
      button: 0, offsetX: 500, offsetY: 300,
      preventDefault() { this.defaultPrevented = true; },
      stopImmediatePropagation() { this.stopped = true; }, ...overrides,
    };
    for (const listener of listeners.get(type) ?? []) {
      listener(event);
      if (event.stopped) break;
    }
    return event;
  }
  return { state, captured, send, ...api };
}

test("touch inspects without mouse hover and keeps the label after release", () => {
  const { state, captured, send } = fixture();
  assert.ok(send("pointerdown").defaultPrevented);
  assert.equal(state.hoverPriceCoordinate, 0.5);
  assert.ok(captured.has(1));
  send("pointermove", { offsetX: 700 });
  assert.equal(state.hoverPriceCoordinate, 0.75);
  send("pointermove", { offsetX: 1100, offsetY: 900 });
  assert.equal(state.hoverPriceCoordinate, 1);
  send("pointerup");
  send("pointerleave");
  assert.equal(state.pricePointerId, null);
  assert.equal(captured.size, 0);
  assert.equal(state.hoverPriceCoordinate, 1);
});

test("additional fingers cannot steal or end inspection", () => {
  const { state, send } = fixture();
  send("pointerdown");
  send("pointerdown", { pointerId: 2, isPrimary: false, offsetX: 40 });
  send("pointermove", { pointerId: 2, offsetX: 900 });
  send("pointerup", { pointerId: 2 });
  assert.equal(state.pricePointerId, 1);
  assert.equal(state.timeDragging, false);
  assert.equal(state.hoverPriceCoordinate, 0.5);
  send("pointercancel");
  assert.equal(state.pricePointerId, null);
  assert.equal(state.hoverPriceCoordinate, null);
});

test("date drag captures touch and recovers from lost capture", () => {
  const { state, send } = fixture();
  send("pointerdown", { offsetX: 40 });
  assert.equal(state.timeDragging, true);
  send("pointermove", { pointerId: 2, offsetX: 900 });
  assert.equal(state.hoverPriceCoordinate, null);
  send("lostpointercapture");
  assert.equal(state.timeDragging, false);
  assert.equal(state.timePointerId, null);
  send("pointerdown");
  assert.equal(state.pricePointerId, 1);
});

test("touch hit targets retain their screen size when the stage is scaled", () => {
  const { pointerHitRadius, priceCoordinateAtPointer } = fixture();
  assert.equal(pointerHitRadius("touch", 18, 24), 96);
  assert.equal(pointerHitRadius("mouse", 18, 24), 18);
  assert.equal(priceCoordinateAtPointer(500, 300), 0.5);
});

test("inspection resumes after settings clear the label during an active drag", () => {
  const { state, send, clearHoverState } = fixture();
  send("pointerdown");
  clearHoverState();
  send("pointermove");
  assert.equal(state.hoverPriceCoordinate, 0.5);
});

for (const axis of [{ x: 0, y: 400 }, { x: 300, y: -200 }]) {
  for (const pointerType of ["mouse", "touch"]) {
    for (const grab of [-0.1, 0.1]) {
      test(`date drag reaches both endpoints with ${pointerType}, axis ${JSON.stringify(axis)}, grab ${grab}`, () => {
        const { state, send } = fixture(axis);
        const point = amount => ({
          offsetX: 100 + axis.x * (amount + grab),
          offsetY: 100 + axis.y * (amount + grab),
          pointerType,
        });
        send("pointerdown", point(0.5));
        assert.equal(state.timeDragTargetDate, state.dates[5], "grabbing must not jump");
        send("pointermove", point(0.5));
        assert.equal(state.timeDragTargetDate, state.dates[5]);
        for (const [amount, index] of [[0, 0], [1, 10], [1.2, 10], [-0.2, 0], [0.7, 7]]) {
          send("pointermove", point(amount));
          assert.equal(state.timeDragTargetDate, state.dates[index]);
        }
        send("pointerup");
        assert.equal(state.timeDragOffsetX, 0);
        assert.equal(state.timeDragOffsetY, 0);
      });
    }
  }
}
