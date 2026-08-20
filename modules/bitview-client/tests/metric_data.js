/**
 * Tests for SeriesData helper methods and date conversion functions.
 * Run: node tests/metric_data.js
 */

import { BitviewClient } from "../index.js";

const client = new BitviewClient("http://localhost:3110");

console.log("Testing SeriesData helpers...\n");

// Fetch a date-based metric
console.log("1. Fetching price data (day1):");
const price = await client.series.price.split.close.usd.by.day1.first(5);
console.log(`   Start: ${price.start}, End: ${price.end}`);

// Test isDateBased
console.log("\n2. isDateBased:");
if (!price.isDateBased) throw new Error("day1 should be date-based");
console.log(`   day1: ${price.isDateBased}`);

// Test indexes() - always returns numbers
console.log("\n3. indexes():");
const indexes = price.indexes();
console.log(`   ${JSON.stringify(indexes)}`);
if (indexes.length !== 5) throw new Error("Expected 5 indexes");
if (indexes[0] !== price.start)
  throw new Error("First index should equal start");

// Test dates() - DateSeriesData method
console.log("\n4. dates():");
const dates = price.dates();
console.log(
  `   First: ${dates[0].toISOString()}, Last: ${dates[dates.length - 1].toISOString()}`,
);
if (dates.length !== 5) throw new Error("Expected 5 dates");
// Day1 index 0 = Jan 3, 2009 (genesis)
if (
  dates[0].getFullYear() !== 2009 ||
  dates[0].getMonth() !== 0 ||
  dates[0].getDate() !== 3
) {
  throw new Error(
    `Expected genesis date (2009-01-03), got ${dates[0].toISOString()}`,
  );
}

// Test keys() - always returns numbers (alias for indexes)
console.log("\n5. keys():");
const keys = price.keys();
if (keys.length !== 5) throw new Error("Expected 5 keys");
if (typeof keys[0] !== "number") throw new Error("Expected number keys");
console.log(`   Length: ${keys.length}, First: ${keys[0]}`);

// Test entries() - returns [number, value] pairs
console.log("\n6. entries():");
const entries = price.entries();
if (typeof entries[0][0] !== "number")
  throw new Error("Expected number entry key");
console.log(`   First: [${entries[0][0]}, ${entries[0][1]}]`);
if (entries[0][1] !== price.data[0])
  throw new Error("First entry value mismatch");

// Test dateEntries() - DateSeriesData method, returns [Date, value] pairs
console.log("\n7. dateEntries():");
const dateEntries = price.dateEntries();
if (!(dateEntries[0][0] instanceof Date))
  throw new Error("Expected Date entry key");
console.log(
  `   First: [${dateEntries[0][0].toISOString()}, ${dateEntries[0][1]}]`,
);

// Test toMap() - returns Map<number, value>
console.log("\n8. toMap():");
const map = price.toMap();
console.log(`   Size: ${map.size}`);
if (map.size !== 5) throw new Error("Expected map size 5");

// Test toDateMap() - DateSeriesData method
console.log("\n9. toDateMap():");
const dateMap = price.toDateMap();
console.log(`   Size: ${dateMap.size}`);
if (dateMap.size !== 5) throw new Error("Expected date map size 5");

// Test Symbol.iterator (for...of) - yields [number, value]
console.log("\n10. for...of iteration:");
let count = 0;
for (const [key, _val] of price) {
  if (count === 0 && typeof key !== "number")
    throw new Error("Expected number keys in iteration");
  count++;
}
console.log(`   Iterated ${count} items`);
if (count !== 5) throw new Error("Expected 5 iterations");

// Test with non-date-based index (height)
console.log("\n11. Testing height-based metric:");
const heightMetric = await client.series.price.spot.usd.by.height.last(3);
console.log(`   Start: ${heightMetric.start}, End: ${heightMetric.end}`);
if (heightMetric.isDateBased)
  throw new Error("height should not be date-based");

// Test keys() - always numbers
const heightKeys = heightMetric.keys();
console.log(`   keys(): ${JSON.stringify(heightKeys)}`);
if (typeof heightKeys[0] !== "number")
  throw new Error("Expected number keys for height");

// Test entries() - [number, value]
const heightEntries = heightMetric.entries();
console.log(
  `   entries()[0]: [${heightEntries[0][0]}, ${heightEntries[0][1]}]`,
);
if (heightEntries[0][0] !== heightMetric.start)
  throw new Error("First entry index mismatch");

// Test toMap() - Map<number, value>
const heightMap = heightMetric.toMap();
if (heightMap.size !== 3) throw new Error("Expected map size 3");
if (heightMap.get(heightMetric.start) !== heightMetric.data[0])
  throw new Error("First value mismatch");

// Test for...of on non-date metric
console.log("\n12. for...of iteration (height):");
let heightCount = 0;
for (const [key, _val] of heightMetric) {
  if (heightCount === 0 && typeof key !== "number")
    throw new Error("Expected number keys for height iteration");
  heightCount++;
}
console.log(`   Iterated ${heightCount} items`);

// Test different date indexes
console.log("\n13. Testing month1:");
const monthMetric =
  await client.series.price.split.close.usd.by.month1.first(3);
