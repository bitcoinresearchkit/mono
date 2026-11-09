// The front page, kept in step with the apps it opens: each app's name and words read from its own page and written
// into the front page's FROM_APPS (its title before " · Bitview", else its launch tag's data-app; its meta
// description), and what it can't take from them checked: every app in the repository on it, every Studio chart and
// section it links to in Studio's sitemap, every series it draws on the server. With --pictures, its pictures made
// again from the apps as they are now (each from its app's own screen, cut where PICTURES says): after an app's
// redesign, as well as its words; its share card (assets/home/og.jpg, 1200×630, as links to it show it), the first
// featured app's banner with Bitview's mark and name; every app's icon (assets/favicon/apps/, Bitview's the MCP
// server's too) and the favicon at /favicon.ico; and every page's share card (assets/cards/NAME.jpg, as CARDS makes it). Each
// page's head checked as a link to it previews and a search engine reads it, the sitemap (sitemap.xml) written from
// the rows, and the stars on GitHub written into every page that counts them. Run it after changing an app, and
// before a release:
//
//   bun showcases/home.mjs [server]              writes FROM_APPS, then checks
//   bun showcases/home.mjs --check [server]      fails if FROM_APPS isn't what it'd be now, or a check fails
//   bun showcases/home.mjs --pictures [server]   also makes its pictures again (a Chromium: CHROMIUM=path, if not found)
//   bun showcases/home.mjs --icons [server]      also makes every app's icon again, its pictures left (a colour changed)
//   ... --page FILE                              another page than showcases/index.html (a draft of it)
//   ... --pictures --only ID                     that app's pictures, icon and card alone (a new app's), the front
//                                                page's share card too if it's its picture
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2), flag = (/** @type {string} */ name) => args.includes(name);
const check = flag("--check"), icons = flag("--icons"), pictures = flag("--pictures") && !icons;
const pageAt = args.indexOf("--page"), PAGE = pageAt >= 0 ? args[pageAt + 1] : join(HERE, "index.html");
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
/** A page's meta tags by name. */
const metaOf = (/** @type {string} */ page) => Object.fromEntries(tags(page.slice(0, page.indexOf("</head>")), "meta").map((tag) => [tag.property ?? tag.name, tag.content]));
function own(/** @type {string} */ href) {
  const file = join(HERE, href, "index.html");
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
  if (check) problems.push(`FROM_APPS isn't what the apps say now: run bun showcases/home.mjs`);
  else (writeFileSync(PAGE, written), (html = written), console.log(`FROM_APPS written: ${apps.length + dev.length} pages read`));
}

// ── The sitemap: the front page and every page in its rows (Studio's places in its own, studio/sitemap.xml) ──

const ORIGIN = "https://bitview.space";
const SITEMAP = join(HERE, "sitemap.xml");
const sitemapXml = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${["/", ...apps, ...dev].map((href) => `<url><loc>${ORIGIN}${href}</loc></url>\n`).join("")}</urlset>\n`;
if ((existsSync(SITEMAP) ? readFileSync(SITEMAP, "utf8") : "") !== sitemapXml) {
  if (check) problems.push("sitemap.xml isn't the front page's rows now: run bun showcases/home.mjs");
  else (writeFileSync(SITEMAP, sitemapXml), console.log(`sitemap.xml written: ${1 + apps.length + dev.length} pages`));
}

// ── Every app on it ──

// (An app: a folder of the repository's with a page, its name not starting with "_", an unlisted one's.)
const git = Bun.spawnSync(["git", "ls-files", "*/index.html"], { cwd: HERE });
if (git.exitCode !== 0) problems.push(`git ls-files failed (${new TextDecoder().decode(git.stderr).trim()}): the apps on the front page unchecked`);
const folders = new TextDecoder().decode(git.stdout).split("\n").filter(Boolean).map((file) => file.split("/")[0]).filter((name) => !name.startsWith("_"));
// (On it: in its rows, APPS or DEV, the footer's links not counting; or unlisted, in UNLISTED: served at its address,
// with its icon, but off the front page and the sitemap, until it's ready to be listed.)
const UNLISTED = ["/ledger/"];
for (const href of UNLISTED) if ([...apps, ...dev].includes(href)) problems.push(`${href}: in the front page's rows and in UNLISTED (take it out of UNLISTED)`);
for (const name of folders)
  if (![...apps, ...dev, ...UNLISTED].includes(`/${name}/`))
    problems.push(`/${name}/: an app not in the front page's rows (add it to APPS or DEV, to UNLISTED to serve it unlisted, or name its folder _${name} to keep it off)`);

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
    // (Every page git knows, but one deleted and not yet committed as such.)
    for (const file of ["index.html", ...new TextDecoder().decode(git.stdout).split("\n").filter(Boolean)].filter((file) => existsSync(join(HERE, file)))) {
      const at = join(HERE, file), was = readFileSync(at, "utf8"), now = was.replace(/<b class="stars-count">[^<]*<\/b>/g, count);
      if (now !== was) (writeFileSync(at, now), written.push(file), at === PAGE && (html = now));
    }
    console.log(`The stars on GitHub: ${repo.stargazers_count}, written as ${atLeast(repo.stargazers_count)}${written.length ? ` into ${written.join(", ")}` : ", as every page has it"}`);
  }
}

// ── Every app's icon ──

