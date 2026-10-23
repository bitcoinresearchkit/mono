// studio.bitview.space, the preview Studio was on, now sends every visit to Studio where the server serves it
// (next.bitview.space/studio/), by Cloudflare's API alone (nothing to install): `bun showcases/studio/deploy.mjs`. A
// Worker of a few lines (no static files): every address, a temporary redirect there (302: a permanent one would stay in browsers' caches
// if where it goes changes), its query kept; a link's #chart= is the browser's to carry, and it does. Charts kept in
// the preview stay in its storage (another origin's): it was a preview.
//
// Its keys are in scripts/.tokens: CF_ACCOUNT_ID, CF_ZONE_ID (bitview.space's), CF_DEPLOY_API_TOKEN, an API token with
// Account › Workers Scripts › Edit, and the purge script's CF_PURGE_API_TOKEN (its host alone purged, never the zone's
// whole).
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
/** Where it is: its Worker, and its domain; where it sends every visit. */
const [name, hostname, to] = ["studio", "studio.bitview.space", "https://next.bitview.space/studio/"];

/** The keys in scripts/.tokens (`export NAME=value` lines), the environment's first. @returns {Record<string, string>} */
function keys() {
  const file = join(HERE, "..", "..", "scripts", ".tokens"), found = /** @type {Record<string, string>} */ ({});
  if (existsSync(file)) for (const [, name, value] of readFileSync(file, "utf8").matchAll(/^\s*(?:export\s+)?([A-Z_]+)=["']?([^"'\n]*)["']?\s*$/gm)) found[name] = value;
  return { ...found, ...Object.fromEntries(Object.entries(process.env).filter(([name]) => name.startsWith("CF_")).map(([name, value]) => [name, String(value)])) };
}

/** The redirect, deployed. */
export async function deploy() {
  const { CF_ACCOUNT_ID: account, CF_ZONE_ID: zone, CF_DEPLOY_API_TOKEN: token, CF_PURGE_API_TOKEN: purge } = keys();
  // (A token's shape checked first: one that isn't would be said back, whole, by the request it can't be sent in.)
  const shaped = (/** @type {string | undefined} */ key) => !key || /^[\w-]+$/.test(key);
  const why = !token ? "no CF_DEPLOY_API_TOKEN in scripts/.tokens (see showcases/studio/deploy.mjs)" : !shaped(token) || !shaped(purge) ? "CF_DEPLOY_API_TOKEN or CF_PURGE_API_TOKEN isn't one (letters, digits, - and _ only)" : !account || !zone ? "no CF_ACCOUNT_ID or CF_ZONE_ID in scripts/.tokens" : "";
  if (why) throw new Error(why);
  /** Cloudflare's API: its result, or what it says went wrong (never what was sent: a failed request says only its kind). */
  const api = async (/** @type {string} */ path, /** @type {RequestInit} */ init = {}, bearer = token) => {
    // (Its account, zone and tokens said as "…", in what Cloudflare says too.)
    const said = (/** @type {string} */ text) => [account, zone, token, purge].reduce((all, key) => (key ? all.replaceAll(key, "…") : all), text), asked = `${init.method ?? "GET"} ${path}`;
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
  // The Worker: its code alone, the files it served before let go of.
  const code = `export default { fetch: (request) => Response.redirect(${JSON.stringify(to)} + new URL(request.url).search, 302) };\n`;
  const worker = new FormData();
  worker.append("metadata", new Blob([JSON.stringify({ main_module: "redirect.js", compatibility_date: "2026-10-11" })], { type: "application/json" }));
  worker.append("redirect.js", new File([code], "redirect.js", { type: "application/javascript+module" }));
  await api(`/accounts/${account}/workers/scripts/${name}`, { method: "PUT", body: worker });
  await api(`/accounts/${account}/workers/domains`, { method: "PUT", headers: json, body: JSON.stringify({ hostname, service: name, environment: "production", zone_id: zone }) });
  // (Deployed whatever the purge does: one that fails says so, the page before served a while.)
  const purged = purge ? await api(`/zones/${zone}/purge_cache`, { method: "POST", headers: json, body: JSON.stringify({ hosts: [hostname] }) }, purge).then(() => "", (/** @type {Error} */ error) => error.message) : "no CF_PURGE_API_TOKEN";
  console.log(`https://${hostname} redirects to ${to}${purged ? ` (its cache not purged, the page before served a while: ${purged})` : ""}`);
}

if (process.argv[1] && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    await deploy();
  } catch (error) {
    console.error(/** @type {Error} */ (error).message);
    process.exit(1);
  }
}
