import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { BitviewClient } from "../../modules/bitview-client/index.js";

// Exercise Studio's actual worker against the generated client catalog.
const html = await readFile(new URL("../studio.html", import.meta.url), "utf8");
const base = html.match(/const base = "([^"]+)";/)[1];
const start = html.indexOf("      function catalogWorkerMain()");
const end = html.indexOf("      let catalogWorker,", start);
assert.ok(start >= 0 && end > start);
let response;
const worker = { postMessage(message) { response = message; } };
new Function("self", `${html.slice(start, end)}; catalogWorkerMain();`)(worker);
let id = 0;
async function request(type, args) {
  await worker.onmessage({ data: { id: ++id, type, args } });
  assert.equal(response.error, undefined, response.error);
  return response.data;
}

const client = new BitviewClient({ baseUrl: base });
const priceSources = {};
for (const [unit, ohlc] of Object.entries(client.series.price.ohlc)) {
  const spot = client.series.price.spot[unit];
  if (!spot) continue;
  priceSources[ohlc.name] = Object.fromEntries(
    [...new Set([...ohlc.indexes(), ...spot.indexes()])].map(index =>
      [index, ohlc.get(index) ? ohlc.name : spot.name]),
  );
}
await request("init", {
  clientUrl: new URL("../../modules/bitview-client/index.js", import.meta.url).href,
  baseUrl: base, priceSources,
});

test("search uses the API ranking and filters unknown or incompatible series", async (t) => {
  const requests = [];
  t.mock.method(globalThis, "fetch", async (input) => {
    requests.push(new URL(input instanceof Request ? input.url : input));
    return new Response(JSON.stringify([
      "market_cap", "not_in_the_generated_client", "timestamp", "sth_realized_price", "market_cap",
    ]), { headers: { "Content-Type": "application/json" } });
  });
  const { items } = await request("search", { query: "short term & price" });
  assert.equal(requests.length, 1);
  assert.equal(requests[0].origin, "http://localhost:3110");
  assert.equal(requests[0].pathname, "/api/series/search");
  assert.equal(requests[0].searchParams.get("q"), "short term & price");
  assert.equal(requests[0].searchParams.get("limit"), "50");
  assert.deepEqual(items.map(item => item.name), ["market_cap", "sth_realized_price"]);
  assert.ok(items.every(item => item.available));
});

test("empty API results stay empty", async (t) => {
  t.mock.method(globalThis, "fetch", async () => new Response("[]"));
  assert.deepEqual((await request("search", { query: "no results" })).items, []);
});

test("API errors reach the picker instead of falling back to local ranking", async (t) => {
  t.mock.method(globalThis, "fetch", async () => new Response("Unavailable", { status: 503 }));
  await worker.onmessage({ data: { id: ++id, type: "search", args: { query: "failed request" } } });
  assert.equal(typeof response.error, "string");
  assert.ok(response.error.length > 0);
  assert.equal(response.data, undefined);
});

test("catalog browsing remains local", async (t) => {
  t.mock.method(globalThis, "fetch", () => { throw new Error("Unexpected network request"); });
  const response = await request("browse", { path: [], offset: 0 });
  assert.ok(response.items.length > 0);
});

test("local instance responds through Studio's worker", { skip: !process.env.STUDIO_TEST_LIVE }, async () => {
  const { items } = await request("search", { query: "realized price sth" });
  assert.equal(items[0]?.name, "sth_realized_price");
  assert.ok(items.every(item => item.available));
});
