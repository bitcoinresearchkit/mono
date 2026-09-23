import { readFile, writeFile, rename } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

const DAY_MS = 86_400_000;
const DAY_ZERO = Date.UTC(2009, 0, 1);
const dateAt = day => new Date(DAY_ZERO + day * DAY_MS).toISOString().slice(0, 10);
export const THRESHOLD = 500_000;
export const seriesNames = [
  'price_ohlc_cents',
  'cointime_urpd_under_6m_supply_density_total_ppm',
  'cointime_urpd_under_6m_supply_density_in_profit_ppm',
  'cointime_urpd_under_6m_supply_density_in_loss_ppm',
];

export function densityBalance(total, profit, loss) {
  if (![total, profit, loss].every(value => Number.isSafeInteger(value) && value >= 0 && value <= 1_000_000) ||
      Math.abs(total - profit - loss) > 1) {
    throw new Error('Invalid or inconsistent density values.');
  }
  return total >= THRESHOLD ? profit - loss : null;
}

export function buildSnapshot(histories, start, end, generatedAt) {
  if (!Array.isArray(histories) || histories.length !== seriesNames.length) {
    throw new Error('Expected price and all three density histories.');
  }
  for (const [index, history] of histories.entries()) {
    if (history.type !== (index === 0 ? 'OHLCCents' : 'PartsPerMillion32') || history.index !== 'day1' ||
        history.start !== 0 || history.end !== end || history.data?.length !== end) {
      throw new Error(`Incomplete or misaligned history: ${seriesNames[index]}`);
    }
  }
  if (histories.some(history => !history.stamp || history.stamp !== histories[0].stamp)) {
    throw new Error('Histories come from different snapshots; retry generation.');
  }
  const rows = histories[0].data.slice(start).map((candle, offset) => {
    if (!Array.isArray(candle) || candle.length !== 4 ||
        !candle.every(value => Number.isSafeInteger(value) && value > 0) ||
        candle[2] > Math.min(candle[0], candle[3]) || candle[1] < Math.max(candle[0], candle[3])) {
      throw new Error(`Invalid price on ${dateAt(start + offset)}.`);
    }
    const [total, profit, loss] = histories.slice(1).map(history => history.data[start + offset]);
    const balance = densityBalance(total, profit, loss);
    return [...candle, balance === null ? null : balance / 10_000, Math.max(0, total - THRESHOLD) / 10_000];
  });
  return {
    generatedAt, start: dateAt(start), through: dateAt(end - 1),
    priceUnit: 'cents',
    columns: ['open', 'high', 'low', 'close', 'signal', 'level'], rows,
  };
}

export async function generateSnapshot({ api = 'http://localhost:3110/api', startDate = '2011-01-01',
  now = new Date(), output = new URL('./index.html', import.meta.url) } = {}) {
  const start = (Date.parse(startDate) - DAY_ZERO) / DAY_MS;
  const end = Math.floor((now.getTime() - DAY_ZERO) / DAY_MS) + 1;
  if (!Number.isInteger(start) || start < 0 || start >= end || dateAt(start) !== startDate) {
    throw new Error('--start must be a date between 2009-01-01 and today.');
  }
  const query = new URLSearchParams({ series: seriesNames.join(','), index: 'day1', start: '0', end: String(end) });
  const response = await fetch(`${api.replace(/\/$/, '')}/series/bulk?${query}`, { signal: AbortSignal.timeout(60_000) });
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

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { values } = parseArgs({ options: {
    api: { type: 'string', default: 'http://localhost:3110/api' },
    start: { type: 'string', default: '2011-01-01' },
  } });
  const snapshot = await generateSnapshot({ api: values.api, startDate: values.start });
  const active = snapshot.rows.filter(row => row[4] !== null).length;
  console.log(`Generated ${snapshot.rows.length} days through ${snapshot.through}; ${active} qualifying observations.`);
}
