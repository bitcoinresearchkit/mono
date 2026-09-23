import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import * as LC from '../../modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs';

const html = await readFile(new URL('../studio.html', import.meta.url), 'utf8');
const theme = { 'chart-surface': html.match(/--chart-surface:\s*([^;]+);/)[1] };
function section(start, end) {
  const from = html.indexOf(start), to = html.indexOf(end, from);
  assert.ok(from >= 0 && to > from);
  return html.slice(from, to);
}
const code = [
  section('        saveState = (changed = true) => {', '        const lineSettings ='),
  section('        const lineSettings =', '        const seriesConfig ='),
  section('        function syncSeriesOrder(', '        function reorderSeries('),
  section('        function buildPoints(', '        function scheduleHistory('),
].join('\n');
const scalar = { type: 'Dollars', start: 0, end: 3, data: [10, null, 12], timestamps: { start: 0, data: [1700000000, 1700000060, 1700000120] } };
const ohlc = { ...scalar, type: 'OHLC<Dollars>', data: [[8, 12, 7, 10], null, [10, 13, 9, 12]] };

function fixture() {
  const panes = [{ index: 0, entries: [], unloaded: [] }];
  const handles = [];
  let range = { from: 0, to: 100 }, saved, failType;
  const chart = {
    addSeries(definition, options, pane) {
      if (definition.type === failType) throw new Error('Renderer failed');
      const handle = {
        pane, options: { ...definition.defaultOptions, ...options }, definition,
        seriesType: () => definition.type,
        applyOptions(next) {
          for (const key of Object.keys(next)) assert.ok(key === 'visible' || (key === 'pointMarkersRadius' && definition.type === 'Line') || key in definition.defaultOptions, `${definition.type}: unsupported ${key}`);
          Object.assign(this.options, next);
        },
        setData(points) {
          for (const point of points) {
            if (Object.keys(point).length === 1) continue;
            const isOHLC = ['Bar', 'Candlestick'].includes(definition.type);
            assert.equal('open' in point, isOHLC, `${definition.type}: wrong data shape`);
            assert.equal('value' in point, !isOHLC, `${definition.type}: wrong data shape`);
          }
          this.points = points;
        },
        setSeriesOrder(index) {
          handles.splice(handles.indexOf(this), 1);
          handles.splice(index, 0, this);
        },
      };
      handles.push(handle);
      return handle;
    },
    removeSeries(handle) {
      assert.ok(handles.includes(handle));
      handles.splice(handles.indexOf(handle), 1);
    },
    timeScale: () => ({ getVisibleLogicalRange: () => range, getVisibleRange: () => ({ from: 1700000000, to: 1700000120 }) }),
    panes: () => panes.map(() => ({ getStretchFactor: () => 1 })),
  };
  const localStorage = { setItem(key, text) { saved = JSON.parse(text); } };
  const api = new Function('LC', 'chart', 'panes', 'localStorage', 'theme', `
    const palette = { white: 'rgb(255, 255, 255)', blue: 'rgb(0, 100, 255)' };
    const priceSources = { price_ohlc: { day1: 'price_ohlc', height: 'price' } };
    const chartTitle = { value: 'Test chart' }, storageKey = 'test', status = {};
    const timeframePicker = { disabled: false };
    let saveState, saveTimer, restoring = false, restoreTimeRange = null, timeframe = 'day1';
    function commitChartState(state) { localStorage.setItem(storageKey, JSON.stringify(state)); }
    function scheduleHistory() {}
    function renderLegend() {}
    ${code}
    return { addSeries, setChartType, setSeriesData, updateVisibility, applySeriesStyle, setting, chartTypes, chartSettings, saveState };
  `)(LC, chart, panes, localStorage, theme);
  return { ...api, panes, handles, pane: panes[0], get saved() { return saved; }, setRange(value) { range = value; }, failRenderer(type) { failType = type; } };
}

