// Studio as one file, for bitview.space: `node showcases/studio/build.mjs [out]` (out: dist/index.html, beside this).
//
// index.html stays as it is, the repo's: it imports the modules beside it and the client this code builds. The file
// made here imports the client and the chart library bitview.space serves (the builds its own page maps, read at each
// load: the page always speaks its server's version, and a release never leaves it on a stale or a missing build);
// everything else is inside it: the fonts, the color names, the search (newer than bitview.space's). Series the page
// names by typed client paths (templates, examples) are written as their names: paths move between versions, names
// are the server's. While bitview.space's client dates points unlike this repo's (its server's rules), the page takes
// this repo's two date helpers in their place.
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, posix, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { gzipSync } from "node:zlib";

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
// Asked for at once (the page's head, not after its 2 MB are read): its server's connection, and its page, which the
// module reads first (the same request: anonymous, as fetch makes it).
replaceOnce(`    <title>Bitview Studio</title>`, `    <link rel="preconnect" href="${SERVER}" crossorigin />\n    <link rel="preload" href="${SERVER}/" as="fetch" crossorigin />\n    <title>Bitview Studio</title>`);

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

// bitview.space's client, to check against.
let served;
try {
  const page = await (await fetch(`${SERVER}/`)).text();
  const map = JSON.parse(/<script type="importmap"[^>]*>([\s\S]*?)<\/script>/.exec(page)?.[1] ?? "{}").imports ?? {};
  const source = await (await fetch(`${SERVER}${map[CLIENT] ?? CLIENT}`)).text();
  served = new (await import(dataUrl("text/javascript", source))).BitviewClient({ baseUrl: SERVER });
} catch (error) {
  console.warn(`Couldn't read ${SERVER}'s client (${error.message}): its names go unchecked, its dates replaced`);
}

// Each of those names, checked against it (a series it lacks would show as not there).
if (served) {
  const names = new Set(), stack = [served.series];
  while (stack.length) {
    const node = stack.pop();
    if (typeof node?.name === "string" && typeof node.indexes === "function") names.add(node.name);
    else if (node && typeof node === "object") for (const key of Object.keys(node)) stack.push(Reflect.get(node, key, {}));
  }
  const missing = [...named].filter((name) => !names.has(name));
  console.log(`${SERVER}'s client ${served.VERSION}: ${missing.length ? `lacks ${missing.join(", ")}` : `has all ${named.size} series the page names`}`);
}

// Its dates, against this repo's client's: every date index, both ways (a bucket's first and last instants), in
// timezones on both sides of UTC (a client on local time agrees in some). Unlike (or unread), the page's client takes
// this repo's `indexToDate` and `dateToIndex` (the page dates points through those two alone).
const repo = new BitviewClient({ baseUrl: SERVER });
const DATE_INDEXES = ["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10"];
const differ = (client) => ["Pacific/Kiritimati", "Pacific/Pago_Pago"].some((zone) => {
  process.env.TZ = zone;
  return DATE_INDEXES.some((index) => [...Array(2000).keys(), 5000, 20000, 400000].some((i) => {
    if (/^year/.test(index) && i > 50) return false;
    const attempt = (run) => { try { return run(); } catch { return "throws"; } };
    const date = repo.indexToDate(index, i), last = new Date(repo.indexToDate(index, i + 1).getTime() - 1);
    return attempt(() => client.indexToDate(index, i).getTime()) !== date.getTime()
      || attempt(() => client.dateToIndex(index, date)) !== i || attempt(() => client.dateToIndex(index, last)) !== i;
  }));
});
if (differ(repo)) throw new Error("This repo's client disagrees with itself: the date check is wrong");
if (!served || differ(served)) {
  if (/\.(?:dates|dateEntries|toDateMap)\(\)/.test(html)) throw new Error("The page dates points through a response's helpers: the replaced two don't cover it");
  const source = readFileSync(join(SHOWCASES, "modules/bitview-client/index.js"), "utf8");
  const from = source.indexOf("// Date conversion constants and helpers"), to = source.indexOf("/**\n * Wrap raw series data");
  if (from < 0 || to < from) throw new Error("This repo's client's date helpers moved: the build can't find them");
  html = html.replace(/\n( *)const client = new BitviewClient\([^\n]*\);\n/, (line, indent) => `${line}${indent}// ${SERVER}'s client dates points unlike its server: this repo's helpers instead (the next release's).\n${indent}{\n${source.slice(from, to)}\nclient.indexToDate = indexToDate;\nclient.dateToIndex = dateToIndex;\n${indent}}\n`);
  if (!html.includes("client.indexToDate = indexToDate;")) throw new Error("The page's client moved: the build can't give it the date helpers");
  console.log(`${SERVER}'s client dates points unlike this repo's: the page takes this repo's date helpers`);
} else console.log(`${SERVER}'s client dates points as this repo's does`);

