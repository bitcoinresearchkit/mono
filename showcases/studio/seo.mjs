// Beside the page, for those who find Studio: sitemap.xml, every address it has (the Charts page, the presets' page,
// each section, each preset: the page's own code makes them, read from it), and llms.txt, its guide for assistants (the
// page's own "about" section, as Markdown, with the presets' sections at their addresses). Made from the presets as a
// server's chain makes them (the local one's by default), and written again whenever the presets or the guide change:
//
//   bun showcases/studio/seo.mjs [server]          writes them
//   bun showcases/studio/seo.mjs --check [server]  fails if they're not what they'd be now
//   bun showcases/studio/seo.mjs --drop [server]   writes them though charts have gone (see below)
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const check = process.argv.includes("--check"), drop = process.argv.includes("--drop");
const SERVER = process.argv.slice(2).find((arg) => !arg.startsWith("--")) ?? "http://127.0.0.1:3110";
// (The address Studio's known by, on every domain it's served on: see ORIGIN in the page.)
const ORIGIN = "https://bitview.space/studio";

const html = readFileSync(join(HERE, "index.html"), "utf8");
const { BitviewClient } = await import(pathToFileURL(join(HERE, "..", "modules", "bitview-client", "index.js")).href);
const { studioPresets, chainFacts } = await import(pathToFileURL(join(HERE, "presets.js")).href);
// (Without the chain's facts the presets would hold what never happened: stopped, not written.)
const facts = await chainFacts(SERVER).catch((error) => {
  throw new Error(`${SERVER}'s chain facts unread (${error.message}): is it running?`);
});
const presets = studioPresets(new BitviewClient({ baseUrl: SERVER }), facts);

const code = /\/\* addresses:[^*]*\*\/([\s\S]*?)\/\* \/addresses \*\//.exec(html)?.[1];
if (!code) throw new Error("No addresses in the page: the sitemap is made from them");
const addressesOf = /** @type {(list: { p: string[] }[]) => { presets: string[], sections: Map<string, string> }} */ (new Function(`${code}\nreturn addressesOf;`)());
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
const held = new Map();
for (const preset of presets) for (let depth = 1; depth < preset.p.length && depth <= 2; depth++) ((key) => held.set(key, (held.get(key) ?? 0) + 1))(JSON.stringify(preset.p.slice(0, depth)));
const sections = [...held].map(([key, count]) => `${JSON.parse(key).length > 1 ? "  " : ""}- ${JSON.parse(key).at(-1)} (${count}): ${ORIGIN}${book.sections.get(key)}`);
// (In plain ASCII where it can be: it's served as a text file is, without a charset.)
const ascii = (/** @type {string} */ text) => text.replace(/[…⋯]/g, "...").replace(/›/g, ">").replace(/[−–—]/g, "-").replace(/[‘’]/g, "'").replace(/[“”]/g, '"');
const urls = ["/", ...book.sections.values(), ...book.presets];

const files = {
  "llms.txt": ascii(`${markdown}\n\n## The presets' sections\n\nEach section's page lists its charts; each chart's own address is in ${ORIGIN}/sitemap.xml.\n\n${sections.join("\n")}\n`),
  "sitemap.xml": `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.map((url) => `<url><loc>${ORIGIN}${url.replaceAll("&", "&amp;")}</loc></url>`).join("\n")}\n</urlset>\n`,
};
const written = (/** @type {string} */ name) => { try { return readFileSync(join(HERE, name), "utf8"); } catch { return ""; } };
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
console.log(`${check ? "Up to date" : "Written"}: llms.txt (${(files["llms.txt"].length / 1024).toFixed(0)} KB), sitemap.xml (${urls.length} addresses), from ${SERVER}'s chain`);
