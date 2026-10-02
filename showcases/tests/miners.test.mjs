import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const html = await readFile(new URL("../calendar/index.html", import.meta.url), "utf8");
const script = html.match(/<script id="calendar-data">([\s\S]*?)<\/script>/);
assert.ok(script, "Missing inline calendar-data script");

const { addDays, addPoolOrdinals, decodeBlocks, groupDays, isoDay, rankPools, requestRanges } = new Function(
  `${script[1]}; return { addDays, addPoolOrdinals, decodeBlocks, groupDays, isoDay, rankPools, requestRanges };`,
)();


const poolBlockCounts = [
  { index: "height", start: 100, end: 105, data: [1, 1, 2, 2, 2] },
  { index: "height", start: 100, end: 105, data: [0, 1, 1, 1, 2] },
  { index: "height", start: 100, end: 105, data: [0, 0, 0, 1, 1] },
];
const attributedBlocks = [
  { height: 100, pool: "antpool", day: "2009-01-03" },
  { height: 101, pool: "foundryusa", day: "2009-01-03" },
  { height: 102, pool: "antpool", day: "2009-01-03" },
  { height: 103, pool: "unknown", day: "2009-01-04" },
  { height: 104, pool: "foundryusa", day: "2009-01-04" },
];
const attributedPools = ["antpool", "foundryusa", "unknown"];

function copyPoolBlockCounts() {
  return poolBlockCounts.map(series => ({ ...series, data: [...series.data] }));
}

test("adds immutable per-pool daily ordinals and cumulative totals", () => {
  const original = attributedBlocks.map(block => ({ ...block }));
  const result = addPoolOrdinals(attributedBlocks, attributedPools, copyPoolBlockCounts());

  assert.deepEqual(result, [
    { height: 100, pool: "antpool", day: "2009-01-03", poolDayIndex: 1, poolTotal: 1 },
    { height: 101, pool: "foundryusa", day: "2009-01-03", poolDayIndex: 1, poolTotal: 1 },
    { height: 102, pool: "antpool", day: "2009-01-03", poolDayIndex: 2, poolTotal: 2 },
    { height: 103, pool: "unknown", day: "2009-01-04", poolDayIndex: 1, poolTotal: 1 },
    { height: 104, pool: "foundryusa", day: "2009-01-04", poolDayIndex: 1, poolTotal: 2 },
  ]);
  assert.notEqual(result, attributedBlocks);
  result.forEach((block, index) => assert.notEqual(block, attributedBlocks[index]));
  assert.deepEqual(attributedBlocks, original);
});

test("returns no work for an empty block range", () => {
  assert.deepEqual(addPoolOrdinals([], [], []), []);
});

test("requires exact height coverage, aligned series, and every represented pool", () => {
  const invalid = [
    copyPoolBlockCounts().map(series => ({ ...series, start: series.start - 1, data: [0, ...series.data] })),
    copyPoolBlockCounts().map(series => ({ ...series, end: series.end + 1 })),
    copyPoolBlockCounts().map(series => ({ ...series, data: series.data.slice(0, -1) })),
  ];
  for (const series of invalid) {
    assert.throws(() => addPoolOrdinals(attributedBlocks, attributedPools, series), /Pool block counts are still synchronizing/);
  }
  assert.throws(() => addPoolOrdinals(attributedBlocks, attributedPools.slice(0, -1), copyPoolBlockCounts()), /Pool block counts are still synchronizing/);
  assert.throws(() => addPoolOrdinals(attributedBlocks, [...attributedPools, "orphan"], [...copyPoolBlockCounts(), { index: "height", start: 100, end: 105, data: [0, 0, 0, 0, 0] }]), /Pool block counts are still synchronizing/);
});

test("rejects invalid totals at a block attributed to that pool", () => {
  for (const value of [null, 1.5, 0, -1]) {
    const series = copyPoolBlockCounts();
    series[0].data[0] = value;
    assert.throws(() => addPoolOrdinals(attributedBlocks, attributedPools, series), /Pool block counts are still synchronizing/);
  }
});

test("rejects cumulative counters whose increments disagree with attributed blocks", () => {
  const missedOwnBlock = copyPoolBlockCounts();
  missedOwnBlock[0].data[2] = 1;
  assert.throws(() => addPoolOrdinals(attributedBlocks, attributedPools, missedOwnBlock), /Pool block counts are still synchronizing/);

  const incrementedForOtherPool = copyPoolBlockCounts();
  incrementedForOtherPool[2].data[3] = 2;
  assert.throws(() => addPoolOrdinals(attributedBlocks, attributedPools, incrementedForOtherPool), /Pool block counts are still synchronizing/);
});

test("starts the pool ordinal at one for the genesis block", () => {
  const blocks = [{ height: 0, pool: "unknown", day: "2009-01-03" }];
  const series = [{ index: "height", start: 0, end: 1, data: [1] }];
  assert.deepEqual(addPoolOrdinals(blocks, ["unknown"], series), [
    { height: 0, pool: "unknown", day: "2009-01-03", poolDayIndex: 1, poolTotal: 1 },
  ]);
});