const monthDates = monthMetric.dates();
console.log(`   First month: ${monthDates[0].toISOString()}`);
// MonthIndex 0 = Jan 1, 2009
if (
  monthDates[0].getFullYear() !== 2009 ||
  monthDates[0].getMonth() !== 0 ||
  monthDates[0].getDate() !== 1
) {
  throw new Error(`Expected 2009-01-01, got ${monthDates[0].toISOString()}`);
}

// Test indexToDate directly
console.log("\n14. Testing indexToDate():");
const genesis = client.indexToDate("day1", 0);
if (
  genesis.getFullYear() !== 2009 ||
  genesis.getMonth() !== 0 ||
  genesis.getDate() !== 3
) {
  throw new Error(`Expected genesis 2009-01-03, got ${genesis.toISOString()}`);
}
const dayOne = client.indexToDate("day1", 1);
if (
  dayOne.getFullYear() !== 2009 ||
  dayOne.getMonth() !== 0 ||
  dayOne.getDate() !== 9
) {
  throw new Error(`Expected day one 2009-01-09, got ${dayOne.toISOString()}`);
}
console.log(`   day1 0: ${genesis.toISOString()}`);
console.log(`   day1 1: ${dayOne.toISOString()}`);

// Test week1
const week0 = client.indexToDate("week1", 0);
const week1 = client.indexToDate("week1", 1);
if (week0.getTime() !== genesis.getTime())
  throw new Error("week1 0 should equal genesis");
console.log(`   week1 0: ${week0.toISOString()}`);
console.log(`   week1 1: ${week1.toISOString()}`);

// Test year1
const year0 = client.indexToDate("year1", 0);
const year1 = client.indexToDate("year1", 1);
if (
  year0.getFullYear() !== 2009 ||
  year0.getMonth() !== 0 ||
  year0.getDate() !== 1
) {
  throw new Error(`Expected 2009-01-01, got ${year0.toISOString()}`);
}
if (year1.getFullYear() !== 2010) throw new Error("year1 1 should be 2010");
console.log(`   year1 0: ${year0.toISOString()}`);
console.log(`   year1 1: ${year1.toISOString()}`);

// Test month3
const q0 = client.indexToDate("month3", 0);
const q1 = client.indexToDate("month3", 1);
if (q0.getMonth() !== 0) throw new Error("month3 0 should be January");
if (q1.getMonth() !== 3) throw new Error("month3 1 should be April");
console.log(`   month3 0: ${q0.toISOString()}`);
console.log(`   month3 1: ${q1.toISOString()}`);

// Test month6
const s0 = client.indexToDate("month6", 0);
const s1 = client.indexToDate("month6", 1);
if (s0.getMonth() !== 0) throw new Error("month6 0 should be January");
if (s1.getMonth() !== 6) throw new Error("month6 1 should be July");
console.log(`   month6 0: ${s0.toISOString()}`);
console.log(`   month6 1: ${s1.toISOString()}`);

// Test year10
const d0 = client.indexToDate("year10", 0);
const d1 = client.indexToDate("year10", 1);
if (d0.getFullYear() !== 2009) throw new Error("year10 0 should be 2009");
if (d1.getFullYear() !== 2019) throw new Error("year10 1 should be 2019");
console.log(`   year10 0: ${d0.toISOString()}`);
console.log(`   year10 1: ${d1.toISOString()}`);

// Test dateToIndex
console.log("\n15. Testing dateToIndex():");
const idx = client.dateToIndex("day1", new Date(Date.UTC(2009, 0, 9)));
if (idx !== 1) throw new Error(`Expected day1 index 1, got ${idx}`);
console.log(`   day1 2009-01-09: ${idx}`);

const monthIdx = client.dateToIndex("month1", new Date(Date.UTC(2010, 0, 1)));
if (monthIdx !== 12)
  throw new Error(`Expected month1 index 12, got ${monthIdx}`);
console.log(`   month1 2010-01-01: ${monthIdx}`);

const yearIdx = client.dateToIndex("year1", new Date(Date.UTC(2019, 0, 1)));
if (yearIdx !== 10) throw new Error(`Expected year1 index 10, got ${yearIdx}`);
console.log(`   year1 2019-01-01: ${yearIdx}`);

// Test roundtrip: indexToDate -> dateToIndex
const testDate = client.indexToDate("day1", 100);
const roundtrip = client.dateToIndex("day1", testDate);
if (roundtrip !== 100)
  throw new Error(`Roundtrip failed: expected 100, got ${roundtrip}`);
console.log(`   Roundtrip day1 100: ${testDate.toISOString()} -> ${roundtrip}`);

// Test slice with Date
console.log("\n16. Testing slice with Date:");
const dateSlice = await client.series.price.split.close.usd.by.day1
  .slice(new Date(Date.UTC(2020, 0, 1)), new Date(Date.UTC(2020, 0, 4)))
  .fetch();
console.log(
  `   Slice start: ${dateSlice.start}, end: ${dateSlice.end}, items: ${dateSlice.data.length}`,
);
if (dateSlice.data.length !== dateSlice.end - dateSlice.start)
  throw new Error("Slice data length mismatch");

console.log("\nAll SeriesData tests passed!");
