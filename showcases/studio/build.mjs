// Studio as one file, for bitview.space: `node showcases/studio/build.mjs [out]` (out: dist/index.html, beside this),
// and its guide as llms.txt beside it.
//
// index.html stays as it is, the repo's: it imports the modules beside it and the client this code builds (it's the
// version a server ships with itself, always in step). The file made here is served on its own, updated whenever, so
// it holds up against whatever bitview.space runs: it imports the client bitview.space serves (the build its own page
// maps, read at each load: made from that server's API, it always speaks the one live); everything else is inside it:
// the chart library (this repo's, tried with Studio), the fonts, the color names, the search (newer than
// bitview.space's). Series the page names by typed client paths are written as their names: paths move between
// versions, names are the server's (one it lacks is said, and shows as missing). While bitview.space's client dates
// points unlike this repo's (its server's rules), the page takes this repo's two date helpers in their place, for as
// long as that client is the one live.
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, posix, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { gzipSync } from "node:zlib";

const SERVER = "https://bitview.space";
const CLIENT = "/scripts/modules/bitview-client/index.js";
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
/** A file of bitview.space's, as text: in a time (a stalled server stalls nothing), and only if it answered with it. */
async function fetchText(path, ms = 30_000) {
  const response = await fetch(`${SERVER}${path}`, { signal: AbortSignal.timeout(ms) });
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  return response.text();
}
/**
 * Fetched code, run in a process of its own that can read only what it's given (no network, no other files, no
 * processes, no environment): `script` (a module's text) prints its answer as JSON.
 */
function sandboxed(script, readable) {
  try {
    const json = execFileSync(process.execPath, ["--permission", ...readable.map((path) => `--allow-fs-read=${path}`), "--input-type=module", "-e", `console.log = console.error; ${script}`], { env: {}, maxBuffer: 1 << 28, timeout: 120_000, stdio: ["ignore", "pipe", "pipe"] });
    return { json, value: JSON.parse(json.toString()) };
  } catch (error) {
    // (What went wrong in there, its last words: not the whole script, which the error's message is.)
    const said = String(error.stderr ?? "").trim().split("\n").filter(Boolean).slice(-3).join(" | ");
    throw new Error(said || (error.signal ? `stopped (${error.signal})` : "it failed"));
  }
}
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

