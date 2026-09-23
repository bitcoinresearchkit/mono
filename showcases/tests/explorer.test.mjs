import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const html = await readFile(new URL('../explorer/index.html', import.meta.url), 'utf8');
function script(id) {
  const source = html.match(new RegExp(`<script id="${id}"[^>]*>([\\s\\S]*?)</script>`));
  assert.ok(source, `Missing inline script: ${id}`);
  return source[1];
}
const { createCatalog, isSeries } = new Function('self', `${script('catalog-worker')}; return { createCatalog, isSeries };`)({});
const { chartPoints, toCSV, humanize, unitForType, axisNumber } = new Function(`${script('explorer-data')}; return { chartPoints, toCSV, humanize, unitForType, axisNumber };`)();

const { BitviewClient } = await import(process.env.EXPLORER_CLIENT_URL || '../../modules/bitview-client/index.js');
const client = new BitviewClient({ baseUrl: 'https://bitview.space', browserCache: false });
const catalog = createCatalog(client.series);

test('the real generated catalog is completely browsable, including raw and structured series', () => {
  let total = 0;
  function walk(node, path = []) {
    const entry = catalog.browse(path).node;
    if (isSeries(node)) {
      total++;
      assert.equal(entry.name, node.name);
      assert.deepEqual(entry.indexes, node.indexes());
      return;
    }
    for (const [key, child] of Object.entries(node)) walk(child, [...path, key]);
  }
  walk(client.series);
  assert.equal(catalog.total, total);
  assert.ok(total > 10000);
  assert.equal(catalog.browse(['blocks', 'blockhash']).node.name, 'blockhash');
});

test('branch and search pagination never omit or duplicate paths', () => {
  const all = catalog.search('supply', 0, 100000).items;
  const pages = [];
  for (let offset = 0; offset < all.length; offset += 100) pages.push(...catalog.search('supply', offset).items);
  assert.deepEqual(pages, all);
  assert.equal(new Set(pages.map(item => item.path.join('/'))).size, pages.length);
  assert.ok(pages.every(item => item.path.join(' ').toLowerCase().includes('supply') || item.name?.includes('supply')));
  const branches = catalog.browse([]);
  assert.deepEqual([...catalog.browse([], 0, 10).items, ...catalog.browse([], 10, 100).items], branches.items);
});

test('search accepts words, camel case, and exact API names without losing aliases', () => {
  const exact = catalog.search('sth_realized_price');
  assert.ok(exact.items.some(item => item.name === 'sth_realized_price'));
  assert.equal(catalog.search('price').items[0].name, 'price');
  assert.equal(catalog.search('sth realized price').total, exact.total);
  assert.ok(catalog.search('opReturn').total > 0);
  assert.equal(catalog.search('zzzz_not_a_real_series_zzzz').total, 0);
  assert.equal(catalog.search('').total, 0);
});

test('navigation rejects inherited properties and paths inside series objects', () => {
  assert.throws(() => catalog.browse(['__proto__']), /not in the current catalog/);
  assert.throws(() => catalog.browse(['constructor']), /not in the current catalog/);
  assert.throws(() => catalog.browse(['price', 'spot', 'usd', 'by']), /not in the current catalog/);
});

test('preview aligns timestamps by absolute index and preserves null gaps and zeroes', () => {
  const result = chartPoints({ start: 12, type: 'Dollars', data: [0, null, 20, true] }, { start: 10, data: [98, 99, 100, 101, 102, 103] });
  assert.deepEqual(result.points, [{ time: 100, value: 0 }, { time: 101 }, { time: 102, value: 20 }, { time: 103, value: 1 }]);
});

test('duplicate timestamps combine candles without losing open, high, or low', () => {
  const result = chartPoints({ start: 0, type: 'OHLCDollars', data: [[10, 20, 5, 15], [15, 25, 8, 20], null] }, { start: 0, data: [100, 100, 101] });
  assert.deepEqual(result.points, [{ time: 100, open: 10, high: 25, low: 5, close: 20 }, { time: 101 }]);
});

test('unmapped, unordered, non-numeric and malformed candle data are table-only', () => {
  const scalar = { start: 0, type: 'Dollars', data: [1, 2] };
  assert.equal(chartPoints(scalar, null).points.length, 0);
  assert.equal(chartPoints(scalar, { start: 0, data: [102, 101] }).points.length, 0);
  assert.equal(chartPoints({ ...scalar, data: [{ sats: 1 }] }, { start: 0, data: [100] }).points.length, 0);
  assert.equal(chartPoints({ ...scalar, type: 'OHLC<Dollars>', data: [[10, 5, 20, 15]] }, { start: 0, data: [100] }).points.length, 0);
  assert.equal(chartPoints({ ...scalar, data: [null, null] }, { start: 0, data: [100, 101] }).points.length, 0);
  assert.match(chartPoints({ ...scalar, data: [] }, null).reason, /^No values/);
});

test('CSV preserves indexes, precision and structures while escaping formulas and quotes', () => {
  const csv = toCSV({ start: 10, data: [1.123456789012, null, { a: '"' }, '=CMD()', -1] }, { start: 10, data: [100, 101, 102, 103, 104] });
  assert.ok(csv.includes('"10","1970-01-01T00:01:40.000Z","1.123456789012"'));
  assert.ok(csv.includes('"\'=CMD()"'));
  assert.ok(csv.includes('"-1"'));
  assert.equal(csv.split('\r\n').length, 6);
});


test('display names expand periods and preserve financial acronyms', () => {
  assert.equal(humanize('dca_stack_10y'), 'DCA stack 10 years');
  assert.equal(humanize('_1y'), '1 year');
  assert.equal(humanize('cagr_3m'), 'CAGR 3 months');
  assert.equal(humanize('sthMvrv'), 'STH MVRV');
  assert.equal(humanize('month1'), 'Month1');
});

test('units come from known value types, including candles, without guessing for generic numbers', () => {
  assert.equal(unitForType('Bitcoin'), 'BTC');
  assert.equal(unitForType('OHLCDollars'), 'USD');
  assert.equal(unitForType('OHLC<Dollars>'), 'USD');
  assert.equal(unitForType('SatsSigned'), 'sats');
  assert.equal(unitForType('StoredF64'), '');
  assert.equal(unitForType('constructor'), '');
  assert.equal(unitForType(), '');
  assert.equal(axisNumber.format(0.00000001), '0.00000001');
  assert.equal(axisNumber.format(0), '0');
});
