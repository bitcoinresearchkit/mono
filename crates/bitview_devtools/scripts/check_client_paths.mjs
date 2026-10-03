// Walks every typed path of client-paths.tsv through the generated JavaScript client and checks
// that the leaf it reaches names the expected series and exposes the expected indexes.
// Reads the recorded baseline: run `cargo api` (or a passing `cargo api -- --check`) first.
// Usage: node crates/bitview_devtools/scripts/check_client_paths.mjs
import { readFileSync } from "node:fs";

const { BitviewClient } = await import(
  new URL("../../../modules/bitview-client/index.js", import.meta.url)
);
const rows = readFileSync(new URL("../snapshots/client-paths.tsv", import.meta.url), "utf8")
  .split("\n")
  .filter((line) => line && !line.startsWith("#"))
  .map((line) => line.split("\t"));
if (rows.length === 0 || rows.some((row) => row.length !== 5)) {
  console.error("client-paths.tsv is empty or malformed: run `cargo api`");
  process.exit(1);
}

const series = new BitviewClient({ baseUrl: "http://fixture.invalid" }).series;
const failures = [];
for (const [name, , javascript, , indexes] of rows) {
  const leaf = javascript.split(".").reduce((node, key) => node?.[key], series);
  const actual = leaf?.indexes ? [...leaf.indexes()].sort().join(",") : undefined;
  const expected = indexes.split(",").filter(Boolean).sort().join(",");
  if (leaf?.name !== name || actual !== expected) {
    failures.push(`${javascript}: expected ${name} [${expected}], got ${leaf?.name} [${actual}]`);
  }
}
for (const failure of failures.slice(0, 20)) console.error(failure);
console.error(`javascript: ${rows.length - failures.length}/${rows.length} typed paths resolve`);
process.exit(failures.length ? 1 : 0);