test("ranks every block, keeps Unknown, and orders tied leaders consistently", () => {
  const blocks = [
    { pool: "unknown" },
    { pool: "antpool" },
    { pool: "unknown" },
    { pool: "antpool" },
    { pool: "foundryusa" },
  ];
  const ranked = rankPools(blocks);

  assert.deepEqual(ranked, [
    { pool: "antpool", count: 2 },
    { pool: "unknown", count: 2 },
    { pool: "foundryusa", count: 1 },
  ]);
  assert.equal(ranked.reduce((total, entry) => total + entry.count, 0), blocks.length);
});

test("breaks ties by pool identifier independently of the browser language", context => {
  const compare = String.prototype.localeCompare;
  context.mock.method(String.prototype, "localeCompare", function(other) {
    return compare.call(this, other, "lt");
  });
  const blocks = ["yaamp", "viabtc", "1thash", "antpool"].map(pool => ({ pool }));
  assert.deepEqual(rankPools(blocks).map(entry => entry.pool), ["1thash", "antpool", "viabtc", "yaamp"]);
});

test("uses the monotonic UTC day for grouping without dropping a raw-header boundary block", () => {
  const rawBeforeMidnight = Date.UTC(2024, 0, 1, 23, 59, 59) / 1000;
  const rawAfterMidnight = Date.UTC(2024, 0, 2, 0, 0, 1) / 1000;
  const blocks = decodeBlocks([
    { index: "height", start: 100, end: 102, data: [rawAfterMidnight, rawBeforeMidnight] },
    { index: "height", start: 100, end: 102, data: [rawAfterMidnight, rawAfterMidnight] },
    { index: "height", start: 100, end: 102, data: ["antpool", "unknown"] },
  ]);

  assert.equal(isoDay(blocks[1].timestamp * 1000), "2024-01-01");
  assert.deepEqual(blocks.map(({ height, pool, day }) => ({ height, pool, day })), [
    { height: 100, pool: "antpool", day: "2024-01-02" },
    { height: 101, pool: "unknown", day: "2024-01-02" },
  ]);
  const days = groupDays(blocks);
  assert.deepEqual([...days.keys()], ["2024-01-02"]);
  assert.equal(days.get("2024-01-02").length, 2);
  assert.equal([...days.values()].flat().length, blocks.length);
});

test("advances calendar dates through leap days and year boundaries in UTC", () => {
  assert.equal(addDays("2023-12-31", 1), "2024-01-01");
  assert.equal(addDays("2024-02-28", 1), "2024-02-29");
  assert.equal(addDays("2024-02-29", 1), "2024-03-01");
  assert.equal(addDays("2024-12-31", 1), "2025-01-01");
});

test("rejects missing, misaligned, and incomplete series instead of silently truncating blocks", () => {
  const complete = [
    { index: "height", start: 100, end: 102, data: [1, 2] },
    { index: "height", start: 100, end: 102, data: [1, 2] },
    { index: "height", start: 100, end: 102, data: ["antpool", "unknown"] },
  ];

  assert.throws(() => decodeBlocks(complete.slice(0, 2)), /Incomplete block data/);
  assert.throws(() => decodeBlocks([
    complete[0],
    { ...complete[1], start: 101 },
    complete[2],
  ]), /Block data is still synchronizing/);
  assert.throws(() => decodeBlocks([
    complete[0],
    complete[1],
    { ...complete[2], data: ["antpool"] },
  ]), /Block data is still synchronizing/);
  assert.throws(() => decodeBlocks([
    complete[0],
    complete[1],
    { ...complete[2], data: ["Antpool", "unknown"] },
  ]), /Bitview returned an incomplete block/);
});

test("splits a complete historical year into four quarter requests", () => {
  assert.deepEqual(requestRanges("2024-01-01", "2025-01-01", "2026-09-24"), [
    { start: "2024-01-01", end: "2024-04-01" },
    { start: "2024-04-01", end: "2024-07-01" },
    { start: "2024-07-01", end: "2024-10-01" },
    { start: "2024-10-01", end: "2025-01-01" },
  ]);
});

test("clamps the genesis year and preserves quarter boundaries", () => {
  assert.deepEqual(requestRanges("2008-01-01", "2010-01-01", "2026-09-24"), [
    { start: "2009-01-03", end: "2009-04-01" },
    { start: "2009-04-01", end: "2009-07-01" },
    { start: "2009-07-01", end: "2009-10-01" },
    { start: "2009-10-01", end: "2010-01-01" },
  ]);
});

test("leaves the current quarter end open so the API includes the tip", () => {
  assert.deepEqual(requestRanges("2026-01-01", "2027-01-01", "2026-09-24"), [
    { start: "2026-01-01", end: "2026-04-01" },
    { start: "2026-04-01", end: "2026-07-01" },
    { start: "2026-07-01" },
  ]);
});
