import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const html = await readFile(new URL("../density.html", import.meta.url), "utf8");
const source = html.slice(html.indexOf('      const API ='), html.indexOf('      async function fetchSeries'));
const { signalFor, buildRows, histogramPoint, showCandles, densitySeriesNames, DAY_ZERO, DAY_SECONDS } = new Function(`${source}\nreturn {signalFor, buildRows, histogramPoint, showCandles, densitySeriesNames, DAY_ZERO, DAY_SECONDS};`)();

test("requires at least 25% total and a strict winning side", () => {
  assert.equal(signalFor(249999, 150000, 99999), 0);
  assert.equal(signalFor(250000, 150000, 100000), 1);
  assert.equal(signalFor(250000, 100000, 150000), -1);
  assert.equal(signalFor(400000, 200000, 200000), 0);
  assert.equal(signalFor(0, 0, 0), 0);
});

test("missing data cannot become a bull, bear, or neutral observation", () => {
  for (const missing of [null, undefined, NaN]) {
    assert.equal(signalFor(missing, 150000, 50000), null);
    assert.equal(signalFor(200000, missing, 50000), null);
    assert.equal(signalFor(200000, 50000, missing), null);
  }
});

test("joins each series by its own start index while retaining price history", () => {
  const candle = [90, 110, 80, 100];
  const rows = buildRows(
    { start: 10, data: [candle, candle, candle, null, [1, 2, 0, 1]] },
    [
      { start: 11, data: [300000, 300000] },
      { start: 10, data: [10000, 225000, 100000] },
      { start: 11, data: [75000, 200000] },
    ],
  );
  assert.deepEqual(rows.map(row => row.signal), [null, 1, -1]);
  assert.deepEqual(rows.map(row => row.time), [10, 11, 12].map(day => DAY_ZERO + day * DAY_SECONDS));
  assert.deepEqual(histogramPoint(rows[0]), { time: rows[0].time });
  assert.equal(histogramPoint(rows[1]).value, 0.75);
  assert.equal(histogramPoint(rows[2]).value, 2 / 3);
  assert.notEqual(histogramPoint(rows[1]).color, histogramPoint(rows[2]).color);
  assert.deepEqual(histogramPoint({ time: 1, signal: 0 }), { time: 1 });
});

test("switches to candles on zoom-in and avoids flicker near the threshold", () => {
  const range = bars => ({ from: 0, to: bars });
  assert.equal(showCandles(range(500), false), true);
  assert.equal(showCandles(range(501), false), false);
  assert.equal(showCandles(range(600), true), true);
  assert.equal(showCandles(range(601), true), false);
  assert.equal(showCandles(null, true), false);
});

test("a zero losing side is a fully dominant 100% share", () => {
  const rows = buildRows(
    { start: 0, data: [[90, 110, 80, 100]] },
    [300000, 300000, 0].map(value => ({ start: 0, data: [value] })),
  );
  assert.equal(rows[0].signal, 1);
  assert.equal(rows[0].strength, 1);
  assert.equal(histogramPoint(rows[0]).value, 1);
});

test("selects supply density series for both modes", () => {
  for (const mode of ["cointime", "coinflow"]) {
    assert.deepEqual(densitySeriesNames(mode), [
      `${mode}_urpd_all_supply_density_total_ppm`,
      `${mode}_urpd_all_supply_density_in_profit_ppm`,
      `${mode}_urpd_all_supply_density_in_loss_ppm`,
    ]);

  }
});
