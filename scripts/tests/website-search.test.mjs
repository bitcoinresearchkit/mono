import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { createChartSearch } from "../../website/scripts/panes/search-client.js";

const workerUrl = new URL("../../website/scripts/panes/search-worker.js", import.meta.url);

test("actual search worker loads its matcher and resolves catalog links", async () => {
  let onMessage;
  const replies = [];
  const previousFetch = globalThis.fetch;
  globalThis.fetch = async (url) => ({
    ok: true,
    json: async () => JSON.parse(await readFile(url, "utf8")),
  });
  globalThis.self = {
    addEventListener: (_, callback) => { onMessage = callback; },
    postMessage: message => replies.push(message),
  };
  try {
    await import(workerUrl);
    await onMessage({ data: { id: 1, needle: "weighted supply density" } });
    assert.equal(replies[0].error, undefined);
    assert.ok(replies[0].results.length >= 4);
    for (const [title, href, blank] of replies[0].results) {
      assert.match(title, /Weighted Supply Density/);
      assert.match(href, /^\/charts\/models\/bedrock\//);
      assert.equal(blank, false);
    }
  } finally {
    globalThis.fetch = previousFetch;
    delete globalThis.self;
  }
});

test("worker startup failures reject pending and subsequent searches", async () => {
  const listeners = new Map();
  let terminated = false;
  globalThis.Worker = class {
    addEventListener(type, callback) { listeners.set(type, callback); }
    postMessage() {}
    terminate() { terminated = true; }
  };
  try {
    const search = createChartSearch();
    const first = assert.rejects(search("price"), /missing module/);
    const second = assert.rejects(search("density"), /missing module/);
    listeners.get("error")({ message: "missing module" });
    await Promise.all([first, second]);
    await assert.rejects(search("supply"), /missing module/);
    assert.equal(terminated, true);
  } finally { delete globalThis.Worker; }
});

test("duplicate chart titles retain their own links", async () => {
  let onMessage;
  let reply;
  const previousFetch = globalThis.fetch;
  globalThis.fetch = async () => ({
    ok: true,
    json: async () => [
      [0, "/charts/first", "Bitcoin Same Title"],
      [8, "second", "Bitcoin Same Title"],
    ],
  });
  globalThis.self = {
    addEventListener: (_, callback) => { onMessage = callback; },
    postMessage: message => { reply = message; },
  };
  try {
    await import(`${workerUrl}?duplicate-titles`);
    await onMessage({ data: { id: 1, needle: "same title" } });
    assert.deepEqual(reply.results.map(([, href]) => href), ["/charts/first", "/charts/second"]);
  } finally {
    globalThis.fetch = previousFetch;
    delete globalThis.self;
  }
});

test("catalog HTTP failures are returned to the caller", async () => {
  let onMessage;
  let reply;
  const previousFetch = globalThis.fetch;
  globalThis.fetch = async () => ({ ok: false });
  globalThis.self = {
    addEventListener: (_, callback) => { onMessage = callback; },
    postMessage: message => { reply = message; },
  };
  try {
    await import(`${workerUrl}?missing-catalog`);
    await onMessage({ data: { id: 7, needle: "price" } });
    assert.deepEqual(reply, { id: 7, error: "Failed to load chart search index" });
  } finally {
    globalThis.fetch = previousFetch;
    delete globalThis.self;
  }
});

test("chart routes match encoded cutoffs, symbols and existing percent escapes", async () => {
  const { decodePathSegment } = await import("../../website/scripts/utils/url.js");
  for (const [url, catalog] of [
    ["%3C4m", "<4m"],
    ["sth-(%3C5m)", "sth-(<5m)"],
    ["supply-density-(%C2%B110%25)", "supply-density-(±10%25)"],
    ["all", "all"],
  ]) assert.equal(decodePathSegment(url), decodePathSegment(catalog));
  assert.equal(decodePathSegment("malformed%"), "malformed%");
});
