import assert from 'node:assert/strict';
import { mkdtemp, copyFile, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import test from 'node:test';
import { buildSnapshot, seriesNames, sources } from './generate.mjs';

const expected = [
  'under_4m_awake_price_cents', 'under_4m_awake_capitalized_price_cents',
  'under_4m_coinflow_price_cents', 'under_4m_coinflow_capitalized_price_cents',
  'sth_awake_price_cents', 'sth_awake_capitalized_price_cents',
  'sth_coinflow_price_cents', 'sth_coinflow_capitalized_price_cents',
  'under_6m_awake_price_cents', 'under_6m_awake_capitalized_price_cents',
  'under_6m_coinflow_price_cents', 'under_6m_coinflow_capitalized_price_cents',
];

function histories(end) {
  return seriesNames.map((name, index) => ({
    type: index === 0 ? 'OHLCCents' : 'Cents', index: 'day1', start: 0, end,
    data: Array.from({ length: end }, (_, day) => {
      if (index === 0) return [100_000, 120_000, 80_000, 110_000];
      if (name.includes('cost_basis')) return name.endsWith('min_cents') ? 1 : 1_000_000;
      if (index - 1 === day % 12) return day < 12 ? 1_000 + day : 20_000 + day;
      return 10_000;
    }),
  }));
}

test('each of the 12 weighted prices can define the cloud, independently of candles and trends', () => {
  assert.deepEqual(sources, expected);
  const snapshot = buildSnapshot(histories(24), 0, 24, '2009-01-24T12:00:00Z');
  snapshot.rows.forEach((row, day) => {
    assert.deepEqual(row.slice(4), day < 12 ? [1_000 + day, 10_000] : [10_000, 20_000 + day]);
  });
  assert.equal(snapshot.bounds.length, 3);
  assert.equal(snapshot.through, '2009-01-24');
});

test('every weighted input is required for the latest partial day', () => {
  for (let index = 1; index <= 12; index++) {
    const data = histories(2);
    data[index].data[1] = null;
    assert.throws(() => buildSnapshot(data, 0, 2, '2009-01-02T12:00:00Z'), /Invalid price/);
  }
});

test('standalone generator writes the partial day and preserves the snapshot on invalid inputs', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'sth-weighted-'));
  const fetch = globalThis.fetch;
  try {
    const generator = pathToFileURL(join(directory, 'generate.mjs'));
    const output = pathToFileURL(join(directory, 'index.html'));
    await copyFile(new URL('./generate.mjs', import.meta.url), generator);
    await writeFile(output, '<script id="chart-data" type="application/json">{}</script>');
    const { generateSnapshot } = await import(generator);
    const data = histories(2);
    globalThis.fetch = async url => {
      assert.equal(new URL(url).searchParams.get('series'), seriesNames.join(','));
      assert.equal(new URL(url).searchParams.get('end'), '2');
      return { ok: true, json: async () => data };
    };
    const options = { startDate: '2009-01-01', now: new Date('2009-01-02T12:00:00Z') };
    assert.equal((await generateSnapshot(options)).through, '2009-01-02');
    const before = await readFile(output, 'utf8');
    data[12].data[1] = null;
    await assert.rejects(generateSnapshot(options), /Invalid price/);
    assert.equal(await readFile(output, 'utf8'), before);
  } finally {
    globalThis.fetch = fetch;
    await rm(directory, { recursive: true, force: true });
  }
});
