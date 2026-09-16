import { readFile, writeFile, rename } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

const DAY_MS = 86_400_000;
const DAY_ZERO = Date.UTC(2009, 0, 1);
const validPrice = value => Number.isSafeInteger(value) && value >= 0;
const dateAt = day => new Date(DAY_ZERO + day * DAY_MS).toISOString().slice(0, 10);
const validCandle = candle => Array.isArray(candle) && candle.length === 4 && candle.every(validPrice) &&
  candle[2] <= Math.min(candle[0], candle[3]) && candle[1] >= Math.max(candle[0], candle[3]);

async function generateCloudSnapshot({ api = 'http://localhost:3110/api', startDate = '2011-03-22',
  now = new Date(), output, seriesNames, buildSnapshot }) {
  const start = (Date.parse(startDate) - DAY_ZERO) / DAY_MS;
  // The API's end is exclusive; include today's in-progress observation.
  const end = (Date.parse(now.toISOString().slice(0, 10)) - DAY_ZERO) / DAY_MS + 1;
  if (!Number.isInteger(start) || start < 0 || start >= end) {
    throw new Error('--start must be a date between 2009-01-01 and today.');
  }
  // Start at genesis so stateful overlays can select before the display range.
  // The API allows 32 series per request; keep the two batches at the same tip.
  const batches = [];
  for (let offset = 0; offset < seriesNames.length; offset += 32) {
    batches.push(seriesNames.slice(offset, offset + 32));
  }
  const histories = (await Promise.all(batches.map(async names => {
    const query = new URLSearchParams({ series: names.join(','), index: 'day1', start: '0', end: String(end) });
    const response = await fetch(`${api.replace(/\/$/, '')}/series/bulk?${query}`, {
      signal: AbortSignal.timeout(60_000),
    });
    if (!response.ok) throw new Error(`Local API returned ${response.status}: ${await response.text()}`);
    return response.json();
  }))).flat();
  if (new Set(histories.map(history => history.stamp)).size !== 1) {
    throw new Error('The backend changed during the refresh; rerun to capture a consistent snapshot.');
  }
  const snapshot = buildSnapshot(histories, start, end, now.toISOString());
  const html = await readFile(output, 'utf8');
  const dataTag = /(<script id="chart-data" type="application\/json">)[\s\S]*?(<\/script>)/g;
  if ([...html.matchAll(dataTag)].length !== 1) throw new Error('Expected one embedded chart snapshot.');
  const json = JSON.stringify(snapshot).replaceAll('<', '\\u003c');
  const temporary = new URL(output.href + '.tmp');
  await writeFile(temporary, html.replace(dataTag, (_, open, close) => open + json + close));
  await rename(temporary, output);
  return snapshot;
}
const weightedSources = cohorts => cohorts.flatMap(cohort =>
  ['awake', 'coinflow'].flatMap(weight =>
    ['price', 'capitalized_price'].map(metric => `${cohort}${weight}_${metric}_cents`),
  ),
);
export const cloudSources = {
  sth: weightedSources(['under_4m_', 'sth_', 'under_6m_']),
  holders: weightedSources(['']),
  lth: weightedSources(['over_4m_', 'lth_', 'over_6m_']),
};
export const sources = Object.values(cloudSources).flat();
const boundsSources = [4, 5, 6].map(months => ({
  name: `<${months}M`,
  series: ['min', 'max'].map(side => `bedrock_under_${months}m_cost_basis_${side}_cents`),
}));
export const seriesNames = ['price_ohlc_cents', ...sources, ...boundsSources.flatMap(source => source.series)];

