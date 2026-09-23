import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const html = await readFile(new URL('../studio.html', import.meta.url), 'utf8');
const source = html.slice(html.indexOf('      const storageKey ='), html.indexOf('      let saveState ='));
function fixture(values = {}) {
  const storage = new Map(Object.entries(values));
  const localStorage = { getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, value) };
  const api = new Function('localStorage', `${source}\nreturn { library, activeChart, newChartRecord, commitChartState, sortedCharts, persistLibrary };`)(localStorage);
  return { ...api, storage, reload: () => fixture(Object.fromEntries(storage)) };
}
const state = title => ({ version: 3, title, timeframe: 'day1', panes: [{ stretch: 2, series: [{ name: 'price_ohlc', chartType: 'bars', chartOptions: {}, color: '#f00' }] }] });

test('migrates the existing chart and preserves its complete layout on reload', () => {
  const legacy = state('Existing chart');
  const f = fixture({ 'bitview-studio-v1': JSON.stringify(legacy) });
  assert.equal(f.library.sort, 'viewed');
  assert.deepEqual(f.activeChart().state, legacy);
  f.persistLibrary();
  assert.deepEqual(f.reload().activeChart().state, legacy);
  assert.equal(f.storage.get('bitview-studio-v1'), JSON.stringify(legacy));
});

test('chart edits remain isolated and selection persists', () => {
  const f = fixture();
  const original = structuredClone(f.activeChart());
  const second = f.newChartRecord(state('Second'));
  f.library.charts.push(second);
  f.library.activeId = second.id;
  f.commitChartState(state('Renamed'));
  assert.deepEqual(f.library.charts[0], original);
  const restored = f.reload();
  assert.equal(restored.library.activeId, second.id);
  assert.equal(restored.activeChart().state.title, 'Renamed');
});

test('sorts names naturally and every date newest first without rearranging storage', () => {
  const f = fixture();
  f.library.charts = [
    { id: 'a', state: state('Chart 10'), created: 10, changed: 30, viewed: 20 },
    { id: 'b', state: state('Chart 2'), created: 30, changed: 20, viewed: 10 },
    { id: 'c', state: state('Alpha'), created: 20, changed: 10, viewed: 30 },
  ];
  for (const [sort, expected] of Object.entries({ name: ['c', 'b', 'a'], created: ['b', 'c', 'a'], changed: ['a', 'b', 'c'], viewed: ['c', 'a', 'b'] })) {
    f.library.sort = sort;
    assert.deepEqual(f.sortedCharts().map(entry => entry.id), expected);
  }
  assert.deepEqual(f.library.charts.map(entry => entry.id), ['a', 'b', 'c']);
});

test('loading or saving unchanged state does not update date changed', () => {
  const f = fixture();
  f.activeChart().changed = 1;
  f.commitChartState(structuredClone(f.activeChart().state));
  assert.equal(f.activeChart().changed, 1);
  f.commitChartState(state('Loaded'), false);
  assert.equal(f.activeChart().changed, 1);
  f.commitChartState(state('Edited'));
  assert.ok(f.activeChart().changed > 1);
});

test('restores valid records when active selection or sorting is missing', () => {
  const f = fixture({ 'bitview-studio-library-v1': JSON.stringify({ version: 1, activeId: 'missing', sort: 'invalid', charts: [{ id: 'good', state: state('Kept') }, { id: 'bad', state: {} }] }) });
  assert.equal(f.library.activeId, 'good');
  assert.equal(f.library.sort, 'viewed');
  assert.equal(f.library.charts.length, 1);
});

test('switching replaces native panes and restores each chart layout, including failed series', async () => {
  const start = html.indexOf('        async function loadChartState(state)');
  const code = html.slice(start, html.indexOf('        switchingChart = true;', start));
  const nativePanes = [{ stretch: 1, getStretchFactor() { return this.stretch; }, setStretchFactor(value) { this.stretch = value; } }];
  const handles = new Set();
  let visibleRange;
  const chart = {
    panes: () => nativePanes,
    addPane() { nativePanes.push({ ...nativePanes[0] }); },
    removePane(index) { nativePanes.splice(index, 1); },
    removeSeries(handle) { assert.ok(handles.delete(handle)); },
    timeScale: () => ({ fitContent() {}, setVisibleRange(range) { visibleRange = range; }, setVisibleLogicalRange(range) { visibleRange = range; } }),
  };
  const api = new Function('chart', 'handles', `
    const panes = [], timeframes = [{ index: 'day1' }, { index: 'height' }], maxPanes = 6;
    const timeframePicker = {}, chartTitle = {}, status = {}, element = { dataset: {} };
    const document = { getElementById: () => element };
    const picker = { open: false }, seriesConfig = { matches: () => false };
    const console = { warn() {} };
    let restoring, generation = 0, saveTimer, historyTimer, historyArmed, timeframe, restoreTimeRange, saved;
    function applyTimeframe() {}
    function scheduleHistory() {}
    function syncSeriesOrder() {}
    function updateVisibility() {}
    function renderLegend() {}
    function setupPane(index) {
      const pane = { index, entries: [], unloaded: [], legend: { remove() {} }, controls: { remove() {} } };
      panes.push(pane);
      return pane;
    }
    async function fetchSeries(name) {
      if (name === 'unavailable') throw new Error('Offline');
      return { start: 0 };
    }
    function addSeries(pane, name, data, options) {
      const api = {}; handles.add(api);
      pane.entries.push({ ...options, name, data, api });
    }
    function saveState(changed) {
      saved = { changed, title: chartTitle.value, timeframe, panes: panes.map(p => [...p.entries, ...p.unloaded].map(e => e.name)) };
    }
    ${code}
    return { loadChartState, panes, get saved() { return saved; }, timeframePicker, chartTitle };
  `)(chart, handles);
  const first = state('Two panels');
  first.panes.push({ series: [{ name: 'other' }, { name: 'unavailable', chartType: 'dots' }] });
  first.timeRange = { from: 100, to: 200 };
  await api.loadChartState(first);
  assert.equal(nativePanes.length, 2);
  assert.deepEqual(visibleRange, first.timeRange);
  assert.equal(api.panes[1].unloaded[0].chartType, 'dots');
  const second = state('Block chart');
  second.timeframe = 'height';
  await api.loadChartState(second);
  assert.equal(nativePanes.length, 1);
  assert.equal(handles.size, 1);
  assert.deepEqual(api.saved, { changed: false, title: 'Block chart', timeframe: 'height', panes: [['price_ohlc']] });
  assert.equal(api.timeframePicker.disabled, false);
  assert.equal(api.chartTitle.disabled, false);
  await api.loadChartState(first);
  assert.equal(nativePanes.length, 2);
  assert.equal(handles.size, 2);
  assert.deepEqual(api.saved.panes, [['price_ohlc'], ['other', 'unavailable']]);
});
