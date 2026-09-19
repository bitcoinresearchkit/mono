import assert from "node:assert/strict";
import { test } from "node:test";

test("active supply in loss chart resolves to a fetchable ratio series", async (t) => {
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
