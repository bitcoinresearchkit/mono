// Studio on Cloudflare, by its API alone (nothing to install): studio.bitview.space shows the page built last, deployed
// by the build (or `bun showcases/studio/deploy.mjs`), and only if its version names the commit as it is (nothing it
// was built from changed since: a build with changes not committed isn't sent).
//
// A Worker of static files alone, the page (as the dashboard's upload makes one): its manifest sent, the page uploaded
// if Cloudflare hasn't it, the Worker put, its domain attached (already so, nothing changes), and its host's cache
// purged (bitview.space's edge keeps pages: the one before would be served on). Its keys are in scripts/.tokens:
// CF_ACCOUNT_ID, CF_ZONE_ID (bitview.space's), CF_DEPLOY_API_TOKEN, an API token with Account › Workers Scripts › Edit,
// and the purge script's CF_PURGE_API_TOKEN (its host alone purged, never the zone's whole).
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
/** Where it goes: its Worker, and its domain. */
const [name, hostname] = ["studio", "studio.bitview.space"];

/** The keys in scripts/.tokens (`export NAME=value` lines), the environment's first. @returns {Record<string, string>} */
function keys() {
  const file = join(HERE, "..", "..", "scripts", ".tokens"), found = /** @type {Record<string, string>} */ ({});
  if (existsSync(file)) for (const [, name, value] of readFileSync(file, "utf8").matchAll(/^\s*(?:export\s+)?([A-Z_]+)=["']?([^"'\n]*)["']?\s*$/gm)) found[name] = value;
  return { ...found, ...Object.fromEntries(Object.entries(process.env).filter(([name]) => name.startsWith("CF_")).map(([name, value]) => [name, String(value)])) };
}

/**
 * The page built last (dist), deployed if it's a commit's (`quiet`: the build's own, skipped with a line when it isn't
 * set up).
 */
