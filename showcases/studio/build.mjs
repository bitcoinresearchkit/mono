// Studio as one file: `node showcases/studio/build.mjs [out]` (out: dist/index.html, beside this). Its guide for
// assistants is in it: its "About Studio's links", the page's own text.
//
// index.html stays as it is, the repo's: it imports the modules beside it and the client this code builds (it's the
// version a server ships with itself, always in step). The file made here is served on its own, updated whenever, so it
// holds up against whatever its server (SERVER) runs: it imports the client that server serves (read at each load: made
// from that server's API, it always speaks the one live); everything else is inside it: the chart library (this repo's,
// tried with Studio), the fonts, the color names, the search, the presets. Series the page names by typed client paths
// are written as their names: paths move between versions, names are the server's (one it lacks is said, and shows as
// missing).
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { gzipSync } from "node:zlib";

const SERVER = "https://next.bitview.space";
const CLIENT = "/modules/bitview-client/index.js";
const HERE = dirname(fileURLToPath(import.meta.url)), SHOWCASES = join(HERE, "..");
const out = process.argv[2] ?? join(HERE, "dist", "index.html");
/** What the page is built from, each file by its own place (no link followed) and as it was read. @type {Map<string, Buffer>} */
const inputs = new Map([[realpathSync(fileURLToPath(import.meta.url)), readFileSync(fileURLToPath(import.meta.url))]]);
/** A file the page is built from, read (and kept, as read, to tell whether the commit has it so). */
const input = (/** @type {string} */ path, /** @type {BufferEncoding | undefined} */ encoding) => {
  const bytes = readFileSync(path);
  inputs.set(realpathSync(path), bytes);
  return encoding ? bytes.toString(encoding) : bytes;
};

let html = String(input(join(HERE, "index.html"), "utf8"));
/** How a link to it previews (put beside it below): read now, so the build's version names it too. */
const ogImage = input(join(HERE, "og.png"));
/** Replaces what must be there exactly once (the page changed under the build otherwise). */
function replaceOnce(from, to) {
  const found = typeof from === "string" ? html.split(from).length - 1 : (html.match(new RegExp(from.source, "g")) ?? []).length;
  if (found !== 1) throw new Error(`Expected ${from} once in index.html, found it ${found} times`);
  html = html.replace(from, () => to);
}
const dataUrl = (type, bytes) => `data:${type};base64,${Buffer.from(bytes).toString("base64")}`;
/**
 * A file of the server's, as text, in a time (a stalled server stalls nothing); one it answers it hasn't, null if
 * it's `optional` (else an error, as anything else that goes wrong is).
 */
async function fetchText(path, optional = false) {
  const response = await fetch(`${SERVER}${path}`, { signal: AbortSignal.timeout(30_000) });
  if (response.ok) return response.text();
  if (optional) return null;
  throw new Error(`${path}: ${response.status}`);
}
/**
 * `work` in a folder of its own, gone after (by its real path: the sandbox's permissions are checked against it, and the
 * system's temp folder can be a link).
 */