test('every built-in renderer receives the correct data and supported options', () => {
  const f = fixture();
  f.addSeries(f.pane, 'price_ohlc', ohlc);
  const entry = f.pane.entries[0];
  for (const [type, expected] of [['line', 'Line'], ['dots', 'Line'], ['area', 'Area'], ['baseline', 'Baseline'], ['histogram', 'Histogram'], ['bars', 'Bar'], ['candles', 'Candlestick']]) {
    f.setChartType(f.pane, entry, type);
    const rendered = entry.ohlcApi || entry.api;
    assert.equal(rendered.seriesType(), expected);
    assert.equal(f.handles.length, entry.ohlcApi ? 2 : 1);
    assert.equal(rendered.points[1].time, scalar.timestamps.data[1]);
    assert.equal(Object.keys(rendered.points[1]).length, 1);
    assert.equal(rendered.options.visible, true);
  }
});

test('candles and bars switch to line when zoomed out and back when zoomed in', () => {
  const f = fixture();
  f.addSeries(f.pane, 'price_ohlc', ohlc, { chartType: 'auto' });
  const entry = f.pane.entries[0];
  assert.equal(entry.chartType, 'candles');
  assert.equal(f.chartTypes.auto, undefined);
  for (const type of ['candles', 'bars']) {
    f.setChartType(f.pane, entry, type);
    for (const count of [500, 501, 1000, 100]) {
      f.setRange({ from: 0, to: count }); f.updateVisibility(f.pane);
      assert.equal(entry.api.seriesType(), 'Line');
      assert.equal(entry.ohlcApi.seriesType(), type === 'bars' ? 'Bar' : 'Candlestick');
      assert.equal(entry.api.options.visible, count > 500);
      assert.equal(entry.ohlcApi.options.visible, count <= 500);
    }
    entry.visible = false; f.updateVisibility(f.pane);
    assert.equal(entry.api.options.visible, false);
    assert.equal(entry.ohlcApi.options.visible, false);
    entry.visible = true; f.updateVisibility(f.pane);
    assert.equal(f.handles.length, 2);
  }
});

test('OHLC renderers are restricted to price, including restored layouts', () => {
  const f = fixture();
  assert.deepEqual(f.chartTypes.bars.settings, []);
  assert.deepEqual(f.chartTypes.candles.settings, []);
  f.addSeries(f.pane, 'supply', scalar, { chartType: 'candles' });
  const entry = f.pane.entries[0];
  assert.equal(entry.chartType, 'line');
  for (const type of ['candles', 'bars', 'auto']) f.setChartType(f.pane, entry, type);
  assert.equal(entry.chartType, 'line');
  f.addSeries(f.pane, 'other_ohlc', ohlc);
  assert.equal(f.pane.entries[1].api.seriesType(), 'Line');
  assert.equal(f.pane.entries[1].api.points[0].value, 10);
});

test('price intervals without OHLC fall back to line and retain the chosen type', () => {
  const f = fixture();
  f.addSeries(f.pane, 'price_ohlc', scalar, { chartType: 'candles' });
  const entry = f.pane.entries[0];
  assert.equal(entry.chartType, 'candles');
  assert.equal(entry.api.seriesType(), 'Line');
  assert.equal(f.saved.panes[0].series[0].chartType, 'candles');
  const restored = fixture();
  restored.addSeries(restored.pane, entry.name, ohlc, f.saved.panes[0].series[0]);
  assert.equal(restored.pane.entries[0].ohlcApi.seriesType(), 'Candlestick');
});

test('type-specific settings, metadata and ordering survive switching and saving', () => {
  const f = fixture();
  f.addSeries(f.pane, 'price_ohlc', ohlc, { title: 'My price', color: 'blue', visible: false, chartOptions: { lineType: LC.LineType.WithSteps, lineStyle: LC.LineStyle.Dashed, lineWidth: 3, base: 7, hollow: false, wickVisible: false, borderVisible: false, openVisible: false, thinBars: false } });
  const entry = f.pane.entries[0];
  f.addSeries(f.pane, 'supply', scalar);
  for (const type of ['area', 'baseline', 'histogram', 'bars', 'candles', 'line']) {
    f.setChartType(f.pane, entry, type);
    assert.equal(f.pane.entries[0], entry);
    assert.equal(f.handles[0], entry.api);
    assert.equal(entry.api.options.visible, false);
    assert.equal(entry.title, 'My price');
    assert.equal(entry.color, 'blue');
    const rendered = entry.ohlcApi || entry.api;
    if (['area', 'baseline', 'line'].includes(type)) {
      assert.equal(entry.api.options.lineType, LC.LineType.WithSteps);
      assert.equal(entry.api.options.lineStyle, LC.LineStyle.Dashed);
      assert.equal(entry.api.options.lineWidth, 3);
    }
    if (type === 'baseline') assert.equal(entry.api.options.baseValue.price, 7);
    if (type === 'histogram') assert.equal(entry.api.options.base, 7);
    if (type === 'candles') {
      assert.equal(rendered.options.upColor, theme['chart-surface']);
      assert.equal(rendered.options.wickVisible, true);
      assert.equal(rendered.options.borderVisible, true);
    }
    if (type === 'bars') {
      assert.equal(rendered.options.openVisible, true);
      assert.equal(rendered.options.thinBars, true);
    }
  }
  const saved = f.saved.panes[0].series[0];
  const restored = fixture();
  restored.addSeries(restored.pane, saved.name, ohlc, saved);
  assert.deepEqual(restored.saved.panes[0].series[0], saved);
});

