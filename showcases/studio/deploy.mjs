// Studio on Cloudflare, by its API alone (nothing to install). The build deploys each page it builds to the preview,
// studio-next.bitview.space; `node showcases/studio/deploy.mjs live` puts a commit on studio.bitview.space: built again,
// apart (so no other build's page goes in its place), and sent only if its version names the commit as it is (nothing it
// was built from changed since) and it has its presets (bitview.space read).
//
// Each is a Worker of static files alone, the page (as the dashboard's upload makes one): its manifest sent, the page
// uploaded if Cloudflare hasn't it, the Worker put, its domain attached (already so, nothing changes). Its keys are in
// scripts/.tokens, as the purge script's: CF_ACCOUNT_ID, CF_ZONE_ID (bitview.space's), and CF_DEPLOY_API_TOKEN, an API
// token with Account › Workers Scripts › Edit.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
/** Where each goes: its Worker, and its domain. */
const TARGETS = /** @type {const} */ ({ next: ["studio-next", "studio-next.bitview.space"], live: ["studio", "studio.bitview.space"] });

/** The keys in scripts/.tokens (`export NAME=value` lines), the environment's first. @returns {Record<string, string>} */
function keys() {
  const file = join(HERE, "..", "..", "scripts", ".tokens"), found = /** @type {Record<string, string>} */ ({});
  if (existsSync(file)) for (const [, name, value] of readFileSync(file, "utf8").matchAll(/^\s*(?:export\s+)?([A-Z_]+)=["']?([^"'\n]*)["']?\s*$/gm)) found[name] = value;
  return { ...found, ...Object.fromEntries(Object.entries(process.env).filter(([name]) => name.startsWith("CF_")).map(([name, value]) => [name, String(value)])) };
}

/**
 * The page deployed: the one built last for the preview (`quiet`: the build's own, skipped with a line when it isn't
 * set up), one built now for live.
 * @param {keyof typeof TARGETS} target
 */
export async function deploy(target, quiet = false) {
  const { CF_ACCOUNT_ID: account, CF_ZONE_ID: zone, CF_DEPLOY_API_TOKEN: token } = keys();
  // (A token's shape checked first: one that isn't would be said back, whole, by the request it can't be sent in.)
  const why = !token ? "no CF_DEPLOY_API_TOKEN in scripts/.tokens (see showcases/studio/deploy.mjs)" : !/^[\w-]+$/.test(token) ? "CF_DEPLOY_API_TOKEN isn't one (letters, digits, - and _ only)" : !account || !zone ? "no CF_ACCOUNT_ID or CF_ZONE_ID in scripts/.tokens" : "";
  if (why) {
    if (quiet) return void console.log(`Preview not deployed: ${why}`);
    throw new Error(why);
  }
  let page;
  if (target === "live") {
    const apart = mkdtempSync(join(tmpdir(), "studio-live-")), file = join(apart, "index.html");
    try {
      execFileSync(process.execPath, [join(HERE, "build.mjs"), file], { stdio: "inherit" });
      page = readFileSync(file);
    } finally {
      rmSync(apart, { recursive: true, force: true });
    }
    const head = execFileSync("git", ["rev-parse", "--short=9", "HEAD"], { cwd: HERE }).toString().trim(), built = /const BUILD = "[^"]* \(([^)]*)\)";/.exec(String(page))?.[1];
    if (built !== head) throw new Error(`Not deployed: built from ${built ? `${built.replace("+", "")} with changes not committed` : "an unknown version"}, not the commit ${head} alone (live is a commit's: commit first)`);
    if (!/<script id="presets" type="application\/gzip">[^<]+<\/script>/.test(String(page))) throw new Error("Not deployed: built without its presets (bitview.space unread): try again");
  } else {
    const file = join(HERE, "dist", "index.html");
    if (!existsSync(file)) throw new Error("Nothing built yet (node showcases/studio/build.mjs)");
    page = readFileSync(file);
  }
  const [name, hostname] = TARGETS[target];
  /** Cloudflare's API: its result, or what it says went wrong (never what was sent: a failed request says only its kind). */
  const api = async (/** @type {string} */ path, /** @type {RequestInit} */ init = {}, bearer = token) => {
    const asked = `${init.method ?? "GET"} ${path.replace(account, "…")}`;
    let response;
    try {
      response = await fetch(`https://api.cloudflare.com/client/v4${path}`, { signal: AbortSignal.timeout(60_000), ...init, headers: { Authorization: `Bearer ${bearer}`, ...init.headers } });
    } catch (error) {
      const { name, cause } = /** @type {Error & { cause?: { code?: string } }} */ (error);
      throw new Error(`${asked}: ${name}${cause?.code ? ` (${cause.code})` : ""}`);
    }
    const body = await response.json().catch(() => ({}));
    if (!response.ok || body.success !== true) throw new Error(`${asked}: ${body.errors?.map((/** @type {any} */ error) => `${error.message} (${error.code})`).join("; ") || response.status}`);
    return body.result;
  };
  const json = { "Content-Type": "application/json" };
  // The page as a file of the Worker's: named by a hash as Cloudflare's own example makes it (SHA-256 of its base64 and
  // its extension, 32 hex digits).
  const base64 = page.toString("base64"), hash = createHash("sha256").update(`${base64}html`).digest("hex").slice(0, 32);
  const session = await api(`/accounts/${account}/workers/scripts/${name}/assets-upload-session`, { method: "POST", headers: json, body: JSON.stringify({ manifest: { "/index.html": { hash, size: page.length } } }) });
  // Uploaded only if Cloudflare hasn't it: none asked for, the session's token is the upload's; else the last upload's.
  let uploaded = session?.jwt;
  for (const bucket of session?.buckets ?? []) {
    const form = new FormData();
    for (const each of bucket) form.append(each, new File([base64], each, { type: "text/html" }));
    uploaded = (await api(`/accounts/${account}/workers/assets/upload?base64=true`, { method: "POST", body: form }, session.jwt))?.jwt;
  }
  if (!uploaded) throw new Error("Cloudflare gave no token for the page's upload");
  // The Worker: its files alone, served as they are (no code, Cloudflare's defaults).
  const worker = new FormData();
  worker.append("metadata", new Blob([JSON.stringify({ assets: { jwt: uploaded, config: {} }, compatibility_date: "2026-10-11" })], { type: "application/json" }));
  await api(`/accounts/${account}/workers/scripts/${name}`, { method: "PUT", body: worker });
  await api(`/accounts/${account}/workers/domains`, { method: "PUT", headers: json, body: JSON.stringify({ hostname, service: name, environment: "production", zone_id: zone }) });
  console.log(`Deployed to https://${hostname}`);
}

if (process.argv[1] && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const target = /** @type {keyof typeof TARGETS} */ (process.argv[2]);
  if (!Object.hasOwn(TARGETS, target)) {
    console.error("node showcases/studio/deploy.mjs next | live");
    process.exit(1);
  }
  try {
    await deploy(target);
  } catch (error) {
    console.error(/** @type {Error} */ (error).message);
    process.exit(1);
  }
}
