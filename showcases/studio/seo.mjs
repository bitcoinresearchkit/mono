// Beside the page, for those who find Studio: sitemap.xml, every address it has (Home, the presets' page,
// each section, each preset: made by addresses.js, the page's own), llms.txt, its guide for assistants (the page's own
// "about" section, as Markdown, with the presets' sections at their addresses), and presets.hash, what the page keys
// its presets' cache by (see below). Made from the presets as a server's chain makes them (the local one's by default;
// its facts kept beside them, chain-facts.json) and the client's series, their titles checked against the rules every
// title keeps (see TITLE_RULES), and written again whenever the presets, the guide or the client change:
//
//   bun showcases/studio/seo.mjs [server]          writes them, by the server's chain now
//   bun showcases/studio/seo.mjs --check           fails if the code makes them otherwise now (by the chain's facts
//                                                  they were written by: what the chain did since fails nothing)
//   bun showcases/studio/seo.mjs --drop [server]   writes them though charts have gone (see below)
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { addressesOf } from "./addresses.js";
import { CHAIN_STATISTICS } from "./chain-statistics.js";
import { shownStep } from "./words.js";

const HERE = dirname(fileURLToPath(import.meta.url));
const check = process.argv.includes("--check"), drop = process.argv.includes("--drop");
const SERVER = process.argv.slice(2).find((arg) => !arg.startsWith("--")) ?? "http://127.0.0.1:3110";
// (The address Studio's known by, on every domain it's served on: see ORIGIN in the page.)
const ORIGIN = "https://bitview.space/studio";

const html = readFileSync(join(HERE, "index.html"), "utf8");
const { BitviewClient } = await import(pathToFileURL(join(HERE, "..", "modules", "bitview-client", "index.js")).href);
const { studioPresets } = await import(pathToFileURL(join(HERE, "presets.js")).href);
const { chainFacts, factsKey, factsOfKey } = await import(pathToFileURL(join(HERE, "facts.js")).href);
const written = (/** @type {string} */ name) => { try { return readFileSync(join(HERE, name), "utf8"); } catch { return ""; } };
// (Checked, by the facts they were written by; written, by the server's now. Without them the presets would hold what
// never happened: stopped.)
const kept = written("chain-facts.json").trim();
if (check && !kept) throw new Error("No chain-facts.json: write them first, bun showcases/studio/seo.mjs");
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
// (Chain statistics' charts keep the names of the site they reproduce: no rule of these is theirs.)
const ruled = presets.filter((preset) => !preset.p.includes(CHAIN_STATISTICS));
/** Every step of every place, once, by its key. @type {Map<string, string[]>} */
const steps = new Map();
for (const preset of ruled) for (let i = 1; i <= preset.p.length; i++) steps.set(JSON.stringify(preset.p.slice(0, i)), preset.p.slice(0, i));
/** Every name shown, each with whether it's a title and its place: titles, legends, the tree's steps as shown. @type {[string, boolean, string[]][]} */
const shown = [
  ...ruled.flatMap((preset) => [[preset.t, true, preset.p], ...preset.c.layers.flatMap((layer) => (layer.label ? [[layer.label, false, preset.p]] : []))]),
  ...[...steps.values()].map((path) => [shownStep(labels, path), false, path]),
];
const broken = shown.flatMap(([text, title, path]) => TITLE_RULES.flatMap(([rule, breaks, titles]) => ((title || !titles) && breaks(text) ? [`${rule}: "${text}" (${path.join(" › ")})`] : [])));
if (broken.length) throw new Error(`${broken.length} name${broken.length === 1 ? "" : "s"} shown break${broken.length === 1 ? "s" : ""} the rules:\n  ${broken.slice(0, 20).join("\n  ")}${broken.length > 20 ? `\n  and ${broken.length - 20} more` : ""}`);
// The presets the empty chart offers first (the page's RECOMMENDED), each one of them: a place renamed would drop it.
const recommended = /const RECOMMENDED = \[([\s\S]*?)\];/.exec(html)?.[1];
if (!recommended) throw new Error("No RECOMMENDED in the page");
const places = new Set(presets.map((preset) => preset.p.join(" › ")));
const unknown = [...recommended.matchAll(/"([^"]+)"/g)].map(([, place]) => place).filter((place) => !places.has(place));
if (unknown.length) throw new Error(`Recommended, but no preset there: ${unknown.join("; ")}`);

