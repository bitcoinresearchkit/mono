import { readFile, writeFile, rename } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

const DAY_MS = 86_400_000;
const DAY_ZERO = Date.UTC(2009, 0, 1);
const validPrice = value => Number.isSafeInteger(value) && value >= 0;
const dateAt = day => new Date(DAY_ZERO + day * DAY_MS).toISOString().slice(0, 10);
const validCandle = candle => Array.isArray(candle) && candle.length === 4 && candle.every(validPrice) &&
  candle[2] <= Math.min(candle[0], candle[3]) && candle[1] >= Math.max(candle[0], candle[3]);

function buildCloudSnapshot(histories, start, end, generatedAt, names, sourceCount = names.length - 1) {
  if (!Array.isArray(histories) || histories.length !== names.length) {
    throw new Error('Expected Bitcoin price and all source histories.');
  }
  for (const [index, history] of histories.entries()) {
    if (history.type !== (index === 0 ? 'OHLCCents' : 'Cents') || history.index !== 'day1' || history.start !== 0 ||
        history.end !== end || history.data?.length !== end) {
      throw new Error(`Incomplete or misaligned daily history: ${names[index]}`);
    }
  }
  const rows = histories[0].data.slice(start).map((candle, offset) => {
    const prices = histories.slice(1, 1 + sourceCount).map(history => history.data[start + offset]);
    if (!validCandle(candle) || ![...candle, ...prices].every(value => validPrice(value) && value > 0)) {
      throw new Error(`Invalid price on ${dateAt(start + offset)}; refusing a partial cloud.`);
    }
    return [...candle, Math.min(...prices), Math.max(...prices)];
  });
  return {
    generatedAt, start: dateAt(start), through: dateAt(end - 1), unit: 'cents',
    columns: ['open', 'high', 'low', 'close', 'min', 'max'], rows,
  };
}

async function generateCloudSnapshot({ api = 'http://localhost:3110/api', startDate = '2011-01-01',
  now = new Date(), output, seriesNames, buildSnapshot }) {
  const start = (Date.parse(startDate) - DAY_ZERO) / DAY_MS;
  // The API's end is exclusive; include today's in-progress observation.
  const end = (Date.parse(now.toISOString().slice(0, 10)) - DAY_ZERO) / DAY_MS + 1;
  if (!Number.isInteger(start) || start < 0 || start >= end) {
    throw new Error('--start must be a date between 2009-01-01 and today.');
  }
  // Start at genesis so stateful overlays can select before the display range.
  const query = new URLSearchParams({ series: seriesNames.join(','), index: 'day1', start: '0', end: String(end) });
  const response = await fetch(`${api.replace(/\/$/, '')}/series/bulk?${query}`, {
    signal: AbortSignal.timeout(60_000),
  });
  if (!response.ok) throw new Error(`Local API returned ${response.status}: ${await response.text()}`);
  const snapshot = buildSnapshot(await response.json(), start, end, now.toISOString());
  const html = await readFile(output, 'utf8');
  const dataTag = /(<script id="chart-data" type="application\/json">)[\s\S]*?(<\/script>)/g;
  if ([...html.matchAll(dataTag)].length !== 1) throw new Error('Expected one embedded chart snapshot.');
  const json = JSON.stringify(snapshot).replaceAll('<', '\\u003c');
  const temporary = new URL(output.href + '.tmp');
  await writeFile(temporary, html.replace(dataTag, (_, open, close) => open + json + close));
  await rename(temporary, output);
  return snapshot;
}


// First day with positive values for all 12 older-cohort sources on the log chart.
const DEFAULT_START = '2011-03-22';

export const sources = ['over_4m', 'lth', 'over_6m'].flatMap(cohort =>
  ['awake', 'coinflow'].flatMap(weight =>
    ['price', 'capitalized_price'].map(metric => `${cohort}_${weight}_${metric}_cents`),
  ),
);
export const seriesNames = ['price_ohlc_cents', ...sources];

export function buildSnapshot(histories, start, end, generatedAt) {
  return buildCloudSnapshot(histories, start, end, generatedAt, seriesNames);
}

export function generateSnapshot(options = {}) {
  return generateCloudSnapshot({
    startDate: DEFAULT_START,
    ...options,
    output: options.output ?? new URL('./index.html', import.meta.url),
    seriesNames,
    buildSnapshot,
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { values } = parseArgs({ options: {
    api: { type: 'string', default: 'http://localhost:3110/api' },
    start: { type: 'string', default: DEFAULT_START },
  } });
  const snapshot = await generateSnapshot({ api: values.api, startDate: values.start });
  const latest = snapshot.rows.at(-1);
  console.log(`Generated ${snapshot.rows.length.toLocaleString()} daily rows: ${snapshot.start} through ${snapshot.through}`);
  console.log(`Latest LTH Cloud: $${(latest[4] / 100).toFixed(2)} – $${(latest[5] / 100).toFixed(2)}`);
}