// (Each app's, the front page's "bitview": Bitview's cube (assets/favicon/app-icon.svg) in its launch tag's colour
// (data-color; the SVG's own orange if none), a file (assets/favicon/apps/NAME.png, as launch.js links it): a home
// screen takes one best; Bitview's also the MCP server's, MCP_ICON. Made with --pictures or --icons.)
const MCP_ICON = join(HERE, "..", "crates", "bitview_mcp", "assets", "logo.png");
const ICONS = /** @type {Record<string, string | undefined>} */ (Object.fromEntries([
  ["bitview", undefined],
  ...folders.map((name) => [name, tags(readFileSync(join(HERE, name, "index.html"), "utf8"), "script").find((tag) => tag.src?.includes("/launch.js"))?.["data-color"]]),
]));

// ── Each page's head: its address, and how a link to it previews ──

/** Every page's share card's making (assets/cards/NAME.jpg, 1200×630, made with --pictures; with the brand, not in
 * the page's folder, which a server route can shadow, as /api/ is the API's): its address, how long it's let
 * settle, its screen's size if not 1200×630 (drawn at twice the density, the card shrunk to fit), and what's done on it
 * first, if anything (Logs: only series answered, no one's addresses in a picture that lasts). */
const CARDS = {
  studio: { url: "/studio/presets/bedrock/awake/floors-colored", screen: [1600, 840], settle: 16000 },
  oracle: { url: "/oracle/", settle: 9000 },
  explorer: { url: "/explorer/", settle: 9000 },
  globe: { url: "/globe/", settle: 9000 },
  trail: { url: "/trail/", settle: 12000 },
  stack: { url: "/stack/", settle: 8000 },
  compass: { url: "/compass/", settle: 6000 },
  calendar: { url: "/calendar/", settle: 9000 },
  catalog: { url: "/catalog/?series=hashrate", settle: 8000 },
  logs: { url: "/logs/", settle: 6000, run: `document.querySelector('button[data-group="statuses"][data-key$=":2"]').click(); document.getElementById("search").value = "/api/series/"; document.getElementById("search").dispatchEvent(new Event("input"))` },
  api: { url: "/api/", settle: 6000 },
  changes: { url: "/changes/", settle: 6000 },
  dev: { url: "/dev/", settle: 4000 },
  brk: { url: "/brk/", settle: 4000 },
  benchmarks: { url: "/benchmarks/", settle: 8000 },
};
// (Every page in the rows: its canonical address and its preview's its own on bitview.space, wherever it's served from
// (next.bitview.space, someone's own server: one page, not copies); its preview named, described and pictured, the
// picture its card, shown large. Studio's address is each place's, set as it opens, so its own isn't checked.)
for (const href of [...apps, ...dev]) {
  const file = join(HERE, href, "index.html");
  if (!existsSync(file)) continue;
  const page = readFileSync(file, "utf8"), head = page.slice(0, page.indexOf("</head>")), url = `${ORIGIN}${href}`;
  const meta = metaOf(page);
  const wrong = (/** @type {string} */ what) => problems.push(`${href}: ${what}`);
  if (href !== "/studio/") {
    if (tags(head, "link").find((tag) => tag.rel === "canonical")?.href !== url) wrong(`its canonical link isn't ${url}`);
    if (meta["og:url"] !== url) wrong(`its og:url isn't ${url}`);
  }
  for (const key of ["og:type", "og:site_name", "og:title", "og:description", "og:image:alt"]) if (!meta[key]) wrong(`no ${key}`);
  if (meta["twitter:card"] !== "summary_large_image") wrong("its twitter:card isn't summary_large_image");
  if (meta["og:image:width"] !== "1200" || meta["og:image:height"] !== "630") wrong("its og:image isn't said 1200×630");
  const image = meta["og:image"]?.startsWith(`${ORIGIN}/`) ? join(HERE, meta["og:image"].slice(ORIGIN.length)) : null;
  const name = href.slice(1, -1);
  if (!(name in CARDS)) wrong("no card's making in CARDS");
  else if (image !== join(HERE, "assets", "cards", `${name}.jpg`)) wrong(`its og:image isn't its card, ${ORIGIN}/assets/cards/${name}.jpg`);
}

// ── Every way into Studio it has, in Studio's sitemap ──