const book = addressesOf(presets);

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
const sections = [...held].map(([key, count]) => {
  const path = JSON.parse(key);
  return `${path.length > 1 ? "  " : ""}- ${shownStep(labels, path)} (${count}): ${ORIGIN}${book.sections.get(key)}`;
});
// (In plain ASCII where it can be: it's served as a text file is, without a charset.)
const ascii = (/** @type {string} */ text) => text.replace(/[…⋯]/g, "...").replace(/›/g, ">").replace(/[−–—]/g, "-").replace(/[‘’]/g, "'").replace(/[“”]/g, '"');
const urls = ["/", ...book.sections.values(), ...book.presets];
// The presets' source, presets.js and every module it imports (followed by their relative imports, as Bun reads them,
// each once), hashed, each by its path from here (the same on every system): the page keys its presets' cache by it
// (with the client's and the chain's facts), so a change to any of them makes them again, with nothing read in the page
// to know it. (Not the client: it's made by the server's build, which never runs this; the page hashes it itself.)
const sources = new Set([join(HERE, "presets.js")]), transpiler = new Bun.Transpiler({ loader: "js" }), sha = createHash("sha256");
for (const file of sources) {
  const text = readFileSync(file, "utf8");
  sha.update(`${relative(HERE, file).replaceAll(sep, "/")}\0${text}\0`);
  for (const { path } of transpiler.scanImports(text)) if (path.startsWith(".")) sources.add(join(dirname(file), path));
}

const files = {
  "llms.txt": ascii(`${markdown}\n\n## The presets' sections\n\nEach section's page lists its charts; each chart's own address is in ${ORIGIN}/sitemap.xml.\n\n${sections.join("\n")}\n`),
  "presets.hash": `${sha.digest("hex")}\n`,
  "chain-facts.json": `${factsKey(facts)}\n`,
  "sitemap.xml": `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.map((url) => `<url><loc>${ORIGIN}${url.replaceAll("&", "&amp;")}</loc></url>`).join("\n")}\n</urlset>\n`,
};
// Charts gone since the sitemap was last written: a preset whose series the server renamed or took away goes without a
// word (its layer reads nothing, so it isn't made). Named, and written over only when meant (--drop).
const now = new Set(urls.map((url) => `${ORIGIN}${url}`));
const gone = [...written("sitemap.xml").matchAll(/<loc>([^<]*)<\/loc>/g)].map(([, loc]) => loc.replaceAll("&amp;", "&")).filter((loc) => !now.has(loc));
if (gone.length && !drop) {
  console.error(`Gone since last written, ${gone.length} addresses:\n${gone.slice(0, 12).map((loc) => `  ${loc}`).join("\n")}${gone.length > 12 ? `\n  and ${gone.length - 12} more` : ""}`);
  console.error("A series renamed or taken away on the server? Fix the presets; meant to go: bun showcases/studio/seo.mjs --drop");
  process.exit(1);
}
const stale = [];
for (const [name, text] of Object.entries(files)) {
  if (!check) writeFileSync(join(HERE, name), text);
  else if (written(name) !== text) stale.push(name);
}
if (stale.length) {
  console.error(`Stale: ${stale.join(", ")}. Write them again: bun showcases/studio/seo.mjs`);
  process.exit(1);
}
console.log(`${check ? "Up to date" : "Written"}: llms.txt (${(files["llms.txt"].length / 1024).toFixed(0)} KB), sitemap.xml (${urls.length} addresses), presets.hash (${sources.size} modules), from ${check ? "the chain's facts they were written by" : `${SERVER}'s chain`}; ${shown.length} names shown keep the rules`);
