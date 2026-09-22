import assert from 'node:assert/strict';
import { test } from 'node:test';
import { BitviewClient } from '../index.js';

const calendarDate = date => [date.getFullYear(), date.getMonth() + 1, date.getDate()]
  .map((value, index) => index ? String(value).padStart(2, '0') : value).join('-');

test('series helpers preserve indexes, values, dates and range requests', async t => {
  const client = new BitviewClient('http://fixture.invalid');
  const requests = [];
  t.mock.method(client, 'getJson', async path => {
    requests.push(path);
    const index = new URL(path, 'http://fixture.invalid').pathname.split('/').at(-1);
    return { index, start: 0, end: 3, version: 1, type: 'Dollars', stamp: '', data: [0, null, 30] };
  });
  for (const index of ['day1', 'height']) {
    const values = await client.series.price.spot.usd.get(index).first(3).fetch();
    assert.equal(values.isDateBased, index === 'day1');
    assert.deepEqual(values.indexes(), [0, 1, 2]);
    assert.deepEqual(values.keys(), [0, 1, 2]);
    const entries = [[0, 0], [1, null], [2, 30]];
    assert.deepEqual(values.entries(), entries);
    assert.deepEqual([...values], entries);
    assert.deepEqual([...values.toMap()], entries);
    if (index === 'day1') {
      const dates = ['2009-01-03', '2009-01-09', '2009-01-10'];
      assert.deepEqual(values.dates().map(calendarDate), dates);
      const dated = dates.map((date, i) => [date, values.data[i]]);
      assert.deepEqual(values.dateEntries().map(([date, value]) => [calendarDate(date), value]), dated);
      assert.deepEqual([...values.toDateMap()].map(([date, value]) => [calendarDate(date), value]), dated);
    }
  }
  await client.series.price.spot.usd.by.day1.slice(
    new Date(2009, 0, 9), new Date(2009, 0, 11),
  ).fetch();
  assert.deepEqual(requests, [
    '/api/series/price/day1?end=3', '/api/series/price/height?end=3',
    '/api/series/price/day1?start=1&end=3',
  ]);
});

test('calendar conversions preserve genesis and period boundaries', () => {
  const client = new BitviewClient('http://fixture.invalid');
  for (const [index, number, expected] of [
    ['day1', 0, '2009-01-03'], ['day1', 1, '2009-01-09'],
    ['week1', 0, '2009-01-03'], ['week1', 1, '2009-01-10'],
    ['month1', 12, '2010-01-01'], ['month3', 1, '2009-04-01'],
    ['month6', 1, '2009-07-01'], ['year1', 10, '2019-01-01'],
    ['year10', 1, '2019-01-01'],
  ]) {
    const date = client.indexToDate(index, number);
    assert.equal(calendarDate(date), expected);
    assert.equal(client.dateToIndex(index, date), number);
  }
});
