// The front page, kept in step with the apps it opens: each app's name and words read from its own page and written
// into the front page's FROM_APPS (its title before " · Bitview", else its launch tag's data-app; its meta
// description), and what it can't take from them checked: every app in the repository on it, every Studio chart and
// section it links to in Studio's sitemap, every series it draws on the server. With --pictures, its pictures made
// again from the apps as they are now (each from its app's own screen, cut where PICTURES says): after an app's
// redesign, as well as its words; its share card (assets/home/og.jpg, 1200×630, as links to it show it), the first
// featured app's banner with Bitview's mark and name; and every app's icon (assets/favicon/apps/). Run it after changing
// an app, and before a release:
//
//   bun showcases/home.mjs [server]              writes FROM_APPS, then checks
//   bun showcases/home.mjs --check [server]      fails if FROM_APPS isn't what it'd be now, or a check fails
//   bun showcases/home.mjs --pictures [server]   also makes its pictures again (a Chromium: CHROMIUM=path, if not found)
//   bun showcases/home.mjs --icons [server]      also makes every app's icon again, its pictures left (a colour changed)
//   ... --page FILE                              another page than showcases/index.html (a draft of it)
//   ... --pictures --only ID                     that app's pictures and icon alone (a new app's), the share card too if
//                                                it's its picture
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2), flag = (/** @type {string} */ name) => args.includes(name);
const check = flag("--check"), pictures = flag("--pictures"), icons = flag("--icons");
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

// ── Every app on it ──

// (An app: a folder of the repository's with a page, its name not starting with "_", an unlisted one's.)
const git = Bun.spawnSync(["git", "ls-files", "*/index.html"], { cwd: HERE });
if (git.exitCode !== 0) problems.push(`git ls-files failed (${new TextDecoder().decode(git.stderr).trim()}): the apps on the front page unchecked`);
const folders = new TextDecoder().decode(git.stdout).split("\n").filter(Boolean).map((file) => file.split("/")[0]).filter((name) => !name.startsWith("_"));
// (On it: in its rows, APPS or DEV, the footer's links not counting.)
for (const name of folders) if (![...apps, ...dev].includes(`/${name}/`)) problems.push(`/${name}/: an app not in the front page's rows (add it to APPS or DEV, or name its folder _${name} to keep it off)`);

// ── Every app's icon ──

