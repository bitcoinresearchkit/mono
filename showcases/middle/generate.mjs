import { readFile, writeFile, rename } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import { BitviewClient } from '../../modules/bitview-client/index.js';

const DAY_MS = 86_400_000;
const DAY_ZERO = Date.UTC(2009, 0, 1);

// One series per request: bulk reads are capped server-side.
// buildSnapshot rejects any history that does not end at `end`.
function fetchHistories(api, names, end) {
  const client = new BitviewClient({ baseUrl: api.replace(/\/api\/?$/, ''), timeout: 60_000 });
  return Promise.all(names.map(name => client.seriesEndpoint(name, 'day1').slice(0, end).fetch({ cache: false })));
}

export const seriesNames = ['price_ohlc_cents', 'awake_capitalized_price_cents', 'true_market_mean_cents'];

export function buildSnapshot(histories, start, end, generatedAt) {
  if (!Array.isArray(histories) || histories.length !== seriesNames.length) throw new Error('Expected three histories.');
  for (const [index, history] of histories.entries()) {
    if (history.type !== (index === 0 ? 'OHLCCents' : 'Cents') || history.index !== 'day1' ||
        history.start !== 0 || history.end !== end || history.data?.length !== end) {
      throw new Error(`Incomplete or misaligned history: ${seriesNames[index]}`);
    }
  }
  const firstPrice = histories[0].data.findIndex(candle => Array.isArray(candle) && candle[3] > 0);
  if (firstPrice < 0) throw new Error('No price history available.');
  start = Math.max(start, firstPrice);
  const rows = histories[0].data.slice(start).map((candle, index) => {
    const cp = histories[1].data[start + index], tmm = histories[2].data[start + index];
    if (!Array.isArray(candle) || candle.length !== 4 ||
        !candle.every(value => Number.isSafeInteger(value) && value > 0) ||
        ![cp, tmm].every(value => Number.isSafeInteger(value) && value >= 0) ||
        candle[2] > Math.min(candle[0], candle[3]) || candle[1] < Math.max(candle[0], candle[3])) {
      throw new Error(`Invalid observation at day ${start + index}.`);
    }
    return [...candle, cp, tmm];
  });
  const dateAt = index => new Date(DAY_ZERO + index * DAY_MS).toISOString().slice(0, 10);
  return { generatedAt, start: dateAt(start), through: dateAt(end - 1), unit: 'cents',
    columns: ['open', 'high', 'low', 'close', 'cp', 'tmm'], rows };
}

export async function generateSnapshot({ api = 'http://localhost:3110/api', startDate = '2009-01-01',
  now = new Date(), output = new URL('./index.html', import.meta.url) } = {}) {
  const start = (Date.parse(startDate) - DAY_ZERO) / DAY_MS;
  const end = Math.floor((now.getTime() - DAY_ZERO) / DAY_MS); // Exclusive: completed UTC days only.
  if (!Number.isInteger(start) || start < 0 || start >= end ||
      new Date(DAY_ZERO + start * DAY_MS).toISOString().slice(0, 10) !== startDate) throw new Error('Invalid start date.');
  const snapshot = buildSnapshot(await fetchHistories(api, seriesNames, end), start, end, now.toISOString());
  const html = await readFile(output, 'utf8');
  const tag = /(<script id="chart-data" type="application\/json">)[\s\S]*?(<\/script>)/g;
  if ([...html.matchAll(tag)].length !== 1) throw new Error('Expected one embedded snapshot.');
  const json = JSON.stringify(snapshot).replaceAll('<', '\\u003c');
  const temporary = new URL(output.href + '.tmp');
  await writeFile(temporary, html.replace(tag, (_, open, close) => open + json + close));
  await rename(temporary, output);
  return snapshot;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { values } = parseArgs({ options: { api: { type: 'string' }, start: { type: 'string' } } });
  const snapshot = await generateSnapshot({ api: values.api, startDate: values.start });
  console.log(`Generated ${snapshot.rows.length} completed daily observations through ${snapshot.through}.`);
}
