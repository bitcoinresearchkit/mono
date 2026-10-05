// Studio as one file, for bitview.space: `node showcases/studio/build.mjs [out]` (out: dist/index.html, beside this).
//
// index.html stays as it is, the repo's: it imports the modules beside it and the client this code builds. The file
// made here imports the client and the chart library bitview.space serves (the builds its own page maps, read at each
// load: the page always speaks its server's version, and a release never leaves it on a stale or a missing build);
// everything else is inside it: the fonts, the color names, the search (newer than bitview.space's). Series the page
// names by typed client paths (templates, examples) are written as their names: paths move between versions, names
// are the server's.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SERVER = "https://bitview.space";
const CLIENT = "/scripts/modules/bitview-client/index.js";
const CHARTS = "/scripts/modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs";
const HERE = dirname(fileURLToPath(import.meta.url)), SHOWCASES = join(HERE, "..");
const out = process.argv[2] ?? join(HERE, "dist", "index.html");

let html = readFileSync(join(HERE, "index.html"), "utf8");
/** Replaces what must be there exactly once (the page changed under the build otherwise). */
function replaceOnce(from, to) {
  const found = typeof from === "string" ? html.split(from).length - 1 : (html.match(new RegExp(from.source, "g")) ?? []).length;
  if (found !== 1) throw new Error(`Expected ${from} once in index.html, found it ${found} times`);
  html = html.replace(from, () => to);
}
const dataUrl = (type, bytes) => `data:${type};base64,${Buffer.from(bytes).toString("base64")}`;
const inlined = (path) => dataUrl("text/javascript", readFileSync(join(SHOWCASES, path)));

// The fonts.
for (const font of ["InterVariable.woff2", "Lilex.woff2"]) replaceOnce(`url("../fonts/${font}")`, `url("${dataUrl("font/woff2", readFileSync(join(SHOWCASES, "fonts", font)))}")`);

// The modules bitview.space doesn't serve, or serves older; the search's worker takes the same one.
const quickmatch = inlined("modules/quickmatch-js/src/index.js");
replaceOnce(`import XKCD from "../modules/xkcd-colors/index.js";`, `import XKCD from "${inlined("modules/xkcd-colors/index.js")}";`);
replaceOnce(`import { QuickMatch, QuickMatchConfig } from "../modules/quickmatch-js/src/index.js";`, `import { QuickMatch, QuickMatchConfig } from "${quickmatch}";`);
replaceOnce(`new URL("../modules/quickmatch-js/src/index.js", location.href).href`, JSON.stringify(quickmatch));
// (A type for the editor only: nothing to load.)
replaceOnce(`/** @type {typeof import("../modules/quickmatch-js/src/index.js")} */ (await import(quickmatch))`, "(await import(quickmatch))");

// The client and the chart library, bitview.space's, by its page's import map.
replaceOnce(`import { BitviewClient } from "../modules/bitview-client/index.js";\n`, "");
replaceOnce(`import * as LC from "../modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs";`, [
  `// The client and the chart library as ${SERVER} serves them now: the builds its own page maps (cached for good, under`,
  `      // names that change with each release), read at each load; their plain paths if the page can't be read.`,
  `      const served = await fetch("${SERVER}/").then((response) => response.text()).then((page) => JSON.parse(/<script type="importmap"[^>]*>([\\s\\S]*?)<\\/script>/.exec(page)?.[1] ?? "{}").imports ?? {}).catch(() => ({}));`,
  `      const [{ BitviewClient }, LC] = await Promise.all([${JSON.stringify(CLIENT)}, ${JSON.stringify(CHARTS)}].map((path) => import(\`${SERVER}\${served[path] ?? path}\`)));`,
].join("\n"));
replaceOnce(/new BitviewClient\(\{ baseUrl: "[^"]*"/, `new BitviewClient({ baseUrl: "${SERVER}"`);

// Typed client paths, as the names they stand for (by this repo's client, whose paths they are).
const { BitviewClient } = await import(pathToFileURL(join(SHOWCASES, "modules/bitview-client/index.js")).href);
const tree = new BitviewClient({ baseUrl: SERVER }).series;
const walk = (node, keys) => keys.reduce((at, key) => at?.[key], node);
const cohortsLine = /\n *const cohorts = tree((?:\.[\w$]+)+);/.exec(html);
const roots = { tree, cohorts: cohortsLine ? walk(tree, cohortsLine[1].slice(1).split(".")) : undefined };
if (cohortsLine) replaceOnce(cohortsLine[0], "");
const named = new Set();
html = html.replace(/(?<![\w$.])(tree|cohorts)((?:\.[A-Za-z_$][\w$]*)+)\.name\b/g, (path, root, keys) => {
  const name = walk(roots[root], keys.slice(1).split("."))?.name;
  if (typeof name !== "string") throw new Error(`${path} names no series in this repo's client`);
  named.add(name);
  return JSON.stringify(name);
});
if (/(?<![\w$.])cohorts\./.test(html)) throw new Error("A path through `cohorts` is left that the build doesn't know");

// Each of those names, checked against bitview.space's client (a series it lacks would show as not there).
try {
  const page = await (await fetch(`${SERVER}/`)).text();
  const map = JSON.parse(/<script type="importmap"[^>]*>([\s\S]*?)<\/script>/.exec(page)?.[1] ?? "{}").imports ?? {};
  const source = await (await fetch(`${SERVER}${map[CLIENT] ?? CLIENT}`)).text();
  const served = new (await import(dataUrl("text/javascript", source))).BitviewClient({ baseUrl: SERVER });
  const names = new Set(), stack = [served.series];
  while (stack.length) {
    const node = stack.pop();
    if (typeof node?.name === "string" && typeof node.indexes === "function") names.add(node.name);
    else if (node && typeof node === "object") for (const key of Object.keys(node)) stack.push(Reflect.get(node, key, {}));
  }
  const missing = [...named].filter((name) => !names.has(name));
  console.log(`${SERVER}'s client ${served.VERSION}: ${missing.length ? `lacks ${missing.join(", ")}` : `has all ${named.size} series the page names`}`);
} catch (error) {
  console.warn(`Couldn't check the names against ${SERVER}'s client: ${error.message}`);
}

const left = html.match(/["'(]\.\.\/(?:modules|fonts)\/[^"')]*/g);
if (left?.length) throw new Error(`Still relative: ${left.join(", ")}`);
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, html);
console.log(`${relative(process.cwd(), out)}: ${(Buffer.byteLength(html) / 1024).toFixed(0)} KB`);