// The chart library, this repo's, in the page: Studio leans on its insides (its series drawn its own way, what it draws
// over them), so it changes only when it's been tried, never with the website's.
replaceOnce(`import * as LC from "../modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs";`, `import * as LC from "${inlined("modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs")}";`);
// The client, bitview.space's: it's made from that server's API, so it always speaks the one live.
replaceOnce(`import { BitviewClient } from "../modules/bitview-client/index.js";\n`, `${[
  `// The client as ${SERVER} serves it now: the build its own page maps (cached for good, under a name that changes`,
  `      // with each release), read at each load; its plain path if the page can't be read (or doesn't answer in a few`,
  `      // seconds), or the mapped build fails. Not loading, the page says so (see #offline): the charts are kept, in this browser.`,
  `      const served = await fetch("${SERVER}/", { signal: AbortSignal.timeout(8000) }).then((response) => response.text()).then((page) => JSON.parse(/<script type="importmap"[^>]*>([\\s\\S]*?)<\\/script>/.exec(page)?.[1] ?? "{}").imports ?? {}).catch(() => ({}));`,
  `      const clientAt = (map) => import(\`${SERVER}\${map[${JSON.stringify(CLIENT)}] ?? ${JSON.stringify(CLIENT)}}\`);`,
  `      const { BitviewClient } = await clientAt(served).catch(() => clientAt({})).catch((error) => {`,
  `        document.body.toggleAttribute("data-offline", true);`,
  `        throw error;`,
  `      });`,
].join("\n")}\n`);
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

// bitview.space's client, checked against (in the sandbox: it's fetched code): the series it has, and its dates
// against this repo's client's: every date index, both ways (a bucket's first and last instants), in timezones on both
// sides of UTC (a client on local time agrees in some). Unlike (or unread), the page's client takes this repo's
// `indexToDate` and `dateToIndex` (the page dates points through those two alone).
const CHECK = `
  const DATE_INDEXES = ["minute10", "minute30", "hour1", "hour4", "hour12", "day1", "day3", "week1", "month1", "month3", "month6", "year1", "year10"];
  const repo = new (await import(REPO)).BitviewClient({ baseUrl: "${SERVER}" }), served = new (await import(SERVED)).BitviewClient({ baseUrl: "${SERVER}" });
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
  const names = new Set(), stack = [served.series];
  while (stack.length) {
    const node = stack.pop();
    if (typeof node?.name === "string" && typeof node.indexes === "function") names.add(node.name);
    else if (node && typeof node === "object") for (const key of Object.keys(node)) stack.push(Reflect.get(node, key, {}));
  }
  process.stdout.write(JSON.stringify({ version: String(served.VERSION), names: [...names], self: differ(repo), differs: differ(served) }));`;
let served;
const checks = realpathSync(mkdtempSync(join(tmpdir(), "bitview-client-")));
try {
  // (Both beside each other there: what the sandbox reads is that folder alone. Unread, the repo's own stands in for
  // the server's, so the date check still checks itself.)
  writeFileSync(join(checks, "repo.mjs"), readFileSync(join(SHOWCASES, "modules/bitview-client/index.js")));
  let read = true;
  try {
    const page = await fetchText("/");
    const map = JSON.parse(/<script type="importmap"[^>]*>([\s\S]*?)<\/script>/.exec(page)?.[1] ?? "{}").imports ?? {};
    writeFileSync(join(checks, "served.mjs"), await fetchText(map[CLIENT] ?? CLIENT));
  } catch (error) {
    read = false;
    writeFileSync(join(checks, "served.mjs"), readFileSync(join(checks, "repo.mjs")));
    console.warn(`Couldn't read ${SERVER}'s client (${error.message}): its names go unchecked, its dates replaced`);
  }
  const checked = sandboxed(CHECK.replace("REPO", JSON.stringify(pathToFileURL(join(checks, "repo.mjs")).href)).replace("SERVED", JSON.stringify(pathToFileURL(join(checks, "served.mjs")).href)), [checks]).value;
  if (checked.self) throw new Error("This repo's client disagrees with itself: the date check is wrong");
  if (read) served = checked;
} finally {
  rmSync(checks, { recursive: true, force: true });
}

// Each of those names, checked against it: one it lacks (this repo ahead of it, mid-release) is said; the page shows it as
// missing, its closest offered, as any series the server doesn't have.
if (served) {
  const names = new Set(served.names), missing = [...named].filter((name) => !names.has(name));
  console[missing.length ? "warn" : "log"](`${SERVER}'s client ${served.version}: ${missing.length ? `lacks ${missing.join(", ")}, which the page names (shown as missing there)` : `has all ${named.size} series the page names`}`);
}
if (!served || served.differs) {
  if (/\.(?:dates|dateEntries|toDateMap)\(\)/.test(html)) throw new Error("The page dates points through a response's helpers: the replaced two don't cover it");
  const source = readFileSync(join(SHOWCASES, "modules/bitview-client/index.js"), "utf8");
  const from = source.indexOf("// Date conversion constants and helpers"), to = source.indexOf("/**\n * Wrap raw series data");
  if (from < 0 || to < from) throw new Error("This repo's client's date helpers moved: the build can't find them");
  // (Only while the client that needed them is the one live: one released since dates its server's points itself. Unread,
  // always: there's no telling which.)
  const when = served ? `if (client.VERSION === ${JSON.stringify(served.version)}) ` : "";
  html = html.replace(/\n( *)const client = new BitviewClient\([^\n]*\);\n/, (line, indent) => `${line}${indent}// ${SERVER}'s client dates points unlike its server: this repo's helpers instead (the next release's).\n${indent}${when}{\n${source.slice(from, to)}\nclient.indexToDate = indexToDate;\nclient.dateToIndex = dateToIndex;\n${indent}}\n`);
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
    const response = await fetch(`${SERVER}/${path}`, { signal: AbortSignal.timeout(30_000) });
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
  const { json, value: presets } = sandboxed(`const { websitePresets } = await import(${JSON.stringify(pathToFileURL(presetsJs).href)}); process.stdout.write(JSON.stringify(await websitePresets(${JSON.stringify(pathToFileURL(`${copy}/`).href)})));`, [copy, presetsJs]);
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
// Each of the page's scripts as it's written now, parsed (what the build put in can't break it: a name declared twice
// fails here, not in a browser).
const parsing = realpathSync(mkdtempSync(join(tmpdir(), "bitview-page-")));
try {
  for (const [i, [, attributes, body]] of [...html.matchAll(/<script(\s[^>]*)?>([\s\S]*?)<\/script>/g)].entries()) {
    if (/type="(?!module)[^"]*"/.test(attributes ?? "") || !body.trim()) continue;
    const file = join(parsing, `script-${i}.${/type="module"/.test(attributes ?? "") ? "mjs" : "cjs"}`);
    writeFileSync(file, body);
    try {
      execFileSync(process.execPath, ["--check", file], { stdio: ["ignore", "ignore", "pipe"] });
    } catch (error) {
      throw new Error(`The page's script ${i} doesn't parse: ${String(error.stderr ?? "").trim().split("\n").filter(Boolean).slice(-2).join(" | ")}`);
    }
  }
} finally {
  rmSync(parsing, { recursive: true, force: true });
}
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, html);
console.log(`${relative(process.cwd(), out)}: ${(Buffer.byteLength(html) / 1024).toFixed(0)} KB`);

// The guide beside the page, as markdown (llms.txt, which the page's head points to): its "About Studio's links"
// section, word for word, so the two never drift. Served with the page, or the head points nowhere.
const about = /<section id="about"[^>]*>([\s\S]*?)<\/section>/.exec(html)?.[1];
if (!about) throw new Error("No guide (the about section) in index.html");
const plain = (fragment) => fragment.replace(/<[^>]+>/g, "").replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&");
const line = (fragment) => plain(fragment).replace(/\s+/g, " ").trim();
const markdown = about
  .replace(/<pre>([\s\S]*?)<\/pre>/g, (_, code) => `\n\`\`\`\n${plain(code).trim()}\n\`\`\`\n`)
  .replace(/<h1>([\s\S]*?)<\/h1>/g, (_, text) => `\n# ${line(text)}\n`)
  .replace(/<h2>([\s\S]*?)<\/h2>/g, (_, text) => `\n## ${line(text)}\n`)
  .replace(/<li>([\s\S]*?)<\/li>/g, (_, text) => `- ${line(text)}\n`)
  .replace(/<p>([\s\S]*?)(?:<\/p>|(?=\n\s*<(?:pre|ul|h\d)))/g, (_, text) => `\n${line(text)}\n`)
  .replace(/<\/?ul>/g, "\n")
  .split("\n").map((text) => text.replace(/^ +/, "")).join("\n")
  .replace(/\n{3,}/g, "\n\n").replace(/(?<=^- [^\n]*)\n\n(?=- )/gm, "\n").trim();
const guide = join(dirname(out), "llms.txt");
writeFileSync(guide, `${markdown}\n`);
console.log(`${relative(process.cwd(), guide)}: the guide, ${(Buffer.byteLength(markdown) / 1024).toFixed(0)} KB (serve it beside the page)`);
