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

test('URPD products belong to their weight owners and Bedrock exposes only its model', () => {
  const { series } = new BitviewClient('http://fixture.invalid');
  assert.deepEqual(Object.keys(series.bedrock).sort(), ['coinflow', 'cointime', 'raw']);
  assert.ok(!('frameworks' in series));
  assert.ok(!('horizon' in series.bedrock.coinflow));
  for (const aggregate of [series.coinflow, series.coinflow.sth, series.coinflow.lth]) {
    assert.ok(!('horizon' in aggregate));
    assert.equal(typeof aggregate.supply.mobile.inLoss.share.by.height.fetch, 'function');
  }
  assert.ok(series.cointime.ageRange.coindaysCreated.under1h);
  for (const owner of ['cointime', 'coinflow']) {
    const urpd = series[owner].urpd;
    for (const [key, cohort] of [
      ['all', 'all'], ['sth', 'sth'], ['lth', 'lth'],
      ['under4m', 'under_4m'], ['under6m', 'under_6m'],
      ['over4m', 'over_4m'], ['over6m', 'over_6m'],
    ]) {
      const prefix = cohort === 'all' ? '' : `${cohort}_`;
      const metrics = urpd[key];
      assert.equal(metrics.costBasis.perCoin.pct50.cents.by.height.path,
        `/api/series/${prefix}${owner}_cost_basis_per_coin_pct50_cents/height`);
      assert.equal(metrics.costBasis.perDollar.pct50.cents.by.height.path,
        `/api/series/${prefix}${owner}_cost_basis_per_dollar_pct50_cents/height`);
      assert.equal(metrics.capitalizedPrice.cents.by.height.path,
        `/api/series/${owner}_urpd_${cohort}_capitalized_price_cents/height`);
      assert.equal(metrics.supplyDensity.total.ppm.by.height.path,
        `/api/series/${owner}_urpd_${cohort}_supply_density_total_ppm/height`);
    }
  }
  assert.equal(series.bedrock.raw.supplyInLossThreshold.pct95.by.day1.path,
    '/api/series/bedrock_raw_supply_in_loss_threshold_pct95_ratio/day1');
  assert.equal(series.bedrock.coinflow.floor.pct95.cents.by.day1.path,
    '/api/series/bedrock_coinflow_floor_pct95_cents/day1');
  assert.equal(series.cohorts.urpd.ageBounds.under4m.min.cents.by.day1.path,
    '/api/series/utxos_urpd_under_4m_cost_basis_min_cents/day1');
});
