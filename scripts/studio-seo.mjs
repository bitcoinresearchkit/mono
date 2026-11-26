// Beside the page, for those who find Studio: sitemap.xml, every address it has (Home, the presets' page,
// each section, each preset: made by addresses.js, the page's own), llms.txt, its guide for assistants (the page's own
// "about" section, as Markdown, with the presets' sections at their addresses), and presets.hash, what the page keys
// its presets' cache by (see below). Made from the presets as a server's chain makes them (the local one's by default;
// its facts kept beside them, chain-facts.json) and the client's series, their titles checked against the rules every
// title keeps (see TITLE_RULES), and written again whenever the presets, the guide or the client change:
//
//   bun scripts/studio-seo.mjs [server]          writes them, by the server's chain now
//   bun scripts/studio-seo.mjs --check           fails if the code makes them otherwise now (by the chain's facts
//                                                  they were written by: what the chain did since fails nothing)
//   bun scripts/studio-seo.mjs --drop [server]   writes them though charts have gone or old addresses changed (below)
//
// And moved.txt: every address Studio once published and has no more, each with what it opens now, kept for good (the
// server answers each with a redirect): a preset's old window address, that preset at it (see addresses.js), the build
// stopped where one changes or another place takes it; a chart or section gone, its successor (see successor).
import { gunzipSync, gzipSync } from "node:zlib";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { addressesOf, slug } from "../website/studio/addresses.js";
import { placeWas } from "../website/studio/places-were.js";
import { studioApplication, presetInfo, sectionInfo, presetDataset, presetDistributions, STUDIO_TITLE, STUDIO_DESCRIPTION, STUDIO_ORIGIN } from "../website/studio/page-info.js";
import { shownStep } from "../website/studio/words.js";

const STUDIO = join(dirname(fileURLToPath(import.meta.url)), "..", "website", "studio");
const check = process.argv.includes("--check"), drop = process.argv.includes("--drop");
const SERVER = process.argv.slice(2).find((arg) => !arg.startsWith("--")) ?? "http://127.0.0.1:3110";
// (The address Studio's known by, on every domain it's served on: see ORIGIN in the page.)
const ORIGIN = STUDIO_ORIGIN;

const html = readFileSync(join(STUDIO, "index.html"), "utf8");
const presetsEdit = readFileSync(join(STUDIO, "presets-edit.js"), "utf8");
const { BitviewClient } = await import(pathToFileURL(join(STUDIO, "..", "modules", "bitview-client", "index.js")).href);
const { studioPresets } = await import(pathToFileURL(join(STUDIO, "presets.js")).href);
const { chainFacts, factsKey, factsOfKey } = await import(pathToFileURL(join(STUDIO, "facts.js")).href);
const writtenBytes = (/** @type {string} */ name) => {
  try { return readFileSync(join(STUDIO, name)); }
  catch (error) { if (error.code === "ENOENT") return Buffer.alloc(0); throw error; }
};
const written = (/** @type {string} */ name) => writtenBytes(name).toString("utf8");
// (Checked, by the facts they were written by; written, by the server's now. Without them the presets would hold what
// never happened: stopped.)
const kept = written("chain-facts.json").trim();
if (check && !kept) throw new Error("No chain-facts.json: write them first, bun scripts/studio-seo.mjs");
const facts = check ? factsOfKey(kept) : await chainFacts(SERVER).catch((error) => {
  throw new Error(`${SERVER}'s chain facts unread (${error.message}): is it running?`);
});
// (What the presets report while they're made, a band the client renamed (see reported in presets.js), stops them here:
// the page goes on without it, so a preset quietly gone breaks this, never only a line in a console.)
const reported = /** @type {string[]} */ ([]);
const client = new BitviewClient({ baseUrl: SERVER });
const { presets, labels } = studioPresets(client, facts, (problem) => reported.push(problem));
if (reported.length) throw new Error(`The presets reported ${reported.length} problem${reported.length === 1 ? "" : "s"}, fix them, then write these again:\n  ${reported.join("\n  ")}`);
/**
 * What every title keeps, each a rule and the titles that break it: one colon at most, the group whose it is after
 * it; a share names its whole ("Share of all coins", "Supply share", never "Share in profit"); a window's days, never
 * months (a window counts fixed days: an age says months, "1 month to under 2 months old", and a calendar period,
 * "by month"); never "all addresses" (the addresses tracked aren't all outputs'). The last two, every name shown too
 * (legends, the tree's): `titles`, a title's alone.
 * @type {[string, (text: string) => boolean, titles?: boolean][]}
 */