// The website's charts, as presets (presets.js): the live website's, as bitview.space serves it now (which can differ
// from this repo's, as its series names do). Its modules are fetched as served, each import followed from its options'
// entry (one by its root path, written as one within the copy), and run in a process of their own that can only read
// that copy and presets.js (no network, no files beyond, no processes, no environment): their charts come back as
// JSON, put in the page compressed (unpacked once, when first wanted). Unreadable, the page has none (and says so).
// (Its real path: the permissions are checked against it, and the system's temp folder can be a link.)
const copy = realpathSync(mkdtempSync(join(tmpdir(), "bitview-website-")));
try {
  const fetched = new Set(), SPECIFIER = /(\bfrom\s*|\bimport\s*\(\s*|\bimport\s*)(["'])([^"']+)\2/g;
  const take = async (path, entry = false) => {
    if (fetched.has(path)) return;
    fetched.add(path);
    const response = await fetch(`${SERVER}/${path}`);
    // (A path read off a comment or a string may be nothing: what's needed and missing fails as it's run.)
    if (!response.ok) {
      if (entry) throw new Error(`${path}: ${response.status}`);
      return;
    }
    let text = await response.text();
    const next = [];
    if (path.endsWith(".js")) text = text.replace(SPECIFIER, (match, lead, quote, specifier) => {
      if (!/^\.{1,2}\//.test(specifier) && !specifier.startsWith("/")) return match;
      const target = new URL(specifier, `${SERVER}/${path}`).pathname.slice(1);
      next.push(target);
      if (!specifier.startsWith("/")) return match;
      const local = posix.relative(posix.dirname(path), target);
      return `${lead}${quote}${local.startsWith(".") ? local : `./${local}`}${quote}`;
    });
    mkdirSync(dirname(join(copy, path)), { recursive: true });
    writeFileSync(join(copy, path), text);
    await Promise.all(next.map((target) => take(target)));
  };
  await take("scripts/options/partial.js", true);
  const presetsJs = join(HERE, "presets.js");
  const run = `console.log = console.error; const { websitePresets } = await import(${JSON.stringify(pathToFileURL(presetsJs).href)}); process.stdout.write(JSON.stringify(await websitePresets(${JSON.stringify(pathToFileURL(`${copy}/`).href)})));`;
  const json = execFileSync(process.execPath, ["--permission", `--allow-fs-read=${copy}`, `--allow-fs-read=${presetsJs}`, "--input-type=module", "-e", run], { env: {}, maxBuffer: 1 << 28, timeout: 120_000, stdio: ["ignore", "pipe", "ignore"] });
  const presets = JSON.parse(json.toString());
  const packed = gzipSync(json, { level: 9 }).toString("base64");
  replaceOnce(`<script id="presets" type="application/gzip"></script>`, `<script id="presets" type="application/gzip">${packed}</script>`);
  console.log(`${SERVER}'s ${presets.length} charts, as presets: ${(packed.length / 1024).toFixed(0)} KB, from ${fetched.size} files`);
} catch (error) {
  // (Said in the page: it has no worker to read them with, so it doesn't try.)
  if (html.includes(`<script id="presets" type="application/gzip"></script>`)) replaceOnce(`<script id="presets" type="application/gzip"></script>`, `<script id="presets" type="application/gzip" data-none></script>`);
  console.warn(`Couldn't make ${SERVER}'s charts into presets (${error.message}): the page has none`);
} finally {
  rmSync(copy, { recursive: true, force: true });
}

const left = html.match(/["'(]\.\.\/(?:modules|fonts)\/[^"')]*/g);
if (left?.length) throw new Error(`Still relative: ${left.join(", ")}`);
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, html);
console.log(`${relative(process.cwd(), out)}: ${(Buffer.byteLength(html) / 1024).toFixed(0)} KB`);