async function inFolder(prefix, work) {
  const folder = realpathSync(mkdtempSync(join(tmpdir(), prefix)));
  try {
    return await work(folder);
  } finally {
    rmSync(folder, { recursive: true, force: true });
  }
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
/**
 * A module of the repo's as a data URL of its text, quoted for the page's code: escaped only where a URL must be ("%",
 * "#", line breaks), so the page's compression takes it as text (as base64 it shrank by a quarter, as text by two
 * thirds); "</script" and "<!--" kept out of the script it's in.
 */
const inlined = (path) => JSON.stringify(`data:text/javascript;charset=utf-8,${input(join(SHOWCASES, path)).toString("utf8").replace(/[%#\r\n\t]/g, (c) => encodeURIComponent(c))}`).replace(/<(\/script|!--)/gi, "\\x3c$1");

// Its brand, Bitview's home: the root of the server that serves it, here a site of its own.
replaceOnce(`<a class="brand" href="/"`, `<a class="brand" href="https://bitview.space"`);
// The fonts, cut to the characters of fonts/ranges.txt (fonts/subset.sh): one the page writes outside them stops the
// build below (else it'd be in another font).
for (const font of ["InterVariable.woff2", "Lilex.woff2"]) replaceOnce(`url("../assets/fonts/${font}")`, `url("${dataUrl("font/woff2", input(join(HERE, "fonts", font)))}")`);
const ranges = String(input(join(HERE, "fonts", "ranges.txt"), "utf8")).trim().split(",").map((range) => range.slice(2).split("-").map((hex) => parseInt(hex, 16)));
/** The text the page writes beyond its own, packed (the presets'): its characters the fonts' too. */
let packedText = "";

// The modules the server doesn't serve, or serves older; the search's worker takes the same one.
const quickmatch = inlined("modules/quickmatch-js/src/index.js");
replaceOnce(`import XKCD from "../modules/xkcd-colors/index.js";`, `import XKCD from ${inlined("modules/xkcd-colors/index.js")};`);
replaceOnce(`import { QuickMatch, QuickMatchConfig } from "../modules/quickmatch-js/src/index.js";`, `import { QuickMatch, QuickMatchConfig } from ${quickmatch};`);
replaceOnce(`new URL("../modules/quickmatch-js/src/index.js", location.href).href`, quickmatch);
// (The QR code's, loaded once a link's is asked for.)
replaceOnce(`import("../modules/lean-qr/2.7.3/index.mjs")`, `import(${inlined("modules/lean-qr/2.7.3/index.mjs")})`);
// (A type for the editor only: nothing to load.)
replaceOnce(`/** @type {typeof import("../modules/quickmatch-js/src/index.js")} */ (await import(quickmatch))`, "(await import(quickmatch))");

// The chart library, this repo's, in the page: Studio leans on its insides (its series drawn its own way, what it draws
// over them), so it changes only when it's been tried, never with the showcases'.
replaceOnce(`import * as LC from "../modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs";`, `import * as LC from ${inlined("modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.mjs")};`);
// The client, the server's: it's made from that server's API, so it always speaks the one live.
replaceOnce(`import { BitviewClient } from "../modules/bitview-client/index.js";\n`, `${[
  `// The client as ${SERVER} serves it now, read at each load. Not loading, the page says so (see #offline): the charts`,
  `      // are kept, in this browser. (Tried by four addresses, each started 6 s after the one before or at once as it`,
  `      // fails, none given up on: a slow connection still loads, the first in wins. A first try may fail or stall (a browser`,
  `      // just started, its add-ons not ready yet), and a failed import is the page's for good by its address: the others`,
  `      // differ. All failed: offline.)`,
  `      const at = (again) => \`${SERVER}${CLIENT}\${again ? \`?again=\${again}\` : ""}\`;`,
  `      const { BitviewClient } = await new Promise((done, fail) => {`,
  `        const addresses = [0, 1, 2, 3].map(at);`,
  `        let started = 0, failed = 0, timer = 0;`,
  `        const next = () => {`,
  `          clearTimeout(timer);`,
  `          if (started === addresses.length) return;`,
  `          import(addresses[started++]).then((client) => ((started = addresses.length), clearTimeout(timer), done(client)), (error) => (++failed === addresses.length ? fail(error) : next()));`,
  `          timer = setTimeout(next, 6000);`,
  `        };`,
  `        next();`,
  `      }).catch((error) => {`,
  `        document.body.toggleAttribute("data-offline", true);`,
  `        throw error;`,
  `      });`,
].join("\n")}\n`);
replaceOnce("new BitviewClient({ baseUrl: location.origin", `new BitviewClient({ baseUrl: "${SERVER}"`);
// Asked for at once (the page's head, not after its 2 MB are read): its server's connection, and its client.
replaceOnce(`    <title>Bitview Studio: charts of Bitcoin's on-chain data</title>`, `    <link rel="preconnect" href="${SERVER}" crossorigin />\n    <link rel="modulepreload" href="${SERVER}${CLIENT}" crossorigin />\n    <title>Bitview Studio: charts of Bitcoin's on-chain data</title>`);

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

// The served client (in the sandbox: it's fetched code), read for the series it has: those the page names, and the
// presets' (below), checked against them. Unread, unchecked (said).
const NAMES = `
  const served = new (await import(SERVED)).BitviewClient({ baseUrl: "${SERVER}" });
  const names = new Set(), stack = [served.series];
  while (stack.length) {
    const node = stack.pop();
    if (typeof node?.name === "string" && typeof node.indexes === "function") names.add(node.name);
    else if (node && typeof node === "object") for (const key of Object.keys(node)) stack.push(Reflect.get(node, key, {}));
  }
  process.stdout.write(JSON.stringify({ version: String(served.VERSION), names: [...names] }));`;
const served = await inFolder("bitview-client-", async (checks) => {
  try {
    writeFileSync(join(checks, "served.mjs"), await fetchText(CLIENT));
  } catch (error) {
    console.warn(`Couldn't read ${SERVER}'s client (${error.message}): its names go unchecked`);
    return undefined;
  }
  return /** @type {{ version: string, names: string[] }} */ (sandboxed(NAMES.replace("SERVED", JSON.stringify(pathToFileURL(join(checks, "served.mjs")).href)), [checks]).value);
});

// Each of those names, checked against it: one it lacks (this repo ahead of it, mid-release) is said; the page shows it as
// missing, its closest offered, as any series the server doesn't have.
if (served) {
  const names = new Set(served.names), missing = [...named].filter((name) => !names.has(name));
  console[missing.length ? "warn" : "log"](`${SERVER}'s client ${served.version}: ${missing.length ? `lacks ${missing.join(", ")}, which the page names (shown as missing there)` : `has all ${named.size} series the page names`}`);
}

// The presets (presets.js), made from this repo's client, its typed paths the series it has: every series Studio can
// chart in one of them, and each the empty chart recommends there (the build stops otherwise); in the page compressed
// (unpacked once, when first wanted).
/** The presets, as made (for the sitemap below). @type {{ p: string[], t: string, c: { layers: any[] } }[]} */
let presetList = [];
{
  input(join(HERE, "presets.js"));
  const { studioPresets, leftOut, chainFacts } = await import(pathToFileURL(join(HERE, "presets.js")).href);
  // (The chain's facts as the server says them now; unread, the presets made without them, every member in, as said.)
  const client = new BitviewClient({ baseUrl: SERVER }), facts = await chainFacts(SERVER).catch((error) => void console.warn(`${SERVER}'s chain facts unread (${error.message}): presets of what never happened kept`));
  presetList = studioPresets(client, facts);
  const missing = leftOut(client, presetList, facts);
  // (What a chart can't hold the page leaves out without a word: here, it stops the build.)
  const over = presetList.filter((preset) => preset.c.layers.length > 40 || (preset.c.panes?.length ?? 1) > 12);
  if (over.length) throw new Error(`Presets over 40 layers or 12 panes: ${over.map((preset) => preset.p.join(" › ")).join(", ")}`);
  if (missing.length) throw new Error(`${missing.length} series in no preset: ${missing.slice(0, 12).join(", ")}${missing.length > 12 ? "…" : ""}`);
  const recommended = /const RECOMMENDED = \[([\s\S]*?)\];/.exec(html)?.[1];
  if (!recommended) throw new Error("No RECOMMENDED in the page");
  const places = new Set(presetList.map((preset) => preset.p.join(" › ")));
  const gone = [...recommended.matchAll(/"([^"]+)"/g)].map(([, place]) => place).filter((place) => !places.has(place));
  if (gone.length) throw new Error(`Recommended, but no preset: ${gone.join(", ")}`);
  const json = (packedText = JSON.stringify(presetList)), packed = gzipSync(json, { level: 9 }).toString("base64");
  replaceOnce(`<script id="presets" type="application/gzip"></script>`, `<script id="presets" type="application/gzip">${packed}</script>`);
  console.log(`${presetList.length} presets, every series in one: ${(packed.length / 1024).toFixed(0)} KB`);
  // (Their series, as the served client has them: one it lacks, said, as the page's own are above.)
  if (served) {
    const names = new Set(served.names), lacking = [...new Set(presetList.flatMap((preset) => preset.c.layers.map((layer) => layer.series)).filter((name) => name && !names.has(name)))];
    if (lacking.length) console.warn(`${SERVER}'s client lacks ${lacking.length} series the presets show (shown as missing there): ${lacking.slice(0, 12).join(", ")}${lacking.length > 12 ? "…" : ""}`);
  }
}

// The build it is, to tell which one runs (⌘K's "Studio's version"): its commit, a "+" when anything it was built from
// isn't, byte for byte as read, what the commit holds there (changed, or not in it at all); and when, as the page writes
// dates.
{
  const root = realpathSync(execFileSync("git", ["rev-parse", "--show-toplevel"], { cwd: HERE }).toString().trim());
  const commit = execFileSync("git", ["rev-parse", "--short=9", "HEAD"], { cwd: HERE }).toString().trim();
  const committed = (/** @type {string} */ path, /** @type {Buffer} */ bytes) => {
    const at = relative(root, path);
    try {
      return !at.startsWith("..") && execFileSync("git", ["show", `HEAD:${at}`], { cwd: root, maxBuffer: 1 << 28, stdio: ["ignore", "pipe", "ignore"] }).equals(bytes);
    } catch {
      return false;
    }
  };
  const changed = [...inputs].every(([path, bytes]) => committed(path, bytes)) ? "" : "+";
  const [, day, month, year, time] = new Date().toUTCString().split(" ");
  replaceOnce(`const BUILD = "here, from the repo";`, `const BUILD = ${JSON.stringify(`${Number(day)} ${month} ${year}, ${time.slice(0, 5)} UTC (${commit}${changed})`)};`);
}
const left = html.match(/["'(]\.\.\/(?:modules|assets)\/[^"')]*/g);
if (left?.length) throw new Error(`Still relative: ${left.join(", ")}`);
const unfonted = [...new Set(html + packedText)].filter((c) => c >= " " && !ranges.some(([from, to = from]) => /** @type {number} */ (c.codePointAt(0)) >= from && /** @type {number} */ (c.codePointAt(0)) <= to));
if (unfonted.length) throw new Error(`Characters outside fonts/ranges.txt (add their blocks there, then run fonts/subset.sh): ${unfonted.map((c) => `${c} U+${(c.codePointAt(0) ?? 0).toString(16).toUpperCase().padStart(4, "0")}`).join(", ")}`);
// Each of the page's scripts as it's written now, parsed (what the build put in can't break it: a name declared twice
// fails here, not in a browser).
await inFolder("bitview-page-", (parsing) => {
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
});
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, html);
console.log(`${relative(process.cwd(), out)}: ${(Buffer.byteLength(html) / 1024).toFixed(0)} KB`);

// Beside the page, for those who find it (the page itself is the one file of the app): llms.txt, its text for
// assistants (the page's own, its "about" section, as Markdown: one text, written once); sitemap.xml, every address it
// has (the Charts page, the presets' page, each section, each preset: the page's own code makes them, read from it);
// robots.txt, the sitemap's address (the preview's says no: see deploy.mjs); og.png, how a link to it previews;
// version.txt, the build (an open page tells a newer one by it).
{
  const ORIGIN = "https://studio.bitview.space", folder = dirname(out);
  const about = /<section id="about"[^>]*>([\s\S]*?)<\/section>/.exec(html)?.[1];
  if (!about) throw new Error("No about section in the page: llms.txt is made from it");
  const entities = { amp: "&", lt: "<", gt: ">", quot: '"', "#39": "'", nbsp: " " };
  const plain = (/** @type {string} */ text) => text.replace(/<[^>]+>/g, "").replace(/&(amp|lt|gt|quot|#39|nbsp);/g, (_, name) => entities[/** @type {keyof typeof entities} */ (name)]).replace(/[ \t]+/g, " ").trim();
  const markdown = [...about.matchAll(/<(h1|h2|h3|p|li|pre)(?:\s[^>]*)?>([\s\S]*?)<\/\1>/g)].map(([, tag, body]) => (tag === "pre" ? `\`\`\`js\n${plain(body.replace(/<br\s*\/?>/g, "\n")).replace(/ ?\n ?/g, "\n")}\n\`\`\`` : tag === "h1" ? `# ${plain(body)}` : tag === "h2" ? `## ${plain(body)}` : tag === "h3" ? `### ${plain(body)}` : tag === "li" ? `- ${plain(body)}` : plain(body))).join("\n\n").replace(/\n\n(?=- )/g, "\n");
  const code = /\/\* addresses:[^*]*\*\/([\s\S]*?)\/\* \/addresses \*\//.exec(html)?.[1];
  if (!code) throw new Error("No addresses in the page: the sitemap is made from them");
  const addressesOf = /** @type {(list: { p: string[] }[]) => { presets: string[], sections: Map<string, string> }} */ (new Function(`${code}\nreturn addressesOf;`)());
  const book = addressesOf(presetList);
  // (Its presets' sections, two levels down, each at its address with how many charts it holds: where to send someone
  // asking for one; the charts themselves, in the sitemap.)
  const held = new Map();
  for (const preset of presetList) for (let depth = 1; depth < preset.p.length && depth <= 2; depth++) ((key) => held.set(key, (held.get(key) ?? 0) + 1))(JSON.stringify(preset.p.slice(0, depth)));
  const sections = [...held].map(([key, count]) => `${JSON.parse(key).length > 1 ? "  " : ""}- ${JSON.parse(key).at(-1)} (${count}): ${ORIGIN}${book.sections.get(key)}`);
  // (In plain ASCII where it can be: it's served without a charset, as a text file is.)
  const ascii = (/** @type {string} */ text) => text.replace(/[…⋯]/g, "...").replace(/›/g, ">").replace(/[−–—]/g, "-").replace(/[‘’]/g, "'").replace(/[“”]/g, '"');
  writeFileSync(join(folder, "llms.txt"), ascii(`${markdown}${sections.length ? `\n\n## The presets' sections\n\nEach section's page lists its charts; each chart's own address is in ${ORIGIN}/sitemap.xml.\n\n${sections.join("\n")}` : ""}\n`));
  const urls = ["/", ...(presetList.length ? [...book.sections.values(), ...book.presets] : [])];
  writeFileSync(join(folder, "sitemap.xml"), `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.map((url) => `<url><loc>${ORIGIN}${url.replaceAll("&", "&amp;")}</loc></url>`).join("\n")}\n</urlset>\n`);
  writeFileSync(join(folder, "robots.txt"), `User-agent: *\nAllow: /\n\nSitemap: ${ORIGIN}/sitemap.xml\n`);
  writeFileSync(join(folder, "og.png"), ogImage);
  // (version.txt: the build, as the page says it (BUILD): an open page finds a newer one served by it.)
  const build = /const BUILD = ("[^"]*");/.exec(html)?.[1];
  if (!build) throw new Error("No BUILD in the page: version.txt is made from it");
  writeFileSync(join(folder, "version.txt"), `${JSON.parse(build)}\n`);
  console.log(`Beside it: llms.txt (${(markdown.length / 1024).toFixed(0)} KB), sitemap.xml (${urls.length} addresses), robots.txt, og.png, version.txt`);
}

// The page built, on the preview (studio-next.bitview.space; see deploy.mjs): a build elsewhere (an out given), not.
if (!process.argv[2]) {
  try {
    await (await import("./deploy.mjs")).deploy("next", true);
  } catch (error) {
    console.warn(`Preview not deployed: ${/** @type {Error} */ (error).message.split("\n")[0]}`);
  }
}

