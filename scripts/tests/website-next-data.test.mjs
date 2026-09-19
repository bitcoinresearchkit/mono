import assert from "node:assert/strict";
import { test } from "node:test";
import { BitviewClient } from "../../modules/bitview-client/index.js";
import { createBlockPreviewFilterData } from "../../website_next/explore/block/preview/filters/data.js";
import { FILTERS } from "../../website_next/explore/block/preview/filters/model.js";
import { encodeBase58Check } from "../../website_next/wallets/derive/base58.js";
import { generateAddressesFromWalletSource } from "../../website_next/wallets/derive/index.js";
import { findUsablePrefixBucket } from "../../website_next/wallets/lookup/bucket.js";

const apiSchema = {
  paths: {
    "/api/blocks/tip/height": {
      get: {
        summary: "Current block height",
        responses: { "200": { content: { "application/json": { schema: { type: "integer" } } } } },
      },
    },
  },
};
const metricCatalog = {
  market: { price: { name: "bitcoin_price", kind: "Dollars", indexes: ["height"] } },
};

for (const [kind, catalog, query, expected] of [
  ["api", apiSchema, "block height", "GET /api/blocks/tip/height"],
  ["metrics", metricCatalog, "bitcoin price", "market.price"],
]) {
  test(`${kind} worker imports its matcher and searches the supplied catalog`, async (t) => {
    let onMessage;
    const replies = [];
    t.mock.method(globalThis, "fetch", async () => Response.json(catalog));
    const previousSelf = globalThis.self;
    globalThis.self = {
      addEventListener: (_, callback) => { onMessage = callback; },
      postMessage: (message) => replies.push(message),
    };
    t.after(() => {
      if (previousSelf === undefined) delete globalThis.self;
      else globalThis.self = previousSelf;
    });
    await import(`../../website_next/ask/tools/${kind}/worker.js`);
    await onMessage({ data: { id: "search", type: "search", data: {
      url: "https://fixture.invalid/catalog", queries: [query], limit: 8, prefixes: [],
    } } });
    const result = replies.at(-1);
    assert.equal(result.status, "complete", JSON.stringify(result));
    assert.ok(result.data.some((item) => (item.key ?? item.path) === expected));
  });
}

test("block filters fetch real client count and transaction membership endpoints", async (t) => {
  const paths = [];
  t.mock.method(globalThis, "fetch", async (url) => {
    const parsed = new URL(url);
    paths.push(parsed.pathname);
    const data = parsed.pathname.endsWith("/tx_index") ? [1, 2] : [1];
    return Response.json({ data, index: "height", start: 0, end: data.length, total: data.length });
  });
  const filters = createBlockPreviewFilterData(100, { start: 200, end: 202 }, new AbortController().signal);
  const counts = await filters.loadCounts();
  const byKey = (key) => FILTERS.find((filter) => filter.key === key);
  for (const key of ["behavior:cpfp_parent", "behavior:coinjoin", "policy:nonstandard", "sighash:anyone_can_pay"]) {
    assert.equal(counts[byKey(key).index], 1, key);
  }
  for (const key of ["input:one", "output:one"]) {
    assert.deepEqual([...await filters.loadMembership(byKey(key))], [1, 0]);
  }
  assert.ok(paths.some((path) => path.endsWith("/input_count/tx_index")));
  assert.ok(paths.some((path) => path.endsWith("/output_count/tx_index")));
  assert.ok(paths.every((path) => !path.includes("undefined")));
});

test("derived wallet addresses reach the client's payload validation and prefix lookup", async (t) => {
  // Synthetic public-only extended key with the secp256k1 generator and a fixed chain code.
  const payload = new Uint8Array(78);
  new DataView(payload.buffer).setUint32(0, 0x0488b21e);
  payload.fill(1, 13, 45);
  payload.set(Buffer.from("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798", "hex"), 45);
  const xpub = await encodeBase58Check(payload);
  const client = new BitviewClient("https://fixture.invalid");
  const requestedTypes = [];
  t.mock.method(client, "getAddressHashPrefixMatches", async (addrType, prefix) => {
    requestedTypes.push(addrType);
    return { addrType, prefix, truncated: false, addresses: [] };
  });
  for (const [source, options, expected] of [
    [xpub, { script: "p2pkh" }, "p2pkh"],
    [xpub, { script: "p2sh_p2wpkh" }, "p2sh"],
    [xpub, { script: "v0_p2wpkh" }, "p2wpkh"],
    [xpub, { script: "v1_p2tr" }, "p2tr"],
    [`wsh(sortedmulti(1,${xpub}/0/*))`, {}, "p2wsh"],
  ]) {
    const [address] = await generateAddressesFromWalletSource(source, { ...options, count: 1 });
    const bucket = await findUsablePrefixBucket(client, address);
    assert.equal(bucket.addrType, expected);
  }
  assert.deepEqual(requestedTypes, ["p2pkh", "p2sh", "p2wpkh", "p2tr", "p2wsh"]);
});
