import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile, mkdtemp, copyFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { buildSnapshot, seriesNames } from './generate.mjs';

const html = await readFile(new URL('./index.html', import.meta.url), 'utf8');
const functionCode = html.slice(html.indexOf('  function cumulativeShares('), html.indexOf('  const element ='));
const cumulativeShares = new Function(`${functionCode}; return cumulativeShares;`)();
function histories() {
  return seriesNames.map((_, i) => ({ type: i ? 'Cents' : 'OHLCCents', index: 'day1', start: 0, end: 2,
    stamp: '2009-01-03T12:00:00Z', data: i ? [100, 120] : [[90, 110, 80, 100], [100, 130, 90, 110]] }));
}

test('cumulative percentages count ties, keep each denominator aligned, and restart for a new range', () => {
  const input = [{close:100,cp:100,tmm:110},{close:90,cp:100,tmm:80},{close:120,cp:100,tmm:110}];
  const rows = cumulativeShares(input);
  assert.deepEqual(rows.map(r=>[r.cpCount,r.tmmCount,r.days]), [[1,0,1],[1,1,2],[2,2,3]]);
  assert.equal(rows[2].cpShare, 200/3);
  assert.deepEqual(cumulativeShares(input.slice(1)).map(r=>[r.cpShare,r.tmmShare]), [[0,100],[50,100]]);
  assert.equal(input[0].cpCount, undefined);
});

test('snapshots require all-holder cointime CP and TMM, complete matching histories, and valid prices', () => {
  assert.deepEqual(seriesNames, ['price_ohlc_cents','awake_capitalized_price_cents','true_market_mean_cents']);
  assert.deepEqual(buildSnapshot(histories(),0,2).rows[0], [90,110,80,100,100,100]);
  for (const mutate of [h=>h[1].data.pop(), h=>h[1].data[0]=null, h=>h[2].stamp='other', h=>h[0].data[0]=[90,80,80,100]]) {
    const h=histories(); mutate(h); assert.throws(()=>buildSnapshot(h,0,2));
  }
});

test('standalone generator excludes the current UTC day and preserves the HTML on input failure', async () => {
  const directory=await mkdtemp(join(tmpdir(),'middle-'));
  const originalFetch=globalThis.fetch;
  try {
    const module=pathToFileURL(join(directory,'generate.mjs'));
    const output=pathToFileURL(join(directory,'index.html'));
    await copyFile(new URL('./generate.mjs',import.meta.url),module);
    await writeFile(output,'<script id="chart-data" type="application/json">{}</script>');
    const {generateSnapshot}=await import(module);
    let data=histories();
    globalThis.fetch=async url=>{
      assert.equal(new URL(url).searchParams.get('end'),'2');
      return {ok:true,json:async()=>data};
    };
    const options={startDate:'2009-01-01',now:new Date('2009-01-03T12:00:00Z')};
    assert.equal((await generateSnapshot(options)).through,'2009-01-02');
    const good=await readFile(output,'utf8');
    data[1].data[1]=null;
    await assert.rejects(generateSnapshot(options));
    assert.equal(await readFile(output,'utf8'),good);
  } finally { globalThis.fetch=originalFetch; await rm(directory,{recursive:true,force:true}); }
});

test('starts with the first available Bitcoin price and excludes unavailable models from both counts', () => {
  const h = histories();
  h[0].data[0] = [0,0,0,0];
  assert.equal(buildSnapshot(h,0,2).start, '2009-01-02');
  const rows = cumulativeShares([{close:6,cp:5,tmm:0},{close:7,cp:6,tmm:1}]);
  assert.deepEqual(rows.map(r=>[r.days,r.cpShare,r.tmmShare]), [[0,null,null],[1,100,100]]);
  h[0].data[0] = [3,6,3,6];
  h[2].data[0] = 0;
  assert.equal(buildSnapshot(h,0,2).rows[0][5], 0);
});

test('winner compares distance on either side of 50%, preserves exact ties, and restarts with the window', () => {
  const input = [
    {close:100,cp:90,tmm:90},
    {close:100,cp:110,tmm:90},
    {close:100,cp:110,tmm:110},
    {close:100,cp:110,tmm:110},
  ];
  const rows = cumulativeShares(input);
  assert.deepEqual(rows.map(r=>r.advantage), [0,50,0,-25]);
  assert.deepEqual(cumulativeShares(input.slice(2)).map(r=>r.advantage), [0,0]);
  assert.equal(cumulativeShares([{close:100,cp:0,tmm:90}])[0].advantage, null);
});