const TITLE_RULES = [
  ["more than one colon", (title) => (title.match(/:/g) ?? []).length > 1, true],
  ["a share that names no whole", (title) => [...title.matchAll(/\bshare\b/gi)].some((m) => !/^ of /i.test(title.slice(m.index + 5)) && !/[a-z)] $/i.test(title.slice(0, m.index))), true],
  ["a month that isn't an age", (text) => /(?<!\b(?:by|per|each) )\bmonths?\b(?![^,:]*\bold\b)/.test(text)],
  ["all addresses", (text) => /\ball addresses\b/i.test(text)],
];
/** Every step of every place, once, by its key. @type {Map<string, string[]>} */
const steps = new Map();
for (const preset of presets) for (let i = 1; i <= preset.p.length; i++) steps.set(JSON.stringify(preset.p.slice(0, i)), preset.p.slice(0, i));
/** Every name shown, each with whether it's a title and its place: titles, legends, the tree's steps as shown. @type {[string, boolean, string[]][]} */
const shown = [
  ...presets.flatMap((preset) => [[preset.t, true, preset.p], ...preset.c.layers.flatMap((layer) => (layer.label ? [[layer.label, false, preset.p]] : []))]),
  ...[...steps.values()].map((path) => [shownStep(labels, path), false, path]),
];
const broken = shown.flatMap(([text, title, path]) => TITLE_RULES.flatMap(([rule, breaks, titles]) => ((title || !titles) && breaks(text) ? [`${rule}: "${text}" (${path.join(" › ")})`] : [])));
if (broken.length) throw new Error(`${broken.length} name${broken.length === 1 ? "" : "s"} shown break${broken.length === 1 ? "s" : ""} the rules:\n  ${broken.slice(0, 20).join("\n  ")}${broken.length > 20 ? `\n  and ${broken.length - 20} more` : ""}`);
// One subject a preset, all of it in view: at most two panes shown, the price's (hidden over the chain's workings)
// aside, and no pane of its own hidden, so nothing is drawn that a reader can't see.
const PRICE = client.series.price.ohlc.name, paneOf = (/** @type {{ pane?: number }} */ layer) => layer.pane ?? 1;
const unseen = presets.filter(({ c }) => ((shown) => shown.size > 2 || Array.from({ length: Math.max(...c.layers.map(paneOf)) }, (_, i) => i + 1).some((pane) => !shown.has(pane) && !c.layers.some((layer) => paneOf(layer) === pane && layer.series === PRICE)))(new Set(c.layers.filter((layer) => !layer.hidden).map(paneOf))));
if (unseen.length) throw new Error(`${unseen.length} preset${unseen.length === 1 ? "" : "s"} with a pane hidden or past two:\n  ${unseen.slice(0, 20).map(({ p }) => p.join(" › ")).join("\n  ")}`);
// The presets the empty chart offers first (the app's RECOMMENDED), each one of them: a place renamed would drop it.
const recommended = /const RECOMMENDED = \[([\s\S]*?)\];/.exec(presetsEdit)?.[1];
if (!recommended) throw new Error("No RECOMMENDED in presets-edit.js");
const places = new Set(presets.map((preset) => preset.p.join(" › ")));
const unknown = [...recommended.matchAll(/"([^"]+)"/g)].map(([, place]) => place).filter((place) => !places.has(place));
if (unknown.length) throw new Error(`Recommended, but no preset there: ${unknown.join("; ")}`);

