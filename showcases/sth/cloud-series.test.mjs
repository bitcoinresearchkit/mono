import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { customSeriesDefaultOptions } from '../modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs';

const html = readFileSync(new URL('./index.html', import.meta.url), 'utf8');
const rendererSource = html.slice(html.indexOf('class CloudRenderer'), html.indexOf('const DAY_SECONDS'));
const CloudSeries = new Function('customSeriesDefaultOptions', rendererSource + '\nreturn CloudSeries;')(customSeriesDefaultOptions);
const boundsLineData = new Function('customSeriesDefaultOptions', rendererSource + '\nreturn boundsLineData;')(customSeriesDefaultOptions);
const boundsEndpoints = new Function('customSeriesDefaultOptions', rendererSource + '\nreturn boundsEndpoints;')(customSeriesDefaultOptions);

test('dots mark both sides of switches and gaps, with one dot for a singleton segment', () => {
  const data = boundsLineData([[8000, 0], [8100, 0], [8200, 0], [12000, 1], [12100, 1], null, [8500, 0]], 100, 10);
  assert.deepEqual(boundsEndpoints(data), [
    { time: 100, value: 80 }, { time: 120, value: 82 },
    { time: 130, value: 120 }, { time: 140, value: 121 },
    { time: 160, value: 85 },
  ]);
  assert.deepEqual(boundsEndpoints(boundsLineData([null, [0, 0], null], 100, 10)), []);
});

test('bound switches and missing observations break the line without losing either endpoint', () => {
  assert.deepEqual(boundsLineData([[8000, 0], [8100, 0], [12000, 1], [12100, 1], null, [8500, 0]], 100, 10), [
    { time: 100, value: 80 },
    { time: 110, value: 81, color: 'transparent' },
    { time: 120, value: 120 },
    { time: 130, value: 121, color: 'transparent' },
    { time: 140 },
    { time: 150, value: 85 },
  ]);
  assert.deepEqual(boundsLineData([[0, 0], [100, 0]], 100, 10), [{ time: 100 }, { time: 110, value: 1 }]);
});

test('range changes never draw stale coordinates from offscreen bars', () => {
  const series = new CloudSeries();
  const bars = Array.from({ length: 5 }, (_, i) => ({
    x: i * 100,
    originalData: { lower: 100 + i, upper: 110 + i },
  }));
  let path;
  let filled;
  const context = {
    beginPath() { path = []; },
    moveTo(x, y) { path.push([x, y]); },
    lineTo(x, y) { path.push([x, y]); },
    closePath() {},
    fill() { filled = path; },
    stroke() {},
  };
  const target = {
    useBitmapCoordinateSpace(draw) {
      draw({ context, horizontalPixelRatio: 2, verticalPixelRatio: 2 });
    },
  };
  const draw = visibleRange => {
    series.update({ bars, visibleRange }, series.defaultOptions());
    series.renderer().draw(target, price => 200 - price);
  };

  draw({ from: 0, to: 5 });
  // The chart refreshes only visible coordinates when zooming or panning.
  for (let i = 1; i < 4; i++) bars[i].x = (i - 1) * 10;
  draw({ from: 1, to: 4 });

  assert.equal(filled.length, 6);
  assert.ok(filled.every(([x, y]) => x >= 0 && x <= 40 && Number.isFinite(y)));
  assert.deepEqual(filled.map(([x]) => x), [0, 20, 40, 40, 20, 0]);
});
