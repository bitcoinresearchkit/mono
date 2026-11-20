// Run with Bun: bun generate-prices.js CACHE_DIR [OUTPUT]
// Reads the tape beside this script (prices.txt) and writes OUTPUT (default: that file), replacing
// two spans with the last trade at or before each block's monotonic time, the running maximum of
// header times: the Mt. Gox trade for each height from FIRST_PRICE (74,641, the tape's first priced
// block; the archive's first trade, 2010-07-17, is about 6,000 blocks earlier) through 2011, and
// the Bitfinex trade for each height from 337,525 (after Bitstamp's halt) up to the oracle's seed
// hour. Every other row stays as the tape has it: the unpriced prefix, heights 160,037 to 337,524
// and the seed hour through the seed (339,999). The replaced spans come only from the pinned
// INPUTS, so a second run changes nothing.

import { mkdir, rename } from "node:fs/promises";
import { join } from "node:path";

/** Pinned inputs. The digest is of the bytes cached as `file`; CACHE_DIR keeps them between runs. */
const INPUTS = {
  gox: {
    // Bitcoincharts' Mt. Gox BTC/USD trades, as the Internet Archive captured them on 2014-03-30.
    file: "mtgox.csv.gz",
    url: "https://web.archive.org/web/20140330235726id_/http://api.bitcoincharts.com/v1/csv/mtgoxUSD.csv.gz",
    sha256: "2ee859b2508ac28e361f31eca508c345cdee20dd67a4aca05e39172132af85fd",
  },
  bitfinex: {
    // Bitfinex BTC/USD public trades, 2015-01-04 through 2015-01-22 UTC, as JSON.stringify of the
    // API rows ordered by millisecond, then trade ID.
    file: "bitfinex.json",
    url: "https://api-pub.bitfinex.com/v2/trades/tBTCUSD/hist",
    sha256: "2caceb4cb2d1feff76474ddea660fa7bd423e7c59ccc609529e435fae51b18b0",
  },
  timestamps: {
    // Bitview's mainnet timestamp_monotonic, heights 0 through 339,999, as JSON.stringify of the
    // concatenated values.
    file: "timestamps.json",
    url: "https://bitview.space/api/series/timestamp_monotonic/height/data",
    sha256: "fac66cac44c45e6334a98edfbfc297399791a4618a9bbe6cac56f3bf0b3481d1",
  },
};
const HEIGHTS = 340_000;
const FIRST_PRICE = 74_641;
const BITFINEX_START = 337_525;
const SEED = HEIGHTS - 1;
const END_OF_2011 = Date.parse("2012-01-01T00:00:00Z") / 1000;
const HOUR = 3600;
const TAPE = join(import.meta.dir, "prices.txt");

/** @typedef {[time: number, price: number]} Trade */
/** @typedef {[id: number, millis: number, amount: number, price: number]} BitfinexTrade */

/** @param {string} path @param {ArrayBuffer | string} bytes */
async function writeAtomic(path, bytes) {
  const temporary = `${path}.${process.pid}.tmp`;
  await Bun.write(temporary, bytes);
  await rename(temporary, path);
}

/** @param {string} url */
async function request(url) {
  const response = await fetch(url, {
    headers: { "User-Agent": "Bitview historical price tape (bitcoinresearchkit/mono)" },
    signal: AbortSignal.timeout(180_000),
  });
  if (!response.ok) throw new Error(`HTTP ${response.status} from ${url}`);
  return response;
}

/**
 * @param {{file: string, sha256: string}} spec
 * @param {() => Promise<ArrayBuffer | string>} download
 */
async function input({ file: name, sha256 }, download) {
  const path = join(cache, name);
  const file = Bun.file(path);
  const cached = await file.exists();
  const bytes = cached ? await file.arrayBuffer() : await download();
  const actual = Bun.CryptoHasher.hash("sha256", bytes, "hex");
  if (actual !== sha256) {
    throw new Error(`${path}: SHA-256 ${actual}, expected ${sha256}`);
  }
  if (!cached) await writeAtomic(path, bytes);
  console.log(`${name} sha256 ${actual}`);
  return Bun.file(path);
}

async function downloadBitfinex() {
  let start = Date.parse("2015-01-04T00:00:00Z");
  const end = Date.parse("2015-01-23T00:00:00Z") - 1;
  const limit = 10_000;
  /** @type {Map<number, BitfinexTrade>} */
  const trades = new Map();
  while (start <= end) {
    /** @type {BitfinexTrade[]} */
    const page = await (await request(`${INPUTS.bitfinex.url}?start=${start}&end=${end}&limit=${limit}&sort=1`)).json();
    for (const trade of page) trades.set(trade[0], trade);
    if (page.length < limit) break;
    // Include the last millisecond again so a page boundary cannot cut a tie.
    const next = page.at(-1)[1];
    if (next <= start) throw new Error(`Bitfinex pagination stalled at ${start}`);
    start = next;
    await Bun.sleep(4000);
  }
  return JSON.stringify([...trades.values()].sort((a, b) => a[1] - b[1] || a[0] - b[0]));
}