const book = addressesOf(presets);
// Old addresses, each opening what's there now: a preset moved (see places-were.js) keeps each it had, never over a
// place there now. Written to moved.txt, the one table the server and the page both send old addresses by.
const addressOf = (/** @type {string[]} */ path) => `/presets/${path.map(slug).join("/")}`;
const alias = (/** @type {string} */ address, /** @type {{ path: string[] } | { preset: number, window?: string }} */ to) => book.at.has(address) || book.at.set(address, to);
const sameStart = (/** @type {string[]} */ a, /** @type {string[]} */ b) => ((at) => (at < 0 ? a : a.slice(0, at)))(a.findIndex((step, d) => step !== b[d]));
const goneSections = new Map();
for (const [i, preset] of presets.entries()) for (const { path: old, window: folded } of placeWas(preset.p)) {
  alias(addressOf(old), folded ? { preset: i, window: folded } : { preset: i });
  // (Its windows had their own addresses beside the chart: each window's name there, as the chart was named then (a
  // window named after the chart, "Supply, all time", was named after its old name) and where the window's place
  // went since. A chart folded into this one had none of this one's windows.)
  if (!folded) for (const [name, window] of Object.entries(preset.w ?? {})) {
    const leaf = /** @type {string} */ (preset.p.at(-1)), then = name.startsWith(leaf) ? [[...old.slice(0, -1), old.at(-1) + name.slice(leaf.length)]] : [];
    for (const place of [[...old.slice(0, -1), name], ...then, ...placeWas([...preset.p.slice(0, -1), name]).map((was) => was.path)]) alias(addressOf(place), { preset: i, window });
  }
  // (A section gone opens the deepest section that holds all of its charts now.)
  for (let depth = 1; depth < old.length; depth++) {
    const key = JSON.stringify(old.slice(0, depth)), parent = preset.p.slice(0, -1);
    if (!book.sections.has(key)) goneSections.set(key, { old: old.slice(0, depth), now: sameStart(goneSections.get(key)?.now ?? parent, parent) });
  }
}
for (const { old, now } of goneSections.values()) alias(`${addressOf(old)}/`, { path: now });