export async function deploy(quiet = false) {
  const { CF_ACCOUNT_ID: account, CF_ZONE_ID: zone, CF_DEPLOY_API_TOKEN: token, CF_PURGE_API_TOKEN: purge } = keys();
  // (A token's shape checked first: one that isn't would be said back, whole, by the request it can't be sent in.)
  const shaped = (/** @type {string | undefined} */ key) => !key || /^[\w-]+$/.test(key);
  const why = !token ? "no CF_DEPLOY_API_TOKEN in scripts/.tokens (see showcases/studio/deploy.mjs)" : !shaped(token) || !shaped(purge) ? "CF_DEPLOY_API_TOKEN or CF_PURGE_API_TOKEN isn't one (letters, digits, - and _ only)" : !account || !zone ? "no CF_ACCOUNT_ID or CF_ZONE_ID in scripts/.tokens" : "";
  if (why) {
    if (quiet) return void console.log(`Not deployed: ${why}`);
    throw new Error(why);
  }
  if (!existsSync(join(HERE, "dist", "index.html"))) throw new Error("Nothing built yet (bun showcases/studio/build.mjs)");
  // (The page, and the files the build puts beside it for those who find it: see build.mjs.)
  const BESIDE = ["llms.txt", "sitemap.xml", "robots.txt", "og.png", "version.txt"], folder = join(HERE, "dist");
  const page = readFileSync(join(folder, "index.html"));
  /** @type {Record<string, Buffer>} */
  const files = {};
  for (const file of BESIDE) if (existsSync(join(folder, file))) files[`/${file}`] = readFileSync(join(folder, file));
  // (A commit's alone: its version names it as it is, nothing it was built from changed since.)
  const head = execFileSync("git", ["rev-parse", "--short=9", "HEAD"], { cwd: HERE }).toString().trim(), built = /const BUILD = "[^"]* \(([^)]*)\)";/.exec(String(page))?.[1];
  if (built !== head) throw new Error(!built ? "built from an unknown version" : built.endsWith("+") ? "built with changes not committed: commit, then build again" : `built from ${built}, not ${head}: build again`);
  files["/index.html"] = page;
  /** Cloudflare's API: its result, or what it says went wrong (never what was sent: a failed request says only its kind). */
  const api = async (/** @type {string} */ path, /** @type {RequestInit} */ init = {}, bearer = token) => {
    // (Its account and zone said as "…", in what Cloudflare says too.)
    const said = (/** @type {string} */ text) => text.replaceAll(account, "…").replaceAll(zone, "…"), asked = `${init.method ?? "GET"} ${path}`;
    let response;
    try {
      response = await fetch(`https://api.cloudflare.com/client/v4${path}`, { signal: AbortSignal.timeout(60_000), ...init, headers: { Authorization: `Bearer ${bearer}`, ...init.headers } });
    } catch (error) {
      const { name, cause } = /** @type {Error & { cause?: { code?: string } }} */ (error);
      throw new Error(said(`${asked}: ${name}${cause?.code ? ` (${cause.code})` : ""}`));
    }
    const body = await response.json().catch(() => ({}));
    if (!response.ok || body.success !== true) throw new Error(said(`${asked}: ${body.errors?.map((/** @type {any} */ error) => `${error.message} (${error.code})`).join("; ") || response.status}`));
    return body.result;
  };
  const json = { "Content-Type": "application/json" };
  // Each file of the Worker's: named by a hash as Cloudflare's own example makes it (SHA-256 of its base64 and its
  // extension, 32 hex digits).
  const TYPES = /** @type {Record<string, string>} */ ({ html: "text/html", txt: "text/plain", xml: "application/xml", png: "image/png" });
  const each = Object.entries(files).map(([path, bytes]) => {
    const extension = /** @type {string} */ (path.split(".").at(-1)), base64 = bytes.toString("base64");
    return { path, base64, size: bytes.length, type: TYPES[extension] ?? "application/octet-stream", hash: createHash("sha256").update(`${base64}${extension}`).digest("hex").slice(0, 32) };
  });
  const session = await api(`/accounts/${account}/workers/scripts/${name}/assets-upload-session`, { method: "POST", headers: json, body: JSON.stringify({ manifest: Object.fromEntries(each.map(({ path, hash, size }) => [path, { hash, size }])) }) });
  // Uploaded only if Cloudflare hasn't them: none asked for, the session's token is the upload's; else the last upload's.
  let uploaded = session?.jwt;
  for (const bucket of session?.buckets ?? []) {
    const form = new FormData();
    for (const hash of bucket) {
      const file = each.find((item) => item.hash === hash);
      if (file) form.append(hash, new File([file.base64], hash, { type: file.type }));
    }
    uploaded = (await api(`/accounts/${account}/workers/assets/upload?base64=true`, { method: "POST", body: form }, session.jwt))?.jwt;
  }
  if (!uploaded) throw new Error("Cloudflare gave no token for the files' upload");
  // The Worker: its files alone, served as they are (no code); any address that isn't one of them, the page (which opens
  // what it names: /presets/…; an old /chart/…, as /presets/…).
  const worker = new FormData();
  worker.append("metadata", new Blob([JSON.stringify({ assets: { jwt: uploaded, config: { not_found_handling: "single-page-application" } }, compatibility_date: "2026-10-11" })], { type: "application/json" }));
  await api(`/accounts/${account}/workers/scripts/${name}`, { method: "PUT", body: worker });
  await api(`/accounts/${account}/workers/domains`, { method: "PUT", headers: json, body: JSON.stringify({ hostname, service: name, environment: "production", zone_id: zone }) });
  // (Deployed whatever the purge does: one that fails says so, the page before served a while.)
  const purged = purge ? await api(`/zones/${zone}/purge_cache`, { method: "POST", headers: json, body: JSON.stringify({ hosts: [hostname] }) }, purge).then(() => "", (/** @type {Error} */ error) => error.message) : "no CF_PURGE_API_TOKEN";
  console.log(`Deployed to https://${hostname}${purged ? ` (its cache not purged, the page before served a while: ${purged})` : ""}`);
}

if (process.argv[1] && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    await deploy();
  } catch (error) {
    console.error(/** @type {Error} */ (error).message);
    process.exit(1);
  }
}