test('a failed switch preserves the previous renderer', () => {
  const f = fixture();
  f.addSeries(f.pane, 'supply', scalar);
  const entry = f.pane.entries[0], original = entry.api;
  f.failRenderer('Area');
  assert.throws(() => f.setChartType(f.pane, entry, 'area'), /Renderer failed/);
  assert.equal(entry.api, original);
  assert.equal(entry.chartType, 'line');
  assert.deepEqual(f.handles, [original]);
});

test('dots shows only point markers and line restores the connecting stroke', () => {
  const f = fixture();
  f.addSeries(f.pane, 'supply', scalar);
  const entry = f.pane.entries[0];
  assert.ok(!f.chartTypes.line.settings.includes('pointMarkersVisible'));
  f.setChartType(f.pane, entry, 'dots');
  assert.equal(entry.api.seriesType(), 'Line');
  assert.equal(entry.api.options.lineVisible, false);
  assert.equal(entry.api.options.pointMarkersVisible, true);
  assert.equal(f.saved.panes[0].series[0].chartType, 'dots');
  f.setChartType(f.pane, entry, 'line');
  assert.equal(entry.api.options.lineVisible, true);
  assert.equal(entry.api.options.pointMarkersVisible, false);
});

test('dot AUTO follows website zoom sizing; fixed sizes persist across zoom and reload', () => {
  const f = fixture();
  f.addSeries(f.pane, 'supply', scalar, { chartType: 'dots' });
  const entry = f.pane.entries[0];
  assert.equal(f.setting(entry, 'dotSize'), 'auto');
  for (const [count, radius] of [[1001, 1], [1000, 1.5], [201, 1.5], [200, 2], [101, 2], [100, 3]]) {
    f.setRange({ from: 0, to: count }); f.updateVisibility(f.pane);
    assert.equal(entry.api.options.pointMarkersRadius, radius, String(count));
  }
  entry.chartOptions.dotSize = 4;
  f.applySeriesStyle(entry);
  for (const count of [50, 2000]) {
    f.setRange({ from: 0, to: count }); f.updateVisibility(f.pane);
    assert.equal(entry.api.options.pointMarkersRadius, 4);
  }
  f.saveState();
  const restored = fixture();
  restored.addSeries(restored.pane, entry.name, scalar, f.saved.panes[0].series[0]);
  assert.equal(restored.pane.entries[0].api.options.pointMarkersRadius, 4);
  f.setChartType(f.pane, entry, 'line');
  f.setChartType(f.pane, entry, 'dots');
  assert.equal(entry.api.options.pointMarkersRadius, 4);
  entry.chartOptions.dotSize = 'auto';
  f.applySeriesStyle(entry);
  assert.equal(entry.api.options.pointMarkersRadius, 1);
});

test('candles and bars are adjacent; histogram supports price and non-price data', () => {
  const f = fixture();
  assert.deepEqual(Object.keys(f.chartTypes).slice(0, 2), ['candles', 'bars']);
  assert.equal(f.chartTypes.histogram.priceOnly, undefined);
  for (const [name, data] of [['price_ohlc', ohlc], ['supply', scalar], ['other_ohlc', ohlc]]) {
    f.addSeries(f.pane, name, data, { chartType: 'histogram' });
    const entry = f.pane.entries.at(-1);
    assert.equal(entry.api.seriesType(), 'Histogram');
    assert.equal(entry.api.options.visible, true);
  }
});
