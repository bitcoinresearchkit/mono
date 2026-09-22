import assert from 'node:assert/strict';
import { test } from 'node:test';
import { BitviewClient } from '../index.js';

test('generated catalog exposes matching named and indexed endpoints', () => {
  const client = new BitviewClient('http://fixture.invalid');
  let series = 0;
  function visit(node) {
    if (typeof node.indexes === 'function') {
      series++;
      for (const index of node.indexes()) {
        const expected = `/api/series/${node.name}/${index}`;
        assert.equal(node.by[index].path, expected);
        assert.equal(node.get(index).path, expected);
      }
      return;
    }
    for (const child of Object.values(node)) {
      if (child && typeof child === 'object') visit(child);
    }
  }
  visit(client.series);
  assert.ok(series > 0);
});