const sitemap = new Set([...readFileSync(join(HERE, "studio", "sitemap.xml"), "utf8").matchAll(/<loc>[^<]*\/studio\/presets\/([^<]*)<\/loc>/g)].map(([, path]) => path));
const topics = [...html.matchAll(/\["([a-z0-9-]+)", "[^"]+", "#[0-9a-f]{6}"(?:, "([a-z0-9_]+)")?\]/g)];
// (Each address whole, to its quote or space: a chart renamed by a suffix isn't taken for its former self; one made in the
// page from a template, as the topics' are, read from TOPICS instead.)
const studioPaths = [...new Set([...[...html.matchAll(/\/studio\/presets\/([^"'`\s<>)]*)/g)].map(([, path]) => path).filter((path) => !path.includes("${")), ...topics.map(([, slug]) => `${slug}/`)])];
if (!topics.length) problems.push("No TOPICS rows found: the page changed? (this script reads them)");
for (const path of studioPaths) if (!sitemap.has(path)) problems.push(`/studio/presets/${path}: not in Studio's sitemap, exactly (a chart or section renamed?)`);

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

/** Each picture's making: its app's address (its screen's size, if not 1440×900), how long it's let settle, what's hidden
 * ("text": a canvas's words, "scene": everything but its canvas, and its tags), and where its banner is cut, in its
 * screen's points: [x, y, width, height]. A banner 1,900 wide, the first title's also 2,880 (@2x). */
const PICTURES = {
  studio: { url: "/studio/presets/bedrock/awake/floors-colored", screen: [2400, 1150], settle: 16000, focus: "#frame, #chart canvas", margin: 1, banner: true, run: `{ const P = CanvasRenderingContext2D.prototype, stroke = P.stroke; P.stroke = function (...a) { if (!this.getLineDash().length) stroke.apply(this, a); }; const b = document.querySelector("#chart").getBoundingClientRect(); document.body.prepend(Object.assign(document.createElement("div"), { id: "frame" })); document.head.append(Object.assign(document.createElement("style"), { textContent: "#frame { position: fixed; left: 265px; top: 80px; width: 1721px; height: 968px } #chart { position: fixed !important; left: " + b.x + "px !important; top: " + b.y + "px !important; width: " + b.width + "px !important; height: " + b.height + "px !important }" })); document.querySelector("#chart canvas").dispatchEvent(new WheelEvent("wheel", { bubbles: true, cancelable: true, deltaY: 0, clientX: 1986, clientY: 500 })); }`, clip: { bitrate: 400_000, run: `{ const c = document.querySelector("#chart canvas"); Element.prototype.setPointerCapture = () => {}; window.ev = (type, x) => c.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, clientX: x, clientY: 500, pointerId: 4, pointerType: "pen", isPrimary: true, button: 0, buttons: 1 })); ev("pointerdown", 1000); }`, each: `(p) => ev("pointermove", 1000 + 240 * (p - Math.sin(2 * Math.PI * p) / (2 * Math.PI) * 0.5))` } },
  oracle: {"url": "/oracle/", "screen": [1800, 1100], "settle": 9000, "focus": "#frame, #plate", "margin": 1, "ground": "#0e1110", "banner": true, "run": "document.body.prepend(Object.assign(document.createElement('div'), {id:'frame'})); document.head.append(Object.assign(document.createElement('style'), {textContent: '#frame {position:fixed;left:0;top:0;width:1800px;height:1012px} #plate {position:fixed;left:420px;top:51px;width:1380px;height:600px} #spectrum {filter:brightness(1.8)} #spectrum text, #spectrum .rule, #spectrum .fit-area, #spectrum .fit-line, #spectrum .window, #spectrum .here {display:none!important}'}));", "clip": {"seconds": 7, "bitrate": 470000, "each": "(p) => { const step=Math.floor(p*80); if(window.__toothStep!==step){document.querySelector('#plate').dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight'}));window.__toothStep=step;} }", "run": "for(let i=0;i<40;i++) document.querySelector('#plate').dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowLeft'}));"}},
  explorer: { url: "/explorer/", screen: [1920, 1080], settle: 9000, focus: "#frame, #map", margin: 1, banner: true, ran: 6000, run: `document.body.prepend(Object.assign(document.createElement("div"), { id: "frame" })); document.head.append(Object.assign(document.createElement("style"), { textContent: "#frame { position: fixed; left: 0; top: 0; width: 1920px; height: 1080px } #map { position: fixed !important; left: 120px; top: -620px; width: 1800px !important; border-radius: 0 !important }" }));`, clip: { bitrate: 200_000, run: `{ const f = window.fetch; window.held = []; window.fetch = (u, o) => f(u, o).then((r) => String(u).includes("/diff/") ? new Promise((ok) => held.push(() => ok(r))) : r); new Promise((ok) => setTimeout(ok, 5000)); }`, each: `(p) => { if ((window.step = (window.step ?? -1) + 1) === 60) held.splice(0).forEach((go) => go()); }` } },
  globe: { url: "/globe/", screen: [1920, 1080], settle: 9000, hide: "text", focus: "#frame, #globe", margin: 1, banner: true, run: `document.body.prepend(Object.assign(document.createElement("div"), { id: "frame" })); document.head.append(Object.assign(document.createElement("style"), { textContent: "#frame { position: fixed; left: 0; top: 0; width: 1920px; height: 1080px } #globe { position: fixed !important; left: 488px; top: -222px; width: 1724px !important; height: 1724px !important; max-width: none !important; max-height: none !important; mask: none !important; -webkit-mask: none !important }" }));`, clip: { bitrate: 470_000, run: `window.drag = (type, x, y) => document.querySelector("#globe").dispatchEvent(new PointerEvent(type, { bubbles: true, clientX: x, clientY: y, pointerId: 1, isPrimary: true, pointerType: "mouse", button: 0, buttons: 1 })); drag("pointerdown", 900, 500);`, each: `(p) => { const e = p - Math.sin(2 * Math.PI * p) / (2 * Math.PI) * 0.6; drag("pointermove", 900 + e * 180, 500 + e * 24); }` } },
  trail: { url: "/trail/", settle: 12000, hide: "scene", focus: "#frame, #terrain", margin: 1, banner: true, ran: 7000, run: `{ for (const P of [CanvasRenderingContext2D.prototype, OffscreenCanvasRenderingContext2D.prototype]) { const stroke = P.stroke; P.stroke = function (...a) { if (!/0\\.22\\)$/.test(String(this.strokeStyle)) && Math.abs(this.lineWidth - 0.45) > 0.01) stroke.apply(this, a); }; } document.body.prepend(Object.assign(document.createElement("div"), { id: "frame" })); document.head.append(Object.assign(document.createElement("style"), { textContent: "#frame { position: fixed; left: 0; top: 95px; width: 1330px; height: 748px }" })); document.querySelector("#dimension-2d").click(); document.querySelector("#dimension-3d").click(); const c = document.querySelector("#terrain"); c.setPointerCapture = () => {}; window.ev = (type, f) => c.dispatchEvent(new PointerEvent(type, { bubbles: true, cancelable: true, composed: true, clientX: 140 + 400 * f, clientY: 560 + 240 * f, pointerId: 9, pointerType: "mouse", isPrimary: true, button: 0, buttons: 1 })); }`, clip: { bitrate: 490_000, run: `ev("pointerdown", 1); ev("pointermove", 1); new Promise((ok) => setTimeout(ok, 2000))`, each: `(p) => ev("pointermove", 1 - 0.18 * (1 - Math.cos(2 * Math.PI * p)))` } },
  stack: {"url": "/stack/?from=2020-01-01&to=2025-12-31", "screen": [1800, 1100], "settle": 10000, "focus": "#chart", "margin": 1.1, "banner": true, "run": "document.head.append(Object.assign(document.createElement('style'), {textContent: '#plots {position:fixed;left:260px;top:260px;width:1300px} #chart {height:540px!important} #chart .cross > line {display:none} #chart .cross, #chart .ends[visibility=hidden] {display:none} #chart .cross[visibility=visible] {display:block}'}));", "clip": {"seconds": 7, "bitrate": 650000, "each": "(p) => { const el=document.querySelector('#plots'),r=el.getBoundingClientRect(); el.dispatchEvent(new PointerEvent('pointermove',{pointerType:'mouse',clientX:r.left+r.width*(0.12+0.8*p),clientY:r.top+r.height/2})); }"}},
  compass: {"url": "/compass/", "screen": [2000, 1200], "settle": 8000, "focus": "#frame, #compass", "margin": 1, "banner": true, "run": "document.body.prepend(Object.assign(document.createElement('div'), {id:'frame'})); document.head.append(Object.assign(document.createElement('style'), {textContent: '#frame {position:fixed;left:382px;top:380px;width:1077px;height:606px} #compass {position:fixed;left:560px;top:180px;width:1000px;height:800px;filter:brightness(2.1) saturate(1.35)} #compass text, #compass .icon, #compass [data-t] {display:none!important}'})); document.querySelector('#compass').setAttribute('viewBox','-240 -60 940 720'); const slider=document.querySelector('#slider'); slider.value=String(Number(slider.max)-1450); slider.dispatchEvent(new Event('input'));", "clip": {"seconds": 7, "bitrate": 470000, "run": "document.querySelector('[data-replay=atlas]').click();"}},
  calendar: {"url": "/calendar/?view=day&date=2025-06-15", "screen": [1800, 1100], "settle": 9000, "focus": ".dial", "margin": 1, "ground": "#f3ede2", "banner": true, "run": "document.documentElement.style.colorScheme='light'; document.head.append(Object.assign(document.createElement('style'), {textContent: '.dial {position:fixed;left:660px;top:170px;width:800px;height:720px;max-width:none;margin:0} .dial text, .dial .hour {display:none!important}'})); document.querySelector('.dial svg').setAttribute('viewBox','-90 -10 500 420');", "clip": {"seconds": 7, "bitrate": 900000, "each": "(p) => { const phase=Math.floor(p*4); if(window.__poolPhase!==phase){const buttons=[...document.querySelectorAll('#leaders button')]; buttons[Math.min(phase,buttons.length-1)]?.dispatchEvent(new PointerEvent('pointerenter',{pointerType:'mouse'}));window.__poolPhase=phase;} }"}},
};
if (onlyAt >= 0 && !(ONLY && (Object.hasOwn(PICTURES, ONLY) || Object.hasOwn(ICONS, ONLY) || Object.hasOwn(CARDS, ONLY)))) throw new Error(`--only ${ONLY ?? ""}: not an app (${[...new Set([...Object.keys(PICTURES), ...Object.keys(ICONS), ...Object.keys(CARDS)])].join(", ")})`);
const featured = [...(/const FEATURED = \[([^\]]*)\]/.exec(html)?.[1] ?? "").matchAll(/"([a-z]+)"/g)].map(([, id]) => id);
const appIds = [...(/const APPS = [^\[]*\[([\s\S]*?)\n      \]/.exec(html)?.[1] ?? "").matchAll(/id: "([a-z]+)"/g)].map(([, id]) => id);
for (const id of appIds) if (!(id in PICTURES)) problems.push(`${id}: no picture's making in PICTURES`);
for (const id of featured) if (!(/** @type {any} */ (PICTURES)[id]?.banner)) problems.push(`${id}: featured, but no banner's making in PICTURES`);

// ── The first title, in the page itself: FEATURED's first, as its own page names it and as APPS says it ──

const first = featured[0], entry = new RegExp(`\\{ id: "${first}", kind: "([^"]*)", line: "([^"]*)", href: "([^"]*)"`).exec(html);
const shown = {
  name: plain(/id="bb-title"[^>]*>([^<]*)</.exec(html)?.[1] ?? "", "the billboard"),
  kind: /id="bb-kind">([^<]*)</.exec(html)?.[1],
  line: /id="bb-blurb">([^<]*)</.exec(html)?.[1],
  href: /id="bb-open" href="([^"]*)"/.exec(html)?.[1],
  picture: /<div class="art"[^>]*><img[^>]* src="\/assets\/home\/banners\/([a-z]+)\.jpg"/.exec(html)?.[1],
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
  img.art { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; object-position: 70% 35%; transform: scale(1.3); transform-origin: 100% 0; }
  .shade { position: absolute; inset: 0; background: linear-gradient(to right, #0d0c0b 0, rgb(13 12 11 / 0.92) 34%, rgb(13 12 11 / 0.35) 62%, transparent 82%), linear-gradient(to top, rgb(13 12 11 / 0.7), transparent 40%); }
  .words { position: absolute; left: 72px; bottom: 76px; width: 560px; }
  .brand { display: flex; align-items: center; gap: 22px; font-size: 76px; font-weight: 750; letter-spacing: -0.04em; }
  .brand img { width: 68px; height: 79px; }
  p { margin: 22px 0 0; color: #d6cfc4; font-size: 28px; line-height: 1.35; letter-spacing: -0.01em; }
</style><img class="art" src="/assets/home/banners/${id}.jpg"><div class="shade"></div>
<div class="words"><div class="brand"><img src="/assets/logo/logo.svg">Bitview</div><p>Bitcoin's chain, read and measured: live apps, thousands of charts, and every series as data.</p></div>`;

// ── Its share card, named in its head ──

if (!/<meta property="og:image" content="https:\/\/bitview\.space\/assets\/home\/og\.jpg"/.test(html)) problems.push("The page's og:image isn't its share card, /assets/home/og.jpg");
if (!existsSync(join(HERE, "assets", "home", "og.jpg"))) problems.push("No share card, assets/home/og.jpg: run with --pictures");

/** How sharp a screen is shot: a poster's width from a recipe's 960-odd points. */
const DENSITY = 2;
/** The most a clip may weigh: each ships inside the binary and the published crate, and crates.io caps a crate at 10 MB. */
const VIDEO_BYTES = 450_000;
/** Each clip: its frames a second, its picture's size, the bitrate it's encoded at. */
const CLIP = { fps: 30, width: 1280, height: 720, bitrate: 650_000 };

/** Runs in every page, doing nothing until a clip starts: then the page's animation clock is ours, a frame stepped when
 * we ask, so a clip is smooth however long a screenshot takes. */
const CLOCK = `(() => {
  const real = { raf: requestAnimationFrame.bind(window), caf: cancelAnimationFrame.bind(window), now: performance.now.bind(performance), date: Date.now };
  let on = false, t = 0, start = 0, wall = 0, id = 0; const queue = new Map();
  window.requestAnimationFrame = (cb) => on ? (queue.set(++id, cb), id) : real.raf(cb);
  window.cancelAnimationFrame = (i) => on ? void queue.delete(i) : real.caf(i);
  performance.now = () => on ? t : real.now();
  Date.now = () => on ? wall + t - start : real.date();
  window.__clock = {
    freeze() { start = t = real.now(); wall = real.date(); on = true; for (const a of document.getAnimations()) a.pause(); },
    step(ms) { t += ms; const due = [...queue.values()]; queue.clear(); for (const cb of due) cb(t); for (const a of document.getAnimations()) { a.pause(); a.currentTime = (a.currentTime ?? 0) + ms; } },
  };
})();`;

/** The clip's encoder, in a tab of its own: a browser's H.264, frames in, length-prefixed samples and their avcC out. */
const ENCODER = `<!doctype html><script>
window.enc = {
  start(width, height, bitrate, fps) {
    this.fps = fps; this.n = 0; this.samples = []; this.config = null; this.error = null;
    this.encoder = new VideoEncoder({ output: (chunk, meta) => { const bytes = new Uint8Array(chunk.byteLength); chunk.copyTo(bytes); this.samples.push([chunk.type === "key", bytes]); if (meta?.decoderConfig?.description) this.config = new Uint8Array(meta.decoderConfig.description); }, error: (e) => (this.error = String(e)) });
    this.encoder.configure({ codec: "avc1.640028", width, height, bitrate, framerate: fps, bitrateMode: "variable", latencyMode: "quality", avc: { format: "avc" } });
  },
  async add(data) {
    const image = await createImageBitmap(new Blob([Uint8Array.from(atob(data), (c) => c.charCodeAt(0))], { type: "image/jpeg" }));
    const frame = new VideoFrame(image, { timestamp: Math.round((this.n * 1e6) / this.fps) });
    this.encoder.encode(frame, { keyFrame: this.n % (this.fps * 3) === 0 });
    frame.close(); image.close(); this.n++;
    while (this.encoder.encodeQueueSize > 4) await new Promise((ok) => setTimeout(ok, 4));
  },
  async finish() {
    await this.encoder.flush();
    if (this.error) throw new Error(this.error);
    const b64 = (u) => { let s = ""; for (let i = 0; i < u.length; i += 32768) s += String.fromCharCode(...u.subarray(i, i + 32768)); return btoa(s); };
    return JSON.stringify({ config: b64(this.config), samples: this.samples.map(([key, bytes]) => [key, b64(bytes)]) });
  },
};
</script>`;

if (pictures || icons) await makePictures();
// (Every featured app's poster, its tile's cover and its clip there, and no clip over what it may weigh.)
for (const id of Object.keys(PICTURES)) for (const file of [`banners/${id}.jpg`, `covers/${id}.jpg`, `videos/${id}.mp4`]) {
  const path = join(HERE, "assets", "home", file);
  if (!existsSync(path)) problems.push(`${id}: no assets/home/${file}: run with --pictures`);
  else if (file.endsWith(".mp4") && statSync(path).size > VIDEO_BYTES) problems.push(`${id}: its clip is ${statSync(path).size} bytes, over ${VIDEO_BYTES}`);
}
// (Each of those pages, and the front page with Studio's, says its clip as a link previews it: Discord plays it, on og:type
// video.other; everywhere else shows the og:image.)
for (const [href, page, id] of [["/", html, "studio"], ...Object.keys(PICTURES).map((id) => [`/${id}/`, readFileSync(join(HERE, id, "index.html"), "utf8"), id])]) {
  const meta = metaOf(page), src = `${ORIGIN}/assets/home/videos/${id}.mp4`;
  if (meta["og:type"] !== "video.other") problems.push(`${href}: its og:type isn't video.other`);
  if (meta["og:video"] !== src) problems.push(`${href}: its og:video isn't ${src}`);
  if (meta["og:video:type"] !== "video/mp4" || meta["og:video:width"] !== "1280" || meta["og:video:height"] !== "720") problems.push(`${href}: its og:video isn't said video/mp4, 1280×720`);
}
// (Every app's icon there, the favicon, and every card, once any were made.)
for (const name of Object.keys(ICONS)) if (!existsSync(join(HERE, "assets", "favicon", "apps", `${name}.png`))) problems.push(`${name}: no icon, assets/favicon/apps/${name}.png: run with --icons`);
if (!existsSync(join(HERE, "favicon.ico"))) problems.push("No favicon.ico: run with --icons");
for (const name of Object.keys(CARDS)) if (!existsSync(join(HERE, "assets", "cards", `${name}.jpg`))) problems.push(`${name}: no card, assets/cards/${name}.jpg: run with --pictures --only ${name}`);

if (problems.length) {
  console.error(`${problems.length} to see to:\n${problems.map((p) => `  - ${p}`).join("\n")}`);
  process.exit(1);
}
console.log(`In step: ${apps.length} apps and ${dev.length} developers' pages, ${studioPaths.length} ways into Studio, ${ids.size} series.`);

/** The pictures: a Chromium, headless, each app's screen as a dark one with less motion, each picture cut from it at the
 * density its size needs. */
async function makePictures() {
  const binary = process.env.CHROMIUM ?? ["/Applications/Chromium.app/Contents/MacOS/Chromium", "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/usr/bin/chromium", "/usr/bin/google-chrome"].find(existsSync);
  if (!binary) return void problems.push("No Chromium for --pictures or --icons: set CHROMIUM to one");
  const profile = mkdtempSync(join(tmpdir(), "home-pictures-"));
  /** @type {ReturnType<typeof Bun.spawn> | null} */
  let browser = null;
  try {
    browser = Bun.spawn([binary, "--headless=new", "--use-mock-keychain", "--password-store=basic", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--hide-scrollbars", "about:blank"], { stderr: "ignore", stdout: "ignore" });
    let port = "";
    for (let tries = 0; !port && tries < 100; tries++) {
      await Bun.sleep(100);
      if (existsSync(join(profile, "DevToolsActivePort"))) port = readFileSync(join(profile, "DevToolsActivePort"), "utf8").split("\n")[0];
    }
    if (!port) return void problems.push("The Chromium for --pictures didn't start");
    const out = join(HERE, "assets", "home");
    for (const folder of ["videos", "covers"]) mkdirSync(join(out, folder), { recursive: true });
    for (const [id, recipe] of Object.entries(pictures ? PICTURES : {})) {
      if ((ONLY && id !== ONLY) || !featured.includes(id)) continue;
      // (An app that fails: said, its files kept as they were, the others made.)
      const tab = await screen(port, `${SERVER}${recipe.url}`, recipe.screen ?? [1440, 900], DENSITY, recipe.settle, /** @type {any} */ (recipe).hide).catch((error) => (problems.push(`${id}'s pictures: ${error.message}`), null));
      if (!tab) continue;
      try {
        const box = await isolate(tab, recipe);
        for (const [file, width, quality] of /** @type {[string, number, number][]} */ ([[`banners/${id}.jpg`, 1920, 78], [`covers/${id}.jpg`, 640, 76]])) {
          const shot = await tab.send("Page.captureScreenshot", { format: "jpeg", quality, captureBeyondViewport: true, clip: { ...box, scale: width / (box.width * DENSITY) } });
          writeFileSync(join(out, file), Buffer.from(shot.data, "base64"));
          console.log(`made ${file}`);
        }
        await record(port, tab, box, `videos/${id}.mp4`, recipe.clip);
      } catch (error) {
        problems.push(`${id}'s pictures: ${/** @type {Error} */ (error).message}`);
      } finally {
        await tab.close();
      }
    }
    // (Each app's icon, at 512 px: a home screen's 180, a manifest's 512.)
    const svg = readFileSync(join(HERE, "assets", "favicon", "app-icon.svg"), "utf8");
    for (const [name, color] of Object.entries(pictures || icons ? ICONS : {})) {
      if (ONLY && name !== ONLY) continue;
      const icon = (color ? svg.replace(/--color: [^;"]+/, `--color: ${color}`) : svg).replace("<svg", '<svg width="512" height="512"');
      const tab = await screen(port, `${SERVER}/llms.txt`, [512, 512], 1, 300, undefined, `<style>body { margin: 0 } svg { display: block }</style>${icon}`);
      try {
        const shot = await tab.send("Page.captureScreenshot", { format: "png", clip: { x: 0, y: 0, width: 512, height: 512, scale: 1 } });
        writeFileSync(join(HERE, "assets", "favicon", "apps", `${name}.png`), Buffer.from(shot.data, "base64"));
        console.log(`made apps/${name}.png`);
        // (Bitview's is the MCP server's icon too: its page's and the MCP registry's.)
        if (name === "bitview") writeFileSync(MCP_ICON, Buffer.from(shot.data, "base64")), console.log("made bitview_mcp's logo.png");
      } finally {
        await tab.close();
      }
    }
    // (The favicon at /favicon.ico, for what asks there rather than reading a page's: favicon.svg at 16, 32 and 48 px,
    // clear around it, each a PNG in one ICO.)
    if (!ONLY || ONLY === "bitview") {
      const svg = readFileSync(join(HERE, "assets", "favicon", "favicon.svg"), "utf8"), sizes = [16, 32, 48], pngs = [];
      for (const size of sizes) {
        const tab = await screen(port, `${SERVER}/llms.txt`, [64, 64], 1, 300, undefined, `<style>html, body { margin: 0; background: transparent } svg { display: block }</style>${svg.replace("<svg", `<svg width="${size}" height="${size}"`)}`);
        try {
          await tab.send("Emulation.setDefaultBackgroundColorOverride", { color: { r: 0, g: 0, b: 0, a: 0 } });
          pngs.push(Buffer.from((await tab.send("Page.captureScreenshot", { format: "png", clip: { x: 0, y: 0, width: size, height: size, scale: 1 } })).data, "base64"));
        } finally {
          await tab.close();
        }
      }
      writeFileSync(join(HERE, "favicon.ico"), ico(sizes, pngs));
      console.log("made favicon.ico");
    }
    // (Every page's share card: its own screen, as it opens.)
    for (const [name, recipe] of Object.entries(pictures ? CARDS : {})) {
      if (ONLY && name !== ONLY) continue;
      const [width, height] = /** @type {any} */ (recipe).screen ?? [1200, 630];
      const tab = await screen(port, `${SERVER}${recipe.url}`, [width, height], 2, recipe.settle).catch((error) => (problems.push(`${name}'s card: ${error.message}`), null));
      if (!tab) continue;
      try {
        if ("run" in recipe) await tab.send("Runtime.evaluate", { expression: recipe.run }), await Bun.sleep(1000);
        const shot = await tab.send("Page.captureScreenshot", { format: "jpeg", quality: 88, clip: { x: 0, y: 0, width, height, scale: 1200 / width / 2 } });
        writeFileSync(join(HERE, "assets", "cards", `${name}.jpg`), Buffer.from(shot.data, "base64"));
        console.log(`made cards/${name}.jpg`);
      } finally {
        await tab.close();
      }
    }
    // (The share card: the billboard's first picture, Bitview's mark and name over it, set as the page is.)
    if (!pictures || (ONLY && ONLY !== featured[0])) return;
    const card = await screen(port, `${SERVER}/llms.txt`, [1200, 630], 2, 1500, undefined, SHARE_CARD(featured[0]));
    try {
      const shot = await card.send("Page.captureScreenshot", { format: "jpeg", quality: 84, clip: { x: 0, y: 0, width: 1200, height: 630, scale: 0.5 } });
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

/** The app's own drawing and nothing else: the page hidden but for it, on the ground it sits on. The area to shoot: 16:9 around it. */
async function isolate(/** @type {any} */ tab, /** @type {any} */ recipe) {
  const ground = recipe.ground ?? "#0d0c0b", focus = recipe.focus;
  if (recipe.run) (await tab.send("Runtime.evaluate", { expression: recipe.run, awaitPromise: true }), await Bun.sleep(recipe.ran ?? 1500));
  const { result } = await tab.send("Runtime.evaluate", { returnByValue: true, expression: `(() => {
    const style = document.createElement("style");
    style.textContent = \`html, body { background: ${ground} !important } body * { visibility: hidden !important; transition: none !important } ${focus}, ${focus} * { visibility: visible !important }\`;
    document.head.append(style);
    const r = document.querySelector(${JSON.stringify(focus)}).getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height, W: innerWidth, H: innerHeight };
  })()` });
  const r = result.value, tall = 16 / 9, width = Math.ceil(Math.max(r.width * (recipe.margin ?? 1.1), r.height * (recipe.margin ?? 1.1) * tall)), height = Math.ceil(width / tall);
  return { x: Math.round(r.x + r.width / 2 - width / 2), y: Math.round(r.y + r.height / 2 - height / 2), width, height };
}

/** A clip of the isolated drawing: the page's clock frozen, stepped a frame at a time, each screenshot encoded, written as an MP4. */
async function record(/** @type {string} */ port, /** @type {any} */ tab, /** @type {any} */ box, /** @type {string} */ file, /** @type {any} */ clip = {}) {
  const seconds = clip.seconds ?? 7, frames = Math.round(seconds * CLIP.fps), bitrate = clip.bitrate ?? CLIP.bitrate;
  const encoder = await screen(port, `${SERVER}/llms.txt`, [64, 64], 1, 200, undefined, ENCODER);
  try {
    const call = async (/** @type {string} */ expression) => {
      const { result, exceptionDetails } = await encoder.send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
      if (exceptionDetails) throw new Error(exceptionDetails.exception?.description ?? exceptionDetails.text);
      return result.value;
    };
    await call(`enc.start(${CLIP.width}, ${CLIP.height}, ${bitrate}, ${CLIP.fps})`);
    if (clip.run) await tab.send("Runtime.evaluate", { expression: clip.run, awaitPromise: true });
    await tab.send("Runtime.evaluate", { expression: "__clock.freeze()" });
    for (let n = 0; n < frames; n++) {
      await tab.send("Runtime.evaluate", { expression: `__clock.step(${1000 / CLIP.fps});${clip.each ? `(${clip.each})(${n / frames});` : ""}`, awaitPromise: true });
      const shot = await tab.send("Page.captureScreenshot", { format: "jpeg", quality: 90, captureBeyondViewport: true, clip: { ...box, scale: CLIP.width / (box.width * DENSITY) } });
      await call(`enc.add(${JSON.stringify(shot.data)})`);
    }
    const { config, samples } = JSON.parse(await call("enc.finish()"));
    writeFileSync(join(HERE, "assets", "home", file), mp4(Buffer.from(config, "base64"), samples.map((/** @type {[boolean, string]} */ [key, data]) => /** @type {[boolean, Buffer]} */ ([key, Buffer.from(data, "base64")]))));
    console.log(`made ${file}`);
  } finally {
    await encoder.close();
  }
}

/** An MP4 of H.264 samples, its index first so it can play as it arrives. */
function mp4(/** @type {Buffer} */ avcC, /** @type {[boolean, Buffer][]} */ samples) {
  const u32 = (/** @type {number} */ n) => { const b = Buffer.alloc(4); b.writeUInt32BE(n >>> 0); return b; }, u16 = (/** @type {number} */ n) => { const b = Buffer.alloc(2); b.writeUInt16BE(n); return b; };
  const box = (/** @type {string} */ type, /** @type {Buffer[]} */ ...body) => Buffer.concat([u32(8 + body.reduce((n, b) => n + b.length, 0)), Buffer.from(type), ...body]);
  const full = (/** @type {string} */ type, /** @type {number} */ flags, /** @type {Buffer[]} */ ...body) => box(type, u32(flags), ...body);
  const zero = (/** @type {number} */ n) => Buffer.alloc(n), timescale = CLIP.fps * 512, duration = samples.length * 512;
  const matrix = Buffer.concat([u32(0x10000), zero(12), u32(0x10000), zero(12), u32(0x40000000)]);
  const ftyp = box("ftyp", Buffer.from("isom"), u32(512), Buffer.from("isomiso2avc1mp41"));
  const moov = (/** @type {number} */ dataAt) => box("moov",
    full("mvhd", 0, zero(8), u32(timescale), u32(duration), u32(0x10000), u16(0x100), zero(10), matrix, zero(24), u32(2)),
    box("trak",
      full("tkhd", 3, zero(8), u32(1), zero(4), u32(duration), zero(16), matrix, u32(CLIP.width << 16), u32(CLIP.height << 16)),
      box("mdia",
        full("mdhd", 0, zero(8), u32(timescale), u32(duration), u16(0x55c4), zero(2)),
        full("hdlr", 0, zero(4), Buffer.from("vide"), zero(12), Buffer.from("Bitview\0")),
        box("minf",
          full("vmhd", 1, zero(8)),
          box("dinf", full("dref", 0, u32(1), full("url ", 1))),
          box("stbl",
            full("stsd", 0, u32(1), box("avc1", zero(6), u16(1), zero(16), u16(CLIP.width), u16(CLIP.height), u32(0x480000), u32(0x480000), zero(4), u16(1), zero(32), u16(0x18), u16(0xffff), box("avcC", avcC))),
            full("stts", 0, u32(1), u32(samples.length), u32(512)),
            full("stss", 0, u32(samples.filter(([key]) => key).length), ...samples.flatMap(([key], i) => (key ? [u32(i + 1)] : []))),
            full("stsc", 0, u32(1), u32(1), u32(samples.length), u32(1)),
            full("stsz", 0, u32(0), u32(samples.length), ...samples.map(([, data]) => u32(data.length))),
            full("stco", 0, u32(1), u32(dataAt)))))));
  const dataAt = ftyp.length + moov(0).length + 8;
  return Buffer.concat([ftyp, moov(dataAt), box("mdat", ...samples.map(([, data]) => data))]);
}

/** A page in a new tab, as a phone-free desk sees it, dark, less motion; settled; its words or all but its scene hidden;
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
  await send("Page.addScriptToEvaluateOnNewDocument", { source: CLOCK });
  // (A canvas's words: not drawn; a scene's tags, its rounded boxes, too.)
  if (hide) await send("Page.addScriptToEvaluateOnNewDocument", { source: `const P = CanvasRenderingContext2D.prototype; P.fillText = P.strokeText = ${hide === "scene" ? "P.roundRect = " : ""}function () {};` });
  const went = await send("Page.navigate", { url });
  if (went.errorText) throw new Error(`${url}: ${went.errorText}`);
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
