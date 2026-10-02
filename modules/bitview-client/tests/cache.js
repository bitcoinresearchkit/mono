// Offline HTTP/cache regression: run with node tests/cache.js.
import assert from 'node:assert/strict';
import { once } from 'node:events';
import { createServer } from 'node:http';
import { after, before, test } from 'node:test';
import { BitviewClient } from '../index.js';

const resources = new Map();
const calls = [];
let baseUrl;
const server = createServer((request, response) => {
  const resource = resources.get(request.url);
  const validator = request.headers['if-none-match'];
  const unchanged = resource.etag && validator === resource.etag && !resource.ignoreConditional;
  const status = resource.status ?? (unchanged ? 304 : 200);
  const body = status === 304 ? '' : resource.body;
  const headers = {
    'Content-Type': resource.type ?? 'application/json',
    'Cache-Control': 'public, no-cache',
  };
  if (resource.etag && !(status === 304 && resource.omit304Etag)) headers.ETag = resource.etag;
  calls.push({ path: request.url, validator, status, bytes: Buffer.byteLength(body) });
  response.writeHead(status, headers);
  response.end(body);
});

before(async () => {
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  baseUrl = `http://127.0.0.1:${server.address().port}`;
});
after(async () => {
  server.closeAllConnections();
  await new Promise(resolve => server.close(resolve));
});

function resource(path, options = {}) {
  const value = { etag: 'W/"v1"', body: '[1,2,3]', ...options };
  resources.set(path, value);
  return value;
}

function trackBodies(t) {
  const fetch = globalThis.fetch;
  const counts = { parsed: 0, cancelled: 0 };
  globalThis.fetch = async (...args) => {
    const response = await fetch(...args);
    const json = response.json.bind(response);
    response.json = () => { counts.parsed++; return json(); };
    if (response.body) {
      const cancel = response.body.cancel.bind(response.body);
      response.body.cancel = () => { counts.cancelled++; return cancel(); };
    }
    return response;
  };
  t.after(() => { globalThis.fetch = fetch; });
  return counts;
}

test('retained ETags produce 304s without parsing or a second callback', async t => {
  const source = resource('/values', { omit304Etag: true });
  const counts = trackBodies(t);
  const client = new BitviewClient(baseUrl);
  const values = [];
  const read = () => client.getJson('/values', { onValue: value => values.push(value) });
  const first = await read();
  assert.deepEqual(first, [1, 2, 3]);
  values.length = 0;
  assert.strictEqual(await read(), first);
  assert.deepEqual(values, [first]);
  assert.equal(counts.parsed, 1);
  assert.deepEqual(calls.filter(call => call.path === '/values').map(call =>
    [call.validator, call.status, call.bytes]), [
    [undefined, 200, 7], ['W/"v1"', 304, 0],
  ]);

  source.etag = '"v2"';
  source.body = '[4,5,6]';
  values.length = 0;
  const changed = await read();
  assert.deepEqual(changed, [4, 5, 6]);
  assert.notStrictEqual(changed, first);
  assert.deepEqual(values, [first, changed]);
  assert.strictEqual(await read(), changed);
  assert.equal(counts.parsed, 2);
  assert.deepEqual(calls.at(-1), { path: '/values', validator: '"v2"', status: 304, bytes: 0 });
});









test('cross-origin browsers keep native revalidation and cancel unneeded bodies', async t => {
  globalThis.location = new URL('https://dashboard.example');
  t.after(() => { delete globalThis.location; });
  const counts = trackBodies(t);
  resource('/cross-origin');
  const client = new BitviewClient(baseUrl);
  const first = await client.getJson('/cross-origin');
  assert.strictEqual(await client.getJson('/cross-origin'), first);
  assert.equal(calls.at(-1).validator, undefined);
  assert.equal(counts.parsed, 1);
  assert.equal(counts.cancelled, 1);
});



test('a browser-cache race reuses its parsed result and releases the network body', async t => {
  resource('/browser-cache');
  const counts = trackBodies(t);
  const client = new BitviewClient(baseUrl);
  await client._browserCachePromise;
  client._browserCache = {
    match: async () => new Response('[1,2,3]', { headers: { ETag: 'W/"v1"' } }),
    put: () => assert.fail('unchanged response must not rewrite the browser cache'),
  };
  const values = [];
  const result = await client.getJson('/browser-cache', { onValue: value => values.push(value) });
  assert.deepEqual(result, [1, 2, 3]);
  assert.deepEqual(values, [result]);
  assert.equal(counts.parsed, 0);
  assert.equal(counts.cancelled, 1);
});