const about = /<section id="about"[^>]*>([\s\S]*?)<\/section>/.exec(html)?.[1];
if (!about) throw new Error("No about section in the page: llms.txt is made from it");
// (Its tags gone, a tag being < and a letter: the guide's comparisons (B > 3.5, < >= <=) kept as written.)
const entities = { amp: "&", lt: "<", gt: ">", quot: '"', "#39": "'", nbsp: " " };
const plain = (/** @type {string} */ text) => text.replace(/<\/?[a-z][^>]*>/gi, "").replace(/&(amp|lt|gt|quot|#39|nbsp);/g, (_, name) => entities[/** @type {keyof typeof entities} */ (name)]).replace(/[ \t]+/g, " ").trim();
const markdown = [...about.matchAll(/<(h1|h2|h3|p|li|pre)(?:\s[^>]*)?>([\s\S]*?)<\/\1>/g)]
  .map(([, tag, body]) => (tag === "pre" ? `\`\`\`js\n${plain(body.replace(/<br\s*\/?>/g, "\n")).replace(/ ?\n ?/g, "\n")}\n\`\`\`` : tag === "h1" ? `# ${plain(body)}` : tag === "h2" ? `## ${plain(body)}` : tag === "h3" ? `### ${plain(body)}` : tag === "li" ? `- ${plain(body)}` : plain(body)))
  .join("\n\n")
  .replace(/\n\n(?=- )/g, "\n");
// (Its presets' sections, two levels down, each at its address with how many charts it holds: where to send someone
// asking for one; the charts themselves, in the sitemap.)
// (Each by its name as shown: its label, where it has one; see shownStep.)
const held = new Map();
for (const preset of presets) for (let depth = 1; depth < preset.p.length && depth <= 2; depth++) {
  const key = JSON.stringify(preset.p.slice(0, depth));
  held.set(key, (held.get(key) ?? 0) + 1);
}
// A page's Markdown twin: its address and `.md`, a section's (a folder's page) being its index.html, as the server
// answers them (`twin` in crates/bitview_website/src/page_info.rs) and llmstxt.org has readers look.
const twin = (/** @type {string} */ address) => `${address}${address.endsWith("/") ? "index.html" : ""}.md`;
const sections = [...held].map(([key, count]) => {
  const path = JSON.parse(key);
  return `${path.length > 1 ? "  " : ""}- ${shownStep(labels, path)} (${count}): ${ORIGIN}${twin(book.sections.get(key))}`;
});
// (In plain ASCII where it can be: it's served as a text file is, without a charset.)
const ascii = (/** @type {string} */ text) => text.replace(/[…⋯]/g, "...").replace(/›/g, ">").replace(/[−–—]/g, "-").replace(/[‘’]/g, "'").replace(/[“”]/g, '"');
const urls = ["/", ...book.sections.values(), ...book.presets];
// What the presets are made from, presets.js, the facts' reader (facts.js: the page's worker makes them by it) and
// every module they import (followed by their relative imports, as Bun reads them, each once), hashed, each by its path
// from here (the same on every system): the page keys its presets' cache by it (with the client's own stamp, HASH, and
// the chain's facts), so a change to any of them makes them again, with nothing read in the page to know it.
const sources = new Set([join(STUDIO, "presets.js"), join(STUDIO, "facts.js")]), transpiler = new Bun.Transpiler({ loader: "js" }), sha = createHash("sha256");
for (const file of sources) {
  const text = readFileSync(file, "utf8");
  sha.update(`${relative(STUDIO, file).replaceAll(sep, "/")}\0${text}\0`);
  for (const { path } of transpiler.scanImports(text)) if (path.startsWith(".")) sources.add(join(dirname(file), path));
}

// A section's direct children, in the page's order: sections before charts.
const children = new Map([...book.sections.keys()].map((key) => [key, new Map()]));
for (const [key, address] of book.sections) {
  const path = JSON.parse(key);
  if (path.length) children.get(JSON.stringify(path.slice(0, -1))).set(address, shownStep(labels, path));
}
for (const [i, preset] of presets.entries()) children.get(JSON.stringify(preset.p.slice(0, -1))).set(book.presets[i], preset.t);
const pageInfo = {};
for (const [key, address] of book.sections) {
  const path = JSON.parse(key), links = children.get(key), info = sectionInfo(path, labels, [...links.values()]);
  const canonical = `${ORIGIN}${address}`;
  pageInfo[address] = { ...info, canonical, json_ld: studioApplication(canonical, info.description),
    markdown: `# ${path.length ? shownStep(labels, path) : "Studio presets"}\n\n${info.description}\n\n[Open this page](${canonical})\n\n## Charts and sections\n\n${[...links].map(([address, name]) => `- [${name}](${ORIGIN}${address})`).join("\n")}\n` };
}
for (const [i, preset] of presets.entries()) {
  const canonical = `${ORIGIN}${book.presets[i]}`, info = presetInfo(preset, labels), distributions = presetDistributions(preset, client.series);
  pageInfo[book.presets[i]] = { ...info, canonical, json_ld: presetDataset(preset, labels, client.series, canonical),
    markdown: `# ${preset.t}\n\n${info.description}\n\n[Open this chart](${canonical})\n\n## What the chart shows\n\n${preset.c.layers.map((layer, i) => `- ${String.fromCharCode(65 + i)}: ${layer.label ?? layer.series ?? (layer.formula ? `Formula ${layer.formula}` : layer.heatmap ? `Cost basis distribution: ${layer.heatmap}` : layer.profile ? `Cost basis profile: ${layer.profile.join(", ")}` : `Cloud between ${(layer.cloud ?? []).join(" and ")}`)}${layer.hidden ? " (hidden initially)" : ""}`).join("\n")}\n\n## Data sources\n\n${distributions.map((source) => `- ${source.name}: [${source.encodingFormat === "text/csv" ? "CSV" : "JSON"}](${source.contentUrl})`).join("\n")}\n` };
}
pageInfo["/"] = { title: STUDIO_TITLE, description: STUDIO_DESCRIPTION, canonical: `${ORIGIN}/`, json_ld: studioApplication(`${ORIGIN}/`, STUDIO_DESCRIPTION), markdown: `# Bitview Studio\n\n${STUDIO_DESCRIPTION}\n\n[Browse the presets](${ORIGIN}/presets/)\n\n[Guide for assistants](${ORIGIN}/llms.txt)\n` };

const pageInfoText = `${JSON.stringify(pageInfo)}\n`;
const files = {
  "page-info.json.gz": gzipSync(pageInfoText, { level: 9 }),
  "llms.txt": ascii(`${markdown}\n\n## The presets' sections\n\nEach section's page lists its charts; each chart's own address is in ${ORIGIN}/sitemap.xml. Every page is also there as Markdown, at its address and .md (after a section's, which ends in a slash, index.html.md), as below.\n\n${sections.join("\n")}\n`),
  "presets.hash": `${sha.digest("hex")}\n`,
  "chain-facts.json": `${factsKey(facts)}\n`,
  "sitemap.xml": `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.map((url) => `<url><loc>${ORIGIN}${url.replaceAll("&", "&amp;")}</loc></url>`).join("\n")}\n</urlset>\n`,
};
// The addresses the presets once had (a chart's `was`, see addresses.js), each with what it opens: a preset's own
// address at its window. Kept in moved.txt, each opening the same for good: one that opens another now (renamed,
// taken by a new place) or nothing stops the build, written over only when meant (--drop).
if (book.clashes.length) {
  console.error(`Old addresses another place has now, ${book.clashes.length}:\n${book.clashes.map((clash) => `  ${clash}`).join("\n")}`);
  console.error("Name the new place otherwise: an old address is shared, and opens what it did.");
  process.exit(1);
}
/** What an address opens: a preset's address (at a window, if one), a section's, or nothing. @param {string} path */
const opens = (path) => {
  const to = book.at.get(path);
  if (!to) return "nothing";
  if (!("preset" in to)) return /** @type {string} */ (book.sections.get(JSON.stringify(to.path)));
  return to.window ? `${book.presets[to.preset]}#window=${to.window}` : book.presets[to.preset];
};
const live = new Set(urls), before = writtenBytes("page-info.json.gz");
const pageInfoBefore = before.length ? JSON.parse(gunzipSync(before).toString("utf8")) : {};
/** A page's series, in its order, each once. @param {any} info @returns {string[]} */
const seriesOf = (info) => [...new Set((info?.json_ld?.distribution ?? []).map((/** @type {{ name: string }} */ source) => source.name))];
/**
 * Where an address gone opens now: of the charts under the nearest section still there, the one showing the gone
 * page's series earliest in its order (its first, else its second, ...), then the one showing fewest besides (the
 * closest, not the busiest), then the first in the page's order; the section itself when none shows any of them.
 * @param {string} address
 */
const successor = (address) => {
  let section = address;
  do section = section.replace(/[^/]+\/?$/, ""); while (!live.has(section));
  const series = seriesOf(pageInfoBefore[address]);
  /** @type {{ chart: string, hits: boolean[], size: number } | null} */
  let best = null;
  for (const chart of book.presets.filter((chart) => chart.startsWith(section))) {
    const shows = new Set(seriesOf(pageInfo[chart])), hits = series.map((name) => shows.has(name));
    const first = best ? hits.findIndex((hit, i) => hit !== best.hits[i]) : -1;
    if (!best || (first >= 0 ? hits[first] : shows.size < best.size)) best = { chart, hits, size: shows.size };
  }
  return best?.hits.includes(true) ? best.chart : section;
};
const rows = written("moved.txt").split("\n").filter(Boolean).map((line) => line.split("\t"));
// An old address another opens now, kept once it's been one: a place moved (see places-were.js) has old addresses for
// what's under it now too, never had.
const sitemapBefore = [...written("sitemap.xml").matchAll(/<loc>([^<]*)<\/loc>/g)].map(([, loc]) => loc.replaceAll("&amp;", "&").slice(ORIGIN.length));
const known = new Set([...sitemapBefore, ...rows.map(([path]) => path)]);
const moved = new Map();
for (const [path] of book.at) if (known.has(path) && opens(path) !== path) moved.set(path, opens(path));
// (What a row opened is followed: a chart moved since opens at its new address, its window or state kept.)
const followed = (/** @type {string} */ to) => ((place, state) => (state === undefined ? opens(place) : `${opens(place).split("#")[0]}#${state}`))(...to.split("#"));
const changed = rows.filter(([path, to]) => book.at.has(path) && opens(path) !== followed(to));
if (changed.length) {
  console.error(`Moved addresses that open another place now, or nothing, ${changed.length}:\n${changed.slice(0, 12).map(([path, to]) => `  ${path}: ${to} → ${opens(path)}`).join("\n")}${changed.length > 12 ? `\n  and ${changed.length - 12} more` : ""}`);
  if (!drop) {
    console.error("Keep the places the presets had (a chart's `was`, presets.js), as their addresses are shared; meant to change: bun scripts/studio-seo.mjs --drop");
    process.exit(1);
  }
}
// The rest kept, each opening its place while that's there, else where its place moved (its own window kept), else its
// place's successor (a new page at one takes it).
for (const [path, to] of rows) {
  const [place, state] = to.split("#"), now = live.has(place) ? place : moved.get(place) ?? successor(place);
  if (!moved.has(path) && !live.has(path)) moved.set(path, state === undefined ? now : `${now.split("#")[0]}#${state}`);
}
// Charts gone since the sitemap was last written: a preset whose series the server renamed or took away goes without a
// word (its layer reads nothing, so it isn't made). Named, and each sent to its successor only when meant (--drop).
const gone = sitemapBefore.filter((path) => !live.has(path) && !moved.has(path));
if (gone.length && !drop) {
  console.error(`Gone since last written, ${gone.length} addresses:\n${gone.slice(0, 12).map((path) => `  ${path} → ${successor(path)}`).join("\n")}${gone.length > 12 ? `\n  and ${gone.length - 12} more` : ""}`);
  console.error("A series renamed or taken away on the server? Fix the presets; meant to go: bun scripts/studio-seo.mjs --drop");
  process.exit(1);
}
for (const path of gone) moved.set(path, successor(path));
files["moved.txt"] = [...moved].sort(([a], [b]) => (a < b ? -1 : 1)).map(([path, to]) => `${path}\t${to}\n`).join("");
const stale = [];
for (const [name, text] of Object.entries(files)) {
  if (!check) writeFileSync(join(STUDIO, name), text);
  else {
    const kept = writtenBytes(name);
    if (name === "page-info.json.gz" ? !kept.length || gunzipSync(kept).toString("utf8") !== pageInfoText : !kept.equals(Buffer.from(text))) stale.push(name);
  }
}
if (stale.length) {
  console.error(`Stale: ${stale.join(", ")}. Write them again: bun scripts/studio-seo.mjs`);
  process.exit(1);
}
console.log(`Page info: ${Object.keys(pageInfo).length} addresses, ${Buffer.byteLength(files["page-info.json.gz"])} bytes`);
console.log(`${check ? "Up to date" : "Written"}: llms.txt (${(files["llms.txt"].length / 1024).toFixed(0)} KB), sitemap.xml (${urls.length} addresses), presets.hash (${sources.size} modules), from ${check ? "the chain's facts they were written by" : `${SERVER}'s chain`}; ${shown.length} names shown keep the rules`);