// A point is [price in cents, displayed side: 0=min / 1=max]. An expanding
// extreme selects the opposite bound; an aging-out contraction does not.
export function trendBounds(minima, maxima) {
  let side = null;
  let previous = null;
  return minima.map((min, index) => {
    const pair = [min, maxima[index]];
    if (!pair.every(validPrice) || pair[0] > pair[1]) {
      side = null;
      previous = null;
      return null;
    }
    if (previous) {
      const lowerMin = pair[0] < previous[0];
      const higherMax = pair[1] > previous[1];
      // Both expanding on the same day leaves their ordering unknown.
      if (lowerMin !== higherMax) side = lowerMin ? 1 : 0;
    }
    previous = pair;
    return side === null ? null : [pair[side], side];
  });
}

export function buildSnapshot(histories, start, end, generatedAt) {
  if (!Array.isArray(histories) || histories.length !== seriesNames.length) {
    throw new Error('Expected Bitcoin price, all three clouds, and trend bounds.');
  }
  for (const [index, history] of histories.entries()) {
    if (history.type !== (index === 0 ? 'OHLCCents' : 'Cents') || history.index !== 'day1' ||
        history.start !== 0 || history.end !== end || history.data?.length !== end) {
      throw new Error(`Incomplete or misaligned daily history: ${seriesNames[index]}`);
    }
  }
  const rows = histories[0].data.slice(start).map((candle, offset) => {
    if (!validCandle(candle) || !candle.every(value => value > 0)) {
      throw new Error(`Invalid Bitcoin price on ${dateAt(start + offset)}.`);
    }
    return candle;
  });
  let offset = 1;
  const clouds = Object.entries(cloudSources).map(([id, names]) => {
    const inputs = histories.slice(offset, offset + names.length);
    offset += names.length;
    const ranges = rows.map((_, index) => {
      const prices = inputs.map(history => history.data[start + index]);
      if (!prices.every(value => validPrice(value) && value > 0)) {
        throw new Error(`Invalid ${id} price on ${dateAt(start + index)}; refusing a partial cloud.`);
      }
      return [Math.min(...prices), Math.max(...prices)];
    });
    return { id, ranges };
  });
  const snapshot = {
    generatedAt, start: dateAt(start), through: dateAt(end - 1), unit: 'cents',
    columns: ['open', 'high', 'low', 'close'], rows, clouds,
  };
  const bounds = boundsSources.map(({ name }, index) => {
    const [minima, maxima] = histories.slice(1 + sources.length + index * 2, 3 + sources.length + index * 2)
      .map(history => history.data);
    for (let day = 0; day < end; day++) {
      const pair = [minima[day], maxima[day]];
      if (pair.every(value => value === null)) continue;
      if (!pair.every(validPrice) || pair[0] > pair[1]) {
        throw new Error(`Invalid ${name} bounds on ${dateAt(day)}.`);
      }
    }
    if (!validPrice(minima[end - 1]) || !validPrice(maxima[end - 1])) {
      throw new Error(`Missing latest ${name} bounds; run the updated local indexer first.`);
    }
    return { name, points: trendBounds(minima, maxima).slice(start) };
  });
  return { ...snapshot, bounds };
}

export function generateSnapshot(options = {}) {
  return generateCloudSnapshot({
    ...options,
    output: options.output ?? new URL('./index.html', import.meta.url),
    seriesNames,
    buildSnapshot,
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { values } = parseArgs({ options: {
    api: { type: 'string', default: 'http://localhost:3110/api' },
    start: { type: 'string', default: '2011-03-22' },
  } });
  const snapshot = await generateSnapshot({ api: values.api, startDate: values.start });
  console.log(`Generated ${snapshot.rows.length.toLocaleString()} daily rows including today's partial data: ${snapshot.start} through ${snapshot.through}`);
  for (const { id, ranges } of snapshot.clouds) {
    const [lower, upper] = ranges.at(-1);
    console.log(`${id}: $${(lower / 100).toFixed(2)} – $${(upper / 100).toFixed(2)}`);
  }
  console.log(`Trend bounds: ${snapshot.bounds.map(({ name, points }) => `${name} ${points.at(-1)?.[1] === 0 ? 'min' : points.at(-1)?.[1] === 1 ? 'max' : 'untouched'}`).join(', ')}`);
}
