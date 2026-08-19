import { BitviewClient } from "../index.js";

const client = new BitviewClient("http://localhost:3110");

console.log("Testing idiomatic API...\n");

// Test getter access (property)
console.log("1. Getter access (.by.dateindex):");
const all = await client.series.price.split.close.usd.by.day1;
console.log(`   Got: ${all.data.length} items\n`);

// Test dynamic access (bracket notation)
console.log("2. Dynamic access (.by['dateindex']):");
const allDynamic = await client.series.price.split.close.usd.by.day1;
console.log(`   Got: ${allDynamic.data.length} items\n`);

// Test fetch all (explicit .fetch())
console.log("3. Explicit .fetch():");
const allExplicit = await client.series.price.split.close.usd.by.day1.fetch();
console.log(`   Got: ${allExplicit.data.length} items\n`);

// Test first(n)
console.log("4. First 5 items (.first(5)):");
const first5 = await client.series.price.split.close.usd.by.day1.first(5);
console.log(
  `   Start: ${first5.start}, End: ${first5.end}, Got: ${first5.data.length} items\n`,
);

// Test last(n)
console.log("5. Last 5 items (.last(5)):");
const last5 = await client.series.price.split.close.usd.by.day1.last(5);
console.log(
  `   Start: ${last5.start}, End: ${last5.end}, Got: ${last5.data.length} items\n`,
);

// Test slice(start, end)
console.log("6. Slice 10-20 (.slice(10, 20)):");
const sliced = await client.series.price.split.close.usd.by.day1.slice(10, 20);
console.log(
  `   Start: ${sliced.start}, End: ${sliced.end}, Got: ${sliced.data.length} items\n`,
);

// Test get(index) - single item
console.log("7. Single item (.get(100)):");
const single = await client.series.price.split.close.usd.by.day1.get(100);
console.log(
  `   Start: ${single.start}, End: ${single.end}, Got: ${single.data.length} item(s)\n`,
);

// Test skip(n).take(m) chaining
console.log("8. Skip and take (.skip(100).take(10)):");
const skipTake = await client.series.price.split.close.usd.by.day1
  .skip(100)
  .take(10);
console.log(
  `   Start: ${skipTake.start}, End: ${skipTake.end}, Got: ${skipTake.data.length} items\n`,
);

// Test fetchCsv
console.log("9. Fetch as CSV (.last(3).fetchCsv()):");
const csv = await client.series.price.split.close.usd.by.day1
  .last(3)
  .fetchCsv();
console.log(`   CSV preview: ${csv.substring(0, 100)}...\n`);

console.log("All tests passed!");
