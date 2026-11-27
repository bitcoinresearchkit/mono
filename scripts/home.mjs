// The front page, kept in step with the apps it opens: each app's name and words read from its own page and written
// into the front page's FROM_APPS (its title before " · Bitview", else its launch tag's data-app; its meta
// description), and what it can't take from them checked: every app in the repository on it, every Studio chart and
// section it or another page links to in Studio's sitemap, every series it draws on the server, the pool logos' list
// every page draws from (assets/pools/logos.js) against that folder's files; who uses Bitview (assets/used-by/list.js)
// against its logos, and written where it's shown (docs/README.md, /dev's strip, /api's line). With --pictures, its pictures made
// again from the apps as they are now (each from its app's own screen, cut where PICTURES says): after an app's
// redesign, as well as its words; its share card (assets/home/og.jpg, 1200×630, as links to it show it), the first
// featured app's banner with Bitview's mark and name; every app's icon (assets/favicon/apps/, Bitview's the MCP
// server's too, and the touch icons at /apple-touch-icon.png and its -precomposed twin) and the favicon at /favicon.ico; and every page's share card (assets/cards/NAME.jpg, as CARDS makes it). Each
// page's head checked as a link to it previews and a search engine reads it, the sitemap (sitemap.xml) written from
// the rows, every preloaded file there, fallback fonts kept inline in every page that uses them and the craft rules (balanced headings, pretty text, equal-width digits) in every page, and the stars on GitHub written into every page that counts them. Run it after changing an app, and
// before a release:
//
//   bun scripts/home.mjs [server]              writes FROM_APPS and Bottoms' stored ranking (from bitview.space), then checks
//   bun scripts/home.mjs --check [server]      fails if FROM_APPS isn't what it'd be now, or a check fails
//   bun scripts/home.mjs --pictures [server]   also makes its pictures again (a Chromium: CHROMIUM=path, if not found), shot from this checkout's pages with bitview.space's data (server serves only the series check)
//   bun scripts/home.mjs --icons [server]      also makes every app's icon again, its pictures left (a colour changed)
//   ... --page FILE                              another page than website/index.html (a draft of it)
//   ... --pictures --only ID                     that app's pictures, icon and card alone (a new app's), the front
//                                                page's share card too if it's its picture
import { existsSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO = join(dirname(fileURLToPath(import.meta.url)), "..");
const SITE = join(REPO, "website");
const PROD = "https://bitview.space";
const args = process.argv.slice(2), flag = (/** @type {string} */ name) => args.includes(name);
const check = flag("--check"), icons = flag("--icons"), pictures = flag("--pictures") && !icons;
const pageAt = args.indexOf("--page"), PAGE = pageAt >= 0 ? args[pageAt + 1] : join(SITE, "index.html");
const onlyAt = args.indexOf("--only"), ONLY = onlyAt >= 0 ? args[onlyAt + 1] : null;
const SERVER = args.find((arg, i) => !arg.startsWith("--") && args[i - 1] !== "--page" && args[i - 1] !== "--only") ?? "http://127.0.0.1:3110";

let html = readFileSync(PAGE, "utf8");
/** What's wrong, all of it, said at the end. @type {string[]} */
const problems = [];

// ── Each app's name and words, from its own page ──

const named = /** @type {Record<string, string>} */ ({ amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", nbsp: " ", copy: "©", hellip: "…", mdash: "—", ndash: "–", rsquo: "’", lsquo: "‘", rdquo: "”", ldquo: "“" });
/** Text as a reader sees it: its entities, named or numbered, as characters; its spaces one. An entity it doesn't know, said. */
const plain = (/** @type {string} */ text, /** @type {string} */ where) =>
  text.replace(/&(#x[0-9a-f]+|#\d+|[a-z]+);/gi, (whole, name) => {
    if (name[0] === "#") return String.fromCodePoint(name[1] === "x" || name[1] === "X" ? parseInt(name.slice(2), 16) : Number(name.slice(1)));
    if (name in named) return named[name];
    problems.push(`${where}: an entity this script doesn't know, ${whole} (add it to named)`);
    return whole;
  }).replace(/\s+/g, " ").trim();
/** A tag's attributes, as HTML reads them: in any order, quoted either way or not at all. */
const attributes = (/** @type {string} */ tag) => Object.fromEntries([...tag.matchAll(/([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+))/g)].map(([, key, a, b, c]) => [key.toLowerCase(), a ?? b ?? c]));
/** The page's tags of a name, each its attributes. */
const tags = (/** @type {string} */ page, /** @type {string} */ name) => [...page.matchAll(new RegExp(`<${name}\\b([^>]*)>`, "gi"))].map(([, inside]) => attributes(inside));
/** An app's own page, by its address on the front page ("/oracle/"): its name and its words. */
function own(/** @type {string} */ href) {
  const file = join(SITE, href, "index.html");
  if (!existsSync(file)) return problems.push(`${href}: no such app (no ${file})`), null;
  const page = readFileSync(file, "utf8");
  const title = plain(/<title>([^<]*)<\/title>/i.exec(page)?.[1] ?? "", href);
  const app = tags(page, "script").find((tag) => tag.src?.includes("/launch.js"))?.["data-app"];
  const name = title.endsWith(" · Bitview") ? title.slice(0, -" · Bitview".length) : app && plain(app, href);
  const about = tags(page, "meta").find((tag) => tag.name?.toLowerCase() === "description")?.content;
  if (!name) problems.push(`${href}: no name (neither a title "… · Bitview" nor a data-app on its launch tag)`);
  if (!about) problems.push(`${href}: no meta description, its words on the front page`);
  return { name: name ?? "", about: plain(about ?? "", href) };
}
/** A list of the page's titles (APPS, DEV): each one's address. */
const hrefsOf = (/** @type {string} */ list) => [...(new RegExp(`const ${list} = [^\\[]*\\[([\\s\\S]*?)\\n      \\]`).exec(html)?.[1] ?? "").matchAll(/href: "(\/[a-z0-9-]+\/)"/g)].map(([, href]) => href);
const apps = hrefsOf("APPS"), dev = hrefsOf("DEV");
if (!apps.length) throw new Error(`No APPS in ${PAGE}: is it the front page?`);
// (An app's name and words; a developer's title named by the page, its words its page's.)
const from = Object.fromEntries([...apps.map((href) => [href, own(href)]), ...dev.map((href) => [href, { about: own(href)?.about ?? "" }])]);
// (Each value as JSON, its "<" escaped: no text of an app's can close the page's script.)
const literal = (/** @type {unknown} */ value) => JSON.stringify(value).replaceAll("<", "\\u003c");
const block = `/* from the apps */\n      const FROM_APPS = {\n${Object.entries(from).map(([href, said]) => `        ${literal(href)}: { ${Object.entries(/** @type {object} */ (said ?? {})).map(([key, value]) => `${literal(key)}: ${literal(value)}`).join(", ")} },`).join("\n")}\n      };\n      /* /from the apps */`;
const marked = /\/\* from the apps \*\/[\s\S]*?\/\* \/from the apps \*\//;
if (!marked.test(html)) throw new Error(`No "/* from the apps */" block in ${PAGE}`);
const written = html.replace(marked, () => block);
if (written !== html) {
  if (check) problems.push(`FROM_APPS isn't what the apps say now: run bun scripts/home.mjs`);
  else (writeFileSync(PAGE, written), (html = written), console.log(`FROM_APPS written: ${apps.length + dev.length} pages read`));
}

// ── The sitemap: the front page and every page in its rows (Studio's places in its own, studio/sitemap.xml) ──

const ORIGIN = "https://bitview.space";
const SITEMAP = join(SITE, "sitemap.xml");
const sitemapXml = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${["/", ...apps, ...dev].map((href) => `<url><loc>${ORIGIN}${href}</loc></url>\n`).join("")}</urlset>\n`;
if ((existsSync(SITEMAP) ? readFileSync(SITEMAP, "utf8") : "") !== sitemapXml) {
  if (check) problems.push("sitemap.xml isn't the front page's rows now: run bun scripts/home.mjs");
  else (writeFileSync(SITEMAP, sitemapXml), console.log(`sitemap.xml written: ${1 + apps.length + dev.length} pages`));
}

// ── Bottoms' stored ranking (bottoms/ranking.json): shown at once while the page ranks today's data, and where its
// reads start. Its worker run here on bitview.space's data, its last ranking kept, written when it changed (its day
// aside). Checked only for being there: a ranking older than the data is the page's to say. ──

if (check) { if (!existsSync(join(SITE, "bottoms", "ranking.json"))) problems.push("bottoms/ranking.json is missing: run bun scripts/home.mjs"); }
else {
  const RANKING = join(SITE, "bottoms", "ranking.json"), real = globalThis.fetch, was = globalThis.self;
  Bun.plugin({ name: "site root", setup: (build) => void build.onResolve({ filter: /^\/modules\// }, (at) => ({ path: join(SITE, at.path) })) });
  // (The worker asks its own origin, here bitview.space, and its own files as a server would answer them.)
  const own = (/** @type {URL} */ url) => (existsSync(fileURLToPath(url)) ? new Response(Bun.file(fileURLToPath(url))) : new Response(null, { status: 404 }));
  globalThis.fetch = Object.assign((/** @type {any} */ url, /** @type {any} */ init) => (url instanceof URL && url.protocol === "file:" ? Promise.resolve(own(url)) : real(typeof url === "string" && url.startsWith("/") ? PROD + url : url, init)), real);
  /** @type {any} */ let last = null;
  const ended = new Promise((done, fail) => {
    /** @type {any} */ (globalThis).self = { postMessage: (/** @type {any} */ message) => message.type === "ranked" && message.live ? (last = message) : message.type === "candles" ? done(last) : message.type === "failed" && fail(new Error(`bottoms/worker.js: ${message.why}`)) };
  });
  await import(join(SITE, "bottoms", "worker.js"));
  const { day, lows, ranked, ceilings } = await ended;
  (globalThis.fetch = real), (globalThis.self = was);
  const old = existsSync(RANKING) ? JSON.parse(readFileSync(RANKING, "utf8")) : null, now = { lows, ranked, ceilings };
  if (JSON.stringify(old && { lows: old.lows, ranked: old.ranked, ceilings: old.ceilings }) !== JSON.stringify(now)) writeFileSync(RANKING, `${JSON.stringify({ day, ...now })}\n`), console.log(`bottoms/ranking.json written: ${ranked.length} lines ranked`);
}

// ── Every app on it ──

// (An app: a folder of the repository's with a page, its name not starting with "_", an unlisted one's.)
const git = Bun.spawnSync(["git", "ls-files", "*/index.html"], { cwd: SITE });
if (git.exitCode !== 0) problems.push(`git ls-files failed (${new TextDecoder().decode(git.stderr).trim()}): the apps on the front page unchecked`);
// (Every page git knows, but one deleted and not yet committed as such.)
const pages = new TextDecoder().decode(git.stdout).split("\n").filter((file) => file && existsSync(join(SITE, file)));

// ── Every page, read once: the front page (or its --page draft), every app's page and the 404 (a write below keeps its text current) ──

/** Absolute path → text. */
const sources = new Map();
for (const at of new Set([resolve(PAGE), join(SITE, "index.html"), join(SITE, "404.html"), ...pages.map((file) => join(SITE, file))]))
  if (existsSync(at)) sources.set(at, at === resolve(PAGE) ? html : readFileSync(at, "utf8"));
const label = (/** @type {string} */ at) => (at.startsWith(SITE) ? relative(SITE, at) : at);

// ── Every file a page preloads, there (a font or module renamed would leave the hint asking for nothing) ──

for (const [at, page] of sources)
  for (const { rel, href } of tags(page, "link"))
    if ((rel === "preload" || rel === "modulepreload") && href?.startsWith("/") && !existsSync(join(SITE, href.split("?")[0]))) problems.push(`${label(at)}: preloads ${href}, no such file`);
// ── Metric-matched local faces, kept inline while the web fonts load ──

const fallbackFonts = `/* fallback fonts */
      /* Local faces keep text visible while the web fonts load. Width fits use a general Latin sample;
         vertical metrics come from Inter 4.1 and Lilex 2.620, divided by size-adjust.
         Menlo needs its full/PostScript name in Chromium. Other mono faces share its calibration. */
      @font-face {
        font-family: "Inter Fallback";
        src: local("ArialMT"), local("Arial"), local("Helvetica"), local("Roboto");
        font-weight: 100 600;
        size-adjust: 108.088066%;
        ascent-override: 89.625991%;
        descent-override: 22.316149%;
        line-gap-override: 0%;
      }
      @font-face {
        font-family: "Inter Fallback";
        src: local("Arial-BoldMT"), local("Arial Bold"), local("Helvetica Bold"), local("Roboto Bold");
        font-weight: 601 900;
        size-adjust: 99.756055%;
        ascent-override: 97.111900%;
        descent-override: 24.180080%;
        line-gap-override: 0%;
      }
      @font-face {
        font-family: "Lilex Fallback";
        src: local("Menlo"), local("Menlo-Regular"), local("SF Mono"), local("SFMono-Regular"), local("Consolas"), local("DejaVu Sans Mono"), local("Liberation Mono");
        font-weight: 500 700;
        size-adjust: 99.659367%;
        ascent-override: 100.241455%;
        descent-override: 30.202881%;
        line-gap-override: 0%;
      }
      /* /fallback fonts */`;
// ── The craft rules every page has (balanced headings, pretty text, equal-width digits), inline like the fallback fonts: no request ──

// (`html body` is more specific than any page's `body { font: … }`, which resets font-variant-numeric, so the digits rule
// wins wherever it sits; an element's own `font:` shorthand resets it again, and ui-checks' digits check finds those.)
const craftRules = `/* craft rules */
      /* Written into every page by home.mjs: headings balance their lines and text pretties its last, so no one-word last
         lines (zero specificity, a page's own text-wrap wins); digits are equal width, so a number that changes never
         shifts what sits beside it (a face without equal-width digits, like Instrument Serif, can't: its box is fixed). */
      :where(h1, h2, h3, h4, h5, h6) {
        text-wrap: balance;
      }
      :where(p, li, dd, blockquote, figcaption) {
        text-wrap: pretty;
      }
      html body {
        font-variant-numeric: tabular-nums;
      }
      /* /craft rules */`;
// (Each block replaced whole, so a page holds exactly one opening marker and one closing marker after it, or the run
// stops at it rather than cut what lies between. A page without one that wants it gets it at the top of its first <style>.)
const BLOCKS = [
  { name: "fallback fonts", text: fallbackFonts, wanted: (/** @type {string} */ page) => /Inter Fallback|Lilex Fallback/.test(page) },
  { name: "craft rules", text: craftRules, wanted: () => true },
];
for (const [at, was] of sources) {
  let now = was;
  for (const { name, text, wanted } of BLOCKS) {
    const open = `/* ${name} */`, close = `/* /${name} */`, from = now.indexOf(open), to = now.indexOf(close);
    if (from < 0 && to < 0) {
      if (wanted(now)) now = now.replace(/<style>\n?/, (tag) => `${tag}      ${text}\n`);
    } else if (from < 0 || to < from || now.split(open).length !== 2 || now.split(close).length !== 2) {
      problems.push(`${label(at)}: needs exactly one "${open}" and one "${close}" after it`);
    } else now = now.slice(0, from) + text + now.slice(to + close.length);
    if (name === "craft rules" && !/<style>/.test(now)) problems.push(`${label(at)}: no <style> to hold the craft rules`);
  }
  if (now === was) continue;
  if (check) problems.push(`${label(at)}: its inline blocks (fallback fonts, craft rules) are stale: run bun scripts/home.mjs`);
  else (writeFileSync(at, now), sources.set(at, now), at === resolve(PAGE) && (html = now), console.log(`${label(at)}: inline blocks written`));
}

const folders = pages.map((file) => file.split("/")[0]).filter((name) => !name.startsWith("_"));
// (On it: in its rows, APPS or DEV, or linked from its footer; or unlisted, in UNLISTED: served at its address,
// with its icon, but off the front page and the sitemap, until it's ready to be listed.)
const footer = tags(/<footer\b[^>]*>([\s\S]*?)<\/footer>/i.exec(html)?.[1] ?? "", "a").map((tag) => tag.href);
const UNLISTED = ["/ledger/"];
for (const href of UNLISTED) if ([...apps, ...dev, ...footer].includes(href)) problems.push(`${href}: on the front page and in UNLISTED (take it out of UNLISTED)`);
for (const name of folders)
  if (![...apps, ...dev, ...footer, ...UNLISTED].includes(`/${name}/`))
    problems.push(`/${name}/: a page not on the front page (add it to APPS or DEV, link it in the footer, add it to UNLISTED, or name its folder _${name} to keep it off)`);

// ── The stars on GitHub ──

// (How many have starred the repository, as GitHub says now, written into every page's count, a link's
// <b class="stars-count">: pages never ask GitHub themselves, so nothing waits on it, moves as it answers or fails, and no
// node tells it who reads them. Two figures and a "+", rounded down, so a count stays true as more star it. Left as it
// is when GitHub doesn't answer; never checked, as it moves on its own.)
/** A count as two figures and a "+", rounded down: 176 "170+", 1,234 "1.2k+", 47,321 "47k+", 1,250,000 "1.2M+". */
const atLeast = (/** @type {number} */ n) => {
  const power = n >= 1e6 ? 2 : n >= 1e3 ? 1 : 0, value = n / 1000 ** power;
  const step = 10 ** (Math.floor(Math.log10(Math.max(value, 1))) - 1);
  return `${+(Math.floor(value / step + 1e-9) * step).toFixed(1)}${["", "k", "M"][power]}+`;
};
if (!check) {
  const repo = await fetch("https://api.github.com/repos/bitcoinresearchkit/mono", { signal: AbortSignal.timeout(10_000) }).then((r) => (r.ok ? r.json() : null)).catch(() => null);
  if (typeof repo?.stargazers_count !== "number") console.log("The stars on GitHub: GitHub didn't answer, every page's count left as it is");
  else {
    const count = `<b class="stars-count">${atLeast(repo.stargazers_count)}</b>`, written = [];
    for (const [at, was] of sources) {
      const now = was.replace(/<b class="stars-count">[^<]*<\/b>/g, count);
      if (now !== was) (writeFileSync(at, now), sources.set(at, now), written.push(label(at)), at === resolve(PAGE) && (html = now));
    }
    console.log(`The stars on GitHub: ${repo.stargazers_count}, written as ${atLeast(repo.stargazers_count)}${written.length ? ` into ${written.join(", ")}` : ", as every page has it"}`);
  }
}

// ── Every app's icon ──

// (Each app's, the front page's "bitview": Bitview's cube (assets/favicon/app-icon.svg) in its launch tag's colour
// (data-color; the SVG's own orange if none), a file (assets/favicon/apps/NAME.png, as launch.js links it): a home
// screen takes one best; Bitview's also the MCP server's, MCP_ICON. Made with --pictures or --icons. A page without
// a launch tag, as the footer's legal ones, links no icon, so has none.)
const MCP_ICON = join(REPO, "crates", "bitview_server", "mcp", "logo.png");
/** The touch icons at the root, Bitview's own icon: iOS asks for the precomposed one first, then the other, and link
 * previews for either. */
const TOUCH_ICONS = ["apple-touch-icon.png", "apple-touch-icon-precomposed.png"];
const ICONS = /** @type {Record<string, string | undefined>} */ (Object.fromEntries([
  ["bitview", undefined],
  ...folders.flatMap((name) => {
    const launch = tags(readFileSync(join(SITE, name, "index.html"), "utf8"), "script").find((tag) => tag.src?.includes("/launch.js"));
    return launch ? [[name, launch["data-color"]]] : [];
  }),
]));

// ── Each page's head: its address, and how a link to it previews ──

/** Every page's share card's making (assets/cards/NAME.jpg, 1200×630, made with --pictures; with the brand, not in
 * the page's folder, which a server route can shadow, as /api/ is the API's): its address, its
 * screen's size if not 1200×630 (drawn at twice the density, the card shrunk to fit), and what's done on it
 * first, if anything (Logs: only series answered, no one's addresses in a picture that lasts). */
const CARDS = {
  studio: { url: "/studio/presets/holders/age-bands/compare/realized-cap-share", screen: [1600, 840] },
  oracle: { url: "/oracle/" },
  explorer: { url: "/explorer/" },
  globe: { url: "/globe/" },
  trail: { url: "/trail/" },
  stack: { url: "/stack/" },
  compass: { url: "/compass/" },
  bedrock: { url: "/bedrock/" },
  bottoms: { url: "/bottoms/" },
  calendar: { url: "/calendar/" },
  miners: { url: "/miners/" },
  catalog: { url: "/catalog/?series=hashrate" },
  logs: { url: "/logs/", run: `document.querySelector('button[data-group="statuses"][data-key$=":2"]').click(); document.getElementById("search").value = "/api/series/"; document.getElementById("search").dispatchEvent(new Event("input"))` },
  api: { url: "/api/" },
  changes: { url: "/changes/" },
  dev: { url: "/dev/" },
  brk: { url: "/brk/" },
  benchmarks: { url: "/benchmarks/" },
};
// (Every page in the rows: its canonical address and its preview's its own on bitview.space, wherever it's served from
// (next.bitview.space, someone's own server: one page, not copies); its preview named, described and pictured, the
// picture its card, shown large. Studio's address is each place's, set as it opens, so its own isn't checked.)
for (const href of [...apps, ...dev]) {
  const file = join(SITE, href, "index.html");
  if (!existsSync(file)) continue;
  const page = readFileSync(file, "utf8"), head = page.slice(0, page.indexOf("</head>")), url = `${ORIGIN}${href}`;
  const meta = Object.fromEntries(tags(head, "meta").map((tag) => [tag.property ?? tag.name, tag.content]));
  const wrong = (/** @type {string} */ what) => problems.push(`${href}: ${what}`);
  if (href !== "/studio/") {
    if (tags(head, "link").find((tag) => tag.rel === "canonical")?.href !== url) wrong(`its canonical link isn't ${url}`);
    if (meta["og:url"] !== url) wrong(`its og:url isn't ${url}`);
  }
  for (const key of ["og:type", "og:site_name", "og:title", "og:description", "og:image:alt"]) if (!meta[key]) wrong(`no ${key}`);
  if (meta["twitter:card"] !== "summary_large_image") wrong("its twitter:card isn't summary_large_image");
  if (meta["og:image:width"] !== "1200" || meta["og:image:height"] !== "630") wrong("its og:image isn't said 1200×630");
  const image = meta["og:image"]?.startsWith(`${ORIGIN}/`) ? join(SITE, meta["og:image"].slice(ORIGIN.length)) : null;
  const name = href.slice(1, -1);
  if (!(name in CARDS)) wrong("no card's making in CARDS");
  else if (image !== join(SITE, "assets", "cards", `${name}.jpg`)) wrong(`its og:image isn't its card, ${ORIGIN}/assets/cards/${name}.jpg`);
}

// ── Every way into Studio a page has (this one's, and every other app's), in Studio's sitemap ──

const sitemap = new Set([...readFileSync(join(SITE, "studio", "sitemap.xml"), "utf8").matchAll(/<loc>[^<]*\/studio\/presets\/([^<]*)<\/loc>/g)].map(([, path]) => path));
const topics = [...html.matchAll(/\["([a-z0-9-]+)", "[^"]+", "#[0-9a-f]{6}"(?:, "([a-z0-9_]+)")?\]/g)];
// (Each address whole, to its quote, space, a sentence's comma or semicolon, or the view after its "#": a chart renamed by a
// suffix isn't taken for its former self; one made in the page from a template, as the topics' are, read from TOPICS
// instead.)
const studioPaths = [...new Set([
  ...[...sources.values()].flatMap((page) => [...page.matchAll(/\/studio\/presets\/([^"'`\s<>),;#]*)/g)].map(([, path]) => path)).filter((path) => !path.includes("${")),
  ...topics.map(([, slug]) => `${slug}/`),
])];
if (!topics.length) problems.push("No TOPICS rows found: the page changed? (this script reads them)");
for (const path of studioPaths) if (!sitemap.has(path)) problems.push(`/studio/presets/${path}: not in Studio's sitemap, exactly (a chart or section renamed?)`);

// ── The pool logos every page draws from (assets/pools/logos.js, written by crates/bitview_primitives/pools.sh), the folder's ──

{
  const { LOGOS, LIGHT } = await import(join(SITE, "assets", "pools", "logos.js"));
  const files = new Set(readdirSync(join(SITE, "assets", "pools")).filter((file) => file.endsWith(".svg")));
  const listed = new Set([...[...LOGOS].map((slug) => `${slug}.svg`), ...[...LIGHT].map((slug) => `${slug}.light.svg`)]);
  if (files.symmetricDifference(listed).size) problems.push("assets/pools/logos.js isn't the folder's logos: run crates/bitview_primitives/pools.sh");
}

// ── Who uses Bitview (assets/used-by/list.js), written where it's shown: the README, the project page's strip, the API
// page's line; and the list checked against its folder's logos ──

{
  const FOLDER = join(SITE, "assets", "used-by");
  const { USERS } = /** @type {typeof import("../website/assets/used-by/list.js")} */ (await import(join(FOLDER, "list.js")));
  const files = new Set(readdirSync(FOLDER).filter((file) => file !== "list.js" && !file.startsWith(".")));
  const listed = USERS.flatMap(({ logo, dark }) => (dark ? [logo, dark] : [logo]));
  /** A logo's width over its height, from its file: a PNG's header, an SVG's viewBox; NaN for anything else. */
  const ratio = (/** @type {string} */ file) => {
    const bytes = readFileSync(join(FOLDER, file));
    if (file.endsWith(".png")) return bytes.readUInt32BE(0) === 0x89504e47 ? bytes.readUInt32BE(16) / bytes.readUInt32BE(20) : NaN;
    const box = bytes.toString().match(/viewBox="[\d.-]+[ ,]+[\d.-]+[ ,]+([\d.]+)[ ,]+([\d.]+)"/);
    return box ? Number(box[1]) / Number(box[2]) : NaN;
  };
  const wrong = [
    ...listed.filter((file, i) => listed.indexOf(file) !== i).map((file) => `${file} listed twice`),
    ...[...files.difference(new Set(listed))].map((file) => `${file} not listed`),
    ...[...new Set(listed).difference(files)].map((file) => `${file} listed but not there`),
    ...listed.filter((file) => !/^[a-z0-9-]+\.(png|svg)$/.test(file)).map((file) => `${file} isn't a plain name.png or name.svg`),
    ...(USERS.length < 3 ? ["fewer than three users, while the API page's line names two and others"] : []),
  ];
  /** @type {Map<string, number>} Each file's ratio, read once every file is there and named as it should be. */
  const ratios = new Map();
  if (!wrong.length) {
    for (const file of listed) ratios.set(file, ratio(file));
    for (const [file, r] of ratios) if (!(r > 0)) wrong.push(`${file} has no size to read (a PNG, or an SVG with a viewBox)`);
    for (const { logo, dark } of USERS)
      if (dark && Math.abs(Number(ratios.get(dark)) / Number(ratios.get(logo)) - 1) > 0.01) wrong.push(`${dark} isn't ${logo}'s shape`);
  }
  if (wrong.length) problems.push(`assets/used-by/list.js isn't the folder's logos: ${wrong.join("; ")}`);
  else {
    /** Text for HTML, an attribute's too. */
    const escape = (/** @type {string} */ text) => text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
    /** Each user with its logo's ratio and its scale (1 unless set): what both writers size it by. */
    const users = USERS.map(({ scale = 1, ...user }) => ({ ...user, scale, ratio: Number(ratios.get(user.logo)) }));
    // The README: each logo as published, its dark one in GitHub's dark mode, 22 px high at scale 1, about GitHub's
    // text (/dev's are 24 px beside its 17 px names), sized so nothing moves as they load, and centred on the line by
    // `absmiddle` (GitHub keeps `align`; "middle" sits on the baseline); a mark with its name beside it (a line may
    // still break between them: browsers allow a break beside any image, a no-break space included, and GitHub strips
    // the styles that could stop it).
    const readme = users.map(({ name, url, logo, dark, mark, scale, ratio }) => {
      const at = "../website/assets/used-by/", height = Math.round(22 * scale), width = Math.round(height * ratio);
      const img = `<img src="${at}${logo}" width="${width}" height="${height}" align="absmiddle" alt="${mark ? "" : escape(name)}" />`;
      const picture = dark ? `<picture><source media="(prefers-color-scheme: dark)" srcset="${at}${dark}" />${img}</picture>` : img;
      return `<a href="${escape(url)}">${picture}${mark ? ` ${escape(name)}` : ""}</a>`;
    });
    // /dev: each logo's shape in the page's ink (a mask, so one file reads on light and dark), its width from its
    // ratio, so nothing moves as it loads; the list twice for the strip's seamless loop, the copy hidden from readers
    // and from Tab; beside the label, the drift's pause (a toggle the page's script shows).
    const strip = (/** @type {boolean} */ copy) =>
      users.map(({ name, url, logo, mark, scale, ratio }) => {
        const style = `--logo: url(/assets/used-by/${logo}); --scale: ${scale}; aspect-ratio: ${+ratio.toFixed(3)}`;
        const label = mark ? "" : ` aria-label="${escape(name)}"`;
        return `<li><a href="${escape(url)}"${label}${copy ? ` tabindex="-1"` : ""}><span class="logo" style="${style}"></span>${mark ? escape(name) : ""}</a></li>`;
      }).join("");
    const [a, b] = users.map((user) => escape(user.name));
    /** Each block replaced whole between its markers, exactly one of each in its file. @type {[string, string][]} */
    const blocks = [
      [join(REPO, "docs", "README.md"), `**Used by**\n\n${readme.join("&emsp;\n")}\n`],
      [join(SITE, "dev", "index.html"), `<div class="used-by" id="used-by"><div class="head"><p class="label" id="used-by-label">Used by</p><button class="pause" type="button" aria-pressed="false" aria-label="Pause the logos" hidden></button></div><div class="strip"><div class="track" style="--users: ${users.length}"><ul aria-labelledby="used-by-label">${strip(false)}</ul><ul aria-hidden="true">${strip(true)}</ul></div></div></div>`],
      [join(SITE, "api", "index.html"), `<template id="used-by"><p class="used-by"><a href="/dev/#used-by">Used by ${a}, ${b} and others →</a></p></template>`],
    ];
    for (const [at, block] of blocks) {
      const file = relative(REPO, at), was = readFileSync(at, "utf8");
      // (Each marker on a line of its own; the block indented as the opening one is.)
      const marked = /^( *)<!-- used by -->\n[\s\S]*?^ *<!-- \/used by -->$/gm, found = [...was.matchAll(marked)];
      if (found.length !== 1 || was.split("<!-- used by -->").length !== 2) {
        problems.push(`${file}: needs exactly one "<!-- used by -->" and one "<!-- /used by -->" after it, each on its own line`);
        continue;
      }
      const indent = found[0][1], lines = [`<!-- used by -->`, ...block.trimEnd().split("\n"), `<!-- /used by -->`];
      const now = was.replace(marked, () => lines.map((line) => (line ? indent + line : line)).join("\n"));
      if (now === was) continue;
      if (check) problems.push(`${file}: "Used by" isn't assets/used-by/list.js now: run bun scripts/home.mjs`);
      else (writeFileSync(at, now), console.log(`${file}: "Used by" written`));
    }
  }
}

// ── Every series it draws, on the server ──

const ids = new Set([
  ...[...html.matchAll(/series: \[([^\]]*)\]/g)].flatMap(([, list]) => [...list.matchAll(/"([a-z0-9_]+)"/g)].map(([, id]) => id)),
  ...topics.flatMap(([, , id]) => (id ? [id] : [])),
  ...[...html.matchAll(/series=([a-z0-9_,]+)/g)].flatMap(([, list]) => list.split(",")),
]);
if (!ids.size) problems.push("No series found: the page changed? (this script reads its CHARTS, TOPICS and series= reads)");
const answers = await Promise.all([...ids].map((id) => fetch(`${SERVER}/api/series/${id}/month1?start=-1`, { signal: AbortSignal.timeout(10_000) }).then((r) => [id, r.status], () => [id, 0])));
if (answers.every(([, status]) => status === 0)) problems.push(`${SERVER} isn't answering: series unchecked (is it running?)`);
else for (const [id, status] of answers) if (status !== 200) problems.push(`series ${id}: the server answers ${status} (renamed?)`);

// ── Its pictures, made again from the apps ──

/** How long a page is let paint once its network has gone quiet (its data in, its chart drawn): a card or a cover's own,
 * no page's guess of its own loading. */
const PAINT = 2000;

/** Each format's quality, Chromium's own encoders: the lowest that shows no loss at twice the density of a phone, in the
 * hardest pictures (Studio's gradients mottle at 30, Oracle's dark cells go muddy at 50 in a JPEG and 60 in a WebP). The
 * share cards stay JPEG (links to them are read by X, which takes no WebP); the pictures our own pages show are WebP. */
const QUALITY = { jpeg: 60, webp: 70 };

/** Each picture's making: its app's address (its screen's size, if not 1440×900), what's hidden
 * ("text": a canvas's words, "scene": everything but its canvas, and its tags), and where its cover and its banner are
 * cut, in its screen's points: [x, y, width, height]. A cover 960×600 and 720 wide (@720, the apps' grid's, from
 * phones to a 1440 desk at 2×); a banner 1,900 wide and 2,880 (@2x). */
const PICTURES = {
  studio: { url: "/studio/presets/holders/age-bands/compare/realized-cap-share", screen: [2600, 1150], cover: [682, 70, 1488, 930], banner: [270, 70, 1900, 930] },
  oracle: { url: "/oracle/", cover: [0, 228, 928, 580], banner: [0, 244, 1440, 628] },
  explorer: { url: "/explorer/", cover: [8, 60, 1344, 840] },
  globe: { url: "/globe/", hide: "text", cover: [166, 44, 720, 450], banner: [43, 64, 966, 450] },
  trail: { url: "/trail/", hide: "scene", cover: [230, 130, 960, 600], banner: [60, 80, 1360, 640] },
  stack: { url: "/stack/", cover: [192, 336, 640, 400], banner: [192, 341, 648, 300] },
  compass: { url: "/compass/", cover: [130, 60, 1088, 680] },
  bedrock: { url: "/bedrock/", cover: [660, 110, 768, 480] },
  bottoms: { url: "/bottoms/", cover: [44, 186, 992, 620] },
  calendar: { url: "/calendar/", cover: [105, 100, 384, 240], banner: [105, 100, 863, 400] },
  miners: { url: "/miners/", cover: [240, 40, 960, 600] },
};
if (onlyAt >= 0 && !(ONLY && (Object.hasOwn(PICTURES, ONLY) || Object.hasOwn(ICONS, ONLY) || Object.hasOwn(CARDS, ONLY)))) throw new Error(`--only ${ONLY ?? ""}: not an app (${[...new Set([...Object.keys(PICTURES), ...Object.keys(ICONS), ...Object.keys(CARDS)])].join(", ")})`);
const featured = [...(/const FEATURED = \[([^\]]*)\]/.exec(html)?.[1] ?? "").matchAll(/"([a-z]+)"/g)].map(([, id]) => id);
const appIds = [...(/const APPS = [^\[]*\[([\s\S]*?)\n      \]/.exec(html)?.[1] ?? "").matchAll(/id: "([a-z]+)"/g)].map(([, id]) => id);
for (const id of appIds) if (!(id in PICTURES)) problems.push(`${id}: no cover's making in PICTURES`);
for (const id of featured) if (!(/** @type {any} */ (PICTURES)[id]?.banner)) problems.push(`${id}: featured, but no banner's making in PICTURES`);

// ── The first title, in the page itself: FEATURED's first, as its own page names it and as APPS says it ──

const first = featured[0], entry = new RegExp(`\\{ id: "${first}", kind: "([^"]*)", line: "([^"]*)", href: "([^"]*)"`).exec(html);
const shown = {
  name: plain(/id="bb-title"[^>]*>([^<]*)</.exec(html)?.[1] ?? "", "the billboard"),
  kind: /id="bb-kind">([^<]*)</.exec(html)?.[1],
  line: /id="bb-blurb">([^<]*)</.exec(html)?.[1],
  href: /id="bb-open" href="([^"]*)"/.exec(html)?.[1],
  picture: /<div class="art"[^>]*><img[^>]* src="\/assets\/home\/banners\/([a-z]+)\.webp"/.exec(html)?.[1],
};
if (!entry) problems.push(`FEATURED's first, ${first}: not in APPS as this script reads it`);
else {
  const want = { name: from[entry[3]]?.name, kind: entry[1], line: entry[2], href: entry[3], picture: first };
  for (const [key, value] of Object.entries(want)) if (shown[/** @type {keyof typeof shown} */ (key)] !== value) problems.push(`The billboard's first title in the page: its ${key} is ${JSON.stringify(shown[/** @type {keyof typeof shown} */ (key)])}, not ${JSON.stringify(value)} (${first}'s)`);
}

/** The share card's page: the app's banner full bleed, darkened left, where Bitview's mark, name and line sit. */
const SHARE_CARD = (/** @type {string} */ id) => `<!doctype html><meta charset="utf-8"><style>
  @font-face { font-family: Inter; src: url("/assets/fonts/InterVariable.woff2") format("woff2"); font-weight: 100 900; }
  html, body { margin: 0; width: 1200px; height: 630px; overflow: hidden; background: #0d0c0b; }
  body { position: relative; font-family: Inter, sans-serif; color: #f6f3ee; -webkit-font-smoothing: antialiased; }
  img.art { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; object-position: 70% 0; }
  .shade { position: absolute; inset: 0; background: linear-gradient(to right, #0d0c0b 0, rgb(13 12 11 / 0.92) 34%, rgb(13 12 11 / 0.35) 62%, transparent 82%), linear-gradient(to top, rgb(13 12 11 / 0.7), transparent 40%); }
  .words { position: absolute; left: 72px; bottom: 76px; width: 560px; }
  .brand { display: flex; align-items: center; gap: 22px; font-size: 76px; font-weight: 750; letter-spacing: -0.04em; }
  .brand img { width: 68px; height: 79px; }
  p { margin: 22px 0 0; color: #d6cfc4; font-size: 28px; line-height: 1.35; letter-spacing: -0.01em; }
</style><img class="art" src="/assets/home/banners/${id}@2x.webp"><div class="shade"></div>
<div class="words"><div class="brand"><img src="/assets/logo/logo.svg">Bitview</div><p>Bitcoin's chain, read and measured: live apps, thousands of charts, and every series as data.</p></div>`;

// ── Its share card, named in its head ──

if (!/<meta property="og:image" content="https:\/\/bitview\.space\/assets\/home\/og\.jpg"/.test(html)) problems.push("The page's og:image isn't its share card, /assets/home/og.jpg");
if (!existsSync(join(SITE, "assets", "home", "og.jpg"))) problems.push("No share card, assets/home/og.jpg: run with --pictures");

if (pictures || icons) await makePictures();
// (Every app's icon there, the favicon, and every card, once any were made.)
for (const name of Object.keys(ICONS)) if (!existsSync(join(SITE, "assets", "favicon", "apps", `${name}.png`))) problems.push(`${name}: no icon, assets/favicon/apps/${name}.png: run with --icons`);
for (const file of ["favicon.ico", ...TOUCH_ICONS]) if (!existsSync(join(SITE, file))) problems.push(`No ${file}: run with --icons`);
for (const name of Object.keys(CARDS)) if (!existsSync(join(SITE, "assets", "cards", `${name}.jpg`))) problems.push(`${name}: no card, assets/cards/${name}.jpg: run with --pictures --only ${name}`);

if (problems.length) {
  console.error(`${problems.length} to see to:\n${problems.map((p) => `  - ${p}`).join("\n")}`);
  process.exit(1);
}
console.log(`In step: ${apps.length} apps and ${dev.length} developers' pages, ${studioPaths.length} ways into Studio, ${ids.size} series.`);

/** This checkout's website on a free port, the way bitviewd serves it (a folder's index.html; an app's own address with no
 * file, as /studio/presets/…, its nearest folder's page), and every other path (the API, /version) from prod. Pictures
 * are this checkout's pages with the data readers see; SERVER's own node may be behind or at another tip, and its numbers
 * would be in every card. A worker's fetches are relative, so only an origin of its own can send them to prod. */
function serveSite() {
  return Bun.serve({ port: 0, idleTimeout: 120, async fetch(req) {
    const { pathname, search } = new URL(req.url), path = decodeURIComponent(pathname);
    let file = resolve(SITE, "." + path);
    if (file.startsWith(SITE) && !existsSync(file) && !path.startsWith("/api/") && !/\.[a-z0-9]+$/i.test(path))
      for (let dir = dirname(file); dir.startsWith(SITE + "/"); dir = dirname(dir)) if (existsSync(join(dir, "index.html"))) { file = dir; break; }
    if (file.startsWith(SITE) && existsSync(file)) {
      if (statSync(file).isDirectory()) file = join(file, "index.html");
      if (existsSync(file)) return new Response(Bun.file(file));
    }
    const r = await fetch(PROD + pathname + search);
    // (fetch hands the body decoded: its encoding and length go with it.)
    const headers = new Headers(r.headers);
    headers.delete("content-encoding"), headers.delete("content-length");
    return new Response(r.body, { status: r.status, headers });
  } });
}

/** The pictures: a Chromium, headless, each app's screen as a dark one with less motion, each picture cut from it at the
 * density its size needs. */
async function makePictures() {
  const binary = process.env.CHROMIUM ?? ["/Applications/Chromium.app/Contents/MacOS/Chromium", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/usr/bin/chromium", "/usr/bin/google-chrome"].find(existsSync);
  if (!binary) return void problems.push("No Chromium for --pictures or --icons: set CHROMIUM to one");
  const site = serveSite(), origin = String(site.url).replace(/\/$/, "");
  const profile = mkdtempSync(join(tmpdir(), "home-pictures-"));
  /** @type {ReturnType<typeof Bun.spawn> | null} */
  let browser = null;
  try {
    browser = Bun.spawn([binary, "--headless=new", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--hide-scrollbars", "about:blank"], { stderr: "ignore", stdout: "ignore" });
    let port = "";
    for (let tries = 0; !port && tries < 100; tries++) {
      await Bun.sleep(100);
      if (existsSync(join(profile, "DevToolsActivePort"))) port = readFileSync(join(profile, "DevToolsActivePort"), "utf8").split("\n")[0];
    }
    if (!port) return void problems.push("The Chromium for --pictures didn't start");
    const out = join(SITE, "assets", "home");
    for (const [id, recipe] of Object.entries(pictures ? PICTURES : {})) {
      if (ONLY && id !== ONLY) continue;
      const wanted = /** @type {[string, number[], number][]} */ ([
        ...(appIds.includes(id) ? [[`covers/${id}.webp`, recipe.cover, 960], [`covers/${id}@720.webp`, recipe.cover, 720]] : []),
        ...(featured.includes(id) && "banner" in recipe ? [[`banners/${id}.webp`, recipe.banner, 1900], [`banners/${id}@2x.webp`, recipe.banner, 2880]] : []),
      ]);
      if (!wanted.length) continue;
      // (One screen at the density the largest cut needs.)
      const density = Math.max(...wanted.map(([, box, width]) => Math.ceil((width / box[2]) * 10) / 10), 1);
      // (An app that fails: said, its pictures kept as they were, the others made.)
      const tab = await screen(port, `${origin}${recipe.url}`, recipe.screen ?? [1440, 900], density, PAINT, /** @type {any} */ (recipe).hide).catch((error) => (problems.push(`${id}'s pictures: ${error.message}`), null));
      if (!tab) continue;
      try {
        for (const [file, [x, y, w, h], width] of wanted) {
          const shot = await tab.send("Page.captureScreenshot", { format: "webp", quality: QUALITY.webp, clip: { x, y, width: w, height: h, scale: width / (w * density) } });
          writeFileSync(join(out, file), Buffer.from(shot.data, "base64"));
          console.log(`made ${file}`);
        }
      } catch (error) {
        problems.push(`${id}'s pictures: ${/** @type {Error} */ (error).message}`);
      } finally {
        await tab.close();
      }
    }
    // (Each app's icon, at 512 px: a home screen's 180, a manifest's 512.)
    const svg = readFileSync(join(SITE, "assets", "favicon", "app-icon.svg"), "utf8");
    for (const [name, color] of Object.entries(pictures || icons ? ICONS : {})) {
      if (ONLY && name !== ONLY) continue;
      const icon = (color ? svg.replace(/--color: [^;"]+/, `--color: ${color}`) : svg).replace("<svg", '<svg width="512" height="512"');
      const tab = await screen(port, `${origin}/llms.txt`, [512, 512], 1, 300, undefined, `<style>body { margin: 0 } svg { display: block }</style>${icon}`);
      try {
        const shot = await tab.send("Page.captureScreenshot", { format: "png", clip: { x: 0, y: 0, width: 512, height: 512, scale: 1 } });
        writeFileSync(join(SITE, "assets", "favicon", "apps", `${name}.png`), Buffer.from(shot.data, "base64"));
        console.log(`made apps/${name}.png`);
        // (Bitview's is the MCP server's icon too: its page's and the MCP registry's; and the one at the root, for what asks
        // there rather than reading a page's, as iOS and link previews do.)
        if (name === "bitview") {
          for (const file of [MCP_ICON, ...TOUCH_ICONS.map((icon) => join(SITE, icon))]) writeFileSync(file, Buffer.from(shot.data, "base64"));
          console.log(`made the MCP server's logo.png, ${TOUCH_ICONS.join(" and ")}`);
        }
      } finally {
        await tab.close();
      }
    }
    // (The favicon at /favicon.ico, for what asks there rather than reading a page's: favicon.svg at 16, 32 and 48 px,
    // clear around it, each a PNG in one ICO.)
    if (!ONLY || ONLY === "bitview") {
      const svg = readFileSync(join(SITE, "assets", "favicon", "favicon.svg"), "utf8"), sizes = [16, 32, 48], pngs = [];
      for (const size of sizes) {
        const tab = await screen(port, `${origin}/llms.txt`, [64, 64], 1, 300, undefined, `<style>html, body { margin: 0; background: transparent } svg { display: block }</style>${svg.replace("<svg", `<svg width="${size}" height="${size}"`)}`);
        try {
          await tab.send("Emulation.setDefaultBackgroundColorOverride", { color: { r: 0, g: 0, b: 0, a: 0 } });
          pngs.push(Buffer.from((await tab.send("Page.captureScreenshot", { format: "png", clip: { x: 0, y: 0, width: size, height: size, scale: 1 } })).data, "base64"));
        } finally {
          await tab.close();
        }
      }
      writeFileSync(join(SITE, "favicon.ico"), ico(sizes, pngs));
      console.log("made favicon.ico");
    }
    // (Every page's share card: its own screen, as it opens.)
    for (const [name, recipe] of Object.entries(pictures ? CARDS : {})) {
      if (ONLY && name !== ONLY) continue;
      const [width, height] = /** @type {any} */ (recipe).screen ?? [1200, 630];
      const tab = await screen(port, `${origin}${recipe.url}`, [width, height], 2, PAINT).catch((error) => (problems.push(`${name}'s card: ${error.message}`), null));
      if (!tab) continue;
      try {
        if ("run" in recipe) await tab.send("Runtime.evaluate", { expression: recipe.run }), await Bun.sleep(1000);
        const shot = await tab.send("Page.captureScreenshot", { format: "jpeg", quality: QUALITY.jpeg, clip: { x: 0, y: 0, width, height, scale: 1200 / width / 2 } });
        writeFileSync(join(SITE, "assets", "cards", `${name}.jpg`), Buffer.from(shot.data, "base64"));
        console.log(`made cards/${name}.jpg`);
      } finally {
        await tab.close();
      }
    }
    // (The share card: the billboard's first picture, Bitview's mark and name over it, set as the page is.)
    if (!pictures || (ONLY && ONLY !== featured[0])) return;
    const card = await screen(port, `${origin}/llms.txt`, [1200, 630], 2, 1500, undefined, SHARE_CARD(featured[0]));
    try {
      const shot = await card.send("Page.captureScreenshot", { format: "jpeg", quality: QUALITY.jpeg, clip: { x: 0, y: 0, width: 1200, height: 630, scale: 0.5 } });
      writeFileSync(join(out, "og.jpg"), Buffer.from(shot.data, "base64"));
      console.log("made og.jpg");
    } finally {
      await card.close();
    }
  } catch (error) {
    problems.push(`--pictures: ${/** @type {Error} */ (error).message}`);
  } finally {
    browser?.kill();
    await browser?.exited;
    await site.stop(true);
    rmSync(profile, { recursive: true, force: true });
  }
}

/** An ICO of PNGs, one a size (each square): its header, a directory entry each (256 px said as 0), then the PNGs. */
function ico(/** @type {number[]} */ sizes, /** @type {Buffer[]} */ pngs) {
  const head = Buffer.alloc(6 + 16 * pngs.length);
  head.writeUInt16LE(1, 2);
  head.writeUInt16LE(pngs.length, 4);
  let offset = head.length;
  pngs.forEach((png, i) => {
    const at = 6 + 16 * i;
    head.writeUInt8(sizes[i] % 256, at);
    head.writeUInt8(sizes[i] % 256, at + 1);
    head.writeUInt16LE(1, at + 4);
    head.writeUInt16LE(32, at + 6);
    head.writeUInt32LE(png.length, at + 8);
    head.writeUInt32LE(offset, at + 12);
    offset += png.length;
  });
  return Buffer.concat([head, ...pngs]);
}

/** A page in a new tab, as a phone-free desk sees it, dark, less motion; its network quiet (a page still loading after a
 * minute fails, loudly) and `settle` ms more to paint; its words or all but its scene hidden;
 * or, given its HTML, that page in its place (its address's origin kept, for its files). */
async function screen(/** @type {string} */ port, /** @type {string} */ url, /** @type {number[]} */ [width, height], /** @type {number} */ density, /** @type {number} */ settle, /** @type {string | undefined} */ hide, /** @type {string | undefined} */ html) {
  const target = await (await fetch(`http://127.0.0.1:${port}/json/new?about:blank`, { method: "PUT" })).json();
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  const deadline = (/** @type {Promise<any>} */ promise, /** @type {string} */ what, ms = 30_000) => Promise.race([promise, Bun.sleep(ms).then(() => Promise.reject(new Error(`${what}: no answer in ${ms / 1000} s`)))]);
  await deadline(new Promise((ok, no) => (ws.addEventListener("open", ok, { once: true }), ws.addEventListener("error", () => no(new Error("its inspector refused")), { once: true }))), "its inspector");
  let id = 0;
  /** Each command waiting: its answer's and its failure's. @type {Map<number, [(value: any) => void, (error: Error) => void]>} */
  const waiting = new Map();
  ws.addEventListener("message", (event) => {
    const message = JSON.parse(String(event.data)), pending = message.id && waiting.get(message.id);
    if (!pending) return;
    waiting.delete(message.id);
    if (message.error) pending[1](new Error(message.error.message));
    else pending[0](message.result ?? {});
  });
  ws.addEventListener("close", () => {
    for (const [, [, no]] of waiting) no(new Error("its inspector closed"));
    waiting.clear();
  });
  const send = (/** @type {string} */ method, params = {}) => deadline(new Promise((ok, no) => {
    waiting.set(++id, [ok, no]);
    ws.send(JSON.stringify({ id, method, params }));
  }), method);
  await send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: density, mobile: false });
  await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: "dark" }, { name: "prefers-reduced-motion", value: "reduce" }] });
  await send("Page.enable");
  await send("Page.setLifecycleEventsEnabled", { enabled: true });
  /** The page loads whose network has gone quiet: no request for half a second. @type {Set<string>} */
  const quiet = new Set();
  ws.addEventListener("message", (event) => {
    const message = JSON.parse(String(event.data));
    if (message.method === "Page.lifecycleEvent" && message.params.name === "networkIdle") quiet.add(message.params.loaderId);
  });
  // (A canvas's words: not drawn; a scene's tags, its rounded boxes, too.)
  if (hide) await send("Page.addScriptToEvaluateOnNewDocument", { source: `const P = CanvasRenderingContext2D.prototype; P.fillText = P.strokeText = ${hide === "scene" ? "P.roundRect = " : ""}function () {};` });
  const went = await send("Page.navigate", { url });
  if (went.errorText) throw new Error(`${url}: ${went.errorText}`);
  for (let waited = 0; !quiet.has(went.loaderId); waited += 100) {
    if (waited >= 60_000) throw new Error(`${url}: its network wasn't quiet in 60 s (still loading?)`);
    await Bun.sleep(100);
  }
  if (html) {
    await Bun.sleep(300);
    const tree = await send("Page.getFrameTree");
    await send("Page.setDocumentContent", { frameId: tree.frameTree.frame.id, html });
  }
  await Bun.sleep(settle);
  if (hide === "scene") await send("Runtime.evaluate", { expression: `for (const e of document.querySelectorAll("body *")) e.style.visibility = "hidden"; for (const c of document.querySelectorAll("canvas")) c.style.visibility = "visible";` });
  await Bun.sleep(500);
  return { send, close: () => (ws.close(), fetch(`http://127.0.0.1:${port}/json/close/${target.id}`)) };
}
