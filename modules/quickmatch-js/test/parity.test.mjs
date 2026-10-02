import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { QuickMatch, QuickMatchConfig } from '../src/index.js';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const hex = (s) => Buffer.from(s).toString('hex');
let seed = 0x51f15e;
const random = (n) => ((seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0) % n);
const words = ['hash', 'rate', 'supply', 'price', 'realized', 'sth', 'lth', 'profit', 'loss', 'market', 'cap', 'day', 'close'];
const pick = () => words[random(words.length)];

for (const separators of ['_- :/', '_. /', ' ', '']) {
  test(`Rust result parity, separators ${JSON.stringify(separators)}`, () => {
    const items = ['', 'price', 'price', 'hash_rate', 'hashrate', 'supply_in_profit', 'sth_realized_price', 'realized_price_sth'];
    for (let i = 0; i < 300; i++) items.push(Array.from({ length: 1 + random(4) }, pick).join(separators[random(separators.length)] ?? '_'));
    const queries = ['', ' ', 'PRIce', 'hashrate', 'suply', 'price price', 'price supply', 'rate hash', 'x'.repeat(1000), '\0price', 'price\0', '\u0085 price \u0085', '\ufeff price \ufeff', '\u2003PRICE\u2003', 'éprice', '🔥 supply', 'price\t'];
    queries.push('realized price sth', 'price realized sth', 'sth price realized', 'price sth realized', 'realized sth price', 'prcie', 'realized prcie sth', 'supply porfit', 'prcie porfit', 'hash rte');
    for (let i = 0; i < 500; i++) {
      let q = Array.from({ length: 1 + random(5) }, pick).join(' ');
      const at = random(q.length);
      switch (random(5)) {
        case 0: q = q.slice(0, at) + q.slice(at + 1); break;
        case 1: q = q.slice(0, at) + 'x' + q.slice(at); break;
        case 2: q = q.slice(0, at); break;
        case 3: q = q.toUpperCase(); break;
      }
      queries.push(q);
    }
    const cases = queries.flatMap(query => [true, false].map(union => ({
      query,
      config: new QuickMatchConfig().withSeparators(random(5) ? separators : '_- :/').withLimit([0, 1, 5, 100, 1000][random(5)])
        .withTrigramBudget([0, 1, 6, 20, 30][random(5)]).withMinScore([0, 1, 2, 5][random(4)]).withUnionFallback(union),
    })));
    const input = [hex(separators), items.length, ...items.map(hex), ...cases.map(({query, config:c}) =>
      [c.limit, c.trigramBudget, c.minScore, c.unionFallback, hex(c.separators), hex(query)].join('\t'))].join('\n') + '\n';
    const rust = spawnSync('cargo', ['run', '--quiet', '--offline', '-p', 'quickmatch', '--example', 'js_parity'], {cwd:root, input, encoding:'utf8', maxBuffer:32*1024*1024});
    assert.equal(rust.status, 0, rust.stderr);
    const expected = rust.stdout.trim().split('\n').map(JSON.parse);
    assert.equal(expected.length, cases.length);
    const matcher = new QuickMatch(items, new QuickMatchConfig().withSeparators(separators));
    cases.forEach(({query, config}, i) => {
      const context = JSON.stringify({query, config});
      assert.deepEqual(matcher.matchesWithIdsAndMatchedWords(query, config), expected[i][0], context);
      assert.deepEqual(matcher.matchesBestWithIdsAndMatchedWords(query, config), expected[i][1], context);
      assert.deepEqual(matcher.matchesExactWithIdsAndMatchedWords(query, config), expected[i][2], context);
      assert.deepEqual(matcher.matchesWithMatchedWords(query, config), expected[i][0].map(([id,count]) => [items[id],count]), context);
      assert.deepEqual(matcher.matchesWith(query, config), expected[i][0].map(([id]) => items[id]), context);
    });
    assert.deepEqual(matcher.matches('price'), matcher.matchesWith('price', new QuickMatchConfig().withSeparators(separators)));
  });
}
