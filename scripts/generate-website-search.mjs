import { readFile, writeFile } from "node:fs/promises";

const root = new URL("../website/", import.meta.url);

// The catalog imports theme/color modules but only reads series metadata.
// Supply their initialization surface without rendering charts or fetching data.
globalThis.document = {
  documentElement: { style: {} },
  getElementById: () => null,
};
globalThis.location = new URL("http://localhost/");
Object.defineProperty(globalThis, "localStorage", {
  value: { getItem: () => null },
});
globalThis.window = {
  document,
  location,
  matchMedia: () => ({ matches: true, addEventListener() {} }),
};
globalThis.getComputedStyle = () => ({ getPropertyValue: () => "" });

const { createPartialOptions } = await import(new URL("scripts/options/partial.js", root));
const { stringToId } = await import(new URL("scripts/utils/format.js", root));
const entries = [];

function walk(tree, path = []) {
  for (const node of tree) {
    const next = [...path, stringToId(node.name)];
    if ("tree" in node) {
      walk(node.tree, next);
    } else {
      const title = node.title || node.name;
      if ("url" in node) entries.push([node.url(), title, 1]);
      else entries.push([node.kind === "explorer" ? "/" : `/${next.join("/")}`, title]);
    }
  }
}
walk(createPartialOptions());

let previous = "";
const packed = entries.map(([href, title, blank]) => {
  let shared = 0;
  while (shared < previous.length && shared < href.length && previous[shared] === href[shared]) {
    shared++;
  }
  const entry = [shared, href.slice(shared), title];
  if (blank) entry.push(blank);
  previous = href;
  return entry;
});
const output = new URL("scripts/options/search-index.json", root);
const contents = JSON.stringify(packed) + "\n";
if (process.argv.includes("--check")) {
  if (await readFile(output, "utf8") !== contents) {
    throw new Error("Search index is stale. Run node scripts/generate-website-search.mjs");
  }
} else {
  await writeFile(output, contents);
}
console.log(`${entries.length} search entries ${process.argv.includes("--check") ? "verified" : "generated"}`);
