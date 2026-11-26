// bun scripts/budget.mjs: Studio's first chart against its byte budget (budget.json:
// KB on the wire before it draws, brotli q11), from the files; exit 1 when a kind is over. The pre-commit hook runs it.
//  - html: the page without its <style>s; css: those, and any stylesheet it links (what's loaded later, isn't);
//  - js: every script the page names, and all they import statically (an import() is what comes after the draw);
//  - font: what it preloads (a face is fetched for the words in it: the page preloads the one a first screen's are in).
import { brotliCompressSync, constants } from "node:zlib";
import { readFileSync } from "node:fs";
import { join, normalize } from "node:path";

const root = join(import.meta.dir, "../website"), budget = JSON.parse(readFileSync(join(import.meta.dir, "budget.json"), "utf8")).kb;
const q11 = (/** @type {Buffer | string} */ b) => brotliCompressSync(b, { params: { [constants.BROTLI_PARAM_QUALITY]: 11, [constants.BROTLI_PARAM_LGWIN]: 24 } }).length;
const fileOf = (/** @type {string} */ url) => join(root, normalize(url.split(/[?#]/)[0]));
const html = readFileSync(join(root, "studio/index.html"), "utf8");
const styles = [...html.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]).join("\n");
const tags = [...html.matchAll(/<(?:script|link)\b[^>]*>/g)].map((m) => m[0]), attr = (/** @type {string} */ tag, /** @type {string} */ name) => tag.match(new RegExp(`\\s${name}="([^"]*)"`))?.[1];
const have = { html: q11(html.replace(/<style[^>]*>[\s\S]*?<\/style>/g, "")), css: q11(styles), js: 0, font: 0 };
for (const t of tags) if (attr(t, "rel") === "stylesheet" && attr(t, "href")?.startsWith("/")) have.css += q11(readFileSync(fileOf(/** @type {string} */ (attr(t, "href")))));

const queue = tags.flatMap((t) => (t.startsWith("<script") ? [attr(t, "src")] : attr(t, "rel") === "modulepreload" ? [attr(t, "href")] : [])).filter((u) => u?.startsWith("/"));
for (const seen = new Set(); queue.length; ) {
  const url = /** @type {string} */ (queue.pop());
  if (seen.has(url)) continue;
  seen.add(url);
  const source = readFileSync(fileOf(url));
  have.js += q11(source);
  for (const [, to] of source.toString().matchAll(/(?:^|[;}\s])(?:import|export)\s*(?:[^'";()]*?from\s*)?["']([^"']+)["']/g)) if (to.startsWith("/") || to.startsWith(".")) queue.push(new URL(to, `http://x${url}`).pathname);
}

for (const t of tags) if (attr(t, "as") === "font") have.font += readFileSync(fileOf(/** @type {string} */ (attr(t, "href")))).length;

let over = false;
console.log("kind      KB now  budget");
for (const [kind, limit] of Object.entries(budget)) {
  const kb = Math.round(have[/** @type {keyof typeof have} */ (kind)] / 100) / 10;
  over ||= kb > limit;
  console.log(kind.padEnd(7), String(kb).padStart(8), String(limit).padStart(7), kb > limit ? ` OVER by ${(kb - limit).toFixed(1)} KB` : "");
}
process.exit(over ? 1 : 0);
