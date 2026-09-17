import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtemp, readFile, writeFile, copyFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { densityBalance, buildSnapshot, seriesNames } from './generate.mjs';

function histories(values = [[500_000, 400_000, 100_000], [499_999, 400_000, 99_999]]) {
  return seriesNames.map((name, index) => ({
    type: index === 0 ? 'OHLCCents' : 'PartsPerMillion32', index: 'day1',
    start: 0, end: values.length, stamp: '2009-01-02T12:00:00Z',
    data: values.map(row => index === 0 ? [100, 120, 90, 110] : row[index - 1]),
  }));
}

test('inclusive 50% cutoff, signed difference, and genuine zero ties', () => {
  assert.equal(densityBalance(499_999, 400_000, 99_999), null);
  assert.equal(densityBalance(500_000, 400_000, 100_000), 300_000);
  assert.equal(densityBalance(500_000, 100_000, 400_000), -300_000);
  assert.equal(densityBalance(500_000, 250_000, 250_000), 0);
  assert.equal(densityBalance(0, 0, 0), null);
  assert.equal(densityBalance(1_000_000, 0, 1_000_000), -1_000_000);
  assert.equal(densityBalance(500_000, 300_000, 200_001), 99_999);
});

test('missing, invalid, or inconsistent values cannot masquerade as an inactive day', () => {
  for (const value of [null, undefined, NaN, -1, 1_000_001, 0.5]) {
    assert.throws(() => densityBalance(value, 300_000, 200_000));
    assert.throws(() => densityBalance(500_000, value, 200_000));
    assert.throws(() => densityBalance(500_000, 300_000, value));
  }
  assert.throws(() => densityBalance(500_000, 400_000, 300_000), /inconsistent/);
});

test('snapshot preserves price on inactive days and uses only <6m cointime 5% inputs', () => {
  assert.ok(seriesNames.slice(1).every(name => name.startsWith('bedrock_cointime_under_6m_supply_density') && !name.includes('10pct')));
  const snapshot = buildSnapshot(histories(), 0, 2, '2009-01-02T12:00:00Z');
  assert.deepEqual(snapshot.rows.map(row => row[4]), [30, null]);
  assert.deepEqual(snapshot.rows.map(row => row.slice(0, 4)), [[100, 120, 90, 110], [100, 120, 90, 110]]);
  assert.equal(snapshot.through, '2009-01-02');
  for (const mutate of [
    data => { data[1].end--; },
    data => { data[1].type = 'Cents'; },
    data => { data[1].stamp = 'old'; },
    data => { data[2].data.pop(); },
    data => { data[0].data[0] = [100, 90, 90, 110]; },
  ]) {
    const data = histories(); mutate(data);
    assert.throws(() => buildSnapshot(data, 0, 2));
  }
});

test('generator works independently, includes the partial day, and preserves output on failure', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'density-page-'));
  const previousFetch = globalThis.fetch;
  try {
    const module = pathToFileURL(join(directory, 'generate.mjs'));
    const output = pathToFileURL(join(directory, 'index.html'));
    await copyFile(new URL('./generate.mjs', import.meta.url), module);
    await writeFile(output, '<script id="chart-data" type="application/json">{}</script>');
    const { generateSnapshot } = await import(module);
    let data = histories();
    globalThis.fetch = async url => {
      const query = new URL(url).searchParams;
      assert.equal(query.get('end'), '2');
      assert.deepEqual(query.get('series').split(','), seriesNames);
      return { ok: true, json: async () => data };
    };
    const options = { startDate: '2009-01-01', now: new Date('2009-01-02T12:00:00Z') };
    const result = await generateSnapshot(options);
    assert.equal(result.rows.length, 2);
    const valid = await readFile(output, 'utf8');
    data = histories(); data[1].data[1] = null;
    await assert.rejects(generateSnapshot(options), /Invalid/);
    assert.equal(await readFile(output, 'utf8'), valid);
  } finally {
    globalThis.fetch = previousFetch;
    await rm(directory, { recursive: true, force: true });
  }
});


test('shareable output contains only price and final indicator values', async () => {
  const snapshot = buildSnapshot(histories([[600_000, 200_000, 400_000], [499_999, 400_000, 99_999]]), 0, 2);
  assert.deepEqual(snapshot.columns, ['open', 'high', 'low', 'close', 'signal', 'level']);
  assert.deepEqual(snapshot.rows.map(row => row.slice(4)), [[-20, 10], [null, 0]]);
  const html = await readFile(new URL('./index.html', import.meta.url), 'utf8');
  assert.equal(html.match(/density|cointime|coinflow|under_6m|in_profit|in_loss|profit|loss|500_000|above 50%|60% pressure|localhost:3110/i)?.[0], undefined);
  const embedded = JSON.parse(html.match(/<script id="chart-data" type="application\/json">([\s\S]*?)<\/script>/)[1]);
  assert.deepEqual(embedded.columns, snapshot.columns);
  assert.ok(embedded.rows.every(row => row.length === 6));
});