async function downloadTimestamps() {
  /** @type {number[]} */
  const timestamps = [];
  const step = 30_000;
  for (let start = 0; start < HEIGHTS; start += step) {
    if (start) await Bun.sleep(400);
    const end = Math.min(start + step, HEIGHTS);
    const page = await (await request(`${INPUTS.timestamps.url}?start=${start}&end=${end}`)).json();
    if (page.length !== end - start) throw new Error(`Bitview timestamp response for heights ${start}..${end - 1} has ${page.length} rows`);
    timestamps.push(...page);
  }
  return JSON.stringify(timestamps);
}

/** @param {ReturnType<typeof Bun.file>} file */
async function readGox(file) {
  const chunks = file.stream().pipeThrough(new DecompressionStream("gzip")).pipeThrough(new TextDecoderStream());
  /** @type {Trade[]} */
  const trades = [];
  let tail = "";
  for await (const chunk of chunks) {
    const lines = (tail + chunk).split("\n");
    tail = lines.pop();
    for (const line of lines) {
      const [time, price, volume] = line.split(",").map(Number);
      if (time >= END_OF_2011) return trades;
      if (!Number.isInteger(time) || time <= 0 || !(price > 0) || !(volume >= 0)) {
        throw new Error(`Invalid Mt. Gox trade: ${line}`);
      }
      if (volume > 0) trades.push([time, price]);
    }
  }
  throw new Error("Mt. Gox archive ends before 2012");
}

/** @param {number} price */
function cents(price) {
  return Math.round(price * 100);
}

/** @param {number} value */
function decimal(value) {
  const dollars = (value / 100).toFixed(2);
  return dollars.endsWith("0") ? dollars.slice(0, -1) : dollars;
}

/**
 * Prices heights `start` to `end - 1` with the last trade at or before each block's monotonic time.
 * @param {string} era
 * @param {Trade[]} trades
 * @param {number} start
 * @param {number} end
 */
function replay(era, trades, start, end) {
  let last = 0;
  for (let h = start; h < end; h++) {
    while (last < trades.length && trades[last][0] <= timestamps[h]) {
      if (last && trades[last][0] < trades[last - 1][0]) throw new Error(`${era}: unordered trade at ${trades[last][0]}`);
      last++;
    }
    if (!last) throw new Error(`${era}: no prior trade at height ${h}`);
    output[h] = decimal(cents(trades[last - 1][1]));
  }
  console.log(`${era}: heights ${start}..${end - 1}`);
}

const [, , cache, destination = TAPE] = Bun.argv;
if (!cache || Bun.argv.length > 4) throw new Error("usage: bun generate-prices.js CACHE_DIR [OUTPUT]");
await mkdir(cache, { recursive: true });
const [goxFile, finexFile, timestampFile] = await Promise.all([
  input(INPUTS.gox, async () => (await request(INPUTS.gox.url)).arrayBuffer()),
  input(INPUTS.bitfinex, downloadBitfinex),
  input(INPUTS.timestamps, downloadTimestamps),
]);
/** @type {number[]} */
const timestamps = await timestampFile.json();
if (timestamps.length !== HEIGHTS || timestamps.some((time, i) => !Number.isInteger(time) || time < timestamps[i - 1])) {
  throw new Error("Expected 340,000 monotonic Bitview timestamps");
}
const output = (await Bun.file(TAPE).text()).trimEnd().split("\n");
if (output.length !== HEIGHTS) throw new Error(`${TAPE}: expected ${HEIGHTS} prices, found ${output.length}`);
const endGox = timestamps.findIndex(time => time >= END_OF_2011);
/** @type {BitfinexTrade[]} */
const bitfinex = await finexFile.json();
const gox = await readGox(goxFile);
console.log(`trades: Mt. Gox ${gox.length}, Bitfinex ${bitfinex.length}`);
replay("Mt. Gox", gox, FIRST_PRICE, endGox);
// Keep the final hour leading up to the calibrated oracle seed as it was published.
const handoff = timestamps.findIndex(time => time >= timestamps[SEED] - HOUR);
replay("Bitfinex", bitfinex.map(([, millis, , price]) => [millis / 1000, price]), BITFINEX_START, handoff);
console.log(`oracle handoff preserved: heights ${handoff}..${SEED}`);

const text = output.join("\n") + "\n";
await writeAtomic(destination, text);
console.log(`sha256 ${Bun.CryptoHasher.hash("sha256", text, "hex")}  ${destination}`);
