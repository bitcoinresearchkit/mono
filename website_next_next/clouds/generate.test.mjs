import assert from 'node:assert/strict';
import { mkdtemp, copyFile, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import test from 'node:test';
import { buildSnapshot, cloudSources, seriesNames } from './generate.mjs';
import { sources as sthSources } from '../sth/generate.mjs';
import { sources as lthSources } from '../lth/generate.mjs';

function histories(end) {
  return seriesNames.map((name, index) => ({
    stamp: 123, type: index === 0 ? 'OHLCCents' : 'Cents', index: 'day1', start: 0, end,
    data: Array.from({ length: end }, (_, day) => {
      if (index === 0) return [100_000, 120_000, 80_000, 110_000];
      if (name.includes('cost_basis')) return name.endsWith('min_cents') ? 1 : 1_000_000 + day;
      return 10_000;
    }),
  }));
}

test('all original weighted inputs can independently define their own cloud', () => {
  assert.deepEqual(cloudSources, { sth: sthSources, holders: [
    'awake_price_cents', 'awake_capitalized_price_cents',
    'coinflow_price_cents', 'coinflow_capitalized_price_cents',
  ], lth: lthSources });
  for (const [id, names] of Object.entries(cloudSources)) {
    for (const name of names) {
      const data = histories(2);
      data[seriesNames.indexOf(name)].data = [1_000, 20_000];
      const snapshot = buildSnapshot(data, 0, 2, '2009-01-02T12:00:00Z');
      for (const cloud of snapshot.clouds) {
        assert.deepEqual(cloud.ranges, cloud.id === id
          ? [[1_000, 10_000], [10_000, 20_000]] : [[10_000, 10_000], [10_000, 10_000]]);
      }
      data[seriesNames.indexOf(name)].data[1] = null;
      assert.throws(() => buildSnapshot(data, 0, 2), /refusing a partial cloud/);
    }
  }
});

test('trend selection survives cropping and stays outside cloud ranges', () => {
  const snapshot = buildSnapshot(histories(3), 2, 3, '2009-01-03T12:00:00Z');
  assert.deepEqual(snapshot.rows, [[100_000, 120_000, 80_000, 110_000]]);
  for (const bound of snapshot.bounds) assert.deepEqual(bound.points, [[1, 0]]);
  for (const cloud of snapshot.clouds) assert.deepEqual(cloud.ranges, [[10_000, 10_000]]);
});

test('standalone generation batches at one tip and preserves output on inconsistent or missing inputs', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'combined-clouds-'));
  const originalFetch = globalThis.fetch;
  try {
    const generator = pathToFileURL(join(directory, 'generate.mjs'));
    const output = pathToFileURL(join(directory, 'index.html'));
    await copyFile(new URL('./generate.mjs', import.meta.url), generator);
    await writeFile(output, '<script id="chart-data" type="application/json">{}</script>');
    const { generateSnapshot } = await import(generator);
    const data = histories(2);
    const requests = [];
    globalThis.fetch = async url => {
      const query = new URL(url).searchParams;
      const names = query.get('series').split(',');
      requests.push(names);
      assert.ok(names.length <= 32);
      assert.equal(query.get('start'), '0');
      assert.equal(query.get('end'), '2');
      return { ok: true, json: async () => names.map(name => data[seriesNames.indexOf(name)]) };
    };
    const options = { startDate: '2009-01-01', now: new Date('2009-01-02T12:00:00Z') };
    const snapshot = await generateSnapshot(options);
    assert.equal(requests.length, 2);
    assert.deepEqual(requests.flat(), seriesNames);
    assert.equal(snapshot.through, '2009-01-02');
    const before = await readFile(output, 'utf8');
    assert.deepEqual(JSON.parse(before.match(/>(.*)</)[1]), snapshot);
    data.at(-1).stamp++;
    await assert.rejects(generateSnapshot(options), /backend changed/);
    assert.equal(await readFile(output, 'utf8'), before);
    data.at(-1).stamp--;
    data[28].data[1] = null;
    await assert.rejects(generateSnapshot(options), /refusing a partial cloud/);
    assert.equal(await readFile(output, 'utf8'), before);
  } finally {
    globalThis.fetch = originalFetch;
    await rm(directory, { recursive: true, force: true });
  }
});