// (Each app's, the front page's "bitview": Bitview's cube (assets/favicon/app-icon.svg) in its launch tag's colour
// (data-color; the SVG's own orange if none), a file (assets/favicon/apps/NAME.png, as launch.js links it): a home
// screen takes one best. Made with --pictures or --icons.)
const ICONS = /** @type {Record<string, string | undefined>} */ (Object.fromEntries([
  ["bitview", undefined],
  ...folders.map((name) => [name, tags(readFileSync(join(HERE, name, "index.html"), "utf8"), "script").find((tag) => tag.src?.includes("/launch.js"))?.["data-color"]]),
]));

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
 * ("text": a canvas's words, "scene": everything but its canvas, and its tags), and where its cover and its banner are
 * cut, in its screen's points: [x, y, width, height]. A cover 960×600; a banner 1,900 wide and 2,880 (@2x). */
const PICTURES = {
  studio: { url: "/studio/presets/bedrock/awake/floors-colored", screen: [2600, 1150], settle: 16000, cover: [677, 120, 1488, 930], banner: [265, 120, 1900, 930] },
  oracle: { url: "/oracle/", settle: 9000, cover: [0, 228, 928, 580], banner: [0, 244, 1440, 628] },
  globe: { url: "/globe/", settle: 9000, hide: "text", cover: [166, 44, 720, 450], banner: [43, 64, 966, 450] },
  trail: { url: "/trail/", settle: 12000, hide: "scene", cover: [230, 130, 960, 600], banner: [60, 80, 1360, 640] },
  stack: { url: "/stack/", settle: 8000, cover: [192, 336, 640, 400], banner: [192, 341, 648, 300] },
  compass: { url: "/compass/", settle: 6000, cover: [130, 60, 1088, 680] },
  calendar: { url: "/calendar/", settle: 9000, cover: [105, 100, 384, 240], banner: [105, 100, 863, 400] },
  series: { url: "/series/?series=hashrate", settle: 8000, cover: [880, 290, 432, 270] },
  logs: { url: "/logs/", settle: 6000, cover: [280, 228, 400, 250] },
};
if (onlyAt >= 0 && !(ONLY && (Object.hasOwn(PICTURES, ONLY) || Object.hasOwn(ICONS, ONLY)))) throw new Error(`--only ${ONLY ?? ""}: not an app (${[...new Set([...Object.keys(PICTURES), ...Object.keys(ICONS)])].join(", ")})`);
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
</style><img class="art" src="/assets/home/banners/${id}@2x.jpg"><div class="shade"></div>
<div class="words"><div class="brand"><img src="/assets/logo/logo.svg">Bitview</div><p>Bitcoin's chain, read and measured: live apps, thousands of charts, and every series as data.</p></div>`;

// ── Its share card, named in its head ──

if (!/<meta property="og:image" content="https:\/\/bitview\.space\/assets\/home\/og\.jpg"/.test(html)) problems.push("The page's og:image isn't its share card, /assets/home/og.jpg");
if (!existsSync(join(HERE, "assets", "home", "og.jpg"))) problems.push("No share card, assets/home/og.jpg: run with --pictures");

if (pictures || icons) await makePictures();
// (Every app's icon there, once any were made.)
for (const name of Object.keys(ICONS)) if (!existsSync(join(HERE, "assets", "favicon", "apps", `${name}.png`))) problems.push(`${name}: no icon, assets/favicon/apps/${name}.png: run with --icons`);

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
    browser = Bun.spawn([binary, "--headless=new", "--remote-debugging-port=0", `--user-data-dir=${profile}`, "--hide-scrollbars", "about:blank"], { stderr: "ignore", stdout: "ignore" });
    let port = "";
    for (let tries = 0; !port && tries < 100; tries++) {
      await Bun.sleep(100);
      if (existsSync(join(profile, "DevToolsActivePort"))) port = readFileSync(join(profile, "DevToolsActivePort"), "utf8").split("\n")[0];
    }
    if (!port) return void problems.push("The Chromium for --pictures didn't start");
    const out = join(HERE, "assets", "home");
    for (const [id, recipe] of Object.entries(icons ? {} : PICTURES)) {
      if (ONLY && id !== ONLY) continue;
      const wanted = /** @type {[string, number[], number, number][]} */ ([
        ...(appIds.includes(id) ? [[`covers/${id}.jpg`, recipe.cover, 960, 80]] : []),
        ...(featured.includes(id) && "banner" in recipe ? [[`banners/${id}.jpg`, recipe.banner, 1900, 78], [`banners/${id}@2x.jpg`, recipe.banner, 2880, 64]] : []),
      ]);
      if (!wanted.length) continue;
      // (One screen at the density the largest cut needs.)
      const density = Math.max(...wanted.map(([, box, width]) => Math.ceil((width / box[2]) * 10) / 10), 1);
      // (An app that fails: said, its pictures kept as they were, the others made.)
      const tab = await screen(port, `${SERVER}${recipe.url}`, recipe.screen ?? [1440, 900], density, recipe.settle, /** @type {any} */ (recipe).hide).catch((error) => (problems.push(`${id}'s pictures: ${error.message}`), null));
      if (!tab) continue;
      try {
        for (const [file, [x, y, w, h], width, quality] of wanted) {
          const shot = await tab.send("Page.captureScreenshot", { format: "jpeg", quality, clip: { x, y, width: w, height: h, scale: width / (w * density) } });
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
    const svg = readFileSync(join(HERE, "assets", "favicon", "app-icon.svg"), "utf8");
    for (const [name, color] of Object.entries(ICONS)) {
      if (ONLY && name !== ONLY) continue;
      const icon = (color ? svg.replace(/--color: [^;"]+/, `--color: ${color}`) : svg).replace("<svg", '<svg width="512" height="512"');
      const tab = await screen(port, `${SERVER}/llms.txt`, [512, 512], 1, 300, undefined, `<style>body { margin: 0 } svg { display: block }</style>${icon}`);
      try {
        const shot = await tab.send("Page.captureScreenshot", { format: "png", clip: { x: 0, y: 0, width: 512, height: 512, scale: 1 } });
        writeFileSync(join(HERE, "assets", "favicon", "apps", `${name}.png`), Buffer.from(shot.data, "base64"));
        console.log(`made apps/${name}.png`);
      } finally {
        await tab.close();
      }
    }
    // (The share card: the billboard's first picture, Bitview's mark and name over it, set as the page is.)
    if (icons || (ONLY && ONLY !== featured[0])) return;
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
